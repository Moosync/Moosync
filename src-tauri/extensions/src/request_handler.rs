use std::{fmt::format, sync::mpsc, time::Duration};

use database::database::Database;
use preferences::preferences::PreferenceConfig;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tauri::{AppHandle, Manager, State};
use types::{
    errors::errors::{MoosyncError, Result},
    songs::{GetSongOptions, SearchableSong, Song},
};

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

#[derive(Debug, Serialize, Deserialize, Clone)]
struct BasicRequest {
    #[serde(rename = "type")]
    type_: String,
    #[serde(rename = "extensionName")]
    extension_name: String,
    data: Option<Value>,
    channel: String,
}

pub struct ReplyHandler {
    app_handle: AppHandle,
}

impl ReplyHandler {
    pub fn new(app_handle: AppHandle) -> Self {
        ReplyHandler { app_handle }
    }

    fn is_main_command(&self, type_: &str) -> bool {
        [
            "get-songs",
            "get-entity",
            "add-songs",
            "update-song",
            "add-playlist",
            "add-song-to-playlist",
            "remove-song",
            "get-preferences",
            "get-secure-preferences",
            "set-preferences",
            "set-secure-preferences",
            "register-oauth",
            "open-external",
            "register-account",
            "set-artist-editable-info",
            "set-album-editable-info",
        ]
        .contains(&type_)
    }

    fn is_ui_request(&self, type_: &str) -> bool {
        [
            "get-current-song",
            "get-volume",
            "get-time",
            "get-queue",
            "get-player-state",
            "open-login-modal",
            "close-login-modal",
            "show-toast",
            "update-preferences",
            "extension-updated",
        ]
        .contains(&type_)
    }

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

    pub fn get_entity(&self, data: Value) -> Result<Value> {
        let database: State<'_, Database> = self.app_handle.state();
        let ret = database.get_entity_by_options(serde_json::from_value(data)?)?;
        Ok(serde_json::to_value(ret)?)
    }

    pub fn add_songs(&self, data: Value) -> Result<Value> {
        let database: State<'_, Database> = self.app_handle.state();
        // TODO: Add song
        // let ret = database.(serde_json::from_value(data)?)?;
        Ok(Value::Null)
    }

    pub fn update_song(&self, data: Value) -> Result<Value> {
        let database: State<'_, Database> = self.app_handle.state();
        database.update_songs(vec![serde_json::from_value(data.clone())?])?;
        Ok(data)
    }

    pub fn add_playlist(&self, data: Value) -> Result<Value> {
        let database: State<'_, Database> = self.app_handle.state();
        let ret = database.create_playlist(serde_json::from_value(data)?)?;
        Ok(serde_json::to_value(ret)?)
    }

    pub fn add_to_playlist(&self, data: Value) -> Result<Value> {
        let database: State<'_, Database> = self.app_handle.state();
        let request: AddToPlaylistRequest = serde_json::from_value(data)?;
        database.add_to_playlist(request.playlist_id, request.songs)?;
        Ok(Value::Null)
    }

    pub fn remove_song(&self, data: Value) -> Result<Value> {
        let database: State<'_, Database> = self.app_handle.state();
        let request: Song = serde_json::from_value(data)?;
        if let Some(song_id) = request.song._id {
            database.remove_songs(vec![song_id])?;
        }
        Ok(Value::Null)
    }

    pub fn set_artist_editable_info(&self, data: Value) -> Result<Value> {
        // TODO: Implement
        Ok(Value::Null)
    }

    pub fn set_album_editable_info(&self, data: Value) -> Result<Value> {
        // TODO: Implement
        Ok(Value::Null)
    }

    pub fn get_preferences(&self, package_name: String, data: Value) -> Result<Value> {
        let preferences: State<'_, PreferenceConfig> = self.app_handle.state();
        let request: PreferenceData = serde_json::from_value(data)?;
        let ret =
            preferences.load_selective(format!("extension.{}.{}", package_name, request.key))?;
        if ret.is_null() {
            return Ok(serde_json::to_value(&request.default_value)?);
        }
        Ok(serde_json::to_value(&ret)?)
    }

    pub fn set_preferences(&self, package_name: String, data: Value) -> Result<Value> {
        let preferences: State<'_, PreferenceConfig> = self.app_handle.state();
        let request: PreferenceData = serde_json::from_value(data)?;
        if let Some(value) = request.value {
            preferences
                .save_selective(format!("extension.{}.{}", package_name, request.key), value)?;
        }
        Ok(Value::Null)
    }

    pub fn get_secure(&self, package_name: String, data: Value) -> Result<Value> {
        let preferences: State<'_, PreferenceConfig> = self.app_handle.state();
        let request: PreferenceData = serde_json::from_value(data)?;
        let ret = preferences.get_secure(format!("extension.{}.{}", package_name, request.key))?;
        if ret.is_null() {
            return Ok(serde_json::to_value(&request.default_value)?);
        }
        Ok(serde_json::to_value(&ret)?)
    }

    pub fn set_secure(&self, package_name: String, data: Value) -> Result<Value> {
        let preferences: State<'_, PreferenceConfig> = self.app_handle.state();
        let request: PreferenceData = serde_json::from_value(data)?;
        if let Some(value) = request.value {
            preferences.set_secure(format!("extension.{}.{}", package_name, request.key), value)?;
        }
        Ok(Value::Null)
    }

    pub fn register_oauth(&self, data: Value) -> Result<Value> {
        // TODO: Implement oauth registration
        Ok(Value::Null)
    }

    pub fn open_external(&self, data: Value) -> Result<Value> {
        // TODO: Implement open external
        Ok(Value::Null)
    }

    pub fn register_account(&self, data: Value) -> Result<Value> {
        // TODO: Implement
        Ok(Value::Null)
    }

    fn send_ui_request(&self, request: BasicRequest) -> Result<Value> {
        if self.app_handle.webview_windows().is_empty() {
            return Ok(Value::Null);
        }

        let (tx, rx) = mpsc::channel::<String>();
        self.app_handle
            .once(format!("ui-reply-{}", request.channel), move |f| {
                let payload = f.payload().to_string();
                println!("event payload {:?}", payload);
                let _ = tx.send(payload);
            });
        println!("Sending ui request {:?}", request);
        self.app_handle.emit("ui-requests", request.clone())?;
        println!("sent ui request {:?}", request);

        let res = rx.recv_timeout(Duration::from_secs(1));
        if let Ok(data) = res {
            println!("got ui reply {:?}", data);
            Ok(serde_json::from_str(&data)?)
        } else {
            Ok(Value::Null)
        }

        // Ok(Value::Null)
    }

    pub fn handle_request(&self, value: &Value) -> Result<Vec<u8>> {
        let request: BasicRequest = serde_json::from_value(value.clone())?;
        let mut ret = request.clone();

        let res = if self.is_main_command(&request.type_) {
            if request.data.is_none() {
                return Err(MoosyncError::String("Missing data field".into()));
            }

            match request.type_.as_str() {
                "get-songs" => self.get_songs(request.data.unwrap()),
                "get-entity" => self.get_entity(request.data.unwrap()),
                "add-songs" => self.add_songs(request.data.unwrap()),
                "update-song" => self.update_song(request.data.unwrap()),
                "add-playlist" => self.add_playlist(request.data.unwrap()),
                "add-song-to-playlist" => self.add_to_playlist(request.data.unwrap()),
                "remove-song" => self.remove_song(request.data.unwrap()),
                "get-preferences" => {
                    self.get_preferences(request.extension_name, request.data.unwrap())
                }
                "get-secure-preferences" => {
                    self.get_secure(request.extension_name, request.data.unwrap())
                }
                "set-preferences" => {
                    self.set_preferences(request.extension_name, request.data.unwrap())
                }
                "set-secure-preferences" => {
                    self.set_secure(request.extension_name, request.data.unwrap())
                }
                "register-oauth" => self.register_oauth(request.data.unwrap()),
                "open-external" => self.open_external(request.data.unwrap()),
                "register-account" => self.register_account(request.data.unwrap()),
                "set-artist-editable-info" => self.set_artist_editable_info(request.data.unwrap()),
                "set-album-editable-info" => self.set_album_editable_info(request.data.unwrap()),
                _ => unreachable!(),
            }?
        } else if self.is_ui_request(&request.type_) {
            self.send_ui_request(request)?
        } else {
            return Err(MoosyncError::String("Not a valid request".into()));
        };

        ret.data = Some(res);
        Ok(serde_json::to_vec(&ret)?)
    }
}
