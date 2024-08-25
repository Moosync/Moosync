use database::database::Database;
use futures::channel::oneshot;
use preferences::preferences::PreferenceConfig;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tauri::{AppHandle, Emitter, Listener, Manager, State};
use types::{
    errors::errors::{MoosyncError, Result},
    extensions::ExtensionUIRequest,
    songs::{GetSongOptions, SearchableSong, Song},
};

use crate::{providers::handler::ProviderHandler, window::handler::WindowHandler};

#[derive(Debug, Serialize, Deserialize)]
struct AddToPlaylistRequest {
    #[serde(rename = "playlistID")]
    playlist_id: String,
    songs: Vec<Song>,
}

#[derive(Debug, Serialize, Deserialize)]
struct PreferenceData {
    key: String,
    value: Option<Value>,
    #[serde(rename = "defaultValue")]
    default_value: Option<Value>,
}

#[derive(Clone)]
pub struct ReplyHandler {
    app_handle: AppHandle,
}

impl ReplyHandler {
    #[tracing::instrument(level = "trace", skip(app_handle))]
    pub fn new(app_handle: AppHandle) -> Self {
        ReplyHandler { app_handle }
    }

    #[tracing::instrument(level = "trace", skip(self, type_))]
    fn is_main_command(&self, type_: &str) -> bool {
        [
            "getSongs",
            "getEntity",
            "updateSong",
            "update-song",
            "addPlaylist",
            "addSongToPlaylist",
            "removeSong",
            "getPreferences",
            "getSecurePreferences",
            "setPreferences",
            "setSecurePreferences",
            "registerOauth",
            "openExternal",
            "registerAccount",
            "setArtistEditableInfo",
            "setAlbumEditableInfo",
        ]
        .contains(&type_)
    }

    #[tracing::instrument(level = "trace", skip(self, type_))]
    fn is_update(&self, type_: &str) -> bool {
        type_ == "extensionUpdated"
    }

    #[tracing::instrument(level = "trace", skip(self, type_))]
    fn is_ui_request(&self, type_: &str) -> bool {
        [
            "getCurrentSong",
            "getVolume",
            "getTime",
            "getQueue",
            "getPlayerState",
            "openLoginModal",
            "closeLoginModal",
            "showToast",
        ]
        .contains(&type_)
    }

    #[tracing::instrument(level = "trace", skip(self, data))]
    pub fn get_songs(&self, data: Value) -> Result<Value> {
        let database: State<'_, Database> = self.app_handle.state();
        let mut request: GetSongOptions = serde_json::from_value(data)?;

        if request.album.is_none()
            && request.artist.is_none()
            && request.album.is_none()
            && request.song.is_none()
        {
            request.song = Some(SearchableSong::default());
        }

        let ret = database.get_songs_by_options(request)?;
        Ok(serde_json::to_value(ret)?)
    }

    #[tracing::instrument(level = "trace", skip(self, data))]
    pub fn get_entity(&self, data: Value) -> Result<Value> {
        let database: State<'_, Database> = self.app_handle.state();
        let ret = database.get_entity_by_options(serde_json::from_value(data)?)?;
        Ok(serde_json::to_value(ret)?)
    }

    #[tracing::instrument(level = "trace", skip(self, data))]
    pub fn add_songs(&self, data: Value) -> Result<Value> {
        let database: State<'_, Database> = self.app_handle.state();
        // TODO: Add song
        let ret = database.insert_songs(serde_json::from_value(data.clone())?)?;
        Ok(serde_json::to_value(ret)?)
    }

    #[tracing::instrument(level = "trace", skip(self, data))]
    pub fn update_song(&self, data: Value) -> Result<Value> {
        let database: State<'_, Database> = self.app_handle.state();
        database.update_songs(vec![serde_json::from_value(data.clone())?])?;
        Ok(data)
    }

    #[tracing::instrument(level = "trace", skip(self, data))]
    pub fn add_playlist(&self, data: Value) -> Result<Value> {
        let database: State<'_, Database> = self.app_handle.state();
        let ret = database.create_playlist(serde_json::from_value(data)?)?;
        Ok(serde_json::to_value(ret)?)
    }

    #[tracing::instrument(level = "trace", skip(self, data))]
    pub fn add_to_playlist(&self, data: Value) -> Result<Value> {
        let database: State<'_, Database> = self.app_handle.state();
        let request: AddToPlaylistRequest = serde_json::from_value(data)?;
        database.add_to_playlist(request.playlist_id, request.songs)?;
        Ok(Value::Null)
    }

    #[tracing::instrument(level = "trace", skip(self, data))]
    pub fn remove_song(&self, data: Value) -> Result<Value> {
        let database: State<'_, Database> = self.app_handle.state();
        let request: Song = serde_json::from_value(data)?;
        if let Some(song_id) = request.song._id {
            database.remove_songs(vec![song_id])?;
        }
        Ok(Value::Null)
    }

    #[tracing::instrument(level = "trace", skip(self, _data))]
    pub fn set_artist_editable_info(&self, _data: Value) -> Result<Value> {
        // TODO: Implement
        Ok(Value::Null)
    }

    #[tracing::instrument(level = "trace", skip(self, _data))]
    pub fn set_album_editable_info(&self, _data: Value) -> Result<Value> {
        // TODO: Implement
        Ok(Value::Null)
    }

    #[tracing::instrument(level = "trace", skip(self, package_name, data))]
    pub fn get_preferences(&self, package_name: String, data: Value) -> Result<Value> {
        let preferences: State<'_, PreferenceConfig> = self.app_handle.state();
        let request: PreferenceData = serde_json::from_value(data)?;
        let ret: Result<Value> =
            preferences.load_selective(format!("extension.{}.{}", package_name, request.key));
        Ok(match ret {
            Ok(v) => v,
            Err(_) => serde_json::to_value(&request.default_value)?,
        })
    }

    #[tracing::instrument(level = "trace", skip(self, package_name, data))]
    pub fn set_preferences(&self, package_name: String, data: Value) -> Result<Value> {
        let preferences: State<'_, PreferenceConfig> = self.app_handle.state();
        let request: PreferenceData = serde_json::from_value(data)?;
        preferences.save_selective(
            format!("extension.{}.{}", package_name, request.key),
            request.value,
        )?;
        Ok(Value::Null)
    }

    #[tracing::instrument(level = "trace", skip(self, package_name, data))]
    pub fn get_secure(&self, package_name: String, data: Value) -> Result<Value> {
        let preferences: State<'_, PreferenceConfig> = self.app_handle.state();
        let request: PreferenceData = serde_json::from_value(data)?;
        preferences.get_secure(format!("extension.{}.{}", package_name, request.key))
    }

    #[tracing::instrument(level = "trace", skip(self, package_name, data))]
    pub fn set_secure(&self, package_name: String, data: Value) -> Result<Value> {
        let preferences: State<'_, PreferenceConfig> = self.app_handle.state();
        let request: PreferenceData = serde_json::from_value(data)?;
        preferences.set_secure(
            format!("extension.{}.{}", package_name, request.key),
            request.value,
        )?;

        Ok(Value::Null)
    }

    #[tracing::instrument(level = "trace", skip(self, _data))]
    pub fn register_oauth(&self, _data: Value) -> Result<Value> {
        // TODO: Implement oauth registration
        Ok(Value::Null)
    }

    #[tracing::instrument(level = "trace", skip(self, data))]
    pub fn open_external(&self, data: Value) -> Result<Value> {
        if data.is_string() {
            let window_handler: State<WindowHandler> = self.app_handle.state();
            window_handler.open_external(data.as_str().unwrap().into())?;
        }
        Ok(Value::Null)
    }

    #[tracing::instrument(level = "trace", skip(self, package_name))]
    fn register_account(&self, package_name: String) -> Result<Value> {
        let app_handle = self.app_handle.clone();
        tauri::async_runtime::spawn(async move {
            let provider_handler: State<ProviderHandler> = app_handle.state();
            provider_handler.initialize_provider(package_name).await;
        });
        Ok(Value::Null)
    }

    #[tracing::instrument(level = "trace", skip(self, request))]
    async fn send_ui_request(&self, request: ExtensionUIRequest) -> Result<Value> {
        if self.app_handle.webview_windows().is_empty() {
            return Ok(Value::Null);
        }

        let (tx, rx) = oneshot::channel();
        self.app_handle
            .once(format!("ui-reply-{}", request.channel), move |f| {
                let payload = f.payload().to_string();
                let _ = tx.send(payload);
            });
        tracing::info!("Sending ui request {:?}", request);
        self.app_handle.emit("ui-requests", request.clone())?;
        tracing::info!("sent ui request {:?}", request);

        let res = rx.await;
        if let Ok(data) = res {
            tracing::info!("got ui reply {:?}", data);
            Ok(serde_json::from_str(&data)?)
        } else {
            Ok(Value::Null)
        }

        // Ok(Value::Null)
    }

    #[tracing::instrument(level = "trace", skip(self))]
    pub async fn extension_updated(&self) -> Result<Value> {
        tracing::info!("Got extension updated");
        let provider_handle: State<ProviderHandler> = self.app_handle.state();
        provider_handle.discover_provider_extensions().await?;
        tracing::info!("Updated extension");
        Ok(Value::Null)
    }

    #[tracing::instrument(level = "trace", skip(self, value))]
    pub async fn handle_request(&self, value: &Value) -> Result<Vec<u8>> {
        let request: ExtensionUIRequest = serde_json::from_value(value.clone())?;
        let mut ret = request.clone();

        let res = if self.is_main_command(&request.type_) {
            if request.data.is_none() {
                return Err(MoosyncError::String("Missing data field".into()));
            }

            match request.type_.as_str() {
                "getSongs" => self.get_songs(request.data.unwrap()),
                "getEntity" => self.get_entity(request.data.unwrap()),
                "updateSong" => self.add_songs(request.data.unwrap()),
                "update-song" => self.update_song(request.data.unwrap()),
                "addPlaylist" => self.add_playlist(request.data.unwrap()),
                "addSongToPlaylist" => self.add_to_playlist(request.data.unwrap()),
                "removeSong" => self.remove_song(request.data.unwrap()),
                "getPreferences" => {
                    self.get_preferences(request.extension_name, request.data.unwrap())
                }
                "getSecurePreferences" => {
                    self.get_secure(request.extension_name, request.data.unwrap())
                }
                "setPreferences" => {
                    self.set_preferences(request.extension_name, request.data.unwrap())
                }
                "setSecurePreferences" => {
                    self.set_secure(request.extension_name, request.data.unwrap())
                }
                "registerOauth" => self.register_oauth(request.data.unwrap()),
                "openExternal" => self.open_external(request.data.unwrap()),
                "registerAccount" => self.register_account(request.extension_name),
                "setArtistEditableInfo" => self.set_artist_editable_info(request.data.unwrap()),
                "setAlbumEditableInfo" => self.set_album_editable_info(request.data.unwrap()),
                _ => unreachable!(),
            }
        } else if self.is_update(&request.type_) {
            self.extension_updated().await
        } else if self.is_ui_request(&request.type_) {
            self.send_ui_request(request).await
        } else {
            tracing::info!("Not a valid request {:?}", request);
            Ok(Value::Null)
        };

        match res {
            Ok(v) => ret.data = Some(v),
            Err(e) => {
                tracing::error!("Error handling request {:?}: {}", value, e);
            }
        }

        // TODO: Perform extension error handling
        Ok(serde_json::to_vec(&ret)?)
    }
}
