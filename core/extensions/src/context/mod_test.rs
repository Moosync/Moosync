use std::sync::{Arc, Mutex, RwLock};

use assertables::{assert_err, assert_none, assert_ok};
use extensions_proto::moosync::types::{
    AddPlaylistRequest, AddSongsRequest, AddToPlaylistRequest, BatchHttpRequest, BatchHttpResponse,
    ExtensionAccountDetail, ExtensionsUpdatedRequest, GetAppVersionRequest, GetCacheRequest,
    GetCurrentSongRequest, GetEntityRequest, GetPlayerStateRequest, GetPreferenceRequest,
    GetProcessStatusRequest, GetProcessStatusResponse, GetQueueRequest, GetSecureRequest,
    GetSongRequest, GetTimeRequest, GetVolumeRequest, KillProcessRequest, KillProcessResponse,
    NotifyPlayerEventRequest, NotifyPlayerEventResponse, OpenClientFdRequest, OpenClientFdResponse,
    OpenExternalUrlRequest, PlayerEvent as ProtoPlayerEvent, PlayerState, PreferenceData,
    ReadProcessStdoutRequest, ReadProcessStdoutResponse, ReadSockRequest, ReadSockResponse,
    RegisterOauthRequest, RegisterUserPreferenceRequest, RemoveSongRequest, SetAccountRequest,
    SetCacheRequest, SetPreferenceRequest, SetSecureRequest, SpawnProcessRequest,
    SpawnProcessResponse, SystemTimeRequest, SystemTimeResponse, UnregisterUserPreferenceRequest,
    UpdateSongRequest, WriteProcessStdinRequest, WriteProcessStdinResponse, WriteSockRequest,
    WriteSockResponse, main_command, main_command_response, player_event::Event as PlayerEvent,
};
use preferences_proto::moosync::types::PreferenceItem;
use songs_proto::moosync::types::{EntityResult, GetEntityOptions, GetSongOptions, Playlist, Song};
use tokio::sync::mpsc;
use tracing_test::traced_test;

use crate::{
    cache::ExtensionMemoryCache,
    context::{DispatchCommand, HostCommandContext, ReplyHandler},
    errors::ExtensionError,
    process::ExtensionProcessManager,
};

#[derive(Default)]
struct MockReply {
    pub cache: Mutex<std::collections::HashMap<String, Vec<u8>>>,
    pub proc_mgr: Option<ExtensionProcessManager>,
    pub player_state: Arc<RwLock<PlayerState>>,
    pub events_tx: Arc<RwLock<Option<mpsc::UnboundedSender<PlayerEvent>>>>,
}
impl ReplyHandler for MockReply {
    #[tracing::instrument(level = "debug", skip_all)]
    fn get_song(&self, _: &str, _: GetSongOptions) -> Result<Vec<Song>, ExtensionError> {
        Ok(vec![Song::default()])
    }
    #[tracing::instrument(level = "debug", skip_all)]
    fn get_entity(&self, _: &str, _: GetEntityOptions) -> Result<EntityResult, ExtensionError> {
        Ok(EntityResult::default())
    }
    #[tracing::instrument(level = "debug", skip_all)]
    fn get_current_song(&self, _: &str) -> Result<Option<Song>, ExtensionError> {
        Ok(Some(Song::default()))
    }
    #[tracing::instrument(level = "debug", skip_all)]
    fn get_player_state(&self, _: &str) -> Result<i32, ExtensionError> { Ok(1) }
    #[tracing::instrument(level = "debug", skip_all)]
    fn get_volume(&self, _: &str) -> Result<f64, ExtensionError> { Ok(0.8) }
    #[tracing::instrument(level = "debug", skip_all)]
    fn get_time(&self, _: &str) -> Result<f64, ExtensionError> { Ok(42.0) }
    #[tracing::instrument(level = "debug", skip_all)]
    fn get_queue(&self, _: &str) -> Result<(Vec<Song>, usize), ExtensionError> {
        Ok((vec![Song::default()], 2))
    }
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
    fn add_songs(&self, _: &str, _: Vec<Song>) -> Result<Vec<Song>, ExtensionError> {
        Ok(vec![Song::default()])
    }
    #[tracing::instrument(level = "debug", skip_all)]
    fn remove_song(&self, _: &str, _: Song) -> Result<bool, ExtensionError> { Ok(true) }
    #[tracing::instrument(level = "debug", skip_all)]
    fn update_song(&self, _: &str, s: Song) -> Result<Song, ExtensionError> { Ok(s) }
    #[tracing::instrument(level = "debug", skip_all)]
    fn add_playlist(&self, _: &str, _: Playlist) -> Result<String, ExtensionError> {
        Ok("pl1".to_string())
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
    fn set_account(&self, _: &str, _: ExtensionAccountDetail) -> Result<bool, ExtensionError> {
        Ok(true)
    }
    #[tracing::instrument(level = "debug", skip_all)]
    fn register_user_preference(
        &self,
        _: &str,
        _: Vec<PreferenceItem>,
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
    fn get_app_version(&self, _: &str) -> Result<String, ExtensionError> { Ok("2.0.0".to_string()) }
    #[tracing::instrument(level = "debug", skip_all)]
    fn get_cache(&self, package_name: &str, key: &str) -> Result<Option<Vec<u8>>, ExtensionError> {
        let scoped = format!("{package_name}:{key}");
        Ok(self.cache.lock().unwrap().get(&scoped).cloned())
    }
    #[tracing::instrument(level = "debug", skip_all)]
    fn set_cache(
        &self,
        package_name: &str,
        key: &str,
        data: Vec<u8>,
        _ttl_seconds: u64,
    ) -> Result<bool, ExtensionError> {
        let scoped = format!("{package_name}:{key}");
        self.cache.lock().unwrap().insert(scoped, data);
        Ok(true)
    }
    #[tracing::instrument(level = "debug", skip_all)]
    fn spawn_process(
        &self,
        _: &str,
        req: SpawnProcessRequest,
    ) -> Result<SpawnProcessResponse, ExtensionError> {
        if let Some(ref pm) = self.proc_mgr {
            pm.spawn_process(req)
        } else {
            Err(ExtensionError::NoContext)
        }
    }
    #[tracing::instrument(level = "debug", skip_all)]
    fn write_process_stdin(
        &self,
        _: &str,
        req: WriteProcessStdinRequest,
    ) -> Result<WriteProcessStdinResponse, ExtensionError> {
        if let Some(ref pm) = self.proc_mgr {
            pm.write_process_stdin(req)
        } else {
            Err(ExtensionError::NoContext)
        }
    }
    #[tracing::instrument(level = "debug", skip_all)]
    fn read_process_stdout(
        &self,
        _: &str,
        req: ReadProcessStdoutRequest,
    ) -> Result<ReadProcessStdoutResponse, ExtensionError> {
        if let Some(ref pm) = self.proc_mgr {
            pm.read_process_stdout(req)
        } else {
            Err(ExtensionError::NoContext)
        }
    }
    #[tracing::instrument(level = "debug", skip_all)]
    fn kill_process(
        &self,
        _: &str,
        req: KillProcessRequest,
    ) -> Result<KillProcessResponse, ExtensionError> {
        if let Some(ref pm) = self.proc_mgr {
            pm.kill_process(req)
        } else {
            Err(ExtensionError::NoContext)
        }
    }
    #[tracing::instrument(level = "debug", skip_all)]
    fn get_process_status(
        &self,
        _: &str,
        req: GetProcessStatusRequest,
    ) -> Result<GetProcessStatusResponse, ExtensionError> {
        if let Some(ref pm) = self.proc_mgr {
            pm.get_process_status(req)
        } else {
            Err(ExtensionError::NoContext)
        }
    }
    #[tracing::instrument(level = "debug", skip_all)]
    fn notify_player_event(
        &self,
        _: &str,
        req: NotifyPlayerEventRequest,
    ) -> Result<NotifyPlayerEventResponse, ExtensionError> {
        if let Some(player_event) = req.event {
            if let Some(PlayerEvent::Ended(_)) = player_event.event.as_ref() {
                *self.player_state.write().unwrap() = PlayerState::Stopped;
            }
            if let (Some(tx), Some(event)) =
                (self.events_tx.read().unwrap().as_ref(), player_event.event)
            {
                let _ = tx.send(event);
            }
        }
        Ok(NotifyPlayerEventResponse {})
    }
    #[tracing::instrument(level = "debug", skip_all)]
    fn system_time(
        &self,
        _: &str,
        _: SystemTimeRequest,
    ) -> Result<SystemTimeResponse, ExtensionError> {
        Ok(SystemTimeResponse { timestamp: 12345 })
    }
    #[tracing::instrument(level = "debug", skip_all)]
    fn open_client_fd(
        &self,
        _: &str,
        _: OpenClientFdRequest,
    ) -> Result<OpenClientFdResponse, ExtensionError> {
        Ok(OpenClientFdResponse { sock_id: 7 })
    }
    #[tracing::instrument(level = "debug", skip_all)]
    fn write_sock(
        &self,
        _: &str,
        req: WriteSockRequest,
    ) -> Result<WriteSockResponse, ExtensionError> {
        Ok(WriteSockResponse {
            bytes_written: req.buf.len() as i64,
        })
    }
    #[tracing::instrument(level = "debug", skip_all)]
    fn read_sock(&self, _: &str, _: ReadSockRequest) -> Result<ReadSockResponse, ExtensionError> {
        Ok(ReadSockResponse {
            data: b"ok".to_vec(),
        })
    }
    #[tracing::instrument(level = "debug", skip_all)]
    fn batch_http_request(
        &self,
        _: &str,
        _: BatchHttpRequest,
    ) -> Result<BatchHttpResponse, ExtensionError> {
        Ok(BatchHttpResponse::default())
    }
}

#[test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_dispatch_command_handlers() {
    let mock = MockReply::default();
    let memory_cache = Mutex::new(ExtensionMemoryCache::default());

    let cmd_state = main_command::Command::GetPlayerState(GetPlayerStateRequest {});
    let res_state = cmd_state.dispatch(&HostCommandContext::new(&mock, "pkg", &memory_cache));
    let cmd_vol = main_command::Command::GetVolume(GetVolumeRequest {});
    let res_vol = cmd_vol.dispatch(&HostCommandContext::new(&mock, "pkg", &memory_cache));
    let cmd_time = main_command::Command::GetTime(GetTimeRequest {});
    let res_time = cmd_time.dispatch(&HostCommandContext::new(&mock, "pkg", &memory_cache));
    let cmd_ver = main_command::Command::GetAppVersion(GetAppVersionRequest {});
    let res_ver = cmd_ver.dispatch(&HostCommandContext::new(&mock, "pkg", &memory_cache));

    assert_ok!(res_state.as_ref());
    assert_ok!(res_vol.as_ref());
    assert_ok!(res_time.as_ref());
    assert_ok!(res_ver.as_ref());

    match res_state.unwrap() {
        main_command_response::Response::GetPlayerState(r) => assert_eq!(r.state, 1),
        _ => panic!("Expected GetPlayerState"),
    }
    match res_vol.unwrap() {
        main_command_response::Response::GetVolume(r) => assert_eq!(r.volume, 0.8),
        _ => panic!("Expected GetVolume"),
    }
    match res_time.unwrap() {
        main_command_response::Response::GetTime(r) => assert_eq!(r.time, 42.0),
        _ => panic!("Expected GetTime"),
    }
    match res_ver.unwrap() {
        main_command_response::Response::GetAppVersion(r) => assert_eq!(r.version, "2.0.0"),
        _ => panic!("Expected GetAppVersion"),
    }
}

#[test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_dispatch_song_and_queue_commands() {
    let mock = MockReply::default();
    let memory_cache = Mutex::new(ExtensionMemoryCache::default());

    let cmd_song = main_command::Command::GetSong(GetSongRequest { options: None });
    let res_song = cmd_song.dispatch(&HostCommandContext::new(&mock, "pkg", &memory_cache));
    let cmd_entity = main_command::Command::GetEntity(GetEntityRequest { options: None });
    let res_entity = cmd_entity.dispatch(&HostCommandContext::new(&mock, "pkg", &memory_cache));
    let cmd_curr = main_command::Command::GetCurrentSong(GetCurrentSongRequest {});
    let res_curr = cmd_curr.dispatch(&HostCommandContext::new(&mock, "pkg", &memory_cache));
    let cmd_queue = main_command::Command::GetQueue(GetQueueRequest {});
    let res_queue = cmd_queue.dispatch(&HostCommandContext::new(&mock, "pkg", &memory_cache));

    assert_ok!(res_song.as_ref());
    assert_ok!(res_entity.as_ref());
    assert_ok!(res_curr.as_ref());
    assert_ok!(res_queue.as_ref());

    match res_song.unwrap() {
        main_command_response::Response::GetSong(r) => assert_eq!(r.songs.len(), 1),
        _ => panic!("Expected GetSong"),
    }
    match res_entity.unwrap() {
        main_command_response::Response::GetEntity(r) => assert!(r.entity.is_some()),
        _ => panic!("Expected GetEntity"),
    }
    match res_curr.unwrap() {
        main_command_response::Response::GetCurrentSong(r) => assert!(r.song.is_some()),
        _ => panic!("Expected GetCurrentSong"),
    }
    match res_queue.unwrap() {
        main_command_response::Response::GetQueue(r) => {
            let q = r.queue.unwrap();
            assert_eq!(q.songs.len(), 1);
            assert_eq!(q.index, 2);
        }
        _ => panic!("Expected GetQueue"),
    }
}

#[test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_dispatch_preference_and_secure_commands() {
    let mock = MockReply::default();
    let memory_cache = Mutex::new(ExtensionMemoryCache::default());

    let cmd_get_pref = main_command::Command::GetPreference(GetPreferenceRequest {
        data: Some(PreferenceData {
            key: "k1".to_string(),
            value: None,
        }),
    });
    let res_get_pref = cmd_get_pref.dispatch(&HostCommandContext::new(&mock, "pkg", &memory_cache));

    let cmd_set_pref = main_command::Command::SetPreference(SetPreferenceRequest {
        data: Some(PreferenceData {
            key: "k2".to_string(),
            value: None,
        }),
    });
    let res_set_pref = cmd_set_pref.dispatch(&HostCommandContext::new(&mock, "pkg", &memory_cache));

    let cmd_set_pref_none =
        main_command::Command::SetPreference(SetPreferenceRequest { data: None });
    let res_set_pref_none =
        cmd_set_pref_none.dispatch(&HostCommandContext::new(&mock, "pkg", &memory_cache));

    let cmd_get_sec = main_command::Command::GetSecure(GetSecureRequest {
        data: Some(PreferenceData {
            key: "k3".to_string(),
            value: None,
        }),
    });
    let res_get_sec = cmd_get_sec.dispatch(&HostCommandContext::new(&mock, "pkg", &memory_cache));

    let cmd_set_sec = main_command::Command::SetSecure(SetSecureRequest {
        data: Some(PreferenceData {
            key: "k4".to_string(),
            value: None,
        }),
    });
    let res_set_sec = cmd_set_sec.dispatch(&HostCommandContext::new(&mock, "pkg", &memory_cache));

    let cmd_set_sec_none = main_command::Command::SetSecure(SetSecureRequest { data: None });
    let res_set_sec_none =
        cmd_set_sec_none.dispatch(&HostCommandContext::new(&mock, "pkg", &memory_cache));

    let cmd_reg_pref =
        main_command::Command::RegisterUserPreference(RegisterUserPreferenceRequest {
            prefs: vec![],
        });
    let res_reg_pref = cmd_reg_pref.dispatch(&HostCommandContext::new(&mock, "pkg", &memory_cache));

    let cmd_unreg_pref =
        main_command::Command::UnregisterUserPreference(UnregisterUserPreferenceRequest {
            keys: vec![],
        });
    let res_unreg_pref =
        cmd_unreg_pref.dispatch(&HostCommandContext::new(&mock, "pkg", &memory_cache));

    assert_ok!(res_get_pref.as_ref());
    assert_ok!(res_set_pref.as_ref());
    assert_ok!(res_set_pref_none.as_ref());
    assert_ok!(res_get_sec.as_ref());
    assert_ok!(res_set_sec.as_ref());
    assert_ok!(res_set_sec_none.as_ref());
    assert_ok!(res_reg_pref.as_ref());
    assert_ok!(res_unreg_pref.as_ref());

    match res_set_pref.unwrap() {
        main_command_response::Response::SetPreference(r) => assert!(r.success),
        _ => panic!("Expected SetPreference"),
    }
    match res_set_pref_none.unwrap() {
        main_command_response::Response::SetPreference(r) => assert!(!r.success),
        _ => panic!("Expected SetPreference false"),
    }
    match res_set_sec.unwrap() {
        main_command_response::Response::SetSecure(r) => assert!(r.success),
        _ => panic!("Expected SetSecure"),
    }
    match res_set_sec_none.unwrap() {
        main_command_response::Response::SetSecure(r) => assert!(!r.success),
        _ => panic!("Expected SetSecure false"),
    }
}

#[test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_dispatch_song_mutations_and_playlists() {
    let mock = MockReply::default();
    let memory_cache = Mutex::new(ExtensionMemoryCache::default());

    let cmd_add_songs = main_command::Command::AddSongs(AddSongsRequest { songs: vec![] });
    let res_add_songs =
        cmd_add_songs.dispatch(&HostCommandContext::new(&mock, "pkg", &memory_cache));

    let cmd_rem_song = main_command::Command::RemoveSong(RemoveSongRequest {
        song: Some(Song::default()),
    });
    let res_rem_song = cmd_rem_song.dispatch(&HostCommandContext::new(&mock, "pkg", &memory_cache));

    let cmd_rem_song_none = main_command::Command::RemoveSong(RemoveSongRequest { song: None });
    let res_rem_song_none =
        cmd_rem_song_none.dispatch(&HostCommandContext::new(&mock, "pkg", &memory_cache));

    let cmd_upd_song = main_command::Command::UpdateSong(UpdateSongRequest {
        song: Some(Song::default()),
    });
    let res_upd_song = cmd_upd_song.dispatch(&HostCommandContext::new(&mock, "pkg", &memory_cache));

    let cmd_upd_song_err = main_command::Command::UpdateSong(UpdateSongRequest { song: None });
    let res_upd_song_err =
        cmd_upd_song_err.dispatch(&HostCommandContext::new(&mock, "pkg", &memory_cache));

    let cmd_add_pl = main_command::Command::AddPlaylist(AddPlaylistRequest {
        playlist: Some(Playlist::default()),
    });
    let res_add_pl = cmd_add_pl.dispatch(&HostCommandContext::new(&mock, "pkg", &memory_cache));

    let cmd_add_pl_err = main_command::Command::AddPlaylist(AddPlaylistRequest { playlist: None });
    let res_add_pl_err =
        cmd_add_pl_err.dispatch(&HostCommandContext::new(&mock, "pkg", &memory_cache));

    let cmd_add_to_pl = main_command::Command::AddToPlaylist(AddToPlaylistRequest {
        playlist_id: "pl1".to_string(),
        songs: vec![],
    });
    let res_add_to_pl =
        cmd_add_to_pl.dispatch(&HostCommandContext::new(&mock, "pkg", &memory_cache));

    assert_ok!(res_add_songs.as_ref());
    assert_ok!(res_rem_song.as_ref());
    assert_ok!(res_rem_song_none.as_ref());
    assert_ok!(res_upd_song.as_ref());
    assert_err!(res_upd_song_err.as_ref());
    assert_ok!(res_add_pl.as_ref());
    assert_err!(res_add_pl_err.as_ref());
    assert_ok!(res_add_to_pl.as_ref());

    match res_add_pl.unwrap() {
        main_command_response::Response::AddPlaylist(r) => assert_eq!(r.playlist_id, "pl1"),
        _ => panic!("Expected AddPlaylist"),
    }
}

#[test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_dispatch_oauth_account_and_misc() {
    let mock = MockReply::default();
    let memory_cache = Mutex::new(ExtensionMemoryCache::default());

    let cmd_oauth = main_command::Command::RegisterOauth(RegisterOauthRequest {
        url: "http://oauth".to_string(),
    });
    let res_oauth = cmd_oauth.dispatch(&HostCommandContext::new(&mock, "pkg", &memory_cache));

    let cmd_open = main_command::Command::OpenExternalUrl(OpenExternalUrlRequest {
        url: "http://open".to_string(),
    });
    let res_open = cmd_open.dispatch(&HostCommandContext::new(&mock, "pkg", &memory_cache));

    let cmd_acc = main_command::Command::SetAccount(SetAccountRequest {
        account: Some(ExtensionAccountDetail::default()),
    });
    let res_acc = cmd_acc.dispatch(&HostCommandContext::new(&mock, "pkg", &memory_cache));

    let cmd_acc_err = main_command::Command::SetAccount(SetAccountRequest { account: None });
    let res_acc_err = cmd_acc_err.dispatch(&HostCommandContext::new(&mock, "pkg", &memory_cache));

    let cmd_updated = main_command::Command::ExtensionsUpdated(ExtensionsUpdatedRequest::default());
    let res_updated = cmd_updated.dispatch(&HostCommandContext::new(&mock, "pkg", &memory_cache));

    assert_ok!(res_oauth.as_ref());
    assert_ok!(res_open.as_ref());
    assert_ok!(res_acc.as_ref());
    assert_err!(res_acc_err.as_ref());
    assert_ok!(res_updated.as_ref());
    assert_none!(mock.get_runtime_handle());
}

#[test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_dispatch_cache_hit_avoids_reply_handler() {
    let mock = MockReply::default();
    let memory_cache = Mutex::new(ExtensionMemoryCache::default());
    memory_cache
        .lock()
        .unwrap()
        .put("cached_key".to_string(), b"fast_data".to_vec(), 3600);

    let cmd_get = main_command::Command::GetCache(GetCacheRequest {
        key: "cached_key".to_string(),
    });
    let res = cmd_get.dispatch(&HostCommandContext::new(&mock, "pkg", &memory_cache));
    assert_ok!(res.as_ref());
    match res.unwrap() {
        main_command_response::Response::GetCache(r) => {
            assert_eq!(r.data, Some(b"fast_data".to_vec()));
        }
        _ => panic!("Expected GetCache"),
    }
    assert_eq!(mock.cache.lock().unwrap().len(), 0);
}

#[test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_dispatch_cache_miss_populates_l1() {
    let mock = MockReply::default();
    mock.cache
        .lock()
        .unwrap()
        .insert("pkg:db_key".to_string(), b"db_data".to_vec());
    let memory_cache = Mutex::new(ExtensionMemoryCache::default());

    let cmd_get = main_command::Command::GetCache(GetCacheRequest {
        key: "db_key".to_string(),
    });
    let res = cmd_get.dispatch(&HostCommandContext::new(&mock, "pkg", &memory_cache));
    assert_ok!(res.as_ref());
    match res.unwrap() {
        main_command_response::Response::GetCache(r) => {
            assert_eq!(r.data, Some(b"db_data".to_vec()));
        }
        _ => panic!("Expected GetCache"),
    }
    assert_eq!(
        memory_cache.lock().unwrap().get("db_key"),
        Some(b"db_data".to_vec())
    );
}

#[test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_dispatch_set_cache_updates_l1_and_calls_reply_handler() {
    let mock = MockReply::default();
    let memory_cache = Mutex::new(ExtensionMemoryCache::default());

    let cmd_set = main_command::Command::SetCache(SetCacheRequest {
        key: "new_key".to_string(),
        data: b"new_data".to_vec(),
        ttl_seconds: None,
    });
    let res = cmd_set.dispatch(&HostCommandContext::new(&mock, "pkg", &memory_cache));
    assert_ok!(res.as_ref());

    assert_eq!(
        memory_cache.lock().unwrap().get("new_key"),
        Some(b"new_data".to_vec())
    );
    assert_eq!(
        mock.cache.lock().unwrap().get("pkg:new_key"),
        Some(&b"new_data".to_vec())
    );
}

#[test]
#[cfg(unix)]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_dispatch_process_and_playback_ended_lifecycle() {
    use std::{fs, os::unix::fs::PermissionsExt, sync::RwLock};

    use extensions_proto::moosync::types::PlayerState;
    use tempdir::TempDir;
    use tokio::sync::mpsc;

    use crate::process::ExtensionProcessManager;

    let tmp = TempDir::new("test_dispatch_proc").unwrap();
    let bin_path = tmp.path().join("echo_bin.sh");
    fs::write(&bin_path, "#!/bin/sh\nread line\necho \"OUT:$line\"\n").unwrap();
    let mut perms = fs::metadata(&bin_path).unwrap().permissions();
    perms.set_mode(0o755);
    fs::set_permissions(&bin_path, perms).unwrap();

    let bytes = fs::read(&bin_path).unwrap();
    let hash = crate::process::sha256_hex(&bytes);

    let mut allowed_paths = std::collections::HashMap::new();
    allowed_paths.insert("test".to_string(), tmp.path().to_path_buf());

    let perm = extensions_proto::moosync::types::BinaryPermission {
        name: "echo_bin.sh".to_string(),
        hashes: [("sha256".to_string(), hash)].into_iter().collect(),
        justification: "test".to_string(),
    };
    let proc_mgr = ExtensionProcessManager::new(Some(allowed_paths), vec![perm]);
    let player_state = Arc::new(RwLock::new(PlayerState::Playing));
    let (events_tx, mut events_rx) = mpsc::unbounded_channel();
    let events_tx_holder = Arc::new(RwLock::new(Some(events_tx)));

    let mock = MockReply {
        cache: Default::default(),
        proc_mgr: Some(proc_mgr),
        player_state: player_state.clone(),
        events_tx: events_tx_holder,
    };
    let memory_cache = Mutex::new(ExtensionMemoryCache::default());

    let ctx = HostCommandContext {
        reply_handler: &mock,
        package_name: "pkg",
        memory_cache: &memory_cache,
    };

    let cmd_spawn = main_command::Command::SpawnProcess(SpawnProcessRequest {
        path: "echo_bin.sh".to_string(),
        args: vec![],
        env: Default::default(),
    });
    let res_spawn = cmd_spawn.dispatch(&ctx).unwrap();
    let proc_id = match res_spawn {
        main_command_response::Response::SpawnProcess(r) => r.process_id,
        other => panic!("Unexpected response {:?}", other),
    };
    assert!(proc_id > 0);

    let cmd_write = main_command::Command::WriteProcessStdin(WriteProcessStdinRequest {
        process_id: proc_id,
        data: b"integration_test\n".to_vec(),
    });
    let res_write = cmd_write.dispatch(&ctx).unwrap();
    match res_write {
        main_command_response::Response::WriteProcessStdin(r) => assert_eq!(r.bytes_written, 17),
        other => panic!("Unexpected response {:?}", other),
    }

    std::thread::sleep(std::time::Duration::from_millis(150));

    let cmd_read = main_command::Command::ReadProcessStdout(ReadProcessStdoutRequest {
        process_id: proc_id,
        max_bytes: Some(1024),
    });
    let res_read = cmd_read.dispatch(&ctx).unwrap();
    match res_read {
        main_command_response::Response::ReadProcessStdout(r) => {
            assert_eq!(String::from_utf8_lossy(&r.data), "OUT:integration_test\n");
        }
        other => panic!("Unexpected response {:?}", other),
    }

    let cmd_status = main_command::Command::GetProcessStatus(GetProcessStatusRequest {
        process_id: proc_id,
    });
    assert!(cmd_status.dispatch(&ctx).is_ok());

    let cmd_kill = main_command::Command::KillProcess(KillProcessRequest {
        process_id: proc_id,
    });
    assert!(cmd_kill.dispatch(&ctx).is_ok());

    let cmd_ended = main_command::Command::NotifyPlayerEvent(NotifyPlayerEventRequest {
        event: Some(ProtoPlayerEvent {
            event: Some(PlayerEvent::Ended(true)),
        }),
    });
    let res_ended = cmd_ended.dispatch(&ctx);
    assert!(res_ended.is_ok());
    assert_eq!(*player_state.read().unwrap(), PlayerState::Stopped);
    assert!(events_rx.try_recv().is_ok());
}

#[test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_dispatch_system_time_socket_and_http() {
    let mock = MockReply::default();
    let memory_cache = Mutex::new(ExtensionMemoryCache::default());
    let ctx = HostCommandContext::new(&mock, "pkg", &memory_cache);

    let res_time = main_command::Command::SystemTime(SystemTimeRequest {}).dispatch(&ctx);
    let res_open = main_command::Command::OpenClientFd(OpenClientFdRequest {
        path: "/tmp/test.sock".to_string(),
    })
    .dispatch(&ctx);
    let res_write = main_command::Command::WriteSock(WriteSockRequest {
        sock_id: 7,
        buf: b"abc".to_vec(),
    })
    .dispatch(&ctx);
    let res_read = main_command::Command::ReadSock(ReadSockRequest {
        sock_id: 7,
        read_len: 2,
    })
    .dispatch(&ctx);
    let res_http =
        main_command::Command::BatchHttpRequest(BatchHttpRequest::default()).dispatch(&ctx);

    assert_ok!(res_time.as_ref());
    assert_ok!(res_open.as_ref());
    assert_ok!(res_write.as_ref());
    assert_ok!(res_read.as_ref());
    assert_ok!(res_http.as_ref());
    match res_time.unwrap() {
        main_command_response::Response::SystemTime(r) => assert_eq!(r.timestamp, 12345),
        other => panic!("Unexpected response: {:?}", other),
    }
    match res_open.unwrap() {
        main_command_response::Response::OpenClientFd(r) => assert_eq!(r.sock_id, 7),
        other => panic!("Unexpected response: {:?}", other),
    }
    match res_write.unwrap() {
        main_command_response::Response::WriteSock(r) => assert_eq!(r.bytes_written, 3),
        other => panic!("Unexpected response: {:?}", other),
    }
    match res_read.unwrap() {
        main_command_response::Response::ReadSock(r) => assert_eq!(r.data, b"ok"),
        other => panic!("Unexpected response: {:?}", other),
    }
}
