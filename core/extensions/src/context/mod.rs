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

use std::fmt::Debug;

use extensions_proto::moosync::types::{
    AddPlaylistResponse, AddSongsResponse, AddToPlaylistResponse, ExtensionCommand,
    ExtensionCommandResponse, ExtensionsUpdatedResponse, GetAppVersionResponse,
    GetCurrentSongResponse, GetEntityResponse, GetPlayerStateResponse, GetPreferenceResponse,
    GetQueueResponse, GetSecureResponse, GetSongResponse, GetTimeResponse, GetVolumeResponse,
    OpenExternalUrlResponse, PreferenceData, RegisterOauthResponse, RegisterUserPreferenceResponse,
    RemoveSongResponse, SetPreferenceResponse, SetSecureResponse, UnregisterUserPreferenceResponse,
    UpdateAccountsResponse, UpdateSongResponse, main_command, main_command_response,
};
pub use extism_context::ExtismContext;
use songs_proto::moosync::types::{EntityResult, GetEntityOptions, GetSongOptions, Playlist, Song};
use ui_proto::moosync::types::PreferenceUiData;

use crate::errors::ExtensionError;

pub trait ReplyHandler: Send + Sync + 'static {
    fn get_song(
        &self,
        _package_name: &str,
        _options: GetSongOptions,
    ) -> Result<Vec<Song>, types::errors::MoosyncError> {
        Err(types::errors::MoosyncError::String(
            "Not implemented".to_string(),
        ))
    }
    fn get_entity(
        &self,
        _package_name: &str,
        _options: GetEntityOptions,
    ) -> Result<EntityResult, types::errors::MoosyncError> {
        Err(types::errors::MoosyncError::String(
            "Not implemented".to_string(),
        ))
    }
    fn get_current_song(
        &self,
        _package_name: &str,
    ) -> Result<Option<Song>, types::errors::MoosyncError> {
        Err(types::errors::MoosyncError::String(
            "Not implemented".to_string(),
        ))
    }
    fn get_player_state(&self, _package_name: &str) -> Result<i32, types::errors::MoosyncError> {
        Err(types::errors::MoosyncError::String(
            "Not implemented".to_string(),
        ))
    }
    fn get_volume(&self, _package_name: &str) -> Result<f64, types::errors::MoosyncError> {
        Err(types::errors::MoosyncError::String(
            "Not implemented".to_string(),
        ))
    }
    fn get_time(&self, _package_name: &str) -> Result<f64, types::errors::MoosyncError> {
        Err(types::errors::MoosyncError::String(
            "Not implemented".to_string(),
        ))
    }
    fn get_queue(
        &self,
        _package_name: &str,
    ) -> Result<(Vec<Song>, usize), types::errors::MoosyncError> {
        Err(types::errors::MoosyncError::String(
            "Not implemented".to_string(),
        ))
    }
    fn get_preference(
        &self,
        _package_name: &str,
        _key: &str,
    ) -> Result<
        Option<extensions_proto::struct_proto::google::protobuf::Value>,
        types::errors::MoosyncError,
    > {
        Err(types::errors::MoosyncError::String(
            "Not implemented".to_string(),
        ))
    }
    fn set_preference(
        &self,
        _package_name: &str,
        _key: &str,
        _value: extensions_proto::struct_proto::google::protobuf::Value,
    ) -> Result<bool, types::errors::MoosyncError> {
        Err(types::errors::MoosyncError::String(
            "Not implemented".to_string(),
        ))
    }
    fn get_secure(
        &self,
        _package_name: &str,
        _key: &str,
    ) -> Result<
        Option<extensions_proto::struct_proto::google::protobuf::Value>,
        types::errors::MoosyncError,
    > {
        Err(types::errors::MoosyncError::String(
            "Not implemented".to_string(),
        ))
    }
    fn set_secure(
        &self,
        _package_name: &str,
        _key: &str,
        _value: extensions_proto::struct_proto::google::protobuf::Value,
    ) -> Result<bool, types::errors::MoosyncError> {
        Err(types::errors::MoosyncError::String(
            "Not implemented".to_string(),
        ))
    }
    fn add_songs(
        &self,
        _package_name: &str,
        _songs: Vec<Song>,
    ) -> Result<Vec<Song>, types::errors::MoosyncError> {
        Err(types::errors::MoosyncError::String(
            "Not implemented".to_string(),
        ))
    }
    fn remove_song(
        &self,
        _package_name: &str,
        _song: Song,
    ) -> Result<bool, types::errors::MoosyncError> {
        Err(types::errors::MoosyncError::String(
            "Not implemented".to_string(),
        ))
    }
    fn update_song(
        &self,
        _package_name: &str,
        _song: Song,
    ) -> Result<Song, types::errors::MoosyncError> {
        Err(types::errors::MoosyncError::String(
            "Not implemented".to_string(),
        ))
    }
    fn add_playlist(
        &self,
        _package_name: &str,
        _playlist: Playlist,
    ) -> Result<String, types::errors::MoosyncError> {
        Err(types::errors::MoosyncError::String(
            "Not implemented".to_string(),
        ))
    }
    fn add_to_playlist(
        &self,
        _package_name: &str,
        _playlist_id: String,
        _songs: Vec<Song>,
    ) -> Result<bool, types::errors::MoosyncError> {
        Err(types::errors::MoosyncError::String(
            "Not implemented".to_string(),
        ))
    }
    fn register_oauth(
        &self,
        _package_name: &str,
        _url: String,
    ) -> Result<bool, types::errors::MoosyncError> {
        Err(types::errors::MoosyncError::String(
            "Not implemented".to_string(),
        ))
    }
    fn open_external_url(
        &self,
        _package_name: &str,
        _url: String,
    ) -> Result<bool, types::errors::MoosyncError> {
        Err(types::errors::MoosyncError::String(
            "Not implemented".to_string(),
        ))
    }
    fn update_accounts(
        &self,
        _package_name: &str,
        _account: Option<String>,
    ) -> Result<bool, types::errors::MoosyncError> {
        Err(types::errors::MoosyncError::String(
            "Not implemented".to_string(),
        ))
    }
    fn register_user_preference(
        &self,
        _package_name: &str,
        _prefs: Vec<PreferenceUiData>,
    ) -> Result<bool, types::errors::MoosyncError> {
        Err(types::errors::MoosyncError::String(
            "Not implemented".to_string(),
        ))
    }
    fn unregister_user_preference(
        &self,
        _package_name: &str,
        _keys: Vec<String>,
    ) -> Result<bool, types::errors::MoosyncError> {
        Err(types::errors::MoosyncError::String(
            "Not implemented".to_string(),
        ))
    }
    fn extensions_updated(&self, _package_name: &str) -> Result<(), types::errors::MoosyncError> {
        Err(types::errors::MoosyncError::String(
            "Not implemented".to_string(),
        ))
    }
    fn get_app_version(&self, _package_name: &str) -> Result<String, types::errors::MoosyncError> {
        Err(types::errors::MoosyncError::String(
            "Not implemented".to_string(),
        ))
    }
}

pub(crate) trait DispatchCommand {
    fn dispatch(
        self,
        reply_handler: &dyn ReplyHandler,
        package_name: &str,
    ) -> Result<main_command_response::Response, types::errors::MoosyncError>;
}

impl DispatchCommand for main_command::Command {
    fn dispatch(
        self,
        reply_handler: &dyn ReplyHandler,
        package_name: &str,
    ) -> Result<main_command_response::Response, types::errors::MoosyncError> {
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
                reply_handler
                    .get_queue(package_name)
                    .and_then(|(songs, index)| {
                        #[derive(serde::Serialize)]
                        struct QueueState {
                            songs: Vec<songs_proto::moosync::types::Song>,
                            index: usize,
                        }
                        let state = QueueState { songs, index };
                        let struct_val = serde_json::from_value(
                            serde_json::to_value(&state)
                                .map_err(|e| types::errors::MoosyncError::String(e.to_string()))?,
                        )
                        .map_err(|e| types::errors::MoosyncError::String(e.to_string()))?;
                        Ok(main_command_response::Response::GetQueue(
                            GetQueueResponse {
                                queue: Some(struct_val),
                            },
                        ))
                    })
            }
            main_command::Command::GetPreference(req) => {
                let key = req.data.as_ref().map(|d| d.key.clone()).unwrap_or_default();
                reply_handler
                    .get_preference(package_name, &key)
                    .map(|value| {
                        main_command_response::Response::GetPreference(GetPreferenceResponse {
                            data: Some(PreferenceData { key, value }),
                        })
                    })
            }
            main_command::Command::SetPreference(req) => {
                if let Some(data) = req.data {
                    let val = data.value.unwrap_or_default();
                    reply_handler
                        .set_preference(package_name, &data.key, val)
                        .map(|success| {
                            main_command_response::Response::SetPreference(SetPreferenceResponse {
                                success,
                            })
                        })
                } else {
                    Ok(main_command_response::Response::SetPreference(
                        SetPreferenceResponse { success: false },
                    ))
                }
            }
            main_command::Command::GetSecure(req) => {
                let key = req.data.as_ref().map(|d| d.key.clone()).unwrap_or_default();
                reply_handler.get_secure(package_name, &key).map(|value| {
                    main_command_response::Response::GetSecure(GetSecureResponse {
                        data: Some(PreferenceData { key, value }),
                    })
                })
            }
            main_command::Command::SetSecure(req) => {
                if let Some(data) = req.data {
                    let val = data.value.unwrap_or_default();
                    reply_handler
                        .set_secure(package_name, &data.key, val)
                        .map(|success| {
                            main_command_response::Response::SetSecure(SetSecureResponse {
                                success,
                            })
                        })
                } else {
                    Ok(main_command_response::Response::SetSecure(
                        SetSecureResponse { success: false },
                    ))
                }
            }
            main_command::Command::AddSongs(req) => reply_handler
                .add_songs(package_name, req.songs)
                .map(|songs| main_command_response::Response::AddSongs(AddSongsResponse { songs })),
            main_command::Command::RemoveSong(req) => {
                if let Some(song) = req.song {
                    reply_handler
                        .remove_song(package_name, song)
                        .map(|success| {
                            main_command_response::Response::RemoveSong(RemoveSongResponse {
                                success,
                            })
                        })
                } else {
                    Ok(main_command_response::Response::RemoveSong(
                        RemoveSongResponse { success: false },
                    ))
                }
            }
            main_command::Command::UpdateSong(req) => {
                if let Some(song) = req.song {
                    reply_handler.update_song(package_name, song).map(|song| {
                        main_command_response::Response::UpdateSong(UpdateSongResponse {
                            song: Some(song),
                        })
                    })
                } else {
                    Err(types::errors::MoosyncError::String(
                        "Missing song in update request".to_string(),
                    ))
                }
            }
            main_command::Command::AddPlaylist(req) => {
                if let Some(playlist) = req.playlist {
                    reply_handler
                        .add_playlist(package_name, playlist)
                        .map(|playlist_id| {
                            main_command_response::Response::AddPlaylist(AddPlaylistResponse {
                                playlist_id,
                            })
                        })
                } else {
                    Err(types::errors::MoosyncError::String(
                        "Missing playlist in add playlist request".to_string(),
                    ))
                }
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
            main_command::Command::UpdateAccounts(req) => reply_handler
                .update_accounts(package_name, req.account)
                .map(|success| {
                    main_command_response::Response::UpdateAccounts(UpdateAccountsResponse {
                        success,
                    })
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
        }
    }
}

mod extism_context;

/// Represents the context of a single running extension instance.
#[async_trait::async_trait]
pub(crate) trait ExtensionContext: Debug + Send + Sync {
    async fn execute_command(
        &self,
        command: ExtensionCommand,
    ) -> Result<ExtensionCommandResponse, ExtensionError>;
}
