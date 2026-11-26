use std::{collections::HashMap, fs, process::Command, sync::Mutex};

use extensions_proto::moosync::types::ExtensionManifest;
use prost::Message;
use rstest::rstest;
use tempdir::TempDir;

use crate::{
    MANIFEST_FILE,
    errors::ExtensionError,
    signing::{
        KeyStoreOutcome, MANIFEST_SIGNATURE_FILE, RegistryKeyStorage, SignatureStatus,
        compute_sha256_hex, store_registry_key_checked, validate_package_name,
        verify_package_contents,
    },
};

#[derive(Default)]
struct MemoryKeyStorage(Mutex<HashMap<String, Vec<u8>>>);

impl RegistryKeyStorage for MemoryKeyStorage {
    #[tracing::instrument(level = "debug", skip_all)]
    fn get_registry_key(&self, registry_url: &str) -> Result<Option<Vec<u8>>, ExtensionError> {
        Ok(self.0.lock().unwrap().get(registry_url).cloned())
    }

    #[tracing::instrument(level = "debug", skip_all)]
    fn set_registry_key(
        &self,
        registry_url: &str,
        public_key: &[u8],
    ) -> Result<(), ExtensionError> {
        self.0
            .lock()
            .unwrap()
            .insert(registry_url.to_string(), public_key.to_vec());
        Ok(())
    }

    #[tracing::instrument(level = "debug", skip_all)]
    fn remove_registry_key(&self, registry_url: &str) -> Result<(), ExtensionError> {
        self.0.lock().unwrap().remove(registry_url);
        Ok(())
    }

    #[tracing::instrument(level = "debug", skip_all)]
    fn list_registry_keys(&self) -> Result<Vec<Vec<u8>>, ExtensionError> {
        Ok(self.0.lock().unwrap().values().cloned().collect())
    }
}

#[rstest]
#[case("app.moosync.spotify", true)]
#[case("app.moosync.sample.empty", true)]
#[case("com.example.my_ext1", true)]
#[case("spotify", false)]
#[case("rs.sample", false)]
#[case("App.Moosync.Spotify", false)]
#[case("app.moosync.", false)]
#[case("app.moosync.123", false)]
#[case("app.moosync.my-ext", false)]
#[tracing::instrument(level = "debug", skip_all)]
fn test_validate_package_name(#[case] name: &str, #[case] expected_valid: bool) {
    assert_eq!(validate_package_name(name).is_ok(), expected_valid);
}

#[rstest]
#[tracing::instrument(level = "debug", skip_all)]
fn test_registry_key_store_roundtrip_and_invalid_key() {
    let store = MemoryKeyStorage::default();

    assert!(
        store_registry_key_checked(&store, "https://example.com/manifest.pb", b"not a pgp key")
            .is_err()
    );
    assert!(
        store
            .get_registry_key("https://example.com/manifest.pb")
            .unwrap()
            .is_none()
    );
    assert!(store.list_registry_keys().unwrap().is_empty());
}

#[rstest]
#[tracing::instrument(level = "debug", skip_all)]
fn test_verify_package_contents_unsigned_and_tampered() {
    let temp = TempDir::new("verify_pkg_test").unwrap();
    let wasm_bytes = b"fake wasm payload";
    fs::write(temp.path().join("ext.wasm"), wasm_bytes).unwrap();

    let mut file_hashes = HashMap::new();
    file_hashes.insert("ext.wasm".to_string(), compute_sha256_hex(wasm_bytes));

    let manifest = ExtensionManifest {
        moosync_extension: true,
        package_name: "app.test.verify".to_string(),
        extension_entry: "ext.wasm".to_string(),
        version: "1.0.0".to_string(),
        file_hashes,
        ..Default::default()
    };
    fs::write(temp.path().join(MANIFEST_FILE), manifest.encode_to_vec()).unwrap();

    let verification = verify_package_contents(temp.path(), &manifest, &[]).unwrap();
    assert_eq!(verification.status, SignatureStatus::Unsigned);

    // Extra unhashed file must fail
    fs::write(temp.path().join("extra.bin"), b"evil").unwrap();
    assert!(matches!(
        verify_package_contents(temp.path(), &manifest, &[]),
        Err(ExtensionError::InvalidSignature(_))
    ));
    fs::remove_file(temp.path().join("extra.bin")).unwrap();

    // Tampered wasm file must fail with HashMismatch
    fs::write(temp.path().join("ext.wasm"), b"tampered").unwrap();
    assert!(matches!(
        verify_package_contents(temp.path(), &manifest, &[]),
        Err(ExtensionError::HashMismatch { .. })
    ));
}

#[rstest]
#[tracing::instrument(level = "debug", skip_all)]
fn test_verify_package_contents_with_gpg_signature() {
    if Command::new("gpg").arg("--version").output().is_err() {
        return;
    }

    let temp = TempDir::new("verify_gpg_test").unwrap();
    let gnupg_home = temp.path().join("gnupg");
    fs::create_dir_all(&gnupg_home).unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&gnupg_home, fs::Permissions::from_mode(0o700)).unwrap();
    }

    let batch_cfg = "%no-protection\nKey-Type: eddsa\nKey-Curve: ed25519\nName-Real: Moosync Test\nName-Email: test@moosync.app\n%commit\n";
    let batch_path = temp.path().join("keygen.txt");
    fs::write(&batch_path, batch_cfg).unwrap();

    let gen_out = Command::new("gpg")
        .env("GNUPGHOME", &gnupg_home)
        .args(["--batch", "--gen-key", batch_path.to_str().unwrap()])
        .output()
        .unwrap();
    assert!(gen_out.status.success(), "gpg --gen-key failed");

    let pub_out = Command::new("gpg")
        .env("GNUPGHOME", &gnupg_home)
        .args(["--batch", "--armor", "--export"])
        .output()
        .unwrap();
    assert!(pub_out.status.success());

    let pkg_dir = temp.path().join("pkg");
    fs::create_dir_all(&pkg_dir).unwrap();
    let wasm_bytes = b"signed wasm payload";
    fs::write(pkg_dir.join("ext.wasm"), wasm_bytes).unwrap();

    let mut file_hashes = HashMap::new();
    file_hashes.insert("ext.wasm".to_string(), compute_sha256_hex(wasm_bytes));
    let manifest = ExtensionManifest {
        moosync_extension: true,
        package_name: "app.test.signed".to_string(),
        extension_entry: "ext.wasm".to_string(),
        version: "1.0.0".to_string(),
        file_hashes,
        ..Default::default()
    };
    let manifest_path = pkg_dir.join(MANIFEST_FILE);
    fs::write(&manifest_path, manifest.encode_to_vec()).unwrap();

    let sig_path = pkg_dir.join(MANIFEST_SIGNATURE_FILE);
    let sign_out = Command::new("gpg")
        .env("GNUPGHOME", &gnupg_home)
        .args([
            "--batch",
            "--yes",
            "--detach-sign",
            "--armor",
            "-o",
            sig_path.to_str().unwrap(),
            manifest_path.to_str().unwrap(),
        ])
        .output()
        .unwrap();
    assert!(sign_out.status.success(), "gpg --detach-sign failed");

    let store = MemoryKeyStorage::default();
    store_registry_key_checked(&store, "https://example.com/manifest.pb", &pub_out.stdout).unwrap();
    let keys = store.list_registry_keys().unwrap();

    let verified = verify_package_contents(&pkg_dir, &manifest, &keys).unwrap();
    assert_eq!(verified.status, SignatureStatus::Verified);

    // Without trusted key, signature is present but untrusted -> Unsigned
    let untrusted = verify_package_contents(&pkg_dir, &manifest, &[]).unwrap();
    assert_eq!(untrusted.status, SignatureStatus::Unsigned);

    // Corrupted signature must fail
    fs::write(&sig_path, b"corrupted signature").unwrap();
    assert!(matches!(
        verify_package_contents(&pkg_dir, &manifest, &keys),
        Err(ExtensionError::InvalidSignature(_))
    ));
}

#[rstest]
#[tracing::instrument(level = "debug", skip_all)]
fn test_registry_key_store_detects_key_change() {
    if Command::new("gpg").arg("--version").output().is_err() {
        return;
    }

    let gen_pub = |email: &str| -> Vec<u8> {
        let temp = TempDir::new("gpg_key_change").unwrap();
        let gnupg_home = temp.path().join("gnupg");
        fs::create_dir_all(&gnupg_home).unwrap();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(&gnupg_home, fs::Permissions::from_mode(0o700)).unwrap();
        }
        let cfg = format!(
            "%no-protection\nKey-Type: eddsa\nKey-Curve: ed25519\nName-Real: Moosync\nName-Email: {email}\n%commit\n"
        );
        let cfg_path = temp.path().join("cfg");
        fs::write(&cfg_path, cfg).unwrap();
        assert!(
            Command::new("gpg")
                .env("GNUPGHOME", &gnupg_home)
                .args(["--batch", "--gen-key", cfg_path.to_str().unwrap()])
                .output()
                .unwrap()
                .status
                .success()
        );
        Command::new("gpg")
            .env("GNUPGHOME", &gnupg_home)
            .args(["--batch", "--armor", "--export"])
            .output()
            .unwrap()
            .stdout
    };

    let key1 = gen_pub("one@moosync.app");
    let key2 = gen_pub("two@moosync.app");
    let url = "https://example.com/manifest.pb";

    let store = MemoryKeyStorage::default();

    assert_eq!(
        store_registry_key_checked(&store, url, &key1).unwrap(),
        KeyStoreOutcome::Stored
    );
    assert_eq!(
        store_registry_key_checked(&store, url, &key1).unwrap(),
        KeyStoreOutcome::Unchanged
    );

    let outcome = store_registry_key_checked(&store, url, &key2).unwrap();
    assert!(matches!(outcome, KeyStoreOutcome::Changed { .. }));
    assert_eq!(store.get_registry_key(url).unwrap().unwrap(), key1);

    store.set_registry_key(url, &key2).unwrap();
    assert_eq!(store.get_registry_key(url).unwrap().unwrap(), key2);
}
