// Moosync
// Copyright (C) 2024, 2025  Moosync <support@moosync.app>
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <http://www.gnu.org/licenses/>.

use std::{fs, sync::Arc};

use assertables::assert_is_empty;
use extensions_proto::moosync::types::ExtensionAccountDetail;
use preferences_proto::moosync::types::PreferenceItem;
use rstest::{fixture, rstest};
use songs_proto::moosync::types::{EntityResult, GetEntityOptions, GetSongOptions, Song};
use tempdir::TempDir;
use tracing_test::traced_test;

use crate::{ExtensionHandler, ReplyHandler, errors::ExtensionError};

struct DummyReply;
impl ReplyHandler for DummyReply {
    #[tracing::instrument(level = "debug", skip_all)]
    fn get_song(&self, _: &str, _: GetSongOptions) -> Result<Vec<Song>, ExtensionError> {
        Ok(vec![])
    }
    #[tracing::instrument(level = "debug", skip_all)]
    fn get_entity(&self, _: &str, _: GetEntityOptions) -> Result<EntityResult, ExtensionError> {
        Ok(EntityResult::default())
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
    fn add_playlist(
        &self,
        _: &str,
        _: songs_proto::moosync::types::Playlist,
    ) -> Result<String, ExtensionError> {
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
    let ext_dir = handler_context.handler.extensions_dir.join("test_ext");
    fs::create_dir_all(&ext_dir).unwrap();
    let manifest = extensions_proto::moosync::types::ExtensionManifest {
        name: "test_ext".to_string(),
        display_name: "Test Extension".to_string(),
        version: "1.0.0".to_string(),
        extension_entry: "main.wasm".to_string(),
        icon: "icon.png".to_string(),
        author: Some("Tester".to_string()),
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
    handler_context.handler.find_new_extensions().unwrap();

    let remote_manifests = vec![extensions_proto::moosync::types::ExtensionManifest {
        moosync_extension: false,
        display_name: "Test Extension".to_string(),
        extension_entry: "".to_string(),
        author: None,
        name: "test_ext".to_string(),
        version: "1.1.0".to_string(),
        icon: "".to_string(),
        permissions: None,
        description: None,
        url: Some("https://example.com/test_ext.msxt".to_string()),
        registry: Some("default".to_string()),
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
    assert_eq!(updatables[0].name, "test_ext");
    assert_eq!(updatables[0].version, "1.1.0");
}

#[rstest]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_extension_handler_check_for_updates_older_remote(handler_context: TestHandlerContext) {
    let ext_dir = handler_context.handler.extensions_dir.join("test_ext");
    fs::create_dir_all(&ext_dir).unwrap();
    let manifest = extensions_proto::moosync::types::ExtensionManifest {
        name: "test_ext".to_string(),
        display_name: "Test Extension".to_string(),
        version: "1.2.0".to_string(),
        extension_entry: "main.wasm".to_string(),
        icon: "icon.png".to_string(),
        author: Some("Tester".to_string()),
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
    handler_context.handler.find_new_extensions().unwrap();

    let remote_manifests = vec![extensions_proto::moosync::types::ExtensionManifest {
        moosync_extension: false,
        display_name: "Test Extension".to_string(),
        extension_entry: "".to_string(),
        author: None,
        name: "test_ext".to_string(),
        version: "1.1.0".to_string(),
        icon: "".to_string(),
        permissions: None,
        description: None,
        url: Some("https://example.com/test_ext.msxt".to_string()),
        registry: Some("default".to_string()),
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
    let ext_dir = handler_context.handler.extensions_dir.join("test_ext");
    fs::create_dir_all(&ext_dir).unwrap();
    let manifest = extensions_proto::moosync::types::ExtensionManifest {
        name: "test_ext".to_string(),
        display_name: "Test Extension".to_string(),
        version: "1.0.0".to_string(),
        extension_entry: "main.wasm".to_string(),
        icon: "icon.png".to_string(),
        author: Some("Tester".to_string()),
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
    handler_context.handler.find_new_extensions().unwrap();

    let remote_manifests = vec![extensions_proto::moosync::types::ExtensionManifest {
        moosync_extension: false,
        display_name: "Test Extension".to_string(),
        extension_entry: "".to_string(),
        author: None,
        name: "test_ext".to_string(),
        version: "1.1.0".to_string(),
        icon: "".to_string(),
        permissions: None,
        description: None,
        url: Some("http://127.0.0.1:9/non_existent.msxt".to_string()),
        registry: Some("default".to_string()),
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
            name: "test.pkg".to_string(),
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
        name: "active_ext".to_string(),
        display_name: "Active Extension".to_string(),
        version: "1.0.0".to_string(),
        extension_entry: "main.wasm".to_string(),
        scopes: vec![extensions_proto::moosync::types::ExtensionProviderScope::Search as i32],
        ..Default::default()
    };
    fs::write(
        ext_dir.join("manifest.pb"),
        prost::Message::encode_to_vec(&manifest),
    )
    .unwrap();
    fs::write(ext_dir.join("main.wasm"), b"wasm bytes").unwrap();

    let handler = &handler_context.handler;
    handler.find_new_extensions().unwrap();

    let ext = handler.get_extension("active_ext");
    let ext_mut = handler.get_extension_mut("active_ext");
    assert!(ext.is_ok());
    assert!(ext_mut.is_ok());

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
    assert!(handler.get_extension("active_ext").is_err());

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
        name: "zip_ext".to_string(),
        display_name: "Zip Extension".to_string(),
        version: "2.3.4".to_string(),
        extension_entry: "main.wasm".to_string(),
        ..Default::default()
    };
    fs::write(
        valid_dir.join("manifest.pb"),
        prost::Message::encode_to_vec(&manifest),
    )
    .unwrap();
    fs::write(valid_dir.join("main.wasm"), b"wasm bytes").unwrap();

    let ver = handler.get_extension_version(valid_dir.clone());
    assert_eq!(ver.unwrap(), "2.3.4");

    let empty_dir = temp_dir.path().join("empty");
    fs::create_dir_all(&empty_dir).unwrap();
    let no_ver = handler.get_extension_version(empty_dir);
    assert!(no_ver.is_err());

    let local_install = handler
        .install_extension(crate::ExtensionInfo::Local(
            extensions_proto::moosync::types::ExtensionDetail::default(),
        ))
        .await;
    assert!(local_install.is_ok());

    let zip_file = temp_dir.path().join("zip_ext.msxt");
    zip_extensions::zip_create_from_directory(&zip_file, &valid_dir).unwrap();

    let zip_install = handler
        .install_extension(crate::ExtensionInfo::LocalPath(zip_file))
        .await;
    assert!(zip_install.is_ok());

    let installed_ext = handler.get_extension("zip_ext");
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
            name: "uninstalled_ext".to_string(),
            version: "1.0.0".to_string(),
            ..Default::default()
        },
        extensions_proto::moosync::types::ExtensionManifest {
            name: "bad_remote_ver".to_string(),
            version: "invalid-semver".to_string(),
            ..Default::default()
        },
    ];
    handler.set_remote_manifests(remote_manifests);
    let has_updates = handler.check_for_updates();
    assert!(!has_updates);
}
