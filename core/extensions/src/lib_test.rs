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
use rstest::{fixture, rstest};
use songs_proto::moosync::types::{EntityResult, GetEntityOptions, GetSongOptions, Song};
use tempdir::TempDir;
use tracing_test::traced_test;
use ui_proto::moosync::types::PreferenceUiData;

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
    fn update_accounts(&self, _: &str, _: Option<String>) -> Result<bool, ExtensionError> {
        Ok(true)
    }
    #[tracing::instrument(level = "debug", skip_all)]
    fn register_user_preference(
        &self,
        _: &str,
        _: Vec<PreferenceUiData>,
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

    let mut handler = ExtensionHandler::new(ext_dir, tmp_dir, cache_dir);
    handler.set_reply_handler(Arc::new(DummyReply));
    TestHandlerContext {
        _temp_dir: temp_dir,
        handler,
    }
}

#[rstest]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_extension_handler_init_and_get_all(handler_context: TestHandlerContext) {
    let TestHandlerContext { handler, .. } = handler_context;

    let all = handler.get_all_extensions();

    assert_is_empty!(&all);
    assert!(!handler.has_updates());
}

#[rstest]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_extension_handler_check_for_updates(mut handler_context: TestHandlerContext) {
    let ext_dir = handler_context.handler.extensions_dir.join("test_ext");
    fs::create_dir_all(&ext_dir).unwrap();
    let pkg_json = r#"{
        "name": "test_ext",
        "displayName": "Test Extension",
        "version": "1.0.0",
        "extensionEntry": "main.wasm",
        "icon": "icon.png",
        "author": "Tester",
        "moosyncExtension": true
    }"#;
    fs::write(ext_dir.join("package.json"), pkg_json).unwrap();
    let lock_data = serde_json::json!({
        "registry": "local",
        "disabled": true
    });
    fs::write(
        ext_dir.join("extension.lock"),
        serde_json::to_vec(&lock_data).unwrap(),
    )
    .unwrap();
    handler_context.handler.find_new_extensions().unwrap();

    let mut remote_manifests = std::collections::HashSet::new();
    remote_manifests.insert(extensions_proto::moosync::types::FetchedExtensionManifest {
        name: "Test Extension".to_string(),
        package_name: "test_ext".to_string(),
        version: "1.1.0".to_string(),
        url: "https://example.com/test_ext.msxt".to_string(),
        logo: None,
        description: None,
        registry: Some("default".to_string()),
    });
    handler_context
        .handler
        .set_remote_manifests(remote_manifests);

    let has_updates = handler_context.handler.check_for_updates();
    let updatables = handler_context.handler.get_updatable_extensions();

    assert!(has_updates);
    assert!(handler_context.handler.has_updates());
    assert_eq!(updatables.len(), 1);
    assert_eq!(updatables[0].package_name, "test_ext");
    assert_eq!(updatables[0].version, "1.1.0");
}

#[rstest]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_extension_handler_check_for_updates_older_remote(mut handler_context: TestHandlerContext) {
    let ext_dir = handler_context.handler.extensions_dir.join("test_ext");
    fs::create_dir_all(&ext_dir).unwrap();
    let pkg_json = r#"{
        "name": "test_ext",
        "displayName": "Test Extension",
        "version": "1.2.0",
        "extensionEntry": "main.wasm",
        "icon": "icon.png",
        "author": "Tester",
        "moosyncExtension": true
    }"#;
    fs::write(ext_dir.join("package.json"), pkg_json).unwrap();
    let lock_data = serde_json::json!({
        "registry": "local",
        "disabled": true
    });
    fs::write(
        ext_dir.join("extension.lock"),
        serde_json::to_vec(&lock_data).unwrap(),
    )
    .unwrap();
    handler_context.handler.find_new_extensions().unwrap();

    let mut remote_manifests = std::collections::HashSet::new();
    remote_manifests.insert(extensions_proto::moosync::types::FetchedExtensionManifest {
        name: "Test Extension".to_string(),
        package_name: "test_ext".to_string(),
        version: "1.1.0".to_string(),
        url: "https://example.com/test_ext.msxt".to_string(),
        logo: None,
        description: None,
        registry: Some("default".to_string()),
    });
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
    mut handler_context: TestHandlerContext,
) {
    let res = handler_context.handler.update_all_extensions().await;
    assert!(res.is_ok());
    assert!(!handler_context.handler.has_updates());
}

#[rstest]
#[tokio::test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_extension_handler_update_all_extensions_failed(
    mut handler_context: TestHandlerContext,
) {
    let ext_dir = handler_context.handler.extensions_dir.join("test_ext");
    fs::create_dir_all(&ext_dir).unwrap();
    let pkg_json = r#"{
        "name": "test_ext",
        "displayName": "Test Extension",
        "version": "1.0.0",
        "extensionEntry": "main.wasm",
        "icon": "icon.png",
        "author": "Tester",
        "moosyncExtension": true
    }"#;
    fs::write(ext_dir.join("package.json"), pkg_json).unwrap();
    let lock_data = serde_json::json!({
        "registry": "local",
        "disabled": true
    });
    fs::write(
        ext_dir.join("extension.lock"),
        serde_json::to_vec(&lock_data).unwrap(),
    )
    .unwrap();
    handler_context.handler.find_new_extensions().unwrap();

    let mut remote_manifests = std::collections::HashSet::new();
    remote_manifests.insert(extensions_proto::moosync::types::FetchedExtensionManifest {
        name: "Test Extension".to_string(),
        package_name: "test_ext".to_string(),
        version: "1.1.0".to_string(),
        url: "http://127.0.0.1:9/non_existent.msxt".to_string(),
        logo: None,
        description: None,
        registry: Some("default".to_string()),
    });
    handler_context
        .handler
        .set_remote_manifests(remote_manifests);
    handler_context.handler.check_for_updates();
    assert!(handler_context.handler.has_updates());

    let res = handler_context.handler.update_all_extensions().await;
    assert!(res.is_err());
    assert!(handler_context.handler.has_updates());
}
