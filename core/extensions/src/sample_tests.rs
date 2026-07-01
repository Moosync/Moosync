use std::{
    path::PathBuf,
    sync::{Arc, Mutex},
};

use extensions_proto::moosync::types::{
    ContextMenuActionRequest, ContextMenuActionResponse, CustomRequest, CustomRequestResponse,
    ExtensionCommand, GetAccountsRequest, GetAccountsResponse, GetProviderScopesRequest,
    GetProviderScopesResponse, MainCommand, PerformAccountLoginRequest,
    PerformAccountLoginResponse, PlayerState, PlayerStateChangedRequest,
    PlayerStateChangedResponse, PreferenceArgs, PreferenceChangedRequest,
    PreferenceChangedResponse, RequestedSearchResultRequest, RequestedSearchResultResponse,
    SeekedRequest, SeekedResponse, SongChangedRequest, SongChangedResponse,
    SongQueueChangedRequest, SongQueueChangedResponse, VolumeChangedRequest, VolumeChangedResponse,
    extension_command, extension_command_response, main_command,
};
use songs_proto::moosync::types::{EntityResult, GetEntityOptions, GetSongOptions, Playlist, Song};
use ui_proto::moosync::types::PreferenceUiData;

use crate::{context::ReplyHandler, errors::ExtensionError, ext_runner::ExtensionHandlerInner};

struct TempDir {
    path: PathBuf,
}

impl TempDir {
    #[tracing::instrument(level = "debug", skip_all)]
    fn new() -> Self {
        let mut path = std::env::temp_dir();
        path.push(uuid::Uuid::new_v4().to_string());
        std::fs::create_dir_all(&path).unwrap();
        Self { path }
    }

    #[tracing::instrument(level = "debug", skip_all)]
    fn path(&self) -> &PathBuf { &self.path }
}

impl Drop for TempDir {
    fn drop(&mut self) { let _ = std::fs::remove_dir_all(&self.path); }
}

struct TestReplyRouter {
    handlers: Mutex<std::collections::HashMap<String, Arc<Mutex<Vec<MainCommand>>>>>,
}

#[tracing::instrument(level = "debug", skip_all)]
fn get_global_router() -> Arc<TestReplyRouter> {
    static ROUTER: std::sync::OnceLock<Arc<TestReplyRouter>> = std::sync::OnceLock::new();
    ROUTER
        .get_or_init(|| {
            Arc::new(TestReplyRouter {
                handlers: Mutex::new(std::collections::HashMap::new()),
            })
        })
        .clone()
}

impl TestReplyRouter {
    #[tracing::instrument(level = "debug", skip_all)]
    fn register(&self, pkg: String, captured_commands: Arc<Mutex<Vec<MainCommand>>>) {
        self.handlers.lock().unwrap().insert(pkg, captured_commands);
    }

    #[tracing::instrument(level = "debug", skip_all)]
    fn remove(&self, pkg: &str) { self.handlers.lock().unwrap().remove(pkg); }

    #[tracing::instrument(level = "debug", skip_all)]
    fn get_captured_commands(&self, package_name: &str) -> Option<Arc<Mutex<Vec<MainCommand>>>> {
        self.handlers.lock().unwrap().get(package_name).cloned()
    }
}

impl ReplyHandler for TestReplyRouter {
    #[tracing::instrument(level = "debug", skip_all)]
    fn get_song(
        &self,
        package_name: &str,
        options: GetSongOptions,
    ) -> Result<Vec<Song>, ExtensionError> {
        if let Some(cmds) = self.get_captured_commands(package_name) {
            let mut cmds = cmds.lock().unwrap();
            cmds.push(MainCommand {
                command: Some(main_command::Command::GetSong(
                    extensions_proto::moosync::types::GetSongRequest {
                        options: Some(options),
                    },
                )),
            });
        }
        Ok(vec![])
    }

    #[tracing::instrument(level = "debug", skip_all)]
    fn get_entity(
        &self,
        package_name: &str,
        options: GetEntityOptions,
    ) -> Result<EntityResult, ExtensionError> {
        if let Some(cmds) = self.get_captured_commands(package_name) {
            let mut cmds = cmds.lock().unwrap();
            cmds.push(MainCommand {
                command: Some(main_command::Command::GetEntity(
                    extensions_proto::moosync::types::GetEntityRequest {
                        options: Some(options),
                    },
                )),
            });
        }
        Ok(EntityResult::default())
    }

    #[tracing::instrument(level = "debug", skip_all)]
    fn get_current_song(&self, package_name: &str) -> Result<Option<Song>, ExtensionError> {
        if let Some(cmds) = self.get_captured_commands(package_name) {
            let mut cmds = cmds.lock().unwrap();
            cmds.push(MainCommand {
                command: Some(main_command::Command::GetCurrentSong(
                    extensions_proto::moosync::types::GetCurrentSongRequest {},
                )),
            });
        }
        Ok(None)
    }

    #[tracing::instrument(level = "debug", skip_all)]
    fn get_player_state(&self, package_name: &str) -> Result<i32, ExtensionError> {
        if let Some(cmds) = self.get_captured_commands(package_name) {
            let mut cmds = cmds.lock().unwrap();
            cmds.push(MainCommand {
                command: Some(main_command::Command::GetPlayerState(
                    extensions_proto::moosync::types::GetPlayerStateRequest {},
                )),
            });
        }
        Ok(0)
    }

    #[tracing::instrument(level = "debug", skip_all)]
    fn get_volume(&self, package_name: &str) -> Result<f64, ExtensionError> {
        if let Some(cmds) = self.get_captured_commands(package_name) {
            let mut cmds = cmds.lock().unwrap();
            cmds.push(MainCommand {
                command: Some(main_command::Command::GetVolume(
                    extensions_proto::moosync::types::GetVolumeRequest {},
                )),
            });
        }
        Ok(1.0)
    }

    #[tracing::instrument(level = "debug", skip_all)]
    fn get_time(&self, package_name: &str) -> Result<f64, ExtensionError> {
        if let Some(cmds) = self.get_captured_commands(package_name) {
            let mut cmds = cmds.lock().unwrap();
            cmds.push(MainCommand {
                command: Some(main_command::Command::GetTime(
                    extensions_proto::moosync::types::GetTimeRequest {},
                )),
            });
        }
        Ok(0.0)
    }

    #[tracing::instrument(level = "debug", skip_all)]
    fn get_queue(&self, package_name: &str) -> Result<(Vec<Song>, usize), ExtensionError> {
        if let Some(cmds) = self.get_captured_commands(package_name) {
            let mut cmds = cmds.lock().unwrap();
            cmds.push(MainCommand {
                command: Some(main_command::Command::GetQueue(
                    extensions_proto::moosync::types::GetQueueRequest {},
                )),
            });
        }
        Ok((vec![], 0))
    }

    #[tracing::instrument(level = "debug", skip_all)]
    fn get_preference(
        &self,
        package_name: &str,
        key: &str,
    ) -> Result<Option<extensions_proto::struct_proto::google::protobuf::Value>, ExtensionError>
    {
        if let Some(cmds) = self.get_captured_commands(package_name) {
            let mut cmds = cmds.lock().unwrap();
            cmds.push(MainCommand {
                command: Some(main_command::Command::GetPreference(
                    extensions_proto::moosync::types::GetPreferenceRequest {
                        data: Some(extensions_proto::moosync::types::PreferenceData {
                            key: key.to_string(),
                            value: None,
                        }),
                    },
                )),
            });
        }
        Ok(None)
    }

    #[tracing::instrument(level = "debug", skip_all)]
    fn set_preference(
        &self,
        package_name: &str,
        key: &str,
        value: extensions_proto::struct_proto::google::protobuf::Value,
    ) -> Result<bool, ExtensionError> {
        if let Some(cmds) = self.get_captured_commands(package_name) {
            let mut cmds = cmds.lock().unwrap();
            cmds.push(MainCommand {
                command: Some(main_command::Command::SetPreference(
                    extensions_proto::moosync::types::SetPreferenceRequest {
                        data: Some(extensions_proto::moosync::types::PreferenceData {
                            key: key.to_string(),
                            value: Some(value),
                        }),
                    },
                )),
            });
        }
        Ok(true)
    }

    #[tracing::instrument(level = "debug", skip_all)]
    fn get_secure(
        &self,
        package_name: &str,
        key: &str,
    ) -> Result<Option<extensions_proto::struct_proto::google::protobuf::Value>, ExtensionError>
    {
        if let Some(cmds) = self.get_captured_commands(package_name) {
            let mut cmds = cmds.lock().unwrap();
            cmds.push(MainCommand {
                command: Some(main_command::Command::GetSecure(
                    extensions_proto::moosync::types::GetSecureRequest {
                        data: Some(extensions_proto::moosync::types::PreferenceData {
                            key: key.to_string(),
                            value: None,
                        }),
                    },
                )),
            });
        }
        Ok(None)
    }

    #[tracing::instrument(level = "debug", skip_all)]
    fn set_secure(
        &self,
        package_name: &str,
        key: &str,
        value: extensions_proto::struct_proto::google::protobuf::Value,
    ) -> Result<bool, ExtensionError> {
        if let Some(cmds) = self.get_captured_commands(package_name) {
            let mut cmds = cmds.lock().unwrap();
            cmds.push(MainCommand {
                command: Some(main_command::Command::SetSecure(
                    extensions_proto::moosync::types::SetSecureRequest {
                        data: Some(extensions_proto::moosync::types::PreferenceData {
                            key: key.to_string(),
                            value: Some(value),
                        }),
                    },
                )),
            });
        }
        Ok(true)
    }

    #[tracing::instrument(level = "debug", skip_all)]
    fn add_songs(&self, package_name: &str, songs: Vec<Song>) -> Result<Vec<Song>, ExtensionError> {
        if let Some(cmds) = self.get_captured_commands(package_name) {
            let mut cmds = cmds.lock().unwrap();
            cmds.push(MainCommand {
                command: Some(main_command::Command::AddSongs(
                    extensions_proto::moosync::types::AddSongsRequest { songs },
                )),
            });
        }
        Ok(vec![])
    }

    #[tracing::instrument(level = "debug", skip_all)]
    fn remove_song(&self, package_name: &str, song: Song) -> Result<bool, ExtensionError> {
        if let Some(cmds) = self.get_captured_commands(package_name) {
            let mut cmds = cmds.lock().unwrap();
            cmds.push(MainCommand {
                command: Some(main_command::Command::RemoveSong(
                    extensions_proto::moosync::types::RemoveSongRequest { song: Some(song) },
                )),
            });
        }
        Ok(true)
    }

    #[tracing::instrument(level = "debug", skip_all)]
    fn update_song(&self, package_name: &str, song: Song) -> Result<Song, ExtensionError> {
        if let Some(cmds) = self.get_captured_commands(package_name) {
            let mut cmds = cmds.lock().unwrap();
            cmds.push(MainCommand {
                command: Some(main_command::Command::UpdateSong(
                    extensions_proto::moosync::types::UpdateSongRequest {
                        song: Some(song.clone()),
                    },
                )),
            });
        }
        Ok(song)
    }

    #[tracing::instrument(level = "debug", skip_all)]
    fn add_playlist(
        &self,
        package_name: &str,
        playlist: Playlist,
    ) -> Result<String, ExtensionError> {
        if let Some(cmds) = self.get_captured_commands(package_name) {
            let mut cmds = cmds.lock().unwrap();
            cmds.push(MainCommand {
                command: Some(main_command::Command::AddPlaylist(
                    extensions_proto::moosync::types::AddPlaylistRequest {
                        playlist: Some(playlist),
                    },
                )),
            });
        }
        Ok("test".to_string())
    }

    #[tracing::instrument(level = "debug", skip_all)]
    fn add_to_playlist(
        &self,
        package_name: &str,
        playlist_id: String,
        songs: Vec<Song>,
    ) -> Result<bool, ExtensionError> {
        if let Some(cmds) = self.get_captured_commands(package_name) {
            let mut cmds = cmds.lock().unwrap();
            cmds.push(MainCommand {
                command: Some(main_command::Command::AddToPlaylist(
                    extensions_proto::moosync::types::AddToPlaylistRequest { playlist_id, songs },
                )),
            });
        }
        Ok(true)
    }

    #[tracing::instrument(level = "debug", skip_all)]
    fn register_oauth(&self, package_name: &str, url: String) -> Result<bool, ExtensionError> {
        if let Some(cmds) = self.get_captured_commands(package_name) {
            let mut cmds = cmds.lock().unwrap();
            cmds.push(MainCommand {
                command: Some(main_command::Command::RegisterOauth(
                    extensions_proto::moosync::types::RegisterOauthRequest { url },
                )),
            });
        }
        Ok(true)
    }

    #[tracing::instrument(level = "debug", skip_all)]
    fn open_external_url(&self, package_name: &str, url: String) -> Result<bool, ExtensionError> {
        if let Some(cmds) = self.get_captured_commands(package_name) {
            let mut cmds = cmds.lock().unwrap();
            cmds.push(MainCommand {
                command: Some(main_command::Command::OpenExternalUrl(
                    extensions_proto::moosync::types::OpenExternalUrlRequest { url },
                )),
            });
        }
        Ok(true)
    }

    #[tracing::instrument(level = "debug", skip_all)]
    fn update_accounts(
        &self,
        package_name: &str,
        account: Option<String>,
    ) -> Result<bool, ExtensionError> {
        if let Some(cmds) = self.get_captured_commands(package_name) {
            let mut cmds = cmds.lock().unwrap();
            cmds.push(MainCommand {
                command: Some(main_command::Command::UpdateAccounts(
                    extensions_proto::moosync::types::UpdateAccountsRequest { account },
                )),
            });
        }
        Ok(true)
    }

    #[tracing::instrument(level = "debug", skip_all)]
    fn register_user_preference(
        &self,
        package_name: &str,
        prefs: Vec<PreferenceUiData>,
    ) -> Result<bool, ExtensionError> {
        if let Some(cmds) = self.get_captured_commands(package_name) {
            let mut cmds = cmds.lock().unwrap();
            cmds.push(MainCommand {
                command: Some(main_command::Command::RegisterUserPreference(
                    extensions_proto::moosync::types::RegisterUserPreferenceRequest { prefs },
                )),
            });
        }
        Ok(true)
    }

    #[tracing::instrument(level = "debug", skip_all)]
    fn unregister_user_preference(
        &self,
        package_name: &str,
        keys: Vec<String>,
    ) -> Result<bool, ExtensionError> {
        if let Some(cmds) = self.get_captured_commands(package_name) {
            let mut cmds = cmds.lock().unwrap();
            cmds.push(MainCommand {
                command: Some(main_command::Command::UnregisterUserPreference(
                    extensions_proto::moosync::types::UnregisterUserPreferenceRequest { keys },
                )),
            });
        }
        Ok(true)
    }

    #[tracing::instrument(level = "debug", skip_all)]
    fn extensions_updated(&self, package_name: &str) -> Result<(), ExtensionError> {
        if let Some(cmds) = self.get_captured_commands(package_name) {
            let mut cmds = cmds.lock().unwrap();
            cmds.push(MainCommand {
                command: Some(main_command::Command::ExtensionsUpdated(
                    extensions_proto::moosync::types::ExtensionsUpdatedRequest {},
                )),
            });
        }
        Ok(())
    }

    #[tracing::instrument(level = "debug", skip_all)]
    fn get_app_version(&self, package_name: &str) -> Result<String, ExtensionError> {
        if let Some(cmds) = self.get_captured_commands(package_name) {
            let mut cmds = cmds.lock().unwrap();
            cmds.push(MainCommand {
                command: Some(main_command::Command::GetAppVersion(
                    extensions_proto::moosync::types::GetAppVersionRequest {},
                )),
            });
        }
        Ok("1.17.0".to_string())
    }
}

struct TestCleanupGuard {
    package_name: String,
    dest_ext_path: PathBuf,
    handler: Arc<ExtensionHandlerInner>,
    _temp_dir: TempDir,
}

impl Drop for TestCleanupGuard {
    fn drop(&mut self) {
        self.handler.remove_extension(&self.package_name);
        let _ = std::fs::remove_dir_all(&self.dest_ext_path);
        get_global_router().remove(&self.package_name);
    }
}

async fn setup_extension_at(
    subdir: &str,
    pkg: &str,
) -> (
    Arc<ExtensionHandlerInner>,
    String,
    Arc<Mutex<Vec<MainCommand>>>,
    TestCleanupGuard,
) {
    let temp_dir = TempDir::new();
    let cache_path = temp_dir.path().join("cache");
    let extensions_path = temp_dir.path().join("extensions");
    std::fs::create_dir_all(&extensions_path).unwrap();
    let handler = Arc::new(ExtensionHandlerInner::new(extensions_path, cache_path));

    let runfiles_dir = std::env::var("TEST_SRCDIR").unwrap_or_else(|_| ".".to_string());

    let src_ext_path = {
        let subdir_or_default = if subdir.is_empty() { "rs" } else { subdir };
        // Try bzlmod canonical repo name first (used when building from Moosync
        // workspace), then fall back to the plain module name (used when
        // building from moosync-exts workspace).
        let candidates = [
            PathBuf::from(&runfiles_dir)
                .join("moosync_ext+/sample_extensions")
                .join(subdir_or_default),
            PathBuf::from(&runfiles_dir)
                .join("moosync_ext/sample_extensions")
                .join(subdir_or_default),
            PathBuf::from(&runfiles_dir)
                .join("_main/sample_extensions")
                .join(subdir_or_default),
        ];
        candidates
            .iter()
            .find(|p| p.exists())
            .cloned()
            .unwrap_or_else(|| candidates[0].clone())
    };

    if !src_ext_path.exists() {
        panic!("Source extensions path not found: {:?}", src_ext_path);
    }

    let uuid_str = uuid::Uuid::new_v4().simple().to_string();
    let actual_pkg = format!("{}-{}", pkg.replace(".", "-"), uuid_str);
    let dest_ext_path = temp_dir.path().join("extensions").join(&actual_pkg);
    std::fs::create_dir_all(&dest_ext_path).unwrap();

    for entry in std::fs::read_dir(&src_ext_path).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.is_file() {
            let dest = dest_ext_path.join(path.file_name().unwrap());
            std::fs::copy(&path, &dest).unwrap();
            let mut perms = std::fs::metadata(&dest).unwrap().permissions();
            perms.set_readonly(false);
            std::fs::set_permissions(&dest, perms).unwrap();
        }
    }

    let package_json_path = dest_ext_path.join("package.json");
    let contents = std::fs::read_to_string(&package_json_path).unwrap();
    let mut json_val: serde_json::Value = serde_json::from_str(&contents).unwrap();
    if let Some(obj) = json_val.as_object_mut() {
        obj.insert(
            "name".to_string(),
            serde_json::Value::String(actual_pkg.clone()),
        );
        if !obj.contains_key("icon") {
            obj.insert(
                "icon".to_string(),
                serde_json::Value::String("icon.png".to_string()),
            );
        }
    }
    std::fs::write(
        &package_json_path,
        serde_json::to_string(&json_val).unwrap(),
    )
    .unwrap();

    static ONCE: std::sync::Once = std::sync::Once::new();
    ONCE.call_once(|| {
        tracing_subscriber::fmt()
            .with_env_filter("debug,extism=debug,extism_pdk=debug,cranelift_codegen=warn,wasmtime_cranelift=warn,wasmtime_internal_cranelift=warn,wasmtime=warn")
            .init();
    });

    let captured_commands = Arc::new(Mutex::new(Vec::<MainCommand>::new()));
    get_global_router().register(actual_pkg.clone(), captured_commands.clone());

    let reply_handler = get_global_router() as Arc<dyn ReplyHandler>;
    handler.spawn_extensions(reply_handler);

    let list = handler.get_installed_extensions();
    if !list.iter().any(|e| e.package_name == actual_pkg) {
        panic!("Setup failed: {} not found in {:?}", actual_pkg, list);
    }

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
        if start.elapsed() > std::time::Duration::from_secs(5) {
            break;
        }
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;
    }

    captured_commands.lock().unwrap().clear();

    let cleanup_guard = TestCleanupGuard {
        package_name: actual_pkg.clone(),
        dest_ext_path,
        handler: handler.clone(),
        _temp_dir: temp_dir,
    };

    (handler, actual_pkg, captured_commands, cleanup_guard)
}

async fn setup_extension() -> (
    Arc<ExtensionHandlerInner>,
    String,
    Arc<Mutex<Vec<MainCommand>>>,
    TestCleanupGuard,
) {
    setup_extension_at("", "sample.pkg").await
}

#[tokio::test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_get_provider_scopes() {
    let (handler, pkg, _, _guard) = setup_extension().await;
    let ext = {
        let map = handler.extensions_map.lock().unwrap();
        map.get(&pkg).unwrap().clone()
    };

    let res = ext
        .get_provider_scopes(GetProviderScopesRequest {})
        .await
        .unwrap();
    assert_eq!(res.scopes, vec![13]); // Accounts = 13
}

#[tokio::test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_get_accounts() {
    let (handler, pkg, captured_commands, _guard) = setup_extension().await;
    let ext = {
        let map = handler.extensions_map.lock().unwrap();
        map.get(&pkg).unwrap().clone()
    };

    let res = ext.get_accounts(Default::default()).await.unwrap();
    assert_eq!(res.accounts.len(), 1);
    assert_eq!(res.accounts[0].id, "test_account");
    assert_eq!(res.accounts[0].name, "Test Account");
    assert!(res.accounts[0].logged_in);

    let cmds = captured_commands.lock().unwrap();
    assert_eq!(cmds.len(), 1);
    if let Some(main_command::Command::UpdateAccounts(req)) = &cmds[0].command {
        assert_eq!(req.account, Some(pkg.clone()));
    } else {
        panic!("Wrong command update accounts");
    }
}

#[tokio::test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_perform_account_login() {
    let (handler, pkg, captured_commands, _guard) = setup_extension().await;
    let ext = {
        let map = handler.extensions_map.lock().unwrap();
        map.get(&pkg).unwrap().clone()
    };

    let res = ext
        .perform_account_login(PerformAccountLoginRequest {
            account_id: "id".to_string(),
            login_status: true,
        })
        .await
        .unwrap();
    assert_eq!(res.status, "success");

    let cmds = captured_commands.lock().unwrap();
    if let Some(main_command::Command::RegisterOauth(req)) = &cmds[0].command {
        assert_eq!(req.url, "https://example.com/callback");
    } else {
        panic!("Expected RegisterOauth");
    }
}

#[tokio::test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_custom_request_hash() {
    let (handler, pkg, _, _guard) = setup_extension().await;
    let ext = {
        let map = handler.extensions_map.lock().unwrap();
        map.get(&pkg).unwrap().clone()
    };

    let res = ext
        .custom_request(CustomRequest {
            request_id: "hash_test".to_string(),
            payload: None,
        })
        .await
        .unwrap();
    assert!(res.data.is_none());
}

#[tokio::test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_custom_request_preferences() {
    let (handler, pkg, captured_commands, _guard) = setup_extension().await;
    let ext = {
        let map = handler.extensions_map.lock().unwrap();
        map.get(&pkg).unwrap().clone()
    };

    let res = ext
        .custom_request(CustomRequest {
            request_id: "preferences_test".to_string(),
            payload: None,
        })
        .await
        .unwrap();
    assert!(res.data.is_none());

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
#[tracing::instrument(level = "debug", skip_all)]
async fn test_search() {
    let (handler, pkg, captured_commands, _guard) = setup_extension().await;
    let ext = {
        let map = handler.extensions_map.lock().unwrap();
        map.get(&pkg).unwrap().clone()
    };

    let res = ext
        .get_search_result(RequestedSearchResultRequest {
            query: "test".to_string(),
        })
        .await
        .unwrap();
    assert!(res.songs.is_empty());

    let cmds = captured_commands.lock().unwrap();
    assert!(matches!(
        cmds[0].command,
        Some(main_command::Command::OpenExternalUrl(_))
    ));
}

#[tokio::test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_context_menu_action() {
    let (handler, pkg, captured_commands, _guard) = setup_extension().await;
    let ext = {
        let map = handler.extensions_map.lock().unwrap();
        map.get(&pkg).unwrap().clone()
    };

    let _res = ext
        .context_menu_action(ContextMenuActionRequest {
            action_id: "add_test".to_string(),
        })
        .await
        .unwrap();

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
#[tracing::instrument(level = "debug", skip_all)]
async fn test_preference_changed() {
    let (handler, pkg, captured_commands, _guard) = setup_extension().await;
    let ext = {
        let map = handler.extensions_map.lock().unwrap();
        map.get(&pkg).unwrap().clone()
    };

    let _res = ext
        .preference_changed(PreferenceChangedRequest {
            preference: Some(PreferenceArgs {
                key: "test_key".to_string(),
                value: Default::default(),
            }),
        })
        .await
        .unwrap();

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
#[tracing::instrument(level = "debug", skip_all)]
async fn test_queue_changed() {
    let (handler, pkg, captured_commands, _guard) = setup_extension().await;
    let ext = {
        let map = handler.extensions_map.lock().unwrap();
        map.get(&pkg).unwrap().clone()
    };

    let _res = ext
        .song_queue_changed(SongQueueChangedRequest {
            queue_state: Some(Default::default()),
        })
        .await
        .unwrap();

    let cmds = captured_commands.lock().unwrap();
    assert!(matches!(
        cmds[0].command,
        Some(main_command::Command::GetQueue(_))
    ));
}

#[tokio::test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_volume_changed() {
    let (handler, pkg, captured_commands, _guard) = setup_extension().await;
    let ext = {
        let map = handler.extensions_map.lock().unwrap();
        map.get(&pkg).unwrap().clone()
    };

    let _res = ext
        .volume_changed(VolumeChangedRequest { volume: 1.0 })
        .await
        .unwrap();

    let cmds = captured_commands.lock().unwrap();
    assert!(matches!(
        cmds[0].command,
        Some(main_command::Command::GetVolume(_))
    ));
}

#[tokio::test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_player_state_changed() {
    let (handler, pkg, captured_commands, _guard) = setup_extension().await;
    let ext = {
        let map = handler.extensions_map.lock().unwrap();
        map.get(&pkg).unwrap().clone()
    };

    let _res = ext
        .player_state_changed(PlayerStateChangedRequest {
            state: PlayerState::Playing.into(),
        })
        .await
        .unwrap();

    let cmds = captured_commands.lock().unwrap();
    assert!(matches!(
        cmds[0].command,
        Some(main_command::Command::GetPlayerState(_))
    ));
}

#[tokio::test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_song_changed() {
    let (handler, pkg, captured_commands, _guard) = setup_extension().await;
    let ext = {
        let map = handler.extensions_map.lock().unwrap();
        map.get(&pkg).unwrap().clone()
    };

    let _res = ext
        .song_changed(SongChangedRequest { song: None })
        .await
        .unwrap();

    let cmds = captured_commands.lock().unwrap();
    assert!(matches!(
        cmds[0].command,
        Some(main_command::Command::GetCurrentSong(_))
    ));
}

#[tokio::test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_seeked() {
    let (handler, pkg, captured_commands, _guard) = setup_extension().await;
    let ext = {
        let map = handler.extensions_map.lock().unwrap();
        map.get(&pkg).unwrap().clone()
    };

    let _res = ext.seeked(SeekedRequest { position: 10.0 }).await.unwrap();

    let cmds = captured_commands.lock().unwrap();
    assert!(matches!(
        cmds[0].command,
        Some(main_command::Command::GetTime(_))
    ));
}

// ---------------------------------------------------------------------------
// Parameterized setup — takes a fixture subdirectory and expected package name
// ---------------------------------------------------------------------------

// ---------------------------------------------------------------------------
// Macro: generate the full test suite for any fixture subdirectory + package
// ---------------------------------------------------------------------------

/// Generates the complete sample-extension test suite inside a module.
/// `$mod_name` – Rust module name (e.g. `sample_rs`)
/// `$subdir`   – fixture sub-directory name (e.g. `"rs"`)
/// `$pkg`      – expected package_name in the extension manifest (e.g.
/// `"rs.sample"`)
macro_rules! generate_sample_tests {
    ($mod_name:ident, $subdir:expr, $pkg:expr) => {
        mod $mod_name {
            use super::*;

            async fn setup() -> (
                Arc<ExtensionHandlerInner>,
                String,
                Arc<Mutex<Vec<MainCommand>>>,
                TestCleanupGuard,
            ) {
                setup_extension_at($subdir, $pkg).await
            }

            #[tokio::test]
            #[tracing::instrument(level = "debug", skip_all)]
            async fn test_get_provider_scopes() {
                let (handler, pkg, _, _guard) = setup().await;
                let ext = {
                    let map = handler.extensions_map.lock().unwrap();
                    map.get(&pkg).unwrap().clone()
                };
                let res = ext
                    .get_provider_scopes(GetProviderScopesRequest {})
                    .await
                    .unwrap();
                assert_eq!(res.scopes, vec![13]);
            }

            #[tokio::test]
            #[tracing::instrument(level = "debug", skip_all)]
            async fn test_get_accounts() {
                let (handler, pkg, captured_commands, _guard) = setup().await;
                let ext = {
                    let map = handler.extensions_map.lock().unwrap();
                    map.get(&pkg).unwrap().clone()
                };
                let res = ext.get_accounts(Default::default()).await.unwrap();
                assert_eq!(res.accounts.len(), 1);
                assert_eq!(res.accounts[0].id, "test_account");
                assert_eq!(res.accounts[0].name, "Test Account");
                assert!(res.accounts[0].logged_in);

                let cmds = captured_commands.lock().unwrap();
                assert_eq!(cmds.len(), 1);
                if let Some(main_command::Command::UpdateAccounts(req)) = &cmds[0].command {
                    assert_eq!(req.account, Some(pkg.clone()));
                } else {
                    panic!("Expected UpdateAccounts");
                }
            }

            #[tokio::test]
            #[tracing::instrument(level = "debug", skip_all)]
            async fn test_perform_account_login() {
                let (handler, pkg, captured_commands, _guard) = setup().await;
                let ext = {
                    let map = handler.extensions_map.lock().unwrap();
                    map.get(&pkg).unwrap().clone()
                };
                let res = ext
                    .perform_account_login(PerformAccountLoginRequest {
                        account_id: "id".to_string(),
                        login_status: true,
                    })
                    .await
                    .unwrap();
                assert_eq!(res.status, "success");

                let cmds = captured_commands.lock().unwrap();
                if let Some(main_command::Command::RegisterOauth(req)) = &cmds[0].command {
                    assert_eq!(req.url, "https://example.com/callback");
                } else {
                    panic!("Expected RegisterOauth");
                }
            }

            #[tokio::test]
            #[tracing::instrument(level = "debug", skip_all)]
            async fn test_custom_request_hash() {
                let (handler, pkg, _, _guard) = setup().await;
                let ext = {
                    let map = handler.extensions_map.lock().unwrap();
                    map.get(&pkg).unwrap().clone()
                };
                let res = ext
                    .custom_request(CustomRequest {
                        request_id: "hash_test".to_string(),
                        payload: None,
                    })
                    .await
                    .unwrap();
                assert!(res.data.is_none());
            }

            #[tokio::test]
            #[tracing::instrument(level = "debug", skip_all)]
            async fn test_custom_request_preferences() {
                let (handler, pkg, captured_commands, _guard) = setup().await;
                let ext = {
                    let map = handler.extensions_map.lock().unwrap();
                    map.get(&pkg).unwrap().clone()
                };
                let res = ext
                    .custom_request(CustomRequest {
                        request_id: "preferences_test".to_string(),
                        payload: None,
                    })
                    .await
                    .unwrap();
                assert!(res.data.is_none());

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
            #[tracing::instrument(level = "debug", skip_all)]
            async fn test_search() {
                let (handler, pkg, captured_commands, _guard) = setup().await;
                let ext = {
                    let map = handler.extensions_map.lock().unwrap();
                    map.get(&pkg).unwrap().clone()
                };
                let res = ext
                    .get_search_result(RequestedSearchResultRequest {
                        query: "test".to_string(),
                    })
                    .await
                    .unwrap();
                assert!(res.songs.is_empty());

                let cmds = captured_commands.lock().unwrap();
                assert!(matches!(
                    cmds[0].command,
                    Some(main_command::Command::OpenExternalUrl(_))
                ));
            }

            #[tokio::test]
            #[tracing::instrument(level = "debug", skip_all)]
            async fn test_context_menu_action() {
                let (handler, pkg, captured_commands, _guard) = setup().await;
                let ext = {
                    let map = handler.extensions_map.lock().unwrap();
                    map.get(&pkg).unwrap().clone()
                };
                let _res = ext
                    .context_menu_action(ContextMenuActionRequest {
                        action_id: "add_test".to_string(),
                    })
                    .await
                    .unwrap();

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
            #[tracing::instrument(level = "debug", skip_all)]
            async fn test_preference_changed() {
                let (handler, pkg, captured_commands, _guard) = setup().await;
                let ext = {
                    let map = handler.extensions_map.lock().unwrap();
                    map.get(&pkg).unwrap().clone()
                };
                let _res = ext
                    .preference_changed(PreferenceChangedRequest {
                        preference: Some(PreferenceArgs {
                            key: "test_key".to_string(),
                            value: Default::default(),
                        }),
                    })
                    .await
                    .unwrap();

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
            #[tracing::instrument(level = "debug", skip_all)]
            async fn test_queue_changed() {
                let (handler, pkg, captured_commands, _guard) = setup().await;
                let ext = {
                    let map = handler.extensions_map.lock().unwrap();
                    map.get(&pkg).unwrap().clone()
                };
                let _res = ext
                    .song_queue_changed(SongQueueChangedRequest {
                        queue_state: Some(Default::default()),
                    })
                    .await
                    .unwrap();

                let cmds = captured_commands.lock().unwrap();
                assert!(matches!(
                    cmds[0].command,
                    Some(main_command::Command::GetQueue(_))
                ));
            }

            #[tokio::test]
            #[tracing::instrument(level = "debug", skip_all)]
            async fn test_volume_changed() {
                let (handler, pkg, captured_commands, _guard) = setup().await;
                let ext = {
                    let map = handler.extensions_map.lock().unwrap();
                    map.get(&pkg).unwrap().clone()
                };
                let _res = ext
                    .volume_changed(VolumeChangedRequest { volume: 1.0 })
                    .await
                    .unwrap();

                let cmds = captured_commands.lock().unwrap();
                assert!(matches!(
                    cmds[0].command,
                    Some(main_command::Command::GetVolume(_))
                ));
            }

            #[tokio::test]
            #[tracing::instrument(level = "debug", skip_all)]
            async fn test_player_state_changed() {
                let (handler, pkg, captured_commands, _guard) = setup().await;
                let ext = {
                    let map = handler.extensions_map.lock().unwrap();
                    map.get(&pkg).unwrap().clone()
                };
                let _res = ext
                    .player_state_changed(PlayerStateChangedRequest {
                        state: PlayerState::Playing.into(),
                    })
                    .await
                    .unwrap();

                let cmds = captured_commands.lock().unwrap();
                assert!(matches!(
                    cmds[0].command,
                    Some(main_command::Command::GetPlayerState(_))
                ));
            }

            #[tokio::test]
            #[tracing::instrument(level = "debug", skip_all)]
            async fn test_song_changed() {
                let (handler, pkg, captured_commands, _guard) = setup().await;
                let ext = {
                    let map = handler.extensions_map.lock().unwrap();
                    map.get(&pkg).unwrap().clone()
                };
                let _res = ext
                    .song_changed(SongChangedRequest { song: None })
                    .await
                    .unwrap();

                let cmds = captured_commands.lock().unwrap();
                assert!(matches!(
                    cmds[0].command,
                    Some(main_command::Command::GetCurrentSong(_))
                ));
            }

            #[tokio::test]
            #[tracing::instrument(level = "debug", skip_all)]
            async fn test_seeked() {
                let (handler, pkg, captured_commands, _guard) = setup().await;
                let ext = {
                    let map = handler.extensions_map.lock().unwrap();
                    map.get(&pkg).unwrap().clone()
                };
                let _res = ext.seeked(SeekedRequest { position: 10.0 }).await.unwrap();

                let cmds = captured_commands.lock().unwrap();
                assert!(matches!(
                    cmds[0].command,
                    Some(main_command::Command::GetTime(_))
                ));
            }
        }
    };
}

// Instantiate the full test suite for each language fixture.
generate_sample_tests!(sample_rs, "rs", "rs.sample");
generate_sample_tests!(sample_go, "go", "go.sample");
// generate_sample_tests!(sample_js, "js", "js.sample");
generate_sample_tests!(sample_py, "py", "py.sample");
