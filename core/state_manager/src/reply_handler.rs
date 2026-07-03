use extensions::{ExtensionError, ReplyHandler};
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
    #[tracing::instrument(level = "debug", skip_all)]
    pub fn new(state_manager: StateManager) -> Self {
        let runtime = state_manager.runtime.clone();
        Self {
            state_manager,
            runtime,
        }
    }
}

impl ReplyHandler for StateReplyHandler {
    #[tracing::instrument(level = "debug", skip_all)]
    fn get_song(
        &self,
        _package_name: &str,
        options: GetSongOptions,
    ) -> Result<Vec<Song>, ExtensionError> {
        let db = self.runtime.block_on(self.state_manager.get_database());
        let songs = db
            .get_songs_by_options(options)
            .map_err(|e| ExtensionError::Sanitize(e.to_string()))?;
        Ok(songs)
    }

    #[tracing::instrument(level = "debug", skip_all)]
    fn get_entity(
        &self,
        _package_name: &str,
        options: GetEntityOptions,
    ) -> Result<EntityResult, ExtensionError> {
        let db = self.runtime.block_on(self.state_manager.get_database());
        let entity_res = db
            .get_entity_by_options(options)
            .map_err(|e| ExtensionError::Sanitize(e.to_string()))?;
        Ok(entity_res)
    }

    #[tracing::instrument(level = "debug", skip_all)]
    fn get_current_song(&self, _package_name: &str) -> Result<Option<Song>, ExtensionError> {
        let player = self
            .runtime
            .block_on(self.state_manager.get_player_handler());
        let song = player.get_current_song().cloned();
        Ok(song)
    }

    #[tracing::instrument(level = "debug", skip_all)]
    fn get_player_state(&self, _package_name: &str) -> Result<i32, ExtensionError> {
        let player = self
            .runtime
            .block_on(self.state_manager.get_player_handler());
        let state = player.get_player_state();
        Ok(state)
    }

    #[tracing::instrument(level = "debug", skip_all)]
    fn get_volume(&self, _package_name: &str) -> Result<f64, ExtensionError> {
        let player = self
            .runtime
            .block_on(self.state_manager.get_player_handler());
        let volume = player.get_volume() as f64;
        Ok(volume)
    }

    #[tracing::instrument(level = "debug", skip_all)]
    fn get_time(&self, _package_name: &str) -> Result<f64, ExtensionError> {
        let player = self
            .runtime
            .block_on(self.state_manager.get_player_handler());
        let pos = player
            .get_current_pos()
            .unwrap_or(std::time::Duration::ZERO);
        Ok(pos.as_secs_f64())
    }

    #[tracing::instrument(level = "debug", skip_all)]
    fn get_queue(&self, _package_name: &str) -> Result<(Vec<Song>, usize), ExtensionError> {
        let player = self
            .runtime
            .block_on(self.state_manager.get_player_handler());
        Ok((player.get_queue().to_vec(), player.get_current_idx()))
    }

    #[tracing::instrument(level = "debug", skip_all)]
    fn get_preference(
        &self,
        package_name: &str,
        key: &str,
    ) -> Result<Option<extensions_proto::struct_proto::google::protobuf::Value>, ExtensionError>
    {
        let prefs = self
            .runtime
            .block_on(self.state_manager.get_preference_config());
        let ext_key = preferences::keys::ExtensionKey {
            package_name: package_name.to_string(),
            key: key.to_string(),
        };
        match prefs.inner.load(ext_key) {
            Ok(val) => Ok(Some(val)),
            Err(_) => Ok(None),
        }
    }

    #[tracing::instrument(level = "debug", skip_all)]
    fn set_preference(
        &self,
        package_name: &str,
        key: &str,
        value: extensions_proto::struct_proto::google::protobuf::Value,
    ) -> Result<bool, ExtensionError> {
        let prefs = self
            .runtime
            .block_on(self.state_manager.get_preference_config());
        let ext_key = preferences::keys::ExtensionKey {
            package_name: package_name.to_string(),
            key: key.to_string(),
        };
        prefs
            .inner
            .save(ext_key, value)
            .map_err(|e| ExtensionError::Sanitize(e.to_string()))?;
        Ok(true)
    }

    #[tracing::instrument(level = "debug", skip_all)]
    fn get_secure(
        &self,
        package_name: &str,
        key: &str,
    ) -> Result<Option<extensions_proto::struct_proto::google::protobuf::Value>, ExtensionError>
    {
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
            let serialized = serde_json::to_string(&val)?;
            let proto_val: extensions_proto::struct_proto::google::protobuf::Value =
                serde_json::from_str(&serialized)?;
            Ok(Some(proto_val))
        }
    }

    #[tracing::instrument(level = "debug", skip_all)]
    fn set_secure(
        &self,
        package_name: &str,
        key: &str,
        value: extensions_proto::struct_proto::google::protobuf::Value,
    ) -> Result<bool, ExtensionError> {
        let prefs = self
            .runtime
            .block_on(self.state_manager.get_preference_config());
        let scoped_key = format!("{}.{}", package_name, key);
        let serialized = serde_json::to_string(&value)?;
        let val: serde_json::Value = serde_json::from_str(&serialized)?;
        prefs
            .set_secure(scoped_key, Some(val))
            .map_err(|e| ExtensionError::Sanitize(e.to_string()))?;
        Ok(true)
    }

    #[tracing::instrument(level = "debug", skip_all)]
    fn add_songs(
        &self,
        _package_name: &str,
        songs: Vec<Song>,
    ) -> Result<Vec<Song>, ExtensionError> {
        let db = self.runtime.block_on(self.state_manager.get_database());
        let songs = db
            .insert_songs(songs)
            .map_err(|e| ExtensionError::Sanitize(e.to_string()))?;
        Ok(songs)
    }

    #[tracing::instrument(level = "debug", skip_all)]
    fn remove_song(&self, _package_name: &str, song: Song) -> Result<bool, ExtensionError> {
        let db = self.runtime.block_on(self.state_manager.get_database());
        if let Some(id) = song.get_id() {
            let ids: &[_] = &[id.as_ref().to_string()];
            db.remove_songs(ids)
                .map_err(|e| ExtensionError::Sanitize(e.to_string()))?;
            return Ok(true);
        }
        Ok(false)
    }

    #[tracing::instrument(level = "debug", skip_all)]
    fn update_song(&self, _package_name: &str, song: Song) -> Result<Song, ExtensionError> {
        let db = self.runtime.block_on(self.state_manager.get_database());
        if let Some(inner) = &song.song {
            db.update_song(inner)
                .map_err(|e| ExtensionError::Sanitize(e.to_string()))?;
        }
        Ok(song)
    }

    #[tracing::instrument(level = "debug", skip_all)]
    fn add_playlist(
        &self,
        _package_name: &str,
        playlist: Playlist,
    ) -> Result<String, ExtensionError> {
        let db = self.runtime.block_on(self.state_manager.get_database());
        let playlist_id = db
            .create_playlist(playlist)
            .map_err(|e| ExtensionError::Sanitize(e.to_string()))?;
        Ok(playlist_id)
    }

    #[tracing::instrument(level = "debug", skip_all)]
    fn add_to_playlist(
        &self,
        _package_name: &str,
        playlist_id: String,
        songs: Vec<Song>,
    ) -> Result<bool, ExtensionError> {
        let db = self.runtime.block_on(self.state_manager.get_database());
        db.add_to_playlist(&playlist_id, &songs)
            .map_err(|e| ExtensionError::Sanitize(e.to_string()))?;
        Ok(true)
    }

    #[tracing::instrument(level = "debug", skip_all)]
    fn register_oauth(&self, _package_name: &str, _url: String) -> Result<bool, ExtensionError> {
        Ok(false)
    }

    #[tracing::instrument(level = "debug", skip_all)]
    fn open_external_url(&self, _package_name: &str, url: String) -> Result<bool, ExtensionError> {
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

    #[tracing::instrument(level = "debug", skip_all)]
    fn update_accounts(
        &self,
        _package_name: &str,
        _account: Option<String>,
    ) -> Result<bool, ExtensionError> {
        Ok(false)
    }

    #[tracing::instrument(level = "debug", skip_all)]
    fn register_user_preference(
        &self,
        package_name: &str,
        prefs: Vec<PreferenceUiData>,
    ) -> Result<bool, ExtensionError> {
        let extensions = self
            .runtime
            .block_on(self.state_manager.get_extension_handler_mut());
        let extension = extensions.get_extension(package_name)?;
        extension.register_ui_preferences(prefs);
        Ok(true)
    }

    #[tracing::instrument(level = "debug", skip_all)]
    fn unregister_user_preference(
        &self,
        package_name: &str,
        keys: Vec<String>,
    ) -> Result<bool, ExtensionError> {
        let extensions = self
            .runtime
            .block_on(self.state_manager.get_extension_handler_mut());
        let extension = extensions.get_extension(package_name)?;
        extension.unregister_ui_preferences(keys);
        Ok(true)
    }

    #[tracing::instrument(level = "debug", skip_all)]
    fn extensions_updated(&self, _package_name: &str) -> Result<(), ExtensionError> {
        let state_manager = self.state_manager.clone();
        self.runtime.spawn(async move {
            let extensions = state_manager.get_extension_handler().await;
            extensions.trigger_extensions_updated();
        });
        Ok(())
    }

    #[tracing::instrument(level = "debug", skip_all)]
    fn get_app_version(&self, _package_name: &str) -> Result<String, ExtensionError> {
        let version = option_env!("CARGO_PKG_VERSION").unwrap_or("").to_string();
        Ok(version)
    }
}
