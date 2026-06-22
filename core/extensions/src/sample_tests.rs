use std::{
    path::PathBuf,
    sync::{Arc, Mutex},
};

use extensions_proto::moosync::types::{
    ExtensionCommand, GetProviderScopesRequest, MainCommand, extension_command,
    extension_command_response, main_command,
};
use songs_proto::moosync::types::{EntityResult, GetEntityOptions, GetSongOptions, Playlist, Song};
use ui_proto::moosync::types::PreferenceUiData;

use crate::{context::ReplyHandler, ext_runner::ExtensionHandlerInner};

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

struct TestReplyRouter {
    handlers: Mutex<std::collections::HashMap<String, Arc<Mutex<Vec<MainCommand>>>>>,
}

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
    fn register(&self, pkg: String, captured_commands: Arc<Mutex<Vec<MainCommand>>>) {
        self.handlers.lock().unwrap().insert(pkg, captured_commands);
    }

    fn remove(&self, pkg: &str) { self.handlers.lock().unwrap().remove(pkg); }

    fn get_captured_commands(&self, package_name: &str) -> Option<Arc<Mutex<Vec<MainCommand>>>> {
        self.handlers.lock().unwrap().get(package_name).cloned()
    }
}

impl ReplyHandler for TestReplyRouter {
    fn get_song(
        &self,
        package_name: &str,
        options: GetSongOptions,
    ) -> Result<Vec<Song>, types::errors::MoosyncError> {
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

    fn get_entity(
        &self,
        package_name: &str,
        options: GetEntityOptions,
    ) -> Result<EntityResult, types::errors::MoosyncError> {
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

    fn get_current_song(
        &self,
        package_name: &str,
    ) -> Result<Option<Song>, types::errors::MoosyncError> {
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

    fn get_player_state(&self, package_name: &str) -> Result<i32, types::errors::MoosyncError> {
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

    fn get_volume(&self, package_name: &str) -> Result<f64, types::errors::MoosyncError> {
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

    fn get_time(&self, package_name: &str) -> Result<f64, types::errors::MoosyncError> {
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

    fn get_queue(
        &self,
        package_name: &str,
    ) -> Result<(Vec<Song>, usize), types::errors::MoosyncError> {
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

    fn get_preference(
        &self,
        package_name: &str,
        key: &str,
    ) -> Result<
        Option<extensions_proto::struct_proto::google::protobuf::Value>,
        types::errors::MoosyncError,
    > {
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

    fn set_preference(
        &self,
        package_name: &str,
        key: &str,
        value: extensions_proto::struct_proto::google::protobuf::Value,
    ) -> Result<bool, types::errors::MoosyncError> {
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

    fn get_secure(
        &self,
        package_name: &str,
        key: &str,
    ) -> Result<
        Option<extensions_proto::struct_proto::google::protobuf::Value>,
        types::errors::MoosyncError,
    > {
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

    fn set_secure(
        &self,
        package_name: &str,
        key: &str,
        value: extensions_proto::struct_proto::google::protobuf::Value,
    ) -> Result<bool, types::errors::MoosyncError> {
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

    fn add_songs(
        &self,
        package_name: &str,
        songs: Vec<Song>,
    ) -> Result<Vec<Song>, types::errors::MoosyncError> {
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

    fn remove_song(
        &self,
        package_name: &str,
        song: Song,
    ) -> Result<bool, types::errors::MoosyncError> {
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

    fn update_song(
        &self,
        package_name: &str,
        song: Song,
    ) -> Result<Song, types::errors::MoosyncError> {
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

    fn add_playlist(
        &self,
        package_name: &str,
        playlist: Playlist,
    ) -> Result<String, types::errors::MoosyncError> {
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

    fn add_to_playlist(
        &self,
        package_name: &str,
        playlist_id: String,
        songs: Vec<Song>,
    ) -> Result<bool, types::errors::MoosyncError> {
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

    fn register_oauth(
        &self,
        package_name: &str,
        url: String,
    ) -> Result<bool, types::errors::MoosyncError> {
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

    fn open_external_url(
        &self,
        package_name: &str,
        url: String,
    ) -> Result<bool, types::errors::MoosyncError> {
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

    fn update_accounts(
        &self,
        package_name: &str,
        account: Option<String>,
    ) -> Result<bool, types::errors::MoosyncError> {
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

    fn register_user_preference(
        &self,
        package_name: &str,
        prefs: Vec<PreferenceUiData>,
    ) -> Result<bool, types::errors::MoosyncError> {
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

    fn unregister_user_preference(
        &self,
        package_name: &str,
        keys: Vec<String>,
    ) -> Result<bool, types::errors::MoosyncError> {
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

    fn extensions_updated(&self, package_name: &str) -> Result<(), types::errors::MoosyncError> {
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

    fn get_app_version(&self, package_name: &str) -> Result<String, types::errors::MoosyncError> {
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
}

impl Drop for TestCleanupGuard {
    fn drop(&mut self) {
        self.handler.remove_extension(&self.package_name);
        let _ = std::fs::remove_dir_all(&self.dest_ext_path);
        get_global_router().remove(&self.package_name);
    }
}

fn get_global_handler() -> (Arc<ExtensionHandlerInner>, &'static TempDir) {
    static GLOBAL_TEMP: std::sync::OnceLock<TempDir> = std::sync::OnceLock::new();
    static GLOBAL_INNER: std::sync::OnceLock<Arc<ExtensionHandlerInner>> =
        std::sync::OnceLock::new();

    let temp_dir = GLOBAL_TEMP.get_or_init(|| TempDir::new());
    let inner = GLOBAL_INNER.get_or_init(|| {
        let cache_path = temp_dir.path().join("cache");
        let extensions_path = temp_dir.path().join("extensions");
        std::fs::create_dir_all(&extensions_path).unwrap();
        Arc::new(ExtensionHandlerInner::new(extensions_path, cache_path))
    });

    (inner.clone(), temp_dir)
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
    let (handler, temp_dir) = get_global_handler();

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
    handler
        .spawn_single_extension(&package_json_path, reply_handler)
        .unwrap();

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
async fn test_get_provider_scopes() {
    let (handler, pkg, _, _guard) = setup_extension().await;

    let cmd = ExtensionCommand {
        package_name: pkg.clone(),
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
    let (handler, pkg, captured_commands, _guard) = setup_extension().await;

    let cmd = ExtensionCommand {
        package_name: pkg.clone(),
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
        assert_eq!(req.account, Some(pkg.clone()));
    } else {
        panic!("Expected UpdateAccounts");
    }
}

#[tokio::test]
async fn test_perform_account_login() {
    let (handler, pkg, captured_commands, _guard) = setup_extension().await;

    let cmd = ExtensionCommand {
        package_name: pkg.clone(),
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
    let (handler, pkg, _, _guard) = setup_extension().await;

    let cmd = ExtensionCommand {
        package_name: pkg.clone(),
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
    let (handler, pkg, captured_commands, _guard) = setup_extension().await;

    let cmd = ExtensionCommand {
        package_name: pkg.clone(),
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
    let (handler, pkg, captured_commands, _guard) = setup_extension().await;

    let cmd = ExtensionCommand {
        package_name: pkg.clone(),
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
    let (handler, pkg, captured_commands, _guard) = setup_extension().await;

    let cmd = ExtensionCommand {
        package_name: pkg.clone(),
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
    let (handler, pkg, captured_commands, _guard) = setup_extension().await;

    let cmd = ExtensionCommand {
        package_name: pkg.clone(),
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
    let (handler, pkg, captured_commands, _guard) = setup_extension().await;

    let cmd = ExtensionCommand {
        package_name: pkg.clone(),
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
    let (handler, pkg, captured_commands, _guard) = setup_extension().await;

    let cmd = ExtensionCommand {
        package_name: pkg.clone(),
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
    let (handler, pkg, captured_commands, _guard) = setup_extension().await;

    let cmd = ExtensionCommand {
        package_name: pkg.clone(),
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
    let (handler, pkg, captured_commands, _guard) = setup_extension().await;

    let cmd = ExtensionCommand {
        package_name: pkg.clone(),
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
    let (handler, pkg, captured_commands, _guard) = setup_extension().await;

    let cmd = ExtensionCommand {
        package_name: pkg.clone(),
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

// ---------------------------------------------------------------------------
// Parameterized setup — takes a fixture subdirectory and expected package name
// ---------------------------------------------------------------------------

// The original setup_extension() already delegates to setup_extension_at.
// (It was updated above in the file.)

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
            async fn test_get_provider_scopes() {
                let (handler, pkg, _, _guard) = setup().await;
                let cmd = ExtensionCommand {
                    package_name: pkg.clone(),
                    event: Some(extension_command::Event::GetProviderScopes(
                        GetProviderScopesRequest {},
                    )),
                };
                let resp = handler.handle_extension_command(cmd).await.unwrap();
                if let Some(extension_command_response::Response::GetProviderScopes(res)) =
                    resp.unwrap().response
                {
                    assert_eq!(res.scopes, vec![13]);
                } else {
                    panic!("Wrong response for GetProviderScopes");
                }
            }

            #[tokio::test]
            async fn test_get_accounts() {
                let (handler, pkg, captured_commands, _guard) = setup().await;
                let cmd = ExtensionCommand {
                    package_name: pkg.clone(),
                    event: Some(extension_command::Event::GetAccounts(Default::default())),
                };
                let resp = handler.handle_extension_command(cmd).await.unwrap();
                if let Some(extension_command_response::Response::GetAccounts(res)) =
                    resp.unwrap().response
                {
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
                    assert_eq!(req.account, Some(pkg.clone()));
                } else {
                    panic!("Expected UpdateAccounts");
                }
            }

            #[tokio::test]
            async fn test_perform_account_login() {
                let (handler, pkg, captured_commands, _guard) = setup().await;
                let cmd = ExtensionCommand {
                    package_name: pkg.clone(),
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
                let (handler, pkg, _, _guard) = setup().await;
                let cmd = ExtensionCommand {
                    package_name: pkg.clone(),
                    event: Some(extension_command::Event::CustomRequest(
                        extensions_proto::moosync::types::CustomRequest {
                            request_id: "hash_test".to_string(),
                            payload: None,
                        },
                    )),
                };
                let resp = handler.handle_extension_command(cmd).await.unwrap();
                if let Some(extension_command_response::Response::CustomRequest(res)) =
                    resp.unwrap().response
                {
                    assert!(res.data.is_none());
                } else {
                    panic!("Wrong response for CustomRequest (hash_test)");
                }
            }

            #[tokio::test]
            async fn test_custom_request_preferences() {
                let (handler, pkg, captured_commands, _guard) = setup().await;
                let cmd = ExtensionCommand {
                    package_name: pkg.clone(),
                    event: Some(extension_command::Event::CustomRequest(
                        extensions_proto::moosync::types::CustomRequest {
                            request_id: "preferences_test".to_string(),
                            payload: None,
                        },
                    )),
                };
                let resp = handler.handle_extension_command(cmd).await.unwrap();
                if let Some(extension_command_response::Response::CustomRequest(res)) =
                    resp.unwrap().response
                {
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
                let (handler, pkg, captured_commands, _guard) = setup().await;
                let cmd = ExtensionCommand {
                    package_name: pkg.clone(),
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
                let (handler, pkg, captured_commands, _guard) = setup().await;
                let cmd = ExtensionCommand {
                    package_name: pkg.clone(),
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
                let (handler, pkg, captured_commands, _guard) = setup().await;
                let cmd = ExtensionCommand {
                    package_name: pkg.clone(),
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
                let (handler, pkg, captured_commands, _guard) = setup().await;
                let cmd = ExtensionCommand {
                    package_name: pkg.clone(),
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
                let (handler, pkg, captured_commands, _guard) = setup().await;
                let cmd = ExtensionCommand {
                    package_name: pkg.clone(),
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
                let (handler, pkg, captured_commands, _guard) = setup().await;
                let cmd = ExtensionCommand {
                    package_name: pkg.clone(),
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
                let (handler, pkg, captured_commands, _guard) = setup().await;
                let cmd = ExtensionCommand {
                    package_name: pkg.clone(),
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
                let (handler, pkg, captured_commands, _guard) = setup().await;
                let cmd = ExtensionCommand {
                    package_name: pkg.clone(),
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
        }
    };
}

// Instantiate the full test suite for each language fixture.
generate_sample_tests!(sample_rs, "rs", "rs.sample");
generate_sample_tests!(sample_go, "go", "go.sample");
// generate_sample_tests!(sample_js, "js", "js.sample");
generate_sample_tests!(sample_py, "py", "py.sample");
