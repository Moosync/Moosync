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

use extensions_proto::moosync::types::{
    AddSongsRequest, ExtensionCommand, GetProviderScopesRequest, MainCommand, extension_command,
    extension_command_response, main_command,
};
use songs_proto::moosync::types::{InnerSong, Song};
use ui_proto::moosync::types::{PreferenceTypes, PreferenceUiData};

use crate::{context::ReplyHandler, ext_runner::ExtensionHandlerInner, models::SanitizeCommand};

static INIT: std::sync::Once = std::sync::Once::new();

fn init_env() {
    INIT.call_once(|| unsafe {
        std::env::set_var("XDG_CACHE_HOME", std::env::temp_dir());
    });
}

fn get_sample_wasm_path() -> PathBuf {
    if let Ok(runfiles_dir) = std::env::var("TEST_SRCDIR") {
        let workspace_name =
            std::env::var("TEST_WORKSPACE").unwrap_or_else(|_| "moosync".to_string());
        PathBuf::from(runfiles_dir)
            .join(workspace_name)
            .join("core/extensions/tests/fixtures/sample_extension.wasm")
    } else {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests")
            .join("fixtures")
            .join("sample_extension.wasm")
    }
}

struct TempDir {
    path: PathBuf,
}

impl TempDir {
    fn new() -> Self {
        let mut path = std::env::temp_dir();
        path.push(uuid::Uuid::new_v4().to_string());
        std::fs::create_dir_all(&path).unwrap();
        Self { path }
    }

    fn path(&self) -> &PathBuf { &self.path }
}

impl Drop for TempDir {
    fn drop(&mut self) { let _ = std::fs::remove_dir_all(&self.path); }
}

struct TestReplyHandler;

impl ReplyHandler for TestReplyHandler {
    fn extensions_updated(&self, _package_name: &str) -> Result<(), types::errors::MoosyncError> {
        Ok(())
    }
}

#[test]
fn test_main_command_sanitize() {
    let song = Song {
        song: Some(InnerSong {
            id: Some("123".to_string()),
            path: Some("/path".to_string()),
            ..Default::default()
        }),
        ..Default::default()
    };

    let mut cmd = MainCommand {
        command: Some(main_command::Command::AddSongs(AddSongsRequest {
            songs: vec![song.clone()],
        })),
    };

    cmd.sanitize("test.pkg").unwrap();

    if let Some(main_command::Command::AddSongs(req)) = cmd.command {
        assert_eq!(
            req.songs[0].song.clone().unwrap().id.as_ref().unwrap(),
            "test.pkg:123"
        );
    } else {
        panic!("Wrong command type");
    }
}

#[test]
fn test_find_and_spawn_extensions() {
    init_env();
    let tmp_dir = TempDir::new();
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

    // Copy valid sample WASM fixture to temporary directory
    std::fs::copy(get_sample_wasm_path(), ext_path.join("main.wasm")).unwrap();

    let reply_handler = Arc::new(TestReplyHandler);

    let mut handler = ExtensionHandlerInner::new(extensions_path, tmp_dir.path().join("cache"));

    let installed = handler.get_installed_extensions();
    assert_eq!(installed.len(), 0);

    handler.spawn_extensions(reply_handler);

    let installed = handler.get_installed_extensions();
    assert_eq!(installed.len(), 1);
    assert_eq!(installed[0].package_name, "test.pkg");
    assert_eq!(installed[0].name, "Test Extension");
}

#[tokio::test]
async fn test_handle_extension_command() {
    init_env();
    let tmp_dir = TempDir::new();
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

    // Copy valid sample WASM fixture to temporary directory
    std::fs::copy(get_sample_wasm_path(), ext_path.join("main.wasm")).unwrap();

    let mut handler = ExtensionHandlerInner::new(extensions_path, tmp_dir.path().join("cache"));

    let reply_handler = Arc::new(TestReplyHandler);
    handler.spawn_extensions(reply_handler);

    // Since the spawn_extension runs in a background thread, we wait/sleep a bit
    // for it to start
    std::thread::sleep(std::time::Duration::from_millis(500));

    let cmd = ExtensionCommand {
        package_name: "sample.pkg".to_string(),
        event: Some(extension_command::Event::GetProviderScopes(
            GetProviderScopesRequest {},
        )),
    };

    let resp = handler.handle_extension_command(cmd).await.unwrap();

    assert!(matches!(
        resp.unwrap().response,
        Some(extension_command_response::Response::GetProviderScopes(_))
    ));
}

#[test]
fn test_register_unregister_ui_preferences() {
    init_env();
    let tmp_dir = TempDir::new();
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

    // Copy valid sample WASM fixture to temporary directory
    std::fs::copy(get_sample_wasm_path(), ext_path.join("main.wasm")).unwrap();

    let mut handler = ExtensionHandlerInner::new(extensions_path, tmp_dir.path().join("cache"));

    let reply_handler = Arc::new(TestReplyHandler);
    handler.spawn_extensions(reply_handler);

    let prefs = vec![PreferenceUiData {
        key: "pref1".to_string(),
        title: "Pref 1".to_string(),
        description: "Description".to_string(),
        r#type: PreferenceTypes::Extensions.into(),
        ..Default::default()
    }];

    handler
        .register_ui_preferences("sample.pkg".to_string(), prefs)
        .unwrap();

    // Verify stored
    let installed = handler.get_installed_extensions();
    assert_eq!(installed[0].preferences.len(), 1);
    assert_eq!(installed[0].preferences[0].key, "pref1");

    // Test unregister
    handler
        .unregister_ui_preferences("sample.pkg".to_string(), vec!["pref1".to_string()])
        .unwrap();

    // Verify removed
    let installed = handler.get_installed_extensions();
    assert_eq!(installed[0].preferences.len(), 0);
}

#[test]
fn test_extension_failed_to_start_disables_extension() {
    init_env();
    let tmp_dir = TempDir::new();
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
    // Write valid empty WASM module header
    let empty_wasm = vec![0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00];
    std::fs::write(ext_path.join("main.wasm"), empty_wasm).unwrap();

    let reply_handler = Arc::new(TestReplyHandler);

    let mut handler =
        ExtensionHandlerInner::new(extensions_path.clone(), tmp_dir.path().join("cache"));

    // Find and spawn extensions
    handler.spawn_extensions(reply_handler);

    // Since the spawn_extension runs in a background thread, we wait/sleep a bit
    // for it to run and fail
    std::thread::sleep(std::time::Duration::from_millis(500));

    // Verify that the .disabled file has been created in the extension's folder
    let disabled_file = ext_path.join(".disabled");
    assert!(disabled_file.exists());

    // Verify that GetInstalledExtensions returns the extension as active: false
    let installed = handler.get_installed_extensions();
    assert_eq!(installed.len(), 1);
    assert_eq!(installed[0].package_name, "fail.pkg");
    assert_eq!(installed[0].active, false);
}

#[test]
fn test_extension_activation_deactivation() {
    let tmp_dir = TempDir::new();
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

    // Write valid empty WASM module header
    let empty_wasm = vec![0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00];
    std::fs::write(ext_path.join("main.wasm"), empty_wasm).unwrap();

    // Create a disabled file initially
    let disabled_file = ext_path.join(".disabled");
    std::fs::write(&disabled_file, "").unwrap();

    let reply_handler = Arc::new(TestReplyHandler);

    let mut handler =
        ExtensionHandlerInner::new(extensions_path.clone(), tmp_dir.path().join("cache"));

    handler.spawn_extensions(reply_handler.clone());

    {
        let extensions_map = handler.extensions_map.lock().unwrap();
        let ext = extensions_map.get("test_pkg").unwrap();
        assert!(!ext.active);
        assert!(ext.context.is_none());
    }
    assert!(disabled_file.exists());

    handler
        .set_extension_active("test_pkg", true, reply_handler.clone())
        .unwrap();

    {
        let extensions_map = handler.extensions_map.lock().unwrap();
        let ext = extensions_map.get("test_pkg").unwrap();
        assert!(ext.active);
        assert!(ext.context.is_some());
    }
    assert!(!disabled_file.exists());

    handler
        .set_extension_active("test_pkg", false, reply_handler.clone())
        .unwrap();

    {
        let extensions_map = handler.extensions_map.lock().unwrap();
        let ext = extensions_map.get("test_pkg").unwrap();
        assert!(!ext.active);
        assert!(ext.context.is_none());
    }
    assert!(disabled_file.exists());
}
