use std::{fs, io::Cursor, path::Path, sync::LazyLock};

use extensions_proto::moosync::types::ExtensionManifest;
use pgp::{
    composed::{Deserializable, SignedPublicKey, StandaloneSignature},
    types::PublicKeyTrait,
};
use regex::Regex;
use sha2::{Digest, Sha256};

use crate::{EXTENSION_LOCK_FILE, MANIFEST_FILE, errors::ExtensionError};

/// Detached ASCII-armored OpenPGP signature file for `manifest.pb`.
pub const MANIFEST_SIGNATURE_FILE: &str = "manifest.pb.asc";

static PACKAGE_NAME_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^[a-z][a-z0-9_]*(\.[a-z][a-z0-9_]*){2,}$").unwrap());

/// Signature verification outcome for an extension package.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SignatureStatus {
    Verified,
    Unsigned,
}

/// Result of verifying an extension package's file hashes and OpenPGP
/// signature.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PackageVerification {
    pub status: SignatureStatus,
    pub key_fingerprint: Option<String>,
}

/// Metadata and signature status returned by inspecting a local `.msxt`
/// archive.
#[derive(Debug, Clone)]
pub struct InspectedPackage {
    pub manifest: ExtensionManifest,
    pub signature_status: SignatureStatus,
}

/// Result of comparing an incoming registry public key against the cached key.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum KeyStoreOutcome {
    Stored,
    Unchanged,
    Changed {
        old_fingerprint: String,
        new_fingerprint: String,
    },
}

/// Persistence backend for trusted extension registry OpenPGP public keys.
pub trait RegistryKeyStorage: Send + Sync {
    fn get_registry_key(&self, registry_url: &str) -> Result<Option<Vec<u8>>, ExtensionError>;
    fn set_registry_key(&self, registry_url: &str, public_key: &[u8])
    -> Result<(), ExtensionError>;
    fn remove_registry_key(&self, registry_url: &str) -> Result<(), ExtensionError>;
    fn list_registry_keys(&self) -> Result<Vec<Vec<u8>>, ExtensionError>;
}

/// Validates that `package_name` uses lowercase reverse-domain notation (at
/// least 3 segments).
#[tracing::instrument(level = "debug", skip_all)]
pub fn validate_package_name(package_name: &str) -> Result<(), ExtensionError> {
    if !PACKAGE_NAME_REGEX.is_match(package_name) {
        return Err(ExtensionError::InvalidPackageName(package_name.to_string()));
    }
    Ok(())
}

/// Computes the lowercase hex SHA-256 digest of `bytes`.
#[tracing::instrument(level = "debug", skip_all)]
pub fn compute_sha256_hex(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    hex::encode(hasher.finalize())
}

/// Computes the lowercase hex SHA-256 digest of the file at `path`.
#[tracing::instrument(level = "debug", skip_all)]
pub fn compute_file_sha256_hex(path: &Path) -> Result<String, ExtensionError> {
    let bytes = fs::read(path)?;
    Ok(compute_sha256_hex(&bytes))
}

/// Parses an armored OpenPGP public key and returns its uppercase hex
/// fingerprint.
#[tracing::instrument(level = "debug", skip_all)]
pub fn public_key_fingerprint(public_key_bytes: &[u8]) -> Result<String, ExtensionError> {
    let key = parse_public_key(public_key_bytes)?;
    Ok(hex::encode_upper(key.fingerprint().as_bytes()))
}

/// Stores `public_key` for `registry_url` if unseen, or reports whether it
/// matched or changed.
#[tracing::instrument(level = "debug", skip_all)]
pub fn store_registry_key_checked(
    storage: &dyn RegistryKeyStorage,
    registry_url: &str,
    public_key: &[u8],
) -> Result<KeyStoreOutcome, ExtensionError> {
    let new_fingerprint = public_key_fingerprint(public_key)?;
    if let Some(existing_bytes) = storage.get_registry_key(registry_url)?
        && let Ok(old_fingerprint) = public_key_fingerprint(&existing_bytes)
    {
        if old_fingerprint == new_fingerprint {
            return Ok(KeyStoreOutcome::Unchanged);
        }
        return Ok(KeyStoreOutcome::Changed {
            old_fingerprint,
            new_fingerprint,
        });
    }
    storage.set_registry_key(registry_url, public_key)?;
    Ok(KeyStoreOutcome::Stored)
}

#[tracing::instrument(level = "debug", skip_all)]
fn parse_public_key(public_key_bytes: &[u8]) -> Result<SignedPublicKey, ExtensionError> {
    let (key, _) = SignedPublicKey::from_reader_single(Cursor::new(public_key_bytes))
        .map_err(|err| ExtensionError::InvalidSignature(format!("Invalid public key: {err}")))?;
    Ok(key)
}

#[tracing::instrument(level = "debug", skip_all)]
fn parse_detached_signature(signature_bytes: &[u8]) -> Result<StandaloneSignature, ExtensionError> {
    let (signature, _) = StandaloneSignature::from_reader_single(Cursor::new(signature_bytes))
        .map_err(|err| ExtensionError::InvalidSignature(format!("Invalid signature: {err}")))?;
    Ok(signature)
}

#[tracing::instrument(level = "debug", skip_all)]
fn verify_with_key(
    data: &[u8],
    signature: &StandaloneSignature,
    public_key: &SignedPublicKey,
) -> bool {
    if signature.verify(public_key, data).is_ok() {
        return true;
    }
    public_key
        .public_subkeys
        .iter()
        .any(|subkey| signature.verify(subkey, data).is_ok())
}

/// Returns the uppercase hex fingerprint of the key in `keys` that signed
/// `manifest.pb.asc`, if any.
#[tracing::instrument(level = "debug", skip_all)]
pub fn read_signing_key_fingerprint(
    extracted_dir: &Path,
    keys: &[Vec<u8>],
) -> Result<Option<String>, ExtensionError> {
    let signature_path = extracted_dir.join(MANIFEST_SIGNATURE_FILE);
    if !signature_path.exists() {
        return Ok(None);
    }
    let signature_bytes = fs::read(&signature_path)?;
    let signature = parse_detached_signature(&signature_bytes)?;
    let manifest_bytes = fs::read(extracted_dir.join(MANIFEST_FILE))?;

    for key_bytes in keys {
        match parse_public_key(key_bytes) {
            Ok(public_key) => {
                if verify_with_key(&manifest_bytes, &signature, &public_key) {
                    return Ok(Some(hex::encode_upper(public_key.fingerprint().as_bytes())));
                }
            }
            Err(err) => {
                tracing::error!("Failed to parse cached registry key: {:?}", err);
            }
        }
    }
    Ok(None)
}

#[tracing::instrument(level = "debug", skip_all)]
fn verify_file_hashes(
    extracted_dir: &Path,
    manifest: &ExtensionManifest,
) -> Result<(), ExtensionError> {
    for (filename, expected_hash) in &manifest.file_hashes {
        let file_path = extracted_dir.join(filename);
        if !file_path.exists() {
            return Err(ExtensionError::InvalidSignature(format!(
                "Missing hashed file: {filename}"
            )));
        }
        let actual_hash = compute_file_sha256_hex(&file_path)?;
        if actual_hash != *expected_hash {
            return Err(ExtensionError::HashMismatch {
                expected: expected_hash.clone(),
                actual: actual_hash,
            });
        }
    }
    Ok(())
}

#[tracing::instrument(level = "debug", skip_all)]
fn ensure_no_unhashed_files(
    extracted_dir: &Path,
    manifest: &ExtensionManifest,
) -> Result<(), ExtensionError> {
    if manifest.file_hashes.is_empty() {
        return Ok(());
    }
    for entry in fs::read_dir(extracted_dir)? {
        let path = entry?.path();
        if !path.is_file() {
            continue;
        }
        if let Some(name) = path.file_name().and_then(|n| n.to_str())
            && name != MANIFEST_FILE
            && name != MANIFEST_SIGNATURE_FILE
            && name != EXTENSION_LOCK_FILE
            && !manifest.file_hashes.contains_key(name)
        {
            return Err(ExtensionError::InvalidSignature(format!(
                "Unhashed file in package: {name}"
            )));
        }
    }
    Ok(())
}

/// Verifies all file hashes and checks `manifest.pb.asc` against trusted
/// `keys`.
#[tracing::instrument(level = "debug", skip_all)]
pub fn verify_package_contents(
    extracted_dir: &Path,
    manifest: &ExtensionManifest,
    keys: &[Vec<u8>],
) -> Result<PackageVerification, ExtensionError> {
    verify_file_hashes(extracted_dir, manifest)?;
    ensure_no_unhashed_files(extracted_dir, manifest)?;

    let key_fingerprint = read_signing_key_fingerprint(extracted_dir, keys)?;
    let status = if key_fingerprint.is_some() {
        SignatureStatus::Verified
    } else {
        SignatureStatus::Unsigned
    };
    Ok(PackageVerification {
        status,
        key_fingerprint,
    })
}
