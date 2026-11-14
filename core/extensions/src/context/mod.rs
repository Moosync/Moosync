use std::{fmt::Debug, sync::Mutex};

use extensions_proto::moosync::types::{
    AddPlaylistResponse, AddSongsResponse, AddToPlaylistResponse, ExtensionAccountDetail,
    ExtensionCommand, ExtensionCommandResponse, ExtensionsUpdatedResponse, GetAppVersionResponse,
    GetCacheResponse, GetCurrentSongResponse, GetEntityResponse, GetPlayerStateResponse,
    GetPreferenceResponse, GetProcessStatusRequest, GetProcessStatusResponse, GetQueueResponse,
    GetSecureResponse, GetSongResponse, GetTimeResponse, GetVolumeResponse, KillProcessRequest,
    KillProcessResponse, NotifyPlayerEventRequest, NotifyPlayerEventResponse,
    OpenExternalUrlResponse, PreferenceData, QueueState, ReadProcessStdoutRequest,
    ReadProcessStdoutResponse, RegisterOauthResponse, RegisterUserPreferenceResponse,
    RemoveSongResponse, SetAccountResponse, SetCacheResponse, SetPreferenceResponse,
    SetSecureResponse, SpawnProcessRequest, SpawnProcessResponse, UnregisterUserPreferenceResponse,
    UpdateSongResponse, WriteProcessStdinRequest, WriteProcessStdinResponse, main_command,
    main_command_response,
};
pub use extism_context::ExtismContext;
use preferences_proto::moosync::types::PreferenceItem;
use songs_proto::moosync::types::{EntityResult, GetEntityOptions, GetSongOptions, Playlist, Song};

use crate::{cache::ExtensionMemoryCache, errors::ExtensionError};

pub struct HostCommandContext<'a> {
    pub reply_handler: &'a dyn ReplyHandler,
    pub package_name: &'a str,
    pub memory_cache: &'a Mutex<ExtensionMemoryCache>,
}

#[cfg(test)]
impl<'a> HostCommandContext<'a> {
    #[tracing::instrument(level = "debug", skip_all)]
    pub fn new(
        reply_handler: &'a dyn ReplyHandler,
        package_name: &'a str,
        memory_cache: &'a Mutex<ExtensionMemoryCache>,
    ) -> Self {
        Self {
            reply_handler,
            package_name,
            memory_cache,
        }
    }
}

pub trait ReplyHandler: Send + Sync + 'static {
    fn get_song(
        &self,
        _package_name: &str,
        _options: GetSongOptions,
    ) -> Result<Vec<Song>, ExtensionError>;
    fn get_entity(
        &self,
        _package_name: &str,
        _options: GetEntityOptions,
    ) -> Result<EntityResult, ExtensionError>;
    fn get_current_song(&self, _package_name: &str) -> Result<Option<Song>, ExtensionError>;
    fn get_player_state(&self, _package_name: &str) -> Result<i32, ExtensionError>;
    fn get_volume(&self, _package_name: &str) -> Result<f64, ExtensionError>;
    fn get_time(&self, _package_name: &str) -> Result<f64, ExtensionError>;
    fn get_queue(&self, _package_name: &str) -> Result<(Vec<Song>, usize), ExtensionError>;
    fn get_preference(
        &self,
        _package_name: &str,
        _key: &str,
    ) -> Result<Option<extensions_proto::struct_proto::google::protobuf::Value>, ExtensionError>;
    fn set_preference(
        &self,
        _package_name: &str,
        _key: &str,
        _value: extensions_proto::struct_proto::google::protobuf::Value,
    ) -> Result<bool, ExtensionError>;
    fn get_secure(
        &self,
        _package_name: &str,
        _key: &str,
    ) -> Result<Option<extensions_proto::struct_proto::google::protobuf::Value>, ExtensionError>;
    fn set_secure(
        &self,
        _package_name: &str,
        _key: &str,
        _value: extensions_proto::struct_proto::google::protobuf::Value,
    ) -> Result<bool, ExtensionError>;
    fn add_songs(
        &self,
        _package_name: &str,
        _songs: Vec<Song>,
    ) -> Result<Vec<Song>, ExtensionError>;
    fn remove_song(&self, _package_name: &str, _song: Song) -> Result<bool, ExtensionError>;
    fn update_song(&self, _package_name: &str, _song: Song) -> Result<Song, ExtensionError>;
    fn add_playlist(
        &self,
        _package_name: &str,
        _playlist: Playlist,
    ) -> Result<String, ExtensionError>;
    fn add_to_playlist(
        &self,
        _package_name: &str,
        _playlist_id: String,
        _songs: Vec<Song>,
    ) -> Result<bool, ExtensionError>;
    fn register_oauth(&self, _package_name: &str, _url: String) -> Result<bool, ExtensionError>;
    fn open_external_url(&self, _package_name: &str, _url: String) -> Result<bool, ExtensionError>;
    fn set_account(
        &self,
        _package_name: &str,
        _account: ExtensionAccountDetail,
    ) -> Result<bool, ExtensionError>;
    fn register_user_preference(
        &self,
        _package_name: &str,
        _prefs: Vec<PreferenceItem>,
    ) -> Result<bool, ExtensionError>;
    fn unregister_user_preference(
        &self,
        _package_name: &str,
        _keys: Vec<String>,
    ) -> Result<bool, ExtensionError>;
    fn extensions_updated(&self, _package_name: &str) -> Result<(), ExtensionError>;
    fn get_app_version(&self, _package_name: &str) -> Result<String, ExtensionError>;
    fn get_cache(&self, package_name: &str, key: &str) -> Result<Option<Vec<u8>>, ExtensionError>;
    fn set_cache(
        &self,
        package_name: &str,
        key: &str,
        data: Vec<u8>,
        ttl_seconds: u64,
    ) -> Result<bool, ExtensionError>;
    fn spawn_process(
        &self,
        _package_name: &str,
        _req: SpawnProcessRequest,
    ) -> Result<SpawnProcessResponse, ExtensionError> {
        Err(ExtensionError::NoContext)
    }
    fn write_process_stdin(
        &self,
        _package_name: &str,
        _req: WriteProcessStdinRequest,
    ) -> Result<WriteProcessStdinResponse, ExtensionError> {
        Err(ExtensionError::NoContext)
    }
    fn read_process_stdout(
        &self,
        _package_name: &str,
        _req: ReadProcessStdoutRequest,
    ) -> Result<ReadProcessStdoutResponse, ExtensionError> {
        Err(ExtensionError::NoContext)
    }
    fn kill_process(
        &self,
        _package_name: &str,
        _req: KillProcessRequest,
    ) -> Result<KillProcessResponse, ExtensionError> {
        Err(ExtensionError::NoContext)
    }
    fn get_process_status(
        &self,
        _package_name: &str,
        _req: GetProcessStatusRequest,
    ) -> Result<GetProcessStatusResponse, ExtensionError> {
        Err(ExtensionError::NoContext)
    }
    fn notify_player_event(
        &self,
        _package_name: &str,
        _req: NotifyPlayerEventRequest,
    ) -> Result<NotifyPlayerEventResponse, ExtensionError> {
        Err(ExtensionError::NoContext)
    }
    fn get_runtime_handle(&self) -> Option<tokio::runtime::Handle> { None }
}

pub(crate) trait DispatchCommand {
    fn dispatch(
        self,
        ctx: &HostCommandContext,
    ) -> Result<main_command_response::Response, ExtensionError>;
}

impl DispatchCommand for main_command::Command {
    #[tracing::instrument(level = "debug", skip_all)]
    fn dispatch(
        self,
        ctx: &HostCommandContext,
    ) -> Result<main_command_response::Response, ExtensionError> {
        let reply_handler = ctx.reply_handler;
        let package_name = ctx.package_name;
        let memory_cache = ctx.memory_cache;
        match self {
            main_command::Command::GetSong(req) => reply_handler
                .get_song(package_name, req.options.unwrap_or_default())
                .map(|songs| main_command_response::Response::GetSong(GetSongResponse { songs })),
            main_command::Command::GetEntity(req) => reply_handler
                .get_entity(package_name, req.options.unwrap_or_default())
                .map(|entity| {
                    main_command_response::Response::GetEntity(GetEntityResponse {
                        entity: Some(entity),
                    })
                }),
            main_command::Command::GetCurrentSong(_) => {
                reply_handler.get_current_song(package_name).map(|song| {
                    main_command_response::Response::GetCurrentSong(GetCurrentSongResponse { song })
                })
            }
            main_command::Command::GetPlayerState(_) => {
                reply_handler.get_player_state(package_name).map(|state| {
                    main_command_response::Response::GetPlayerState(GetPlayerStateResponse {
                        state,
                    })
                })
            }
            main_command::Command::GetVolume(_) => {
                reply_handler.get_volume(package_name).map(|volume| {
                    main_command_response::Response::GetVolume(GetVolumeResponse { volume })
                })
            }
            main_command::Command::GetTime(_) => reply_handler
                .get_time(package_name)
                .map(|time| main_command_response::Response::GetTime(GetTimeResponse { time })),
            main_command::Command::GetQueue(_) => {
                reply_handler.get_queue(package_name).map(|(songs, index)| {
                    main_command_response::Response::GetQueue(GetQueueResponse {
                        queue: Some(QueueState {
                            songs,
                            index: index as u64,
                        }),
                    })
                })
            }
            main_command::Command::GetPreference(req) => {
                let key = req.data.map(|d| d.key).unwrap_or_default();
                reply_handler
                    .get_preference(package_name, &key)
                    .map(|value| {
                        main_command_response::Response::GetPreference(GetPreferenceResponse {
                            data: Some(PreferenceData { key, value }),
                        })
                    })
            }
            main_command::Command::SetPreference(req) => req
                .data
                .map_or(Ok(false), |data| {
                    reply_handler.set_preference(
                        package_name,
                        &data.key,
                        data.value.unwrap_or_default(),
                    )
                })
                .map(|success| {
                    main_command_response::Response::SetPreference(SetPreferenceResponse {
                        success,
                    })
                }),
            main_command::Command::GetSecure(req) => {
                let key = req.data.map(|d| d.key).unwrap_or_default();
                reply_handler.get_secure(package_name, &key).map(|value| {
                    main_command_response::Response::GetSecure(GetSecureResponse {
                        data: Some(PreferenceData { key, value }),
                    })
                })
            }
            main_command::Command::SetSecure(req) => req
                .data
                .map_or(Ok(false), |data| {
                    reply_handler.set_secure(
                        package_name,
                        &data.key,
                        data.value.unwrap_or_default(),
                    )
                })
                .map(|success| {
                    main_command_response::Response::SetSecure(SetSecureResponse { success })
                }),
            main_command::Command::RegisterUserPreference(req) => reply_handler
                .register_user_preference(package_name, req.prefs)
                .map(|success| {
                    main_command_response::Response::RegisterUserPreference(
                        RegisterUserPreferenceResponse { success },
                    )
                }),
            main_command::Command::UnregisterUserPreference(req) => reply_handler
                .unregister_user_preference(package_name, req.keys)
                .map(|success| {
                    main_command_response::Response::UnregisterUserPreference(
                        UnregisterUserPreferenceResponse { success },
                    )
                }),
            main_command::Command::AddSongs(req) => reply_handler
                .add_songs(package_name, req.songs)
                .map(|songs| main_command_response::Response::AddSongs(AddSongsResponse { songs })),
            main_command::Command::RemoveSong(req) => req
                .song
                .map_or(Ok(false), |song| {
                    reply_handler.remove_song(package_name, song)
                })
                .map(|success| {
                    main_command_response::Response::RemoveSong(RemoveSongResponse { success })
                }),
            main_command::Command::UpdateSong(req) => {
                let song = req.song.ok_or(ExtensionError::MissingSong)?;
                reply_handler.update_song(package_name, song).map(|song| {
                    main_command_response::Response::UpdateSong(UpdateSongResponse {
                        song: Some(song),
                    })
                })
            }
            main_command::Command::AddPlaylist(req) => {
                let playlist = req.playlist.ok_or(ExtensionError::MissingPlaylist)?;
                reply_handler
                    .add_playlist(package_name, playlist)
                    .map(|playlist_id| {
                        main_command_response::Response::AddPlaylist(AddPlaylistResponse {
                            playlist_id,
                        })
                    })
            }
            main_command::Command::AddToPlaylist(req) => reply_handler
                .add_to_playlist(package_name, req.playlist_id, req.songs)
                .map(|success| {
                    main_command_response::Response::AddToPlaylist(AddToPlaylistResponse {
                        success,
                    })
                }),
            main_command::Command::RegisterOauth(req) => reply_handler
                .register_oauth(package_name, req.url)
                .map(|success| {
                    main_command_response::Response::RegisterOauth(RegisterOauthResponse {
                        success,
                    })
                }),
            main_command::Command::OpenExternalUrl(req) => reply_handler
                .open_external_url(package_name, req.url)
                .map(|success| {
                    main_command_response::Response::OpenExternalUrl(OpenExternalUrlResponse {
                        success,
                    })
                }),
            main_command::Command::SetAccount(req) => {
                let account = req.account.ok_or_else(|| {
                    ExtensionError::Sanitize(
                        "Account detail missing from SetAccountRequest".to_string(),
                    )
                })?;
                reply_handler
                    .set_account(package_name, account)
                    .map(|success| {
                        main_command_response::Response::SetAccount(SetAccountResponse { success })
                    })
            }
            main_command::Command::ExtensionsUpdated(_) => {
                reply_handler.extensions_updated(package_name).map(|_| {
                    main_command_response::Response::ExtensionsUpdated(ExtensionsUpdatedResponse {})
                })
            }
            main_command::Command::GetAppVersion(_) => {
                reply_handler.get_app_version(package_name).map(|version| {
                    main_command_response::Response::GetAppVersion(GetAppVersionResponse {
                        version,
                    })
                })
            }
            main_command::Command::GetCache(req) => {
                if let Some(cached_data) = memory_cache.lock().unwrap().get(&req.key) {
                    return Ok(main_command_response::Response::GetCache(
                        GetCacheResponse {
                            data: Some(cached_data),
                        },
                    ));
                }

                match reply_handler.get_cache(package_name, &req.key) {
                    Ok(Some(data)) => {
                        memory_cache
                            .lock()
                            .unwrap()
                            .put(req.key, data.clone(), 7 * 24 * 3600);
                        Ok(main_command_response::Response::GetCache(
                            GetCacheResponse { data: Some(data) },
                        ))
                    }
                    Ok(None) => Ok(main_command_response::Response::GetCache(
                        GetCacheResponse { data: None },
                    )),
                    Err(e) => Err(e),
                }
            }
            main_command::Command::SetCache(req) => {
                let ttl_seconds = match req.ttl_seconds {
                    Some(0) | None => 7 * 24 * 3600,
                    Some(t) => t,
                };
                memory_cache
                    .lock()
                    .unwrap()
                    .put(req.key.clone(), req.data.clone(), ttl_seconds);
                reply_handler
                    .set_cache(package_name, &req.key, req.data, ttl_seconds)
                    .map(|success| {
                        main_command_response::Response::SetCache(SetCacheResponse { success })
                    })
            }
            main_command::Command::SpawnProcess(req) => reply_handler
                .spawn_process(package_name, req)
                .map(main_command_response::Response::SpawnProcess),
            main_command::Command::WriteProcessStdin(req) => reply_handler
                .write_process_stdin(package_name, req)
                .map(main_command_response::Response::WriteProcessStdin),
            main_command::Command::ReadProcessStdout(req) => reply_handler
                .read_process_stdout(package_name, req)
                .map(main_command_response::Response::ReadProcessStdout),
            main_command::Command::KillProcess(req) => reply_handler
                .kill_process(package_name, req)
                .map(main_command_response::Response::KillProcess),
            main_command::Command::GetProcessStatus(req) => reply_handler
                .get_process_status(package_name, req)
                .map(main_command_response::Response::GetProcessStatus),
            main_command::Command::NotifyPlayerEvent(req) => reply_handler
                .notify_player_event(package_name, req)
                .map(main_command_response::Response::NotifyPlayerEvent),
        }
    }
}

mod extism_context;
#[cfg(test)]
mod extism_context_test;
#[cfg(test)]
mod mod_test;

/// Represents the context of a single running extension instance.
#[async_trait::async_trait]
pub(crate) trait ExtensionContext: Debug + Send + Sync {
    async fn execute_command(
        &self,
        command: ExtensionCommand,
    ) -> Result<ExtensionCommandResponse, ExtensionError>;
}
