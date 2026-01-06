use crate::context::{ExtismContext, ReplyHandler};
use crate::ext_runner::ExtensionHandlerInner;
use extensions_proto::moosync::types::{
    ExtensionCommand, ExtensionsUpdatedResponse, GetProviderScopesRequest, MainCommand,
    MainCommandResponse, RunnerCommand, extension_command, extension_command_response,
    main_command, main_command_response, runner_command, runner_command_response,
};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

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

async fn setup_extension() -> (ExtensionHandlerInner, Arc<Mutex<Vec<MainCommand>>>, TempDir) {
    let tmp_dir = TempDir::new();
    // Cache path remains in temp
    let cache_path = tmp_dir.path().join("cache");

    let runfiles_dir = std::env::var("TEST_SRCDIR").unwrap_or_else(|_| ".".to_string());
    let workspace_name = std::env::var("TEST_WORKSPACE").unwrap_or_else(|_| "moosync".to_string());

    let extensions_path = PathBuf::from(runfiles_dir)
        .join(workspace_name)
        .join("core/extensions/tests/fixtures");

    if !extensions_path.exists() {
        panic!("Extensions path not found: {:?}", extensions_path);
    }

    let captured_commands = Arc::new(Mutex::new(Vec::<MainCommand>::new()));
    let captured_commands_clone = captured_commands.clone();

    let reply_handler: ReplyHandler = Arc::new(Box::new(move |_, cmd| {
        let mut cmds = captured_commands_clone.lock().unwrap();
        cmds.push(cmd);
        Ok(MainCommandResponse {
            response: Some(main_command_response::Response::ExtensionsUpdated(
                ExtensionsUpdatedResponse {},
            )),
        })
    }));

    let extism_context = ExtismContext::new(cache_path, reply_handler);
    let mut handler =
        ExtensionHandlerInner::new_with_context(extensions_path.clone(), Box::new(extism_context));

    let req_find = RunnerCommand {
        command: Some(runner_command::Command::FindNewExtensions(
            Default::default(),
        )),
    };
    handler.handle_runner_command(req_find).unwrap();

    // Verify extension is loaded
    let req_list = RunnerCommand {
        command: Some(runner_command::Command::GetInstalledExtensions(
            Default::default(),
        )),
    };
    let resp = handler.handle_runner_command(req_list).unwrap();
    if let Some(runner_command_response::Response::GetInstalledExtensions(list)) = resp.response {
        if list.extensions.is_empty() {
            panic!(
                "Setup failed: No extensions loaded from {:?}",
                extensions_path
            );
        }
        if !list
            .extensions
            .iter()
            .any(|e| e.package_name == "sample.pkg")
        {
            panic!(
                "Setup failed: sample.pkg not found in {:?}",
                list.extensions
            );
        }
    }

    // Wait for ExtensionsUpdated command to settle
    let start = std::time::Instant::now();
    loop {
        {
            let cmds = captured_commands.lock().unwrap();
            if cmds
                .iter()
                .any(|c| matches!(c.command, Some(main_command::Command::ExtensionsUpdated(_))))
            {
                break;
            }
        }
        if start.elapsed() > std::time::Duration::from_secs(2) {
            //println!("DEBUG: Timeout waiting for ExtensionsUpdated in setup");
            break;
        }
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;
    }

    captured_commands.lock().unwrap().clear();

    (handler, captured_commands, tmp_dir)
}

#[tokio::test]
async fn test_get_provider_scopes() {
    let (mut handler, _, _tmp) = setup_extension().await;

    let cmd = ExtensionCommand {
        package_name: "sample.pkg".to_string(),
        event: Some(extension_command::Event::GetProviderScopes(
            GetProviderScopesRequest {},
        )),
    };

    let resp = handler.handle_extension_command(cmd).await.unwrap();
    if let Some(extension_command_response::Response::GetProviderScopes(res)) =
        resp.unwrap().response
    {
        assert_eq!(res.scopes, vec![13]); // Accounts = 13
    } else {
        panic!("Wrong response for GetProviderScopes");
    }
}

#[tokio::test]
async fn test_get_accounts() {
    let (mut handler, captured_commands, _tmp) = setup_extension().await;

    let cmd = ExtensionCommand {
        package_name: "sample.pkg".to_string(),
        event: Some(extension_command::Event::GetAccounts(Default::default())),
    };

    let resp = handler.handle_extension_command(cmd).await.unwrap();
    if let Some(extension_command_response::Response::GetAccounts(res)) = resp.unwrap().response {
        assert_eq!(res.accounts.len(), 1);
        assert_eq!(res.accounts[0].id, "test_account");
        assert_eq!(res.accounts[0].name, "Test Account");
        assert!(res.accounts[0].logged_in);
    } else {
        panic!("Wrong response for GetAccounts");
    }

    let cmds = captured_commands.lock().unwrap();
    assert_eq!(cmds.len(), 1);
    if let Some(main_command::Command::UpdateAccounts(req)) = &cmds[0].command {
        assert_eq!(req.account, Some("sample.pkg".to_string()));
    } else {
        panic!("Expected UpdateAccounts");
    }
}

#[tokio::test]
async fn test_perform_account_login() {
    let (mut handler, captured_commands, _tmp) = setup_extension().await;

    let cmd = ExtensionCommand {
        package_name: "sample.pkg".to_string(),
        event: Some(extension_command::Event::PerformAccountLogin(
            extensions_proto::moosync::types::PerformAccountLoginRequest {
                account_id: "id".to_string(),
                login_status: true,
            },
        )),
    };

    let resp = handler.handle_extension_command(cmd).await.unwrap();
    if let Some(extension_command_response::Response::PerformAccountLogin(res)) =
        resp.unwrap().response
    {
        assert_eq!(res.status, "success");
    } else {
        panic!("Wrong response for PerformAccountLogin");
    }

    let cmds = captured_commands.lock().unwrap();
    if let Some(main_command::Command::RegisterOauth(req)) = &cmds[0].command {
        assert_eq!(req.url, "https://example.com/callback");
    } else {
        panic!("Expected RegisterOauth");
    }
}

#[tokio::test]
async fn test_custom_request_hash() {
    let (mut handler, _, _tmp) = setup_extension().await;

    let cmd = ExtensionCommand {
        package_name: "sample.pkg".to_string(),
        event: Some(extension_command::Event::CustomRequest(
            extensions_proto::moosync::types::CustomRequest {
                request_id: "hash_test".to_string(),
                payload: None,
            },
        )),
    };

    let resp = handler.handle_extension_command(cmd).await.unwrap();
    if let Some(extension_command_response::Response::CustomRequest(res)) = resp.unwrap().response {
        assert!(res.data.is_none());
    } else {
        panic!("Wrong response for CustomRequest (hash_test)");
    }
}

#[tokio::test]
async fn test_custom_request_preferences() {
    let (mut handler, captured_commands, _tmp) = setup_extension().await;

    let cmd = ExtensionCommand {
        package_name: "sample.pkg".to_string(),
        event: Some(extension_command::Event::CustomRequest(
            extensions_proto::moosync::types::CustomRequest {
                request_id: "preferences_test".to_string(),
                payload: None,
            },
        )),
    };

    let resp = handler.handle_extension_command(cmd).await.unwrap();
    if let Some(extension_command_response::Response::CustomRequest(res)) = resp.unwrap().response {
        assert!(res.data.is_none());
    } else {
        panic!("Wrong response for CustomRequest (preferences)");
    }

    let cmds = captured_commands.lock().unwrap();
    assert_eq!(cmds.len(), 2);
    assert!(matches!(
        cmds[0].command,
        Some(main_command::Command::RegisterUserPreference(_))
    ));
    assert!(matches!(
        cmds[1].command,
        Some(main_command::Command::UnregisterUserPreference(_))
    ));
}

#[tokio::test]
async fn test_search() {
    let (mut handler, captured_commands, _tmp) = setup_extension().await;

    let cmd = ExtensionCommand {
        package_name: "sample.pkg".to_string(),
        event: Some(extension_command::Event::RequestedSearchResult(
            extensions_proto::moosync::types::RequestedSearchResultRequest {
                query: "test".to_string(),
            },
        )),
    };

    let resp = handler.handle_extension_command(cmd).await.unwrap();
    if let Some(extension_command_response::Response::RequestedSearchResult(res)) =
        resp.unwrap().response
    {
        assert!(res.songs.is_empty());
    } else {
        panic!("Wrong response for RequestedSearchResult");
    }

    let cmds = captured_commands.lock().unwrap();
    assert!(matches!(
        cmds[0].command,
        Some(main_command::Command::OpenExternalUrl(_))
    ));
}

#[tokio::test]
async fn test_context_menu_action() {
    let (mut handler, captured_commands, _tmp) = setup_extension().await;

    let cmd = ExtensionCommand {
        package_name: "sample.pkg".to_string(),
        event: Some(extension_command::Event::ContextMenuAction(
            extensions_proto::moosync::types::ContextMenuActionRequest {
                action_id: "add_test".to_string(),
            },
        )),
    };

    let resp = handler.handle_extension_command(cmd).await.unwrap();
    assert!(matches!(
        resp.unwrap().response,
        Some(extension_command_response::Response::ContextMenuAction(_))
    ));

    let cmds = captured_commands.lock().unwrap();
    assert_eq!(cmds.len(), 3);
    assert!(matches!(
        cmds[0].command,
        Some(main_command::Command::AddPlaylist(_))
    ));
    assert!(matches!(
        cmds[1].command,
        Some(main_command::Command::AddSongs(_))
    ));
    assert!(matches!(
        cmds[2].command,
        Some(main_command::Command::AddToPlaylist(_))
    ));
}

#[tokio::test]
async fn test_preference_changed() {
    let (mut handler, captured_commands, _tmp) = setup_extension().await;

    let cmd = ExtensionCommand {
        package_name: "sample.pkg".to_string(),
        event: Some(extension_command::Event::PreferenceChanged(
            extensions_proto::moosync::types::PreferenceChangedRequest {
                preference: Some(extensions_proto::moosync::types::PreferenceArgs {
                    key: "test_key".to_string(),
                    value: Default::default(),
                }),
            },
        )),
    };

    let resp = handler.handle_extension_command(cmd).await.unwrap();
    assert!(matches!(
        resp.unwrap().response,
        Some(extension_command_response::Response::PreferenceChanged(_))
    ));

    let cmds = captured_commands.lock().unwrap();
    assert_eq!(cmds.len(), 2);
    assert!(matches!(
        cmds[0].command,
        Some(main_command::Command::GetPreference(_))
    ));
    assert!(matches!(
        cmds[1].command,
        Some(main_command::Command::GetSecure(_))
    ));
}

#[tokio::test]
async fn test_queue_changed() {
    let (mut handler, captured_commands, _tmp) = setup_extension().await;

    let cmd = ExtensionCommand {
        package_name: "sample.pkg".to_string(),
        event: Some(extension_command::Event::SongQueueChanged(
            extensions_proto::moosync::types::SongQueueChangedRequest {
                queue_state: Some(Default::default()),
            },
        )),
    };

    let resp = handler.handle_extension_command(cmd).await.unwrap();
    assert!(matches!(
        resp.unwrap().response,
        Some(extension_command_response::Response::SongQueueChanged(_))
    ));

    let cmds = captured_commands.lock().unwrap();
    assert!(matches!(
        cmds[0].command,
        Some(main_command::Command::GetQueue(_))
    ));
}

#[tokio::test]
async fn test_volume_changed() {
    let (mut handler, captured_commands, _tmp) = setup_extension().await;

    let cmd = ExtensionCommand {
        package_name: "sample.pkg".to_string(),
        event: Some(extension_command::Event::VolumeChanged(
            extensions_proto::moosync::types::VolumeChangedRequest { volume: 1.0 },
        )),
    };

    let resp = handler.handle_extension_command(cmd).await.unwrap();
    assert!(matches!(
        resp.unwrap().response,
        Some(extension_command_response::Response::VolumeChanged(_))
    ));

    let cmds = captured_commands.lock().unwrap();
    assert!(matches!(
        cmds[0].command,
        Some(main_command::Command::GetVolume(_))
    ));
}

#[tokio::test]
async fn test_player_state_changed() {
    let (mut handler, captured_commands, _tmp) = setup_extension().await;

    let cmd = ExtensionCommand {
        package_name: "sample.pkg".to_string(),
        event: Some(extension_command::Event::PlayerStateChanged(
            extensions_proto::moosync::types::PlayerStateChangedRequest {
                state: extensions_proto::moosync::types::PlayerState::Playing.into(),
            },
        )),
    };

    let resp = handler.handle_extension_command(cmd).await.unwrap();
    assert!(matches!(
        resp.unwrap().response,
        Some(extension_command_response::Response::PlayerStateChanged(_))
    ));

    let cmds = captured_commands.lock().unwrap();
    assert!(matches!(
        cmds[0].command,
        Some(main_command::Command::GetPlayerState(_))
    ));
}

#[tokio::test]
async fn test_song_changed() {
    let (mut handler, captured_commands, _tmp) = setup_extension().await;

    let cmd = ExtensionCommand {
        package_name: "sample.pkg".to_string(),
        event: Some(extension_command::Event::SongChanged(
            extensions_proto::moosync::types::SongChangedRequest { song: None },
        )),
    };

    let resp = handler.handle_extension_command(cmd).await.unwrap();
    assert!(matches!(
        resp.unwrap().response,
        Some(extension_command_response::Response::SongChanged(_))
    ));

    let cmds = captured_commands.lock().unwrap();
    assert!(matches!(
        cmds[0].command,
        Some(main_command::Command::GetCurrentSong(_))
    ));
}

#[tokio::test]
async fn test_seeked() {
    let (mut handler, captured_commands, _tmp) = setup_extension().await;

    let cmd = ExtensionCommand {
        package_name: "sample.pkg".to_string(),
        event: Some(extension_command::Event::Seeked(
            extensions_proto::moosync::types::SeekedRequest { position: 10.0 },
        )),
    };

    let resp = handler.handle_extension_command(cmd).await.unwrap();
    assert!(matches!(
        resp.unwrap().response,
        Some(extension_command_response::Response::Seeked(_))
    ));

    let cmds = captured_commands.lock().unwrap();
    assert!(matches!(
        cmds[0].command,
        Some(main_command::Command::GetTime(_))
    ));
}
