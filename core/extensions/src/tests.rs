use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use crate::context::{MockExtism, ReplyHandler};
use crate::ext_runner::ExtensionHandlerInner;
use crate::models::SanitizeCommand;
use extensions_proto::moosync::types::{
    AddSongsRequest, ExtensionCommand, ExtensionCommandResponse, ExtensionsUpdatedResponse,
    GetProviderScopesRequest, GetProviderScopesResponse, MainCommand, MainCommandResponse,
    RunnerCommand, extension_command, extension_command_response, main_command,
    main_command_response, runner_command, runner_command_response,
};
use songs_proto::moosync::types::{InnerSong, Song};
use ui_proto::moosync::types::{PreferenceTypes, PreferenceUiData};

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

    let _reply_handler: ReplyHandler = Arc::new(Box::new(|_, _| {
        Ok(MainCommandResponse {
            response: Some(main_command_response::Response::ExtensionsUpdated(
                ExtensionsUpdatedResponse {},
            )),
        })
    }));

    let mut handler =
        ExtensionHandlerInner::new_with_context(extensions_path, Box::new(mock_extism));

    // 1. Check initially empty
    let req = RunnerCommand {
        // Use Default::default() for the Empty message
        command: Some(runner_command::Command::GetInstalledExtensions(
            Default::default(),
        )),
    };

    let resp = handler.handle_runner_command(req).unwrap();

    if let Some(runner_command_response::Response::GetInstalledExtensions(installed)) =
        resp.response
    {
        assert_eq!(installed.extensions.len(), 0);
    } else {
        panic!("Wrong response type");
    }

    // 2. Find new extensions
    let req_find = RunnerCommand {
        command: Some(runner_command::Command::FindNewExtensions(
            Default::default(),
        )),
    };
    handler.handle_runner_command(req_find).unwrap();

    // 3. Check again, should have 1
    let req_get = RunnerCommand {
        command: Some(runner_command::Command::GetInstalledExtensions(
            Default::default(),
        )),
    };

    let resp = handler.handle_runner_command(req_get).unwrap();

    if let Some(runner_command_response::Response::GetInstalledExtensions(installed)) =
        resp.response
    {
        let list = installed.extensions;
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].package_name, "test.pkg");
        assert_eq!(list[0].name, "Test Extension");
    } else {
        panic!("Wrong response type");
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
            if let Some(extension_command::Event::GetProviderScopes(_)) = cmd.event {
                Ok(ExtensionCommandResponse {
                    response: Some(extension_command_response::Response::GetProviderScopes(
                        GetProviderScopesResponse { scopes: vec![] },
                    )),
                })
            } else {
                panic!("Wrong command passed to extism");
            }
        });

    let mut handler =
        ExtensionHandlerInner::new_with_context(extensions_path, Box::new(mock_extism));

    let req = RunnerCommand {
        command: Some(runner_command::Command::FindNewExtensions(
            Default::default(),
        )),
    };
    handler.handle_runner_command(req).unwrap();

    let cmd = ExtensionCommand {
        package_name: "test.pkg".to_string(),
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

    let req_find = RunnerCommand {
        command: Some(runner_command::Command::FindNewExtensions(
            Default::default(),
        )),
    };
    handler.handle_runner_command(req_find).unwrap();

    let prefs = vec![PreferenceUiData {
        key: "pref1".to_string(),
        title: "Pref 1".to_string(),
        description: "Description".to_string(),
        r#type: PreferenceTypes::Extensions.into(),
        ..Default::default()
    }];

    handler
        .register_ui_preferences("test.pkg".to_string(), prefs)
        .unwrap();

    // Verify stored
    let req_get = RunnerCommand {
        command: Some(runner_command::Command::GetInstalledExtensions(
            Default::default(),
        )),
    };
    let resp = handler.handle_runner_command(req_get).unwrap();

    if let Some(runner_command_response::Response::GetInstalledExtensions(installed)) =
        resp.response
    {
        assert_eq!(installed.extensions[0].preferences.len(), 1);
        assert_eq!(installed.extensions[0].preferences[0].key, "pref1");
    } else {
        panic!("Wrong response type");
    }

    // Test unregister
    handler
        .unregister_ui_preferences("test.pkg".to_string(), vec!["pref1".to_string()])
        .unwrap();

    // Verify removed
    let req_get_again = RunnerCommand {
        command: Some(runner_command::Command::GetInstalledExtensions(
            Default::default(),
        )),
    };
    let resp = handler.handle_runner_command(req_get_again).unwrap();

    if let Some(runner_command_response::Response::GetInstalledExtensions(installed)) =
        resp.response
    {
        assert_eq!(installed.extensions[0].preferences.len(), 0);
    } else {
        panic!("Wrong response type");
    }
}
