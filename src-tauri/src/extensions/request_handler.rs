use database::database::Database;
use futures::channel::oneshot;
use preferences::preferences::PreferenceConfig;
use serde_json::Value;
use tauri::{AppHandle, Emitter, Listener, Manager, State};
use types::{
    entities::{GetEntityOptions, QueryablePlaylist},
    errors::{MoosyncError, Result},
    extensions::{MainCommand, MainCommandResponse},
    songs::{GetSongOptions, SearchableSong, Song},
    ui::extensions::ExtensionUIRequest,
    ui::extensions::{AddToPlaylistRequest, PreferenceData},
};

use crate::{providers::handler::ProviderHandler, window::handler::WindowHandler};

#[derive(Clone)]
pub struct ReplyHandler {
    app_handle: AppHandle,
}

impl ReplyHandler {
    #[tracing::instrument(level = "trace", skip(app_handle))]
    pub fn new(app_handle: AppHandle) -> Self {
        ReplyHandler { app_handle }
    }

    #[tracing::instrument(level = "trace", skip(self, data))]
    pub fn get_songs(&self, mut data: GetSongOptions) -> Result<MainCommandResponse> {
        let database: State<'_, Database> = self.app_handle.state();
        if data.album.is_none()
            && data.artist.is_none()
            && data.album.is_none()
            && data.song.is_none()
        {
            data.song = Some(SearchableSong::default());
        }

        let ret = database.get_songs_by_options(data)?;
        Ok(MainCommandResponse::GetSong(ret))
    }

    #[tracing::instrument(level = "trace", skip(self, data))]
    pub fn get_entity(&self, data: GetEntityOptions) -> Result<MainCommandResponse> {
        let database: State<'_, Database> = self.app_handle.state();
        let ret = database.get_entity_by_options(data)?;
        Ok(MainCommandResponse::GetEntity(ret))
    }

    #[tracing::instrument(level = "trace", skip(self, data))]
    pub fn add_songs(&self, data: Vec<Song>) -> Result<MainCommandResponse> {
        let database: State<'_, Database> = self.app_handle.state();
        let ret = database.insert_songs(data)?;
        Ok(MainCommandResponse::AddSongs(ret))
    }

    #[tracing::instrument(level = "trace", skip(self, data))]
    pub fn update_song(&self, data: Song) -> Result<MainCommandResponse> {
        let database: State<'_, Database> = self.app_handle.state();
        database.update_songs(vec![data.clone()])?;
        Ok(MainCommandResponse::UpdateSong(data))
    }

    #[tracing::instrument(level = "trace", skip(self, data))]
    pub fn add_playlist(&self, data: QueryablePlaylist) -> Result<MainCommandResponse> {
        let database: State<'_, Database> = self.app_handle.state();
        let ret = database.create_playlist(data)?;
        Ok(MainCommandResponse::AddPlaylist(ret))
    }

    #[tracing::instrument(level = "trace", skip(self, data))]
    pub fn add_to_playlist(
        &self,
        playlist_id: String,
        data: Vec<Song>,
    ) -> Result<MainCommandResponse> {
        let database: State<'_, Database> = self.app_handle.state();
        if let Err(e) = database.add_to_playlist(playlist_id, data) {
            tracing::error!("Failed to add songs to playlist {:?}", e);
            Ok(MainCommandResponse::AddToPlaylist(false))
        } else {
            Ok(MainCommandResponse::AddToPlaylist(true))
        }
    }

    #[tracing::instrument(level = "trace", skip(self, data))]
    pub fn remove_song(&self, data: Song) -> Result<MainCommandResponse> {
        let database: State<'_, Database> = self.app_handle.state();
        if let Some(song_id) = data.song._id {
            database.remove_songs(vec![song_id])?;
        }
        Ok(MainCommandResponse::RemoveSong(true))
    }

    #[tracing::instrument(level = "trace", skip(self, data))]
    pub fn get_preferences(&self, data: PreferenceData) -> Result<MainCommandResponse> {
        let preferences: State<'_, PreferenceConfig> = self.app_handle.state();
        let ret: Result<Value> = preferences.load_selective(data.key.clone());
        Ok(match ret {
            Ok(v) => MainCommandResponse::GetPreference(PreferenceData {
                key: data.key,
                value: Some(v),
                default_value: None,
            }),
            Err(_) => MainCommandResponse::GetPreference(PreferenceData {
                key: data.key,
                value: data.default_value,
                default_value: None,
            }),
        })
    }

    #[tracing::instrument(level = "trace", skip(self, data))]
    pub fn set_preferences(&self, data: PreferenceData) -> Result<MainCommandResponse> {
        let preferences: State<'_, PreferenceConfig> = self.app_handle.state();
        preferences.save_selective(data.key, data.value)?;
        Ok(MainCommandResponse::SetPreference(true))
    }

    #[tracing::instrument(level = "trace", skip(self, data))]
    pub fn get_secure(&self, data: PreferenceData) -> Result<MainCommandResponse> {
        let preferences: State<'_, PreferenceConfig> = self.app_handle.state();
        let val = preferences.get_secure(data.key.clone())?;
        Ok(MainCommandResponse::GetPreference(PreferenceData {
            key: data.key,
            value: val,
            default_value: None,
        }))
    }

    #[tracing::instrument(level = "trace", skip(self, data))]
    pub fn set_secure(&self, data: PreferenceData) -> Result<MainCommandResponse> {
        let preferences: State<'_, PreferenceConfig> = self.app_handle.state();
        preferences.set_secure(data.key, data.value)?;

        Ok(MainCommandResponse::SetSecure(true))
    }

    #[tracing::instrument(level = "trace", skip(self, _data))]
    pub fn register_oauth(&self, _data: String) -> Result<MainCommandResponse> {
        // TODO: Implement oauth registration
        Ok(MainCommandResponse::RegisterOAuth(false))
    }

    #[tracing::instrument(level = "trace", skip(self, data))]
    pub fn open_external(&self, data: String) -> Result<MainCommandResponse> {
        let window_handler: State<WindowHandler> = self.app_handle.state();
        window_handler.open_external(self.app_handle.clone(), data)?;

        Ok(MainCommandResponse::OpenExternalUrl(true))
    }

    fn update_accounts(&self, key: Option<String>) -> Result<MainCommandResponse> {
        let app_handle = self.app_handle.clone();
        if let Some(key) = key {
            tauri::async_runtime::spawn(async move {
                let provider_handler: State<ProviderHandler> = app_handle.state();
                let _ = provider_handler.request_account_status(key).await;
            });
        }
        Ok(MainCommandResponse::UpdateAccounts(true))
    }

    #[tracing::instrument(level = "trace", skip(self, command))]
    async fn send_ui_request(&self, mut command: MainCommand) -> Result<MainCommandResponse> {
        if self.app_handle.webview_windows().is_empty() {
            return Err("No webview spawned yet".into());
        }

        let (tx, rx) = oneshot::channel();
        let request = command.to_ui_request()?;
        self.app_handle
            .once(format!("ui-reply-{}", request.channel), move |f| {
                let payload = f.payload().to_string();
                let _ = tx.send(payload);
            });
        tracing::debug!("Sending ui request {:?}", request);
        self.app_handle.emit("ui-requests", request.clone())?;
        tracing::debug!("sent ui request {:?}", request);

        let res = rx.await;

        match res {
            Ok(data) => match command {
                MainCommand::GetCurrentSong() => Ok(MainCommandResponse::GetCurrentSong(
                    serde_json::from_str(&data)?,
                )),
                MainCommand::GetPlayerState() => Ok(MainCommandResponse::GetPlayerState(
                    serde_json::from_str(&data)?,
                )),
                MainCommand::GetVolume() => {
                    Ok(MainCommandResponse::GetVolume(serde_json::from_str(&data)?))
                }
                MainCommand::GetTime() => {
                    Ok(MainCommandResponse::GetTime(serde_json::from_str(&data)?))
                }
                MainCommand::GetQueue() => {
                    Ok(MainCommandResponse::GetQueue(serde_json::from_str(&data)?))
                }
                _ => Err("Not a ui request".into()),
            },
            Err(_) => Err("Failed to get response from UI".into()),
        }
    }

    #[tracing::instrument(level = "trace", skip(self))]
    pub async fn extension_updated(&self) -> Result<MainCommandResponse> {
        tracing::debug!("Got extension updated");
        let provider_handle: State<ProviderHandler> = self.app_handle.state();
        provider_handle.discover_provider_extensions().await?;
        tracing::debug!("Updated extension");
        Ok(MainCommandResponse::ExtensionsUpdated(true))
    }

    #[tracing::instrument(level = "trace", skip(self))]
    pub async fn handle_request(&self, command: MainCommand) -> Result<MainCommandResponse> {
        tracing::debug!("Got request from extension {:?}", command);

        Ok(match command {
            MainCommand::GetSong(get_song_options) => self.get_songs(get_song_options)?,
            MainCommand::GetEntity(get_entity_options) => self.get_entity(get_entity_options)?,
            MainCommand::GetCurrentSong()
            | MainCommand::GetPlayerState()
            | MainCommand::GetVolume()
            | MainCommand::GetTime()
            | MainCommand::GetQueue() => self.send_ui_request(command).await?,
            MainCommand::GetPreference(preference_data) => self.get_preferences(preference_data)?,
            MainCommand::SetPreference(preference_data) => self.set_preferences(preference_data)?,
            MainCommand::GetSecure(preference_data) => self.get_secure(preference_data)?,
            MainCommand::SetSecure(preference_data) => self.set_secure(preference_data)?,
            MainCommand::AddSongs(vec) => self.add_songs(vec)?,
            MainCommand::RemoveSong(song) => self.remove_song(song)?,
            MainCommand::UpdateSong(song) => self.update_song(song)?,
            MainCommand::AddPlaylist(queryable_playlist) => {
                self.add_playlist(queryable_playlist)?
            }
            MainCommand::AddToPlaylist(add_to_playlist_request) => self.add_to_playlist(
                add_to_playlist_request.playlist_id,
                add_to_playlist_request.songs,
            )?,
            MainCommand::RegisterOAuth(url) => self.register_oauth(url)?,
            MainCommand::OpenExternalUrl(url) => self.open_external(url)?,
            MainCommand::UpdateAccounts(key) => self.update_accounts(key)?,
            MainCommand::ExtensionsUpdated() => self.extension_updated().await?,
        })
    }
}
