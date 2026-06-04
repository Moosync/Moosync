use crate::context::ReplyHandler;
use crate::ext_runner::ExtensionHandlerInner;
use extensions_proto::moosync::types::{
    ExtensionCommand, GetProviderScopesRequest, MainCommand,
    extension_command, extension_command_response,
    main_command,
};
use songs_proto::moosync::types::{Song, Playlist, GetSongOptions, GetEntityOptions, EntityResult};
use ui_proto::moosync::types::PreferenceUiData;
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

struct TestReplyHandler {
    captured_commands: Arc<Mutex<Vec<MainCommand>>>,
}

impl ReplyHandler for TestReplyHandler {
    fn get_song(
        &self,
        _package_name: &str,
        options: GetSongOptions,
    ) -> Result<Vec<Song>, types::errors::MoosyncError> {
        let mut cmds = self.captured_commands.lock().unwrap();
        cmds.push(MainCommand {
            command: Some(main_command::Command::GetSong(
                extensions_proto::moosync::types::GetSongRequest { options: Some(options) },
            )),
        });
        Ok(vec![])
    }

    fn get_entity(
        &self,
        _package_name: &str,
        options: GetEntityOptions,
    ) -> Result<EntityResult, types::errors::MoosyncError> {
        let mut cmds = self.captured_commands.lock().unwrap();
        cmds.push(MainCommand {
            command: Some(main_command::Command::GetEntity(
                extensions_proto::moosync::types::GetEntityRequest { options: Some(options) },
            )),
        });
        Ok(EntityResult::default())
    }

    fn get_current_song(
        &self,
        _package_name: &str,
    ) -> Result<Option<Song>, types::errors::MoosyncError> {
        let mut cmds = self.captured_commands.lock().unwrap();
        cmds.push(MainCommand {
            command: Some(main_command::Command::GetCurrentSong(
                extensions_proto::moosync::types::GetCurrentSongRequest {},
            )),
        });
        Ok(None)
    }

    fn get_player_state(
        &self,
        _package_name: &str,
    ) -> Result<i32, types::errors::MoosyncError> {
        let mut cmds = self.captured_commands.lock().unwrap();
        cmds.push(MainCommand {
            command: Some(main_command::Command::GetPlayerState(
                extensions_proto::moosync::types::GetPlayerStateRequest {},
            )),
        });
        Ok(0)
    }

    fn get_volume(
        &self,
        _package_name: &str,
    ) -> Result<f64, types::errors::MoosyncError> {
        let mut cmds = self.captured_commands.lock().unwrap();
        cmds.push(MainCommand {
            command: Some(main_command::Command::GetVolume(
                extensions_proto::moosync::types::GetVolumeRequest {},
            )),
        });
        Ok(1.0)
    }

    fn get_time(
        &self,
        _package_name: &str,
    ) -> Result<f64, types::errors::MoosyncError> {
        let mut cmds = self.captured_commands.lock().unwrap();
        cmds.push(MainCommand {
            command: Some(main_command::Command::GetTime(
                extensions_proto::moosync::types::GetTimeRequest {},
            )),
        });
        Ok(0.0)
    }

    fn get_queue(
        &self,
        _package_name: &str,
    ) -> Result<(Vec<Song>, usize), types::errors::MoosyncError> {
        let mut cmds = self.captured_commands.lock().unwrap();
        cmds.push(MainCommand {
            command: Some(main_command::Command::GetQueue(
                extensions_proto::moosync::types::GetQueueRequest {},
            )),
        });
        Ok((vec![], 0))
    }

    fn get_preference(
        &self,
        _package_name: &str,
        key: &str,
    ) -> Result<Option<serde_json::Value>, types::errors::MoosyncError> {
        let mut cmds = self.captured_commands.lock().unwrap();
        cmds.push(MainCommand {
            command: Some(main_command::Command::GetPreference(
                extensions_proto::moosync::types::GetPreferenceRequest {
                    data: Some(extensions_proto::moosync::types::PreferenceData {
                        key: key.to_string(),
                        value: None,
                    })
                },
            )),
        });
        Ok(None)
    }

    fn set_preference(
        &self,
        _package_name: &str,
        key: &str,
        value: serde_json::Value,
    ) -> Result<bool, types::errors::MoosyncError> {
        let mut cmds = self.captured_commands.lock().unwrap();
        cmds.push(MainCommand {
            command: Some(main_command::Command::SetPreference(
                extensions_proto::moosync::types::SetPreferenceRequest {
                    data: Some(extensions_proto::moosync::types::PreferenceData {
                        key: key.to_string(),
                        value: Some(serde_json::from_value(value).unwrap()),
                    })
                },
            )),
        });
        Ok(true)
    }

    fn get_secure(
        &self,
        _package_name: &str,
        key: &str,
    ) -> Result<Option<serde_json::Value>, types::errors::MoosyncError> {
        let mut cmds = self.captured_commands.lock().unwrap();
        cmds.push(MainCommand {
            command: Some(main_command::Command::GetSecure(
                extensions_proto::moosync::types::GetSecureRequest {
                    data: Some(extensions_proto::moosync::types::PreferenceData {
                        key: key.to_string(),
                        value: None,
                    })
                },
            )),
        });
        Ok(None)
    }

    fn set_secure(
        &self,
        _package_name: &str,
        key: &str,
        value: serde_json::Value,
    ) -> Result<bool, types::errors::MoosyncError> {
        let mut cmds = self.captured_commands.lock().unwrap();
        cmds.push(MainCommand {
            command: Some(main_command::Command::SetSecure(
                extensions_proto::moosync::types::SetSecureRequest {
                    data: Some(extensions_proto::moosync::types::PreferenceData {
                        key: key.to_string(),
                        value: Some(serde_json::from_value(value).unwrap()),
                    })
                },
            )),
        });
        Ok(true)
    }

    fn add_songs(
        &self,
        _package_name: &str,
        songs: Vec<Song>,
    ) -> Result<Vec<Song>, types::errors::MoosyncError> {
        let mut cmds = self.captured_commands.lock().unwrap();
        cmds.push(MainCommand {
            command: Some(main_command::Command::AddSongs(
                extensions_proto::moosync::types::AddSongsRequest { songs },
            )),
        });
        Ok(vec![])
    }

    fn remove_song(
        &self,
        _package_name: &str,
        song: Song,
    ) -> Result<bool, types::errors::MoosyncError> {
        let mut cmds = self.captured_commands.lock().unwrap();
        cmds.push(MainCommand {
            command: Some(main_command::Command::RemoveSong(
                extensions_proto::moosync::types::RemoveSongRequest { song: Some(song) },
            )),
        });
        Ok(true)
    }

    fn update_song(
        &self,
        _package_name: &str,
        song: Song,
    ) -> Result<Song, types::errors::MoosyncError> {
        let mut cmds = self.captured_commands.lock().unwrap();
        cmds.push(MainCommand {
            command: Some(main_command::Command::UpdateSong(
                extensions_proto::moosync::types::UpdateSongRequest { song: Some(song.clone()) },
            )),
        });
        Ok(song)
    }

    fn add_playlist(
        &self,
        _package_name: &str,
        playlist: Playlist,
    ) -> Result<String, types::errors::MoosyncError> {
        let mut cmds = self.captured_commands.lock().unwrap();
        cmds.push(MainCommand {
            command: Some(main_command::Command::AddPlaylist(
                extensions_proto::moosync::types::AddPlaylistRequest { playlist: Some(playlist) },
            )),
        });
        Ok("test".to_string())
    }

    fn add_to_playlist(
        &self,
        _package_name: &str,
        playlist_id: String,
        songs: Vec<Song>,
    ) -> Result<bool, types::errors::MoosyncError> {
        let mut cmds = self.captured_commands.lock().unwrap();
        cmds.push(MainCommand {
            command: Some(main_command::Command::AddToPlaylist(
                extensions_proto::moosync::types::AddToPlaylistRequest { playlist_id, songs },
            )),
        });
        Ok(true)
    }

    fn register_oauth(
        &self,
        _package_name: &str,
        url: String,
    ) -> Result<bool, types::errors::MoosyncError> {
        let mut cmds = self.captured_commands.lock().unwrap();
        cmds.push(MainCommand {
            command: Some(main_command::Command::RegisterOauth(
                extensions_proto::moosync::types::RegisterOauthRequest { url },
            )),
        });
        Ok(true)
    }

    fn open_external_url(
        &self,
        _package_name: &str,
        url: String,
    ) -> Result<bool, types::errors::MoosyncError> {
        let mut cmds = self.captured_commands.lock().unwrap();
        cmds.push(MainCommand {
            command: Some(main_command::Command::OpenExternalUrl(
                extensions_proto::moosync::types::OpenExternalUrlRequest { url },
            )),
        });
        Ok(true)
    }

    fn update_accounts(
        &self,
        _package_name: &str,
        account: Option<String>,
    ) -> Result<bool, types::errors::MoosyncError> {
        let mut cmds = self.captured_commands.lock().unwrap();
        cmds.push(MainCommand {
            command: Some(main_command::Command::UpdateAccounts(
                extensions_proto::moosync::types::UpdateAccountsRequest { account },
            )),
        });
        Ok(true)
    }

    fn register_user_preference(
        &self,
        _package_name: &str,
        prefs: Vec<PreferenceUiData>,
    ) -> Result<bool, types::errors::MoosyncError> {
        let mut cmds = self.captured_commands.lock().unwrap();
        cmds.push(MainCommand {
            command: Some(main_command::Command::RegisterUserPreference(
                extensions_proto::moosync::types::RegisterUserPreferenceRequest { prefs },
            )),
        });
        Ok(true)
    }

    fn unregister_user_preference(
        &self,
        _package_name: &str,
        keys: Vec<String>,
    ) -> Result<bool, types::errors::MoosyncError> {
        let mut cmds = self.captured_commands.lock().unwrap();
        cmds.push(MainCommand {
            command: Some(main_command::Command::UnregisterUserPreference(
                extensions_proto::moosync::types::UnregisterUserPreferenceRequest { keys },
            )),
        });
        Ok(true)
    }

    fn extensions_updated(
        &self,
        _package_name: &str,
    ) -> Result<(), types::errors::MoosyncError> {
        let mut cmds = self.captured_commands.lock().unwrap();
        cmds.push(MainCommand {
            command: Some(main_command::Command::ExtensionsUpdated(
                extensions_proto::moosync::types::ExtensionsUpdatedRequest {},
            )),
        });
        Ok(())
    }

    fn get_app_version(
        &self,
        _package_name: &str,
    ) -> Result<String, types::errors::MoosyncError> {
        let mut cmds = self.captured_commands.lock().unwrap();
        cmds.push(MainCommand {
            command: Some(main_command::Command::GetAppVersion(
                extensions_proto::moosync::types::GetAppVersionRequest {},
            )),
        });
        Ok("1.17.0".to_string())
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

    static ONCE: std::sync::Once = std::sync::Once::new();
    ONCE.call_once(|| {
        tracing_subscriber::fmt()
            .with_env_filter("debug,extism=debug,extism_pdk=debug,cranelift_codegen=warn,wasmtime_cranelift=warn,wasmtime_internal_cranelift=warn,wasmtime=warn")
            .init();
    });

    let captured_commands = Arc::new(Mutex::new(Vec::<MainCommand>::new()));
    let reply_handler = Arc::new(TestReplyHandler {
        captured_commands: captured_commands.clone(),
    });

    let mut handler = ExtensionHandlerInner::new(
        extensions_path.clone(),
        cache_path,
    );

    handler.spawn_extensions(reply_handler);

    // Verify extension is loaded
    let list = handler.get_installed_extensions();
    if list.is_empty() {
        panic!(
            "Setup failed: No extensions loaded from {:?}",
            extensions_path
        );
    }
    if !list
        .iter()
        .any(|e| e.package_name == "sample.pkg")
    {
        panic!(
            "Setup failed: sample.pkg not found in {:?}",
            list
        );
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
