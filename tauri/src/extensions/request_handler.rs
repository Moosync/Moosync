use database::database::Database;
use extensions::ExtensionHandler;
use extensions_proto::moosync::types::{
    AddPlaylistResponse, AddSongsResponse, AddToPlaylistResponse, ExtensionUiRequest,
    ExtensionsUpdatedResponse, GetAppVersionResponse, GetCurrentSongResponse, GetEntityResponse,
    GetPlayerStateResponse, GetPreferenceResponse, GetQueueResponse, GetSecureResponse,
    GetSongResponse, GetTimeResponse, GetVolumeResponse, MainCommand, MainCommandResponse,
    OpenExternalUrlResponse, PreferenceData, RegisterOauthResponse, RegisterUserPreferenceResponse,
    RemoveSongResponse, SetPreferenceResponse, SetSecureResponse, UnregisterUserPreferenceResponse,
    UpdateAccountsResponse, UpdateSongResponse, main_command, main_command_response,
};
use extensions_proto::struct_proto::google;
use futures::channel::oneshot;
use preferences::preferences::PreferenceConfig;
use songs_proto::moosync::types::{
    GetEntityOptions, GetSongOptions, Playlist, SearchableSong, Song,
};
use tauri::{AppHandle, Emitter, Listener, Manager, State};
use types::{
    errors::{Result, error_helpers},
    prelude::SongsExt,
};
use ui_proto::moosync::types::PreferenceUiData;

use crate::{
    oauth::handler::OAuthHandler, providers::handler::ProviderHandler,
    window::handler::WindowHandler,
};

#[derive(Clone)]
pub struct ReplyHandler {
    app_handle: AppHandle,
}

impl ReplyHandler {
    #[tracing::instrument(level = "debug", skip(app_handle))]
    pub fn new(app_handle: AppHandle) -> Self {
        ReplyHandler { app_handle }
    }

    #[tracing::instrument(level = "debug", skip(self, data))]
    pub fn get_songs(&self, mut data: GetSongOptions) -> Result<main_command_response::Response> {
        let database: State<'_, Database> = self.app_handle.state();
        if data.album.is_none()
            && data.artist.is_none()
            && data.album.is_none()
            && data.song.is_none()
        {
            data.song = Some(SearchableSong::default());
        }

        let ret = database.get_songs_by_options(data)?;
        Ok(main_command_response::Response::GetSong(GetSongResponse {
            songs: ret,
        }))
    }

    #[tracing::instrument(level = "debug", skip(self, data))]
    pub fn get_entity(&self, data: GetEntityOptions) -> Result<main_command_response::Response> {
        let database: State<'_, Database> = self.app_handle.state();
        let ret = database.get_entity_by_options(data)?;
        Ok(main_command_response::Response::GetEntity(
            GetEntityResponse {
                entity: Some(serde_json::from_value(ret)?),
            },
        ))
    }

    #[tracing::instrument(level = "debug", skip(self, data))]
    pub fn add_songs(&self, data: Vec<Song>) -> Result<main_command_response::Response> {
        let database: State<'_, Database> = self.app_handle.state();
        let ret = database.insert_songs(data)?;
        Ok(main_command_response::Response::AddSongs(
            AddSongsResponse { songs: ret },
        ))
    }

    #[tracing::instrument(level = "debug", skip(self, data))]
    pub fn update_song(&self, data: Song) -> Result<main_command_response::Response> {
        let database: State<'_, Database> = self.app_handle.state();
        database.update_songs(vec![data.clone()])?;
        Ok(main_command_response::Response::UpdateSong(
            UpdateSongResponse { song: Some(data) },
        ))
    }

    #[tracing::instrument(level = "debug", skip(self, data))]
    pub fn add_playlist(&self, data: Playlist) -> Result<main_command_response::Response> {
        let database: State<'_, Database> = self.app_handle.state();
        let ret = database.create_playlist(data)?;
        Ok(main_command_response::Response::AddPlaylist(
            AddPlaylistResponse { playlist_id: ret },
        ))
    }

    #[tracing::instrument(level = "debug", skip(self, data))]
    pub fn add_to_playlist(
        &self,
        playlist_id: String,
        data: Vec<Song>,
    ) -> Result<main_command_response::Response> {
        let database: State<'_, Database> = self.app_handle.state();
        let success = if let Err(e) = database.add_to_playlist(playlist_id, data) {
            tracing::error!("Failed to add songs to playlist {:?}", e);
            false
        } else {
            true
        };
        Ok(main_command_response::Response::AddToPlaylist(
            AddToPlaylistResponse { success },
        ))
    }

    #[tracing::instrument(level = "debug", skip(self, data))]
    pub fn remove_song(&self, data: Song) -> Result<main_command_response::Response> {
        let database: State<'_, Database> = self.app_handle.state();
        if let Some(song_id) = data.get_id() {
            database.remove_songs(vec![song_id])?;
        }
        Ok(main_command_response::Response::RemoveSong(
            RemoveSongResponse { success: true },
        ))
    }

    #[tracing::instrument(level = "debug", skip(self, data))]
    pub fn get_preferences(&self, data: PreferenceData) -> Result<main_command_response::Response> {
        let preferences: State<'_, PreferenceConfig> = self.app_handle.state();
        let ret: Result<google::protobuf::Value> = preferences.load_selective(data.key.clone());
        let response_data = PreferenceData {
            key: data.key,
            value: ret.ok(),
        };

        Ok(main_command_response::Response::GetPreference(
            GetPreferenceResponse {
                data: Some(response_data),
            },
        ))
    }

    #[tracing::instrument(level = "debug", skip(self, data))]
    pub fn set_preferences(&self, data: PreferenceData) -> Result<main_command_response::Response> {
        let preferences: State<'_, PreferenceConfig> = self.app_handle.state();
        preferences.save_selective(data.key, data.value)?;

        Ok(main_command_response::Response::SetPreference(
            SetPreferenceResponse { success: true },
        ))
    }

    #[tracing::instrument(level = "debug", skip(self, data))]
    pub fn get_secure(&self, data: PreferenceData) -> Result<main_command_response::Response> {
        let preferences: State<'_, PreferenceConfig> = self.app_handle.state();
        let ret: Result<google::protobuf::Value> = preferences.get_secure(data.key.clone());
        Ok(main_command_response::Response::GetSecure(
            GetSecureResponse {
                data: Some(PreferenceData {
                    key: data.key,
                    value: ret.ok(),
                }),
            },
        ))
    }

    #[tracing::instrument(level = "debug", skip(self, data))]
    pub fn set_secure(&self, data: PreferenceData) -> Result<main_command_response::Response> {
        let preferences: State<'_, PreferenceConfig> = self.app_handle.state();
        preferences.set_secure(data.key, data.value)?;

        Ok(main_command_response::Response::SetSecure(
            SetSecureResponse { success: true },
        ))
    }

    #[tracing::instrument(level = "debug", skip(self))]
    pub fn register_oauth(
        &self,
        data: String,
        ext: String,
    ) -> Result<main_command_response::Response> {
        let oauth: State<'_, OAuthHandler> = self.app_handle.state();
        oauth.register_oauth_path(data, format!("extension:{ext}"));
        Ok(main_command_response::Response::RegisterOauth(
            RegisterOauthResponse { success: true },
        ))
    }

    #[tracing::instrument(level = "debug", skip(self, data))]
    pub fn open_external(&self, data: String) -> Result<main_command_response::Response> {
        let window_handler: State<WindowHandler> = self.app_handle.state();
        window_handler.open_external(self.app_handle.clone(), data)?;

        Ok(main_command_response::Response::OpenExternalUrl(
            OpenExternalUrlResponse { success: true },
        ))
    }

    async fn update_accounts(
        &self,
        key: Option<String>,
    ) -> Result<main_command_response::Response> {
        let app_handle = self.app_handle.clone();
        if let Some(key) = key {
            let provider_handler: State<ProviderHandler> = app_handle.state();
            if let Err(e) = provider_handler
                .request_account_status(format!("extension:{}", &key).as_str())
                .await
            {
                tracing::error!("Failed to get account status from {}: {:?}", key, e);
                return Err("Failed to get account status".into());
            }
        }
        Ok(main_command_response::Response::UpdateAccounts(
            UpdateAccountsResponse { success: true },
        ))
    }

    async fn register_preferences(
        &self,
        package_name: String,
        prefs: Vec<PreferenceUiData>,
    ) -> Result<main_command_response::Response> {
        let ext_handler: State<ExtensionHandler> = self.app_handle.state();
        ext_handler
            .register_ui_preferences(package_name, prefs)
            .await?;
        Ok(main_command_response::Response::RegisterUserPreference(
            RegisterUserPreferenceResponse { success: true },
        ))
    }

    async fn unregister_preferences(
        &self,
        package_name: String,
        pref_keys: Vec<String>,
    ) -> Result<main_command_response::Response> {
        let ext_handler: State<ExtensionHandler> = self.app_handle.state();
        ext_handler
            .unregister_ui_preferences(package_name, pref_keys)
            .await?;
        Ok(main_command_response::Response::UnregisterUserPreference(
            UnregisterUserPreferenceResponse { success: true },
        ))
    }

    #[tracing::instrument(level = "debug", skip(self, command))]
    async fn send_ui_request(
        &self,
        command: MainCommand,
    ) -> Result<main_command_response::Response> {
        if self.app_handle.webview_windows().is_empty() {
            return Err("No webview spawned yet".into());
        }

        let (tx, rx) = oneshot::channel();
        let channel = uuid::Uuid::new_v4().to_string();
        self.app_handle
            .once(format!("ui-reply-{}", channel), move |f| {
                let payload = f.payload().to_string();
                let _ = tx.send(payload);
            });
        tracing::debug!("Sending ui request {:?}", command);
        self.app_handle
            .emit(
                "ui-requests",
                ExtensionUiRequest {
                    channel,
                    r#type: Some(command.clone()),
                },
            )
            .map_err(error_helpers::to_extension_error)?;

        let res = rx.await;

        match res {
            Ok(data) => match command.command {
                Some(main_command::Command::GetCurrentSong(_)) => Ok(
                    main_command_response::Response::GetCurrentSong(GetCurrentSongResponse {
                        song: Some(serde_json::from_str(&data)?),
                    }),
                ),
                Some(main_command::Command::GetPlayerState(_)) => Ok(
                    main_command_response::Response::GetPlayerState(GetPlayerStateResponse {
                        state: serde_json::from_str(&data)?,
                    }),
                ),
                Some(main_command::Command::GetVolume(_)) => Ok(
                    main_command_response::Response::GetVolume(GetVolumeResponse {
                        volume: serde_json::from_str(&data)?,
                    }),
                ),
                Some(main_command::Command::GetTime(_)) => {
                    Ok(main_command_response::Response::GetTime(GetTimeResponse {
                        time: serde_json::from_str(&data)?,
                    }))
                }
                Some(main_command::Command::GetQueue(_)) => Ok(
                    main_command_response::Response::GetQueue(GetQueueResponse {
                        queue: Some(serde_json::from_str(&data)?),
                    }),
                ),
                _ => Err("Not a ui request".into()),
            },
            Err(_) => Err("Failed to get response from UI".into()),
        }
    }

    #[tracing::instrument(level = "debug", skip(self))]
    pub async fn extension_updated(&self) -> Result<main_command_response::Response> {
        tracing::debug!("Got extension updated");
        let provider_handle: State<ProviderHandler> = self.app_handle.state();
        provider_handle.discover_provider_extensions().await?;
        tracing::debug!("Updated extension");
        Ok(main_command_response::Response::ExtensionsUpdated(
            ExtensionsUpdatedResponse {},
        ))
    }

    #[tracing::instrument(level = "debug", skip(self))]
    pub async fn handle_request(
        &self,
        ext: &str,
        command: MainCommand,
    ) -> Result<MainCommandResponse> {
        tracing::debug!("Got request from extension {:?}", command);
        let ext = ext.to_string();

        let res = match command.command {
            Some(main_command::Command::GetSong(req)) => {
                self.get_songs(req.options.unwrap_or_default())
            }
            Some(main_command::Command::GetEntity(req)) => {
                self.get_entity(req.options.unwrap_or_default())
            }
            Some(main_command::Command::GetCurrentSong(_))
            | Some(main_command::Command::GetPlayerState(_))
            | Some(main_command::Command::GetVolume(_))
            | Some(main_command::Command::GetTime(_))
            | Some(main_command::Command::GetQueue(_)) => self.send_ui_request(command).await,
            Some(main_command::Command::GetPreference(req)) => {
                self.get_preferences(req.data.unwrap_or_default())
            }
            Some(main_command::Command::SetPreference(req)) => {
                self.set_preferences(req.data.unwrap_or_default())
            }
            Some(main_command::Command::GetSecure(req)) => {
                self.get_secure(req.data.unwrap_or_default())
            }
            Some(main_command::Command::SetSecure(req)) => {
                self.set_secure(req.data.unwrap_or_default())
            }
            Some(main_command::Command::AddSongs(req)) => self.add_songs(req.songs),
            Some(main_command::Command::RemoveSong(req)) => {
                self.remove_song(req.song.unwrap_or_default())
            }
            Some(main_command::Command::UpdateSong(req)) => {
                self.update_song(req.song.unwrap_or_default())
            }
            Some(main_command::Command::AddPlaylist(req)) => {
                self.add_playlist(req.playlist.unwrap_or_default())
            }
            Some(main_command::Command::AddToPlaylist(req)) => {
                self.add_to_playlist(req.playlist_id, req.songs)
            }
            Some(main_command::Command::RegisterOauth(req)) => self.register_oauth(req.url, ext),
            Some(main_command::Command::OpenExternalUrl(req)) => self.open_external(req.url),
            Some(main_command::Command::UpdateAccounts(req)) => {
                self.update_accounts(req.account).await
            }
            Some(main_command::Command::ExtensionsUpdated(_)) => self.extension_updated().await,
            Some(main_command::Command::RegisterUserPreference(req)) => {
                self.register_preferences(ext, req.prefs).await
            }
            Some(main_command::Command::UnregisterUserPreference(req)) => {
                self.unregister_preferences(ext, req.keys).await
            }
            Some(main_command::Command::GetAppVersion(_)) => Ok(
                main_command_response::Response::GetAppVersion(GetAppVersionResponse {
                    version: env!("APP_VERSION").to_string(),
                }),
            ),
            None => Err("No command provided".into()),
        }?;

        Ok(MainCommandResponse {
            response: Some(res),
        })
    }
}
