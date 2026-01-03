use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use super::models::RunnerCommand;
use crate::context::{MockExtism, ReplyHandler};
use crate::ext_runner::ExtensionHandlerInner;
use crate::models::{ExtensionCommand, ExtensionCommandResponse, RunnerCommandResp};
use serde_json::Value;
use songs_proto::moosync::types::{InnerSong, Song};
use types::extensions::MainCommand;
use types::preferences::{PreferenceTypes, PreferenceUIData};
use types::ui::extensions::PackageNameArgs;

static INIT: std::sync::Once = std::sync::Once::new();

fn init_env() {
    INIT.call_once(|| unsafe {
        std::env::set_var("XDG_CACHE_HOME", std::env::temp_dir());
    });
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

    fn path(&self) -> &PathBuf {
        &self.path
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.path);
    }
}

#[test]
fn test_runner_command_try_from() {
    // Test FindNewExtensions
    let cmd = RunnerCommand::try_from(("findNewExtensions", &Value::Null));
    assert!(matches!(cmd.unwrap(), RunnerCommand::FindNewExtensions));

    // Test GetInstalledExtensions
    let cmd = RunnerCommand::try_from(("getInstalledExtensions", &Value::Null));
    assert!(matches!(
        cmd.unwrap(),
        RunnerCommand::GetInstalledExtensions
    ));

    // Test GetExtensionIcon
    let args = serde_json::to_value(PackageNameArgs {
        package_name: "test.pkg".to_string(),
    })
    .unwrap();
    let cmd = RunnerCommand::try_from(("getExtensionIcon", &args));
    if let RunnerCommand::GetExtensionIcon(pkg_args) = cmd.unwrap() {
        assert_eq!(pkg_args.package_name, "test.pkg");
    } else {
        panic!("Wrong command type");
    }

    // Invalid command
    let cmd = RunnerCommand::try_from(("invalidCommand", &Value::Null));
    assert!(cmd.is_err());
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

    let mut cmd = MainCommand::AddSongs(vec![song.clone()]);
    cmd.sanitize_command("test.pkg").unwrap();

    if let MainCommand::AddSongs(songs) = cmd {
        // Sanitization logic in extensions uses "test.pkg:" prefix
        // Let's verify what sanitize_song does. It usually prefixes the ID if present.
        assert_eq!(
            songs[0].song.as_ref().unwrap().id.as_ref().unwrap(),
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

    // Create a dummy package.json
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

    // Create dummy wasm file
    std::fs::write(ext_path.join("main.wasm"), b"dummy").unwrap();

    let mut mock_extism = MockExtism::new();

    // spawn_extension should be called
    mock_extism
        .expect_spawn_extension()
        .times(1)
        .returning(|_| {
            let wasm = extism::Wasm::data(
                // Minimal valid WASM module
                vec![0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00],
            );
            let manifest = extism::Manifest::new([wasm]);
            let plugin = extism::PluginBuilder::new(manifest).build().unwrap();
            Arc::new(Mutex::new(plugin))
        });

    let _reply_handler: ReplyHandler = Arc::new(Box::new(|_, _| {
        Ok(types::extensions::MainCommandResponse::ExtensionsUpdated(
            true,
        ))
    }));

    let mut handler =
        ExtensionHandlerInner::new_with_context(extensions_path, Box::new(mock_extism));

    // Initially no extensions
    if let RunnerCommandResp::ExtensionList(list) = handler
        .handle_runner_command(RunnerCommand::GetInstalledExtensions)
        .unwrap()
    {
        assert_eq!(list.len(), 0);
    } else {
        panic!("Wrong response");
    }

    // Find and spawn
    handler
        .handle_runner_command(RunnerCommand::FindNewExtensions)
        .unwrap();

    // Now should have 1 extension
    if let RunnerCommandResp::ExtensionList(list) = handler
        .handle_runner_command(RunnerCommand::GetInstalledExtensions)
        .unwrap()
    {
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].package_name, "test.pkg");
        assert_eq!(list[0].name, "Test Extension");
    } else {
        panic!("Wrong response");
    }
}

#[tokio::test]
async fn test_handle_extension_command() {
    init_env();
    let tmp_dir = TempDir::new();
    let extensions_path = tmp_dir.path().join("extensions");
    std::fs::create_dir_all(&extensions_path).unwrap();

    let ext_path = extensions_path.join("test_ext");
    std::fs::create_dir_all(&ext_path).unwrap();

    // Create a dummy package.json
    let manifest = r#"{
        "name": "test.pkg",
        "displayName": "Test Extension",
        "version": "1.0.0",
        "extensionEntry": "main.wasm",
        "moosyncExtension": true,
        "icon": "icon.png"
    }"#;
    std::fs::write(ext_path.join("package.json"), manifest).unwrap();
    std::fs::write(ext_path.join("main.wasm"), b"dummy").unwrap();

    let mut mock_extism = MockExtism::new();

    mock_extism
        .expect_spawn_extension()
        .times(1)
        .returning(|_| {
            let wasm = extism::Wasm::data(vec![0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00]);
            let manifest = extism::Manifest::new([wasm]);
            let plugin = extism::PluginBuilder::new(manifest).build().unwrap();
            Arc::new(Mutex::new(plugin))
        });

    mock_extism
        .expect_execute_command()
        .times(1)
        .returning(|_, _, cmd| {
            if let ExtensionCommand::GetProviderScopes(_) = cmd {
                Ok(ExtensionCommandResponse::GetProviderScopes(vec![]))
            } else {
                panic!("Wrong command passed to extism");
            }
        });

    let mut handler =
        ExtensionHandlerInner::new_with_context(extensions_path, Box::new(mock_extism));
    handler
        .handle_runner_command(RunnerCommand::FindNewExtensions)
        .unwrap();

    let cmd = ExtensionCommand::GetProviderScopes(PackageNameArgs {
        package_name: "test.pkg".to_string(),
    });

    let resp = handler.handle_extension_command(cmd).await.unwrap();
    assert!(matches!(
        resp,
        ExtensionCommandResponse::GetProviderScopes(_)
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
        "name": "test.pkg",
        "displayName": "Test Extension",
        "version": "1.0.0",
        "extensionEntry": "main.wasm",
        "moosyncExtension": true,
        "icon": "icon.png"
    }"#;
    std::fs::write(ext_path.join("package.json"), manifest).unwrap();
    std::fs::write(ext_path.join("main.wasm"), b"dummy").unwrap();

    let mut mock_extism = MockExtism::new();
    mock_extism
        .expect_spawn_extension()
        .times(1)
        .returning(|_| {
            let wasm = extism::Wasm::data(vec![0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00]);
            let manifest = extism::Manifest::new([wasm]);
            let plugin = extism::PluginBuilder::new(manifest).build().unwrap();
            Arc::new(Mutex::new(plugin))
        });

    let mut handler =
        ExtensionHandlerInner::new_with_context(extensions_path, Box::new(mock_extism));
    handler
        .handle_runner_command(RunnerCommand::FindNewExtensions)
        .unwrap();

    // Test register
    let prefs = vec![PreferenceUIData {
        key: "pref1".to_string(),
        title: "Pref 1".to_string(),
        description: "Description".to_string(),
        _type: PreferenceTypes::Extensions, // Using a valid type
        ..Default::default()
    }];

    handler
        .register_ui_preferences("test.pkg".to_string(), prefs)
        .unwrap();

    // Verify stored
    if let RunnerCommandResp::ExtensionList(list) = handler
        .handle_runner_command(RunnerCommand::GetInstalledExtensions)
        .unwrap()
    {
        assert_eq!(list[0].preferences.len(), 1);
        assert_eq!(list[0].preferences[0].key, "pref1");
    } else {
        panic!("Wrong response");
    }

    // Test unregister
    handler
        .unregister_ui_preferences("test.pkg".to_string(), vec!["pref1".to_string()])
        .unwrap();

    // Verify removed
    if let RunnerCommandResp::ExtensionList(list) = handler
        .handle_runner_command(RunnerCommand::GetInstalledExtensions)
        .unwrap()
    {
        assert_eq!(list[0].preferences.len(), 0);
    } else {
        panic!("Wrong response");
    }
}
