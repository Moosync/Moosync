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

use std::{
    fs,
    path::PathBuf,
    sync::{Arc, atomic::AtomicBool},
};

use assertables::{assert_is_empty, assert_len_eq_x};
use rstest::{fixture, rstest};
use songs_proto::moosync::types::{EntityResult, GetEntityOptions, GetSongOptions, Song};
use tempdir::TempDir;
use tracing_test::traced_test;
use ui_proto::moosync::types::PreferenceUiData;

use crate::{ReplyHandler, errors::ExtensionError, extension::Extension};

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

struct TestExtContext {
    pub _temp_dir: TempDir,
    pub manifest_path: PathBuf,
    pub cache_dir: PathBuf,
}

#[fixture]
#[tracing::instrument(level = "debug", skip_all)]
fn ext_context() -> TestExtContext {
    let temp_dir = TempDir::new("moosync_ext_unit").expect("failed to create temp dir");
    let manifest_json = r#"{
        "name": "unit.pkg",
        "displayName": "Unit Test Extension",
        "version": "1.0.0",
        "extensionEntry": "main.wasm",
        "moosyncExtension": true,
        "icon": "icon.png",
        "author": "Tester"
    }"#;
    let manifest_path = temp_dir.path().join("package.json");
    fs::write(&manifest_path, manifest_json).unwrap();
    let lock_data = serde_json::json!({
        "registry": "local",
        "disabled": true
    });
    fs::write(
        temp_dir.path().join("extension.lock"),
        serde_json::to_vec(&lock_data).unwrap(),
    )
    .unwrap();
    let cache_dir = temp_dir.path().join("cache");

    TestExtContext {
        _temp_dir: temp_dir,
        manifest_path,
        cache_dir,
    }
}

#[rstest]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_extension_preferences_and_active_state(ext_context: TestExtContext) {
    let TestExtContext {
        manifest_path,
        cache_dir,
        ..
    } = ext_context;
    let reply = Arc::new(DummyReply);
    let has_started = Arc::new(AtomicBool::new(false));

    let ext = Extension::new(&manifest_path, reply, cache_dir, has_started);
    assert!(ext.is_ok());
    let ext = ext.unwrap();

    assert_eq!(ext.get_package_name(), "unit.pkg");
    assert!(!ext.is_active());
    assert_eq!(ext.get_lock_data().registry, "local");

    ext.register_ui_preferences(vec![PreferenceUiData {
        key: "volume".to_string(),
        title: "Default Volume".to_string(),
        ..Default::default()
    }]);
    let details = ext.get_extension_detail();
    assert_len_eq_x!(&details.preferences, 1);
    assert_eq!(details.preferences[0].key, "volume");

    ext.unregister_ui_preferences(vec!["volume".to_string()]);
    let details_after = ext.get_extension_detail();
    assert_is_empty!(&details_after.preferences);
}
