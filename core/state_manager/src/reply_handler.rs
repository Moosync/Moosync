use std::borrow::Cow;

use extensions::ReplyHandler;
use songs_proto::moosync::types::{EntityResult, GetEntityOptions, GetSongOptions, Playlist, Song};
use tokio::runtime::Handle;
use types::prelude::SongsExt;
use ui_proto::moosync::types::PreferenceUiData;

use crate::StateManager;

pub struct StateReplyHandler {
    state_manager: StateManager,
    runtime: Handle,
}

impl StateReplyHandler {
    pub fn new(state_manager: StateManager) -> Self {
        let runtime = state_manager.runtime.clone();
        Self {
            state_manager,
            runtime,
        }
    }
}

impl ReplyHandler for StateReplyHandler {
    fn get_song(
        &self,
        _package_name: &str,
        options: GetSongOptions,
    ) -> Result<Vec<Song>, types::errors::MoosyncError> {
        let db = self.runtime.block_on(self.state_manager.get_database());
        let songs = db.get_songs_by_options(options)?;
        Ok(songs)
    }

    fn get_entity(
        &self,
        _package_name: &str,
        options: GetEntityOptions,
    ) -> Result<EntityResult, types::errors::MoosyncError> {
        let db = self.runtime.block_on(self.state_manager.get_database());
        let entity_res = db.get_entity_by_options(options)?;
        Ok(entity_res)
    }

    fn get_current_song(
        &self,
        _package_name: &str,
    ) -> Result<Option<Song>, types::errors::MoosyncError> {
        let player = self
            .runtime
            .block_on(self.state_manager.get_player_handler());
        let song = player.get_current_song().cloned();
        Ok(song)
    }

    fn get_player_state(&self, _package_name: &str) -> Result<i32, types::errors::MoosyncError> {
        let player = self
            .runtime
            .block_on(self.state_manager.get_player_handler());
        let state = player.get_player_state();
        Ok(state)
    }

    fn get_volume(&self, _package_name: &str) -> Result<f64, types::errors::MoosyncError> {
        let player = self
            .runtime
            .block_on(self.state_manager.get_player_handler());
        let volume = player.get_volume() as f64;
        Ok(volume)
    }

    fn get_time(&self, _package_name: &str) -> Result<f64, types::errors::MoosyncError> {
        let player = self
            .runtime
            .block_on(self.state_manager.get_player_handler());
        let pos = player
            .get_current_pos()
            .unwrap_or(std::time::Duration::ZERO);
        Ok(pos.as_secs_f64())
    }

    fn get_queue(
        &self,
        _package_name: &str,
    ) -> Result<(Vec<Song>, usize), types::errors::MoosyncError> {
        let player = self
            .runtime
            .block_on(self.state_manager.get_player_handler());
        Ok((player.get_queue().to_vec(), player.get_current_idx()))
    }

    fn get_preference(
        &self,
        package_name: &str,
        key: &str,
    ) -> Result<Option<serde_json::Value>, types::errors::MoosyncError> {
        let prefs = self
            .runtime
            .block_on(self.state_manager.get_preference_config());
        let scoped_key = format!("{}.{}", package_name, key);
        let val: serde_json::Value = prefs
            .load_selective(scoped_key)
            .unwrap_or(serde_json::Value::Null);
        if val.is_null() {
            Ok(None)
        } else {
            Ok(Some(val))
        }
    }

    fn set_preference(
        &self,
        package_name: &str,
        key: &str,
        value: serde_json::Value,
    ) -> Result<bool, types::errors::MoosyncError> {
        let prefs = self
            .runtime
            .block_on(self.state_manager.get_preference_config());
        let scoped_key = format!("{}.{}", package_name, key);
        prefs.save_selective(scoped_key, Some(value))?;
        Ok(true)
    }

    fn get_secure(
        &self,
        package_name: &str,
        key: &str,
    ) -> Result<Option<serde_json::Value>, types::errors::MoosyncError> {
        let prefs = self
            .runtime
            .block_on(self.state_manager.get_preference_config());
        let scoped_key = format!("{}.{}", package_name, key);
        let val: serde_json::Value = prefs
            .get_secure(scoped_key)
            .unwrap_or(serde_json::Value::Null);
        if val.is_null() {
            Ok(None)
        } else {
            Ok(Some(val))
        }
    }

    fn set_secure(
        &self,
        package_name: &str,
        key: &str,
        value: serde_json::Value,
    ) -> Result<bool, types::errors::MoosyncError> {
        let prefs = self
            .runtime
            .block_on(self.state_manager.get_preference_config());
        let scoped_key = format!("{}.{}", package_name, key);
        prefs.set_secure(scoped_key, Some(value))?;
        Ok(true)
    }

    fn add_songs(
        &self,
        _package_name: &str,
        songs: Vec<Song>,
    ) -> Result<Vec<Song>, types::errors::MoosyncError> {
        let db = self.runtime.block_on(self.state_manager.get_database());
        let songs = db.insert_songs(songs)?;
        Ok(songs)
    }

    fn remove_song(
        &self,
        _package_name: &str,
        song: Song,
    ) -> Result<bool, types::errors::MoosyncError> {
        let db = self.runtime.block_on(self.state_manager.get_database());
        if let Some(id) = song.get_id() {
            let ids: &[_] = &[id.as_ref().to_string()];
            db.remove_songs(ids)?;
            return Ok(true);
        }
        Ok(false)
    }

    fn update_song(
        &self,
        _package_name: &str,
        song: Song,
    ) -> Result<Song, types::errors::MoosyncError> {
        let db = self.runtime.block_on(self.state_manager.get_database());
        if let Some(inner) = &song.song {
            db.update_song(inner)?;
        }
        Ok(song)
    }

    fn add_playlist(
        &self,
        _package_name: &str,
        playlist: Playlist,
    ) -> Result<String, types::errors::MoosyncError> {
        let db = self.runtime.block_on(self.state_manager.get_database());
        let playlist_id = db.create_playlist(playlist)?;
        Ok(playlist_id)
    }

    fn add_to_playlist(
        &self,
        _package_name: &str,
        playlist_id: String,
        songs: Vec<Song>,
    ) -> Result<bool, types::errors::MoosyncError> {
        let db = self.runtime.block_on(self.state_manager.get_database());
        db.add_to_playlist(&playlist_id, &songs)?;
        Ok(true)
    }

    fn register_oauth(
        &self,
        _package_name: &str,
        _url: String,
    ) -> Result<bool, types::errors::MoosyncError> {
        Ok(false)
    }

    fn open_external_url(
        &self,
        _package_name: &str,
        url: String,
    ) -> Result<bool, types::errors::MoosyncError> {
        #[cfg(target_os = "macos")]
        let status = std::process::Command::new("open").arg(&url).status();
        #[cfg(target_os = "windows")]
        let status = std::process::Command::new("cmd")
            .args(&["/C", "start", &url])
            .status();
        #[cfg(not(any(target_os = "macos", target_os = "windows")))]
        let status = std::process::Command::new("xdg-open").arg(&url).status();

        let success = status.map(|s| s.success()).unwrap_or(false);
        Ok(success)
    }

    fn update_accounts(
        &self,
        _package_name: &str,
        _account: Option<String>,
    ) -> Result<bool, types::errors::MoosyncError> {
        Ok(false)
    }

    fn register_user_preference(
        &self,
        package_name: &str,
        prefs: Vec<PreferenceUiData>,
    ) -> Result<bool, types::errors::MoosyncError> {
        let extensions = self
            .runtime
            .block_on(self.state_manager.get_extension_handler());
        extensions.register_ui_preferences(package_name.to_string(), prefs)?;
        Ok(true)
    }

    fn unregister_user_preference(
        &self,
        package_name: &str,
        keys: Vec<String>,
    ) -> Result<bool, types::errors::MoosyncError> {
        let extensions = self
            .runtime
            .block_on(self.state_manager.get_extension_handler());
        extensions.unregister_ui_preferences(package_name.to_string(), keys)?;
        Ok(true)
    }

    fn extensions_updated(&self, _package_name: &str) -> Result<(), types::errors::MoosyncError> {
        self.state_manager.on_extensions_updated.run_all(|cb| {
            cb(());
        });
        Ok(())
    }

    fn get_app_version(&self, _package_name: &str) -> Result<String, types::errors::MoosyncError> {
        let version = option_env!("CARGO_PKG_VERSION")
            .unwrap_or("1.17.0")
            .to_string();
        Ok(version)
    }
}
