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

use std::{path::PathBuf, sync::Arc};

use assertables::{assert_is_empty, assert_len_eq_x};
use extensions_proto::moosync::types::GetProviderScopesRequest;
use songs_proto::moosync::types::{EntityResult, GetEntityOptions, GetSongOptions, Playlist, Song};
use tempdir::TempDir;
use tracing_test::traced_test;
use ui_proto::moosync::types::{PreferenceTypes, PreferenceUiData};

use crate::{
    ExtensionError, context::ReplyHandler, ext_runner::ExtensionHandlerInner,
    extension::ExtensionLockData,
};

static INIT: std::sync::Once = std::sync::Once::new();

#[tracing::instrument(level = "debug", skip_all)]
fn init_env() {
    INIT.call_once(|| unsafe {
        std::env::set_var("XDG_CACHE_HOME", std::env::temp_dir());
    });
}

#[tracing::instrument(level = "debug", skip_all)]
fn get_sample_wasm_path() -> PathBuf {
    if let Ok(runfiles_dir) = std::env::var("TEST_SRCDIR") {
        let candidates = [
            PathBuf::from(&runfiles_dir)
                .join("moosync_ext+/sample_extensions/rs/sample_extension.wasm"),
            PathBuf::from(&runfiles_dir)
                .join("moosync_ext/sample_extensions/rs/sample_extension.wasm"),
            PathBuf::from(&runfiles_dir).join("_main/sample_extensions/rs/sample_extension.wasm"),
        ];
        candidates
            .iter()
            .find(|p| p.exists())
            .cloned()
            .unwrap_or_else(|| candidates[0].clone())
    } else {
        panic!("TEST_SRCDIR not set or sample_extension.wasm not found in runfiles")
    }
}

struct TestReplyHandler;

impl ReplyHandler for TestReplyHandler {
    #[tracing::instrument(level = "debug", skip_all)]
    fn get_song(
        &self,
        _package_name: &str,
        _options: GetSongOptions,
    ) -> Result<Vec<Song>, ExtensionError> {
        Ok(vec![])
    }
    #[tracing::instrument(level = "debug", skip_all)]
    fn get_entity(
        &self,
        _package_name: &str,
        _options: GetEntityOptions,
    ) -> Result<EntityResult, ExtensionError> {
        Ok(EntityResult::default())
    }
    #[tracing::instrument(level = "debug", skip_all)]
    fn get_current_song(&self, _package_name: &str) -> Result<Option<Song>, ExtensionError> {
        Ok(None)
    }
    #[tracing::instrument(level = "debug", skip_all)]
    fn get_player_state(&self, _package_name: &str) -> Result<i32, ExtensionError> { Ok(0) }
    #[tracing::instrument(level = "debug", skip_all)]
    fn get_volume(&self, _package_name: &str) -> Result<f64, ExtensionError> { Ok(0.0) }
    #[tracing::instrument(level = "debug", skip_all)]
    fn get_time(&self, _package_name: &str) -> Result<f64, ExtensionError> { Ok(0.0) }
    #[tracing::instrument(level = "debug", skip_all)]
    fn get_queue(&self, _package_name: &str) -> Result<(Vec<Song>, usize), ExtensionError> {
        Ok((vec![], 0))
    }
    #[tracing::instrument(level = "debug", skip_all)]
    fn get_preference(
        &self,
        _package_name: &str,
        _key: &str,
    ) -> Result<Option<extensions_proto::struct_proto::google::protobuf::Value>, ExtensionError>
    {
        Ok(None)
    }
    #[tracing::instrument(level = "debug", skip_all)]
    fn set_preference(
        &self,
        _package_name: &str,
        _key: &str,
        _value: extensions_proto::struct_proto::google::protobuf::Value,
    ) -> Result<bool, ExtensionError> {
        Ok(true)
    }
    #[tracing::instrument(level = "debug", skip_all)]
    fn get_secure(
        &self,
        _package_name: &str,
        _key: &str,
    ) -> Result<Option<extensions_proto::struct_proto::google::protobuf::Value>, ExtensionError>
    {
        Ok(None)
    }
    #[tracing::instrument(level = "debug", skip_all)]
    fn set_secure(
        &self,
        _package_name: &str,
        _key: &str,
        _value: extensions_proto::struct_proto::google::protobuf::Value,
    ) -> Result<bool, ExtensionError> {
        Ok(true)
    }
    #[tracing::instrument(level = "debug", skip_all)]
    fn add_songs(
        &self,
        _package_name: &str,
        _songs: Vec<Song>,
    ) -> Result<Vec<Song>, ExtensionError> {
        Ok(vec![])
    }
    #[tracing::instrument(level = "debug", skip_all)]
    fn remove_song(&self, _package_name: &str, _song: Song) -> Result<bool, ExtensionError> {
        Ok(true)
    }
    #[tracing::instrument(level = "debug", skip_all)]
    fn update_song(&self, _package_name: &str, _song: Song) -> Result<Song, ExtensionError> {
        Ok(Song::default())
    }
    #[tracing::instrument(level = "debug", skip_all)]
    fn add_playlist(
        &self,
        _package_name: &str,
        _playlist: Playlist,
    ) -> Result<String, ExtensionError> {
        Ok("".to_string())
    }
    #[tracing::instrument(level = "debug", skip_all)]
    fn add_to_playlist(
        &self,
        _package_name: &str,
        _playlist_id: String,
        _songs: Vec<Song>,
    ) -> Result<bool, ExtensionError> {
        Ok(true)
    }
    #[tracing::instrument(level = "debug", skip_all)]
    fn register_oauth(&self, _package_name: &str, _url: String) -> Result<bool, ExtensionError> {
        Ok(true)
    }
    #[tracing::instrument(level = "debug", skip_all)]
    fn open_external_url(&self, _package_name: &str, _url: String) -> Result<bool, ExtensionError> {
        Ok(true)
    }
    #[tracing::instrument(level = "debug", skip_all)]
    fn update_accounts(
        &self,
        _package_name: &str,
        _account: Option<String>,
    ) -> Result<bool, ExtensionError> {
        Ok(true)
    }
    #[tracing::instrument(level = "debug", skip_all)]
    fn register_user_preference(
        &self,
        _package_name: &str,
        _prefs: Vec<PreferenceUiData>,
    ) -> Result<bool, ExtensionError> {
        Ok(true)
    }
    #[tracing::instrument(level = "debug", skip_all)]
    fn unregister_user_preference(
        &self,
        _package_name: &str,
        _keys: Vec<String>,
    ) -> Result<bool, ExtensionError> {
        Ok(true)
    }
    #[tracing::instrument(level = "debug", skip_all)]
    fn extensions_updated(&self, _package_name: &str) -> Result<(), ExtensionError> { Ok(()) }
    #[tracing::instrument(level = "debug", skip_all)]
    fn get_app_version(&self, _package_name: &str) -> Result<String, ExtensionError> {
        Ok("".to_string())
    }
}

#[test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_find_and_spawn_extensions() {
    init_env();
    let tmp_dir = TempDir::new("moosync_test_spawn_ext").unwrap();
    let extensions_path = tmp_dir.path().join("extensions");
    std::fs::create_dir_all(&extensions_path).unwrap();

    let ext_path = extensions_path.join("test_ext");
    std::fs::create_dir_all(&ext_path).unwrap();

    let manifest = r#"{
        "name": "test.pkg",
        "displayName": "Test Extension",
        "version": "1.0.0",
        "extensionEntry": "main.wasm",
        "moosyncExtension": true,
        "description": "Test",
        "icon": "icon.png",
        "author": "Author"
    }"#;
    std::fs::write(ext_path.join("package.json"), manifest).unwrap();

    std::fs::copy(get_sample_wasm_path(), ext_path.join("main.wasm")).unwrap();

    let reply_handler = Arc::new(TestReplyHandler);
    let handler = ExtensionHandlerInner::new(extensions_path, tmp_dir.path().join("cache"));

    assert_is_empty!(&handler.get_installed_extensions());

    handler.spawn_extensions(reply_handler);
    let installed = handler.get_installed_extensions();

    assert_len_eq_x!(&installed, 1);
    assert_eq!(installed[0].package_name, "test.pkg");
    assert_eq!(installed[0].name, "Test Extension");
}

#[tokio::test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_handle_extension_command() {
    init_env();
    let tmp_dir = TempDir::new("moosync_test_cmd_ext").unwrap();
    let extensions_path = tmp_dir.path().join("extensions");
    std::fs::create_dir_all(&extensions_path).unwrap();

    let ext_path = extensions_path.join("test_ext");
    std::fs::create_dir_all(&ext_path).unwrap();

    let manifest = r#"{
        "name": "sample.pkg",
        "displayName": "Test Extension",
        "version": "1.0.0",
        "extensionEntry": "main.wasm",
        "moosyncExtension": true,
        "icon": "icon.png"
    }"#;
    std::fs::write(ext_path.join("package.json"), manifest).unwrap();

    std::fs::copy(get_sample_wasm_path(), ext_path.join("main.wasm")).unwrap();

    let handler = ExtensionHandlerInner::new(extensions_path, tmp_dir.path().join("cache"));
    let reply_handler = Arc::new(TestReplyHandler);
    handler.spawn_extensions(reply_handler);

    std::thread::sleep(std::time::Duration::from_millis(500));

    let ext = {
        let map = handler.extensions_map.lock().unwrap();
        map.get("sample.pkg").unwrap().clone()
    };

    let res = ext
        .get_provider_scopes(GetProviderScopesRequest {})
        .await
        .unwrap();

    assert_eq!(res.scopes, vec![13, 0, 1, 2, 3, 4]);
}

#[test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_register_unregister_ui_preferences() {
    init_env();
    let tmp_dir = TempDir::new("moosync_test_ui_prefs").unwrap();
    let extensions_path = tmp_dir.path().join("extensions");
    std::fs::create_dir_all(&extensions_path).unwrap();

    let ext_path = extensions_path.join("test_ext");
    std::fs::create_dir_all(&ext_path).unwrap();

    let manifest = r#"{
        "name": "sample.pkg",
        "displayName": "Test Extension",
        "version": "1.0.0",
        "extensionEntry": "main.wasm",
        "moosyncExtension": true,
        "icon": "icon.png"
    }"#;
    std::fs::write(ext_path.join("package.json"), manifest).unwrap();

    std::fs::copy(get_sample_wasm_path(), ext_path.join("main.wasm")).unwrap();

    let handler = ExtensionHandlerInner::new(extensions_path, tmp_dir.path().join("cache"));
    let reply_handler = Arc::new(TestReplyHandler);
    handler.spawn_extensions(reply_handler);

    let prefs = vec![PreferenceUiData {
        key: "pref1".to_string(),
        title: "Pref 1".to_string(),
        description: "Description".to_string(),
        r#type: PreferenceTypes::Extensions.into(),
        ..Default::default()
    }];

    let ext = handler.get_extension("sample.pkg").unwrap();
    ext.register_ui_preferences(prefs);

    let installed = handler.get_installed_extensions();
    assert_len_eq_x!(&installed[0].preferences, 1);
    assert_eq!(installed[0].preferences[0].key, "pref1");

    ext.unregister_ui_preferences(vec!["pref1".to_string()]);
    let installed_after = handler.get_installed_extensions();
    assert_is_empty!(&installed_after[0].preferences);
}

#[test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_extension_failed_to_start_disables_extension() {
    init_env();
    let tmp_dir = TempDir::new("moosync_test_fail_ext").unwrap();
    let extensions_path = tmp_dir.path().join("extensions");
    std::fs::create_dir_all(&extensions_path).unwrap();

    let ext_path = extensions_path.join("fail.pkg");
    std::fs::create_dir_all(&ext_path).unwrap();

    let manifest = r#"{
        "name": "fail.pkg",
        "displayName": "Fail Extension",
        "version": "1.0.0",
        "extensionEntry": "main.wasm",
        "moosyncExtension": true,
        "icon": "icon.png"
    }"#;
    std::fs::write(ext_path.join("package.json"), manifest).unwrap();
    let empty_wasm = vec![0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00];
    std::fs::write(ext_path.join("main.wasm"), empty_wasm).unwrap();

    let reply_handler = Arc::new(TestReplyHandler);
    let handler = ExtensionHandlerInner::new(extensions_path.clone(), tmp_dir.path().join("cache"));
    handler.spawn_extensions(reply_handler);

    let lock_file = ext_path.join("extension.lock");
    for _ in 0..50 {
        if lock_file.exists()
            && let Ok(bytes) = std::fs::read(&lock_file)
            && let Ok(data) = serde_json::from_slice::<ExtensionLockData>(&bytes)
            && data.disabled
        {
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(100));
    }

    assert!(lock_file.exists());
    let installed = handler.get_installed_extensions();
    assert_len_eq_x!(&installed, 1);
    assert_eq!(installed[0].package_name, "fail.pkg");
    assert!(!installed[0].active);
}

#[test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_extension_activation_deactivation() {
    init_env();
    let tmp_dir = TempDir::new("moosync_test_act_ext").unwrap();
    let extensions_path = tmp_dir.path().join("extensions");
    std::fs::create_dir_all(&extensions_path).unwrap();

    let ext_path = extensions_path.join("test_pkg");
    std::fs::create_dir_all(&ext_path).unwrap();

    let manifest = r#"{
        "name": "test_pkg",
        "displayName": "Test Extension",
        "version": "1.0.0",
        "extensionEntry": "main.wasm",
        "moosyncExtension": true,
        "icon": "icon.png"
    }"#;
    std::fs::write(ext_path.join("package.json"), manifest).unwrap();

    std::fs::copy(get_sample_wasm_path(), ext_path.join("main.wasm")).unwrap();

    let lock_file = ext_path.join("extension.lock");
    let lock_data = ExtensionLockData {
        registry: "local".to_string(),
        disabled: true,
    };
    std::fs::write(&lock_file, serde_json::to_vec_pretty(&lock_data).unwrap()).unwrap();

    let reply_handler = Arc::new(TestReplyHandler);
    let handler = ExtensionHandlerInner::new(extensions_path.clone(), tmp_dir.path().join("cache"));
    handler.spawn_extensions(reply_handler.clone());

    {
        let extensions_map = handler.extensions_map.lock().unwrap();
        let ext = extensions_map.get("test_pkg").unwrap();
        assert!(!ext.is_active());
        assert!(!ext.get_extension_detail().has_started);
    }
    assert!(lock_file.exists());

    {
        let extensions_map = handler.extensions_map.lock().unwrap();
        let ext = extensions_map.get("test_pkg").unwrap();
        ext.set_active(true).unwrap();
    }

    for _ in 0..50 {
        let extensions_map = handler.extensions_map.lock().unwrap();
        if let Some(ext) = extensions_map.get("test_pkg")
            && ext.get_extension_detail().has_started
        {
            break;
        }
        drop(extensions_map);
        std::thread::sleep(std::time::Duration::from_millis(100));
    }

    {
        let extensions_map = handler.extensions_map.lock().unwrap();
        let ext = extensions_map.get("test_pkg").unwrap();
        assert!(ext.is_active());
        assert!(ext.get_extension_detail().has_started);
    }

    {
        let extensions_map = handler.extensions_map.lock().unwrap();
        let ext = extensions_map.get("test_pkg").unwrap();
        ext.set_active(false).unwrap();
    }

    {
        let extensions_map = handler.extensions_map.lock().unwrap();
        let ext = extensions_map.get("test_pkg").unwrap();
        assert!(!ext.is_active());
        assert!(!ext.get_extension_detail().has_started);
    }
}
