use std::{fs, sync::Arc};

use assertables::assert_is_empty;
use extensions_proto::moosync::types::ExtensionAccountDetail;
use preferences_proto::moosync::types::PreferenceItem;
use rstest::{fixture, rstest};
use songs_proto::moosync::types::{Album, Artist, Genre, GetSongOptions, Playlist, Song};
use tempdir::TempDir;
use tracing_test::traced_test;
use zip_extensions::zip_writer::zip_create_from_directory;

use crate::{ExtensionHandler, RegistryKeyStorage, ReplyHandler, errors::ExtensionError};

struct DummyReply;
impl ReplyHandler for DummyReply {
    #[tracing::instrument(level = "debug", skip_all)]
    fn get_song(&self, _: &str, _: GetSongOptions) -> Result<Vec<Song>, ExtensionError> {
        Ok(vec![])
    }
    #[tracing::instrument(level = "debug", skip_all)]
    fn get_albums(&self, _: &str, _: Album, _: bool) -> Result<Vec<Album>, ExtensionError> {
        Ok(vec![])
    }
    fn get_artists(&self, _: &str, _: Artist, _: bool) -> Result<Vec<Artist>, ExtensionError> {
        Ok(vec![])
    }
    fn get_genres(&self, _: &str, _: Genre, _: bool) -> Result<Vec<Genre>, ExtensionError> {
        Ok(vec![])
    }
    fn get_playlists(
        &self,
        _: &str,
        _: Playlist,
        _: bool,
    ) -> Result<Vec<Playlist>, ExtensionError> {
        Ok(vec![])
    }
    #[tracing::instrument(level = "debug", skip_all)]
    fn get_current_song(&self, _: &str) -> Result<Option<Song>, ExtensionError> { Ok(None) }
    #[tracing::instrument(level = "debug", skip_all)]
    fn get_player_state(&self, _: &str) -> Result<i32, ExtensionError> { Ok(0) }
    #[tracing::instrument(level = "debug", skip_all)]
    fn get_volume(&self, _: &str) -> Result<f64, ExtensionError> { Ok(1.0) }
    #[tracing::instrument(level = "debug", skip_all)]
    fn get_time(&self, _: &str) -> Result<f64, ExtensionError> { Ok(0.0) }
    #[tracing::instrument(level = "debug", skip_all)]
    fn get_queue(&self, _: &str) -> Result<(Vec<Song>, usize), ExtensionError> { Ok((vec![], 0)) }
    #[tracing::instrument(level = "debug", skip_all)]
    fn get_preference(
        &self,
        _: &str,
        _: &str,
    ) -> Result<Option<extensions_proto::struct_proto::google::protobuf::Value>, ExtensionError>
    {
        Ok(None)
    }
    #[tracing::instrument(level = "debug", skip_all)]
    fn set_preference(
        &self,
        _: &str,
        _: &str,
        _: extensions_proto::struct_proto::google::protobuf::Value,
    ) -> Result<bool, ExtensionError> {
        Ok(true)
    }
    #[tracing::instrument(level = "debug", skip_all)]
    fn get_secure(
        &self,
        _: &str,
        _: &str,
    ) -> Result<Option<extensions_proto::struct_proto::google::protobuf::Value>, ExtensionError>
    {
        Ok(None)
    }
    #[tracing::instrument(level = "debug", skip_all)]
    fn set_secure(
        &self,
        _: &str,
        _: &str,
        _: extensions_proto::struct_proto::google::protobuf::Value,
    ) -> Result<bool, ExtensionError> {
        Ok(true)
    }
    #[tracing::instrument(level = "debug", skip_all)]
    fn add_songs(&self, _: &str, _: Vec<Song>) -> Result<Vec<Song>, ExtensionError> { Ok(vec![]) }
    #[tracing::instrument(level = "debug", skip_all)]
    fn remove_song(&self, _: &str, _: Song) -> Result<bool, ExtensionError> { Ok(true) }
    #[tracing::instrument(level = "debug", skip_all)]
    fn update_song(&self, _: &str, s: Song) -> Result<Song, ExtensionError> { Ok(s) }
    #[tracing::instrument(level = "debug", skip_all)]
    fn add_playlist(&self, _: &str, _: Playlist) -> Result<String, ExtensionError> {
        Ok("".to_string())
    }
    #[tracing::instrument(level = "debug", skip_all)]
    fn add_to_playlist(&self, _: &str, _: String, _: Vec<Song>) -> Result<bool, ExtensionError> {
        Ok(true)
    }
    #[tracing::instrument(level = "debug", skip_all)]
    fn register_oauth(&self, _: &str, _: String) -> Result<bool, ExtensionError> { Ok(true) }
    #[tracing::instrument(level = "debug", skip_all)]
    fn open_external_url(&self, _: &str, _: String) -> Result<bool, ExtensionError> { Ok(true) }
    #[tracing::instrument(level = "debug", skip_all)]
    fn set_account(&self, _: &str, _: ExtensionAccountDetail) -> Result<bool, ExtensionError> {
        Ok(true)
    }
    #[tracing::instrument(level = "debug", skip_all)]
    fn register_user_preference(
        &self,
        _: &str,
        _: Vec<PreferenceItem>,
    ) -> Result<bool, ExtensionError> {
        Ok(true)
    }
    #[tracing::instrument(level = "debug", skip_all)]
    fn unregister_user_preference(&self, _: &str, _: Vec<String>) -> Result<bool, ExtensionError> {
        Ok(true)
    }
    #[tracing::instrument(level = "debug", skip_all)]
    fn extensions_updated(&self, _: &str) -> Result<(), ExtensionError> { Ok(()) }
    #[tracing::instrument(level = "debug", skip_all)]
    fn get_app_version(&self, _: &str) -> Result<String, ExtensionError> { Ok("1.0.0".to_string()) }
    #[tracing::instrument(level = "debug", skip_all)]
    fn get_cache(&self, _: &str, _: &str) -> Result<Option<Vec<u8>>, ExtensionError> { Ok(None) }
    #[tracing::instrument(level = "debug", skip_all)]
    fn set_cache(&self, _: &str, _: &str, _: Vec<u8>, _: u64) -> Result<bool, ExtensionError> {
        Ok(true)
    }
}

struct TestHandlerContext {
    pub _temp_dir: TempDir,
    pub handler: ExtensionHandler,
}

#[fixture]
#[tracing::instrument(level = "debug", skip_all)]
fn handler_context() -> TestHandlerContext {
    let temp_dir = TempDir::new("moosync_ext_lib").expect("failed to create temp dir");
    let ext_dir = temp_dir.path().join("exts");
    let tmp_dir = temp_dir.path().join("tmp");
    let cache_dir = temp_dir.path().join("cache");
    fs::create_dir_all(&ext_dir).unwrap();
    fs::create_dir_all(&tmp_dir).unwrap();
    fs::create_dir_all(&cache_dir).unwrap();

    let handler = ExtensionHandler::new(ext_dir, tmp_dir, cache_dir);
    handler.set_reply_handler(Arc::new(DummyReply));
    TestHandlerContext {
        _temp_dir: temp_dir,
        handler,
    }
}

#[rstest]
#[tokio::test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_extension_handler_init_and_get_all(handler_context: TestHandlerContext) {
    let TestHandlerContext { handler, .. } = handler_context;

    let all = handler.get_all_extensions().await;

    assert_is_empty!(&all);
    assert!(!handler.has_updates());
}

#[rstest]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_extension_handler_check_for_updates_v1(handler_context: TestHandlerContext) {
    let ext_dir = handler_context.handler.extensions_dir.join("app.test.ext");
    fs::create_dir_all(&ext_dir).unwrap();
    let manifest = extensions_proto::moosync::types::ExtensionManifest {
        package_name: "app.test.ext".to_string(),
        display_name: "Test Extension".to_string(),
        version: "1.0.0".to_string(),
        extension_entry: "main.wasm".to_string(),
        icon: "icon.png".to_string(),
        author: Some("Tester".to_string()),
        file_hashes: Default::default(),
        sha256: None,
        ..Default::default()
    };
    fs::write(
        ext_dir.join("manifest.pb"),
        prost::Message::encode_to_vec(&manifest),
    )
    .unwrap();
    let empty_wasm = vec![0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00];
    fs::write(ext_dir.join("main.wasm"), empty_wasm.clone()).unwrap();
    let lock_data = extensions_proto::moosync::types::ExtensionLockData {
        registry: "local".to_string(),
    };
    fs::write(
        ext_dir.join("extension.lock"),
        prost::Message::encode_to_vec(&lock_data),
    )
    .unwrap();
    handler_context
        .handler
        .load_extension(&ext_dir.join("manifest.pb"), true)
        .unwrap();

    let remote_manifests = vec![extensions_proto::moosync::types::ExtensionManifest {
        moosync_extension: false,
        display_name: "Test Extension".to_string(),
        extension_entry: "".to_string(),
        author: None,
        package_name: "app.test.ext".to_string(),
        version: "1.1.0".to_string(),
        icon: "".to_string(),
        permissions: None,
        description: None,
        url: Some("https://example.com/test_ext.msxt".to_string()),
        registry: Some("default".to_string()),
        file_hashes: Default::default(),
        sha256: None,
        scopes: vec![],
    }];
    handler_context
        .handler
        .set_remote_manifests(remote_manifests);

    let has_updates = handler_context.handler.check_for_updates();
    let updatables = handler_context.handler.get_updatable_extensions();

    assert!(has_updates);
    assert!(handler_context.handler.has_updates());
    assert_eq!(updatables.len(), 1);
    assert_eq!(updatables[0].package_name, "app.test.ext");
    assert_eq!(updatables[0].version, "1.1.0");
}

#[rstest]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_extension_handler_check_for_updates_older_remote(handler_context: TestHandlerContext) {
    let ext_dir = handler_context.handler.extensions_dir.join("app.test.ext");
    fs::create_dir_all(&ext_dir).unwrap();
    let manifest = extensions_proto::moosync::types::ExtensionManifest {
        package_name: "app.test.ext".to_string(),
        display_name: "Test Extension".to_string(),
        version: "1.2.0".to_string(),
        extension_entry: "main.wasm".to_string(),
        icon: "icon.png".to_string(),
        author: Some("Tester".to_string()),
        file_hashes: Default::default(),
        sha256: None,
        ..Default::default()
    };
    fs::write(
        ext_dir.join("manifest.pb"),
        prost::Message::encode_to_vec(&manifest),
    )
    .unwrap();
    let empty_wasm = vec![0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00];
    fs::write(ext_dir.join("main.wasm"), empty_wasm.clone()).unwrap();
    let lock_data = extensions_proto::moosync::types::ExtensionLockData {
        registry: "local".to_string(),
    };
    fs::write(
        ext_dir.join("extension.lock"),
        prost::Message::encode_to_vec(&lock_data),
    )
    .unwrap();
    handler_context
        .handler
        .load_extension(&ext_dir.join("manifest.pb"), true)
        .unwrap();

    let remote_manifests = vec![extensions_proto::moosync::types::ExtensionManifest {
        moosync_extension: false,
        display_name: "Test Extension".to_string(),
        extension_entry: "".to_string(),
        author: None,
        package_name: "app.test.ext".to_string(),
        version: "1.1.0".to_string(),
        icon: "".to_string(),
        permissions: None,
        description: None,
        url: Some("https://example.com/test_ext.msxt".to_string()),
        registry: Some("default".to_string()),
        file_hashes: Default::default(),
        sha256: None,
        scopes: vec![],
    }];
    handler_context
        .handler
        .set_remote_manifests(remote_manifests);

    let has_updates = handler_context.handler.check_for_updates();
    let updatables = handler_context.handler.get_updatable_extensions();

    assert!(!has_updates);
    assert!(!handler_context.handler.has_updates());
    assert_is_empty!(&updatables);
}

#[rstest]
#[tokio::test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_extension_handler_update_all_extensions_no_updates(
    handler_context: TestHandlerContext,
) {
    let res = handler_context.handler.update_all_extensions().await;
    assert!(res.is_ok());
    assert!(!handler_context.handler.has_updates());
}

#[rstest]
#[tokio::test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_extension_handler_update_all_extensions_failed(handler_context: TestHandlerContext) {
    let ext_dir = handler_context.handler.extensions_dir.join("app.test.ext");
    fs::create_dir_all(&ext_dir).unwrap();
    let manifest = extensions_proto::moosync::types::ExtensionManifest {
        package_name: "app.test.ext".to_string(),
        display_name: "Test Extension".to_string(),
        version: "1.0.0".to_string(),
        extension_entry: "main.wasm".to_string(),
        icon: "icon.png".to_string(),
        author: Some("Tester".to_string()),
        file_hashes: Default::default(),
        sha256: None,
        ..Default::default()
    };
    fs::write(
        ext_dir.join("manifest.pb"),
        prost::Message::encode_to_vec(&manifest),
    )
    .unwrap();
    let empty_wasm = vec![0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00];
    fs::write(ext_dir.join("main.wasm"), empty_wasm).unwrap();
    let lock_data = extensions_proto::moosync::types::ExtensionLockData {
        registry: "local".to_string(),
    };
    fs::write(
        ext_dir.join("extension.lock"),
        prost::Message::encode_to_vec(&lock_data),
    )
    .unwrap();
    handler_context
        .handler
        .load_extension(&ext_dir.join("manifest.pb"), true)
        .unwrap();

    let remote_manifests = vec![extensions_proto::moosync::types::ExtensionManifest {
        moosync_extension: false,
        display_name: "Test Extension".to_string(),
        extension_entry: "".to_string(),
        author: None,
        package_name: "app.test.ext".to_string(),
        version: "1.1.0".to_string(),
        icon: "".to_string(),
        permissions: None,
        description: None,
        url: Some("http://127.0.0.1:9/non_existent.msxt".to_string()),
        registry: Some("default".to_string()),
        file_hashes: Default::default(),
        sha256: None,
        scopes: vec![],
    }];
    handler_context
        .handler
        .set_remote_manifests(remote_manifests);
    handler_context.handler.check_for_updates();
    assert!(handler_context.handler.has_updates());

    let res = handler_context.handler.update_all_extensions().await;
    assert!(res.is_err());
    assert!(handler_context.handler.has_updates());
}

#[rstest]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_extension_info_methods_and_traits() {
    use std::{
        collections::hash_map::DefaultHasher,
        hash::{Hash, Hasher},
    };

    let local_info =
        crate::ExtensionInfo::Local(extensions_proto::moosync::types::ExtensionDetail {
            package_name: "test.pkg".to_string(),
            ..Default::default()
        });
    let remote_info =
        crate::ExtensionInfo::Remote(extensions_proto::moosync::types::ExtensionManifest {
            package_name: "test.pkg".to_string(),
            file_hashes: Default::default(),
            sha256: None,
            ..Default::default()
        });
    let path_info = crate::ExtensionInfo::LocalPath(std::path::PathBuf::from("/some/path"));

    assert_eq!(local_info.package_name(), "test.pkg");
    assert_eq!(remote_info.package_name(), "test.pkg");
    assert_eq!(path_info.package_name(), "");

    assert_eq!(local_info, remote_info);
    assert_ne!(local_info, path_info);

    let mut h1 = DefaultHasher::new();
    local_info.hash(&mut h1);
    let mut h2 = DefaultHasher::new();
    remote_info.hash(&mut h2);
    assert_eq!(h1.finish(), h2.finish());
}

#[rstest]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_extension_handler_triggers(handler_context: TestHandlerContext) {
    let handler = &handler_context.handler;

    handler.trigger_accounts_updated(Some("acc_1".to_string()));
    handler.trigger_accounts_updated(None);
    handler.trigger_preferences_updated("pkg.name".to_string());

    assert_eq!(handler.get_active_extensions().len(), 0);
}

#[rstest]
#[tokio::test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_extension_handler_get_active_and_toggle(handler_context: TestHandlerContext) {
    let ext_dir = handler_context.handler.extensions_dir.join("active_ext");
    fs::create_dir_all(&ext_dir).unwrap();
    let manifest = extensions_proto::moosync::types::ExtensionManifest {
        package_name: "app.test.active".to_string(),
        display_name: "Active Extension".to_string(),
        version: "1.0.0".to_string(),
        extension_entry: "main.wasm".to_string(),
        file_hashes: Default::default(),
        sha256: None,
        scopes: vec![extensions_proto::moosync::types::ScopePermission {
            scope: extensions_proto::moosync::types::ExtensionProviderScope::Search as i32,
            justification: "Search".to_string(),
        }],
        ..Default::default()
    };
    fs::write(
        ext_dir.join("manifest.pb"),
        prost::Message::encode_to_vec(&manifest),
    )
    .unwrap();
    fs::write(ext_dir.join("main.wasm"), b"wasm bytes").unwrap();

    let handler = &handler_context.handler;
    handler
        .load_extension(&ext_dir.join("manifest.pb"), true)
        .unwrap();

    let ext = handler.get_extension("app.test.active");
    assert!(ext.is_ok());

    let scoped = handler
        .get_extensions_with_scope(extensions_proto::moosync::types::ExtensionProviderScope::Search)
        .await;
    assert_eq!(scoped.len(), 1);

    let all = handler.get_all_extensions().await;
    assert_eq!(all.len(), 1);

    let ext_arc = ext.unwrap();
    assert_eq!(handler.get_active_extensions().len(), 0);

    let detail = ext_arc.get_extension_detail();
    let toggle_res = handler.toggle_extension(crate::ExtensionInfo::Local(detail));
    assert!(toggle_res.is_ok());
    assert!(handler.get_extension("app.test.active").is_err());

    let invalid_toggle = handler.toggle_extension(crate::ExtensionInfo::Remote(manifest));
    assert!(invalid_toggle.is_err());
}

#[rstest]
#[tokio::test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_extension_version_and_installation(handler_context: TestHandlerContext) {
    let handler = &handler_context.handler;
    let temp_dir = TempDir::new("ext_version_test").unwrap();
    let valid_dir = temp_dir.path().join("valid");
    fs::create_dir_all(&valid_dir).unwrap();
    let manifest = extensions_proto::moosync::types::ExtensionManifest {
        moosync_extension: true,
        package_name: "app.test.zip".to_string(),
        display_name: "Zip Extension".to_string(),
        version: "2.3.4".to_string(),
        extension_entry: "main.wasm".to_string(),
        file_hashes: Default::default(),
        sha256: None,
        ..Default::default()
    };
    fs::write(
        valid_dir.join("manifest.pb"),
        prost::Message::encode_to_vec(&manifest),
    )
    .unwrap();
    fs::write(valid_dir.join("main.wasm"), b"wasm bytes").unwrap();

    let local_install = handler
        .install_extension(crate::ExtensionInfo::Local(
            extensions_proto::moosync::types::ExtensionDetail::default(),
        ))
        .await;
    assert!(local_install.is_ok());

    let zip_file = temp_dir.path().join("zip_ext.msxt");
    zip_create_from_directory(&zip_file, &valid_dir).unwrap();

    let zip_install = handler
        .install_extension(crate::ExtensionInfo::LocalPath(zip_file))
        .await;
    assert!(zip_install.is_ok());

    let installed_ext = handler.get_extension("app.test.zip");
    assert!(installed_ext.is_ok());

    let remote_fetch_res = handler
        .fetch_remote_manifests(std::collections::HashSet::new())
        .await;
    assert!(remote_fetch_res.is_ok());
}

#[rstest]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_check_for_updates_invalid_semver_edge_cases(handler_context: TestHandlerContext) {
    let handler = &handler_context.handler;

    let remote_manifests = vec![
        extensions_proto::moosync::types::ExtensionManifest {
            package_name: "uninstalled_ext".to_string(),
            version: "1.0.0".to_string(),
            file_hashes: Default::default(),
            sha256: None,
            ..Default::default()
        },
        extensions_proto::moosync::types::ExtensionManifest {
            package_name: "bad_remote_ver".to_string(),
            version: "invalid-semver".to_string(),
            file_hashes: Default::default(),
            sha256: None,
            ..Default::default()
        },
    ];
    handler.set_remote_manifests(remote_manifests);
    let has_updates = handler.check_for_updates();
    assert!(!has_updates);
}

#[rstest]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_remove_extension_mismatched_dir_name(handler_context: TestHandlerContext) {
    let handler = &handler_context.handler;
    let ext_dir = handler.extensions_dir.join("mismatched_dir");
    fs::create_dir_all(&ext_dir).unwrap();
    let manifest = extensions_proto::moosync::types::ExtensionManifest {
        package_name: "actual.package.name".to_string(),
        display_name: "Mismatched Dir Extension".to_string(),
        version: "1.0.0".to_string(),
        extension_entry: "main.wasm".to_string(),
        file_hashes: Default::default(),
        sha256: None,
        ..Default::default()
    };
    fs::write(
        ext_dir.join("manifest.pb"),
        prost::Message::encode_to_vec(&manifest),
    )
    .unwrap();
    fs::write(ext_dir.join("main.wasm"), b"wasm bytes").unwrap();
    handler
        .load_extension(&ext_dir.join("manifest.pb"), true)
        .unwrap();

    handler
        .remove_extension("actual.package.name".to_string())
        .unwrap();

    assert!(!ext_dir.exists());
    assert!(handler.get_extension("actual.package.name").is_err());
}

#[rstest]
#[tokio::test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_update_signing_key_mismatch_and_match(handler_context: TestHandlerContext) {
    if std::process::Command::new("gpg")
        .arg("--version")
        .output()
        .is_err()
    {
        return;
    }

    let handler = &handler_context.handler;
    let temp = TempDir::new("update_key_mismatch_test").unwrap();

    let make_key = |sub: &str| -> (std::path::PathBuf, Vec<u8>) {
        let gnupg = temp.path().join(sub);
        fs::create_dir_all(&gnupg).unwrap();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(&gnupg, fs::Permissions::from_mode(0o700)).unwrap();
        }
        let cfg = temp.path().join(format!("{sub}.cfg"));
        fs::write(
            &cfg,
            "%no-protection\nKey-Type: eddsa\nKey-Curve: ed25519\nName-Real: Test\nName-Email: test@moosync.app\n%commit\n",
        )
        .unwrap();
        assert!(
            std::process::Command::new("gpg")
                .env("GNUPGHOME", &gnupg)
                .args(["--batch", "--gen-key", cfg.to_str().unwrap()])
                .output()
                .unwrap()
                .status
                .success()
        );
        let pub_bytes = std::process::Command::new("gpg")
            .env("GNUPGHOME", &gnupg)
            .args(["--batch", "--armor", "--export"])
            .output()
            .unwrap()
            .stdout;
        (gnupg, pub_bytes)
    };

    let (gnupg1, pub1) = make_key("k1");
    let (gnupg2, pub2) = make_key("k2");

    #[derive(Default)]
    struct TestKeyStorage(std::sync::Mutex<std::collections::HashMap<String, Vec<u8>>>);
    impl RegistryKeyStorage for TestKeyStorage {
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

    let key_store = Arc::new(TestKeyStorage::default());
    key_store
        .set_registry_key("https://reg1.example/manifest.pb", &pub1)
        .unwrap();
    key_store
        .set_registry_key("https://reg2.example/manifest.pb", &pub2)
        .unwrap();
    handler.set_registry_key_storage(key_store);

    let build_signed_msxt = |ver: &str, gnupg: &std::path::Path| -> std::path::PathBuf {
        let dir = temp.path().join(format!(
            "pkg_{ver}_{}",
            gnupg.file_name().unwrap().to_str().unwrap()
        ));
        fs::create_dir_all(&dir).unwrap();
        let wasm = b"wasm bytes";
        fs::write(dir.join("main.wasm"), wasm).unwrap();
        let mut hashes = std::collections::HashMap::new();
        hashes.insert(
            "main.wasm".to_string(),
            crate::signing::compute_sha256_hex(wasm),
        );
        let manifest = extensions_proto::moosync::types::ExtensionManifest {
            moosync_extension: true,
            package_name: "app.test.signed_update".to_string(),
            display_name: "Signed Update".to_string(),
            version: ver.to_string(),
            extension_entry: "main.wasm".to_string(),
            file_hashes: hashes,
            ..Default::default()
        };
        let manifest_path = dir.join(crate::MANIFEST_FILE);
        fs::write(&manifest_path, prost::Message::encode_to_vec(&manifest)).unwrap();
        let sig_path = dir.join(crate::MANIFEST_SIGNATURE_FILE);
        assert!(
            std::process::Command::new("gpg")
                .env("GNUPGHOME", gnupg)
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
                .unwrap()
                .status
                .success()
        );
        let zip_path = temp.path().join(format!("ext_{ver}.msxt"));
        zip_create_from_directory(&zip_path, &dir).unwrap();
        zip_path
    };

    let v1_msxt = build_signed_msxt("1.0.0", &gnupg1);
    handler
        .install_extension(crate::ExtensionInfo::LocalPath(v1_msxt))
        .await
        .unwrap();

    // Update signed by a DIFFERENT key must fail with UpdateSigningKeyMismatch
    let v2_diff_key = build_signed_msxt("2.0.0", &gnupg2);
    let err = handler
        .install_extension(crate::ExtensionInfo::LocalPath(v2_diff_key))
        .await
        .unwrap_err();
    assert!(matches!(
        err,
        crate::ExtensionError::UpdateSigningKeyMismatch(_)
    ));

    // Update signed by the SAME key must succeed
    let v2_same_key = build_signed_msxt("2.0.0", &gnupg1);
    assert!(
        handler
            .install_extension(crate::ExtensionInfo::LocalPath(v2_same_key))
            .await
            .is_ok()
    );
}
