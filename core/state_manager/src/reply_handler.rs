use std::time::{SystemTime, UNIX_EPOCH};

use extensions::{ExtensionError, ReplyHandler};
use extensions_proto::{
    moosync::types::{
        BatchHttpRequest, BatchHttpResponse, GetProcessStatusRequest, GetProcessStatusResponse,
        KillProcessRequest, KillProcessResponse, NotifyPlayerEventRequest,
        NotifyPlayerEventResponse, OpenClientFdRequest, OpenClientFdResponse,
        ReadProcessStdoutRequest, ReadProcessStdoutResponse, ReadSockRequest, ReadSockResponse,
        SpawnProcessRequest, SpawnProcessResponse, SystemTimeRequest, SystemTimeResponse,
        WriteProcessStdinRequest, WriteProcessStdinResponse, WriteSockRequest, WriteSockResponse,
        WsCloseRequest, WsCloseResponse, WsConnectRequest, WsConnectResponse, WsSendRequest,
        WsSendResponse,
    },
    struct_proto::google::protobuf::{Value as StructValue, value::Kind as StructValueKind},
};
use preferences_proto::moosync::types::{
    PreferenceItem, preference_value::Value as PreferenceRawValue,
};
use prost::Message;
use songs_proto::moosync::types::{Album, Artist, Genre, GetSongOptions, Playlist, Song};
use tokio::runtime::Handle;
use tracing::Instrument;
use types::prelude::SongsExt;

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

#[tracing::instrument(level = "debug", skip_all)]
fn preference_value_to_struct(raw_val: PreferenceRawValue) -> Option<StructValue> {
    let kind = match raw_val {
        PreferenceRawValue::StringValue(val) => StructValueKind::StringValue(val),
        PreferenceRawValue::BoolValue(val) => StructValueKind::BoolValue(val),
        PreferenceRawValue::NumberValue(val) => StructValueKind::NumberValue(f64::from(val)),
        _ => return None,
    };
    Some(StructValue { kind: Some(kind) })
}

#[tracing::instrument(level = "debug", skip_all)]
fn block_on_db<F: std::future::Future + Send>(future: F) -> F::Output
where
    F::Output: Send,
{
    if let Ok(handle) = tokio::runtime::Handle::try_current()
        && handle.runtime_flavor() == tokio::runtime::RuntimeFlavor::MultiThread
    {
        return tokio::task::block_in_place(|| handle.block_on(future));
    }
    std::thread::scope(|s| {
        s.spawn(|| {
            tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .expect("Failed to create Tokio runtime")
                .block_on(future)
        })
        .join()
        .expect("Database thread panicked")
    })
}

impl ReplyHandler for StateReplyHandler {
    #[tracing::instrument(level = "debug", skip_all)]
    fn get_song(
        &self,
        _package_name: &str,
        options: GetSongOptions,
    ) -> Result<Vec<Song>, ExtensionError> {
        let db = self.state_manager.get_database();
        let songs = block_on_db(db.get_songs_by_options(options))
            .map_err(|e| ExtensionError::Sanitize(e.to_string()))?;
        Ok(songs)
    }

    #[tracing::instrument(level = "debug", skip_all)]
    fn get_albums(
        &self,
        _package_name: &str,
        options: Album,
        inclusive: bool,
    ) -> Result<Vec<Album>, ExtensionError> {
        let db = self.state_manager.get_database();
        block_on_db(db.get_albums(options, inclusive))
            .map_err(|e| ExtensionError::Sanitize(e.to_string()))
    }

    #[tracing::instrument(level = "debug", skip_all)]
    fn get_artists(
        &self,
        _package_name: &str,
        options: Artist,
        inclusive: bool,
    ) -> Result<Vec<Artist>, ExtensionError> {
        let db = self.state_manager.get_database();
        block_on_db(db.get_artists(options, inclusive))
            .map_err(|e| ExtensionError::Sanitize(e.to_string()))
    }

    #[tracing::instrument(level = "debug", skip_all)]
    fn get_genres(
        &self,
        _package_name: &str,
        options: Genre,
        inclusive: bool,
    ) -> Result<Vec<Genre>, ExtensionError> {
        let db = self.state_manager.get_database();
        block_on_db(db.get_genres(options, inclusive))
            .map_err(|e| ExtensionError::Sanitize(e.to_string()))
    }

    #[tracing::instrument(level = "debug", skip_all)]
    fn get_playlists(
        &self,
        _package_name: &str,
        options: Playlist,
        inclusive: bool,
    ) -> Result<Vec<Playlist>, ExtensionError> {
        let db = self.state_manager.get_database();
        block_on_db(db.get_playlists(options, inclusive))
            .map_err(|e| ExtensionError::Sanitize(e.to_string()))
    }

    #[tracing::instrument(level = "debug", skip_all)]
    fn get_current_song(&self, _package_name: &str) -> Result<Option<Song>, ExtensionError> {
        let player = self.state_manager.get_player_handler();
        let song = player.get_current_song();
        Ok(song)
    }

    #[tracing::instrument(level = "debug", skip_all)]
    fn get_player_state(&self, _package_name: &str) -> Result<i32, ExtensionError> {
        let player = self.state_manager.get_player_handler();
        let state = player.get_player_state();
        Ok(state)
    }

    #[tracing::instrument(level = "debug", skip_all)]
    fn get_volume(&self, _package_name: &str) -> Result<f64, ExtensionError> {
        let player = self.state_manager.get_player_handler();
        let volume = player.get_volume() as f64;
        Ok(volume)
    }

    #[tracing::instrument(level = "debug", skip_all)]
    fn get_time(&self, _package_name: &str) -> Result<f64, ExtensionError> {
        let player = self.state_manager.get_player_handler();
        let pos = player
            .get_current_pos()
            .unwrap_or(std::time::Duration::ZERO);
        Ok(pos.as_secs_f64())
    }

    #[tracing::instrument(level = "debug", skip_all)]
    fn get_queue(&self, _package_name: &str) -> Result<(Vec<Song>, usize), ExtensionError> {
        let player = self.state_manager.get_player_handler();
        Ok((player.get_queue().to_vec(), player.get_current_idx()))
    }

    #[tracing::instrument(level = "debug", skip_all)]
    fn get_preference(
        &self,
        package_name: &str,
        key: &str,
    ) -> Result<Option<StructValue>, ExtensionError> {
        let prefs = self.state_manager.get_preference_config();
        let scoped_key = format!("{}.{}", package_name, key);
        if let Some(item) = prefs.get(&scoped_key)
            && let Some(val) = item.value
            && let Some(PreferenceRawValue::StringValue(hex_str)) = val.value
            && let Ok(bytes) = hex::decode(hex_str)
            && let Ok(proto_val) = StructValue::decode(&bytes[..])
        {
            return Ok(Some(proto_val));
        }

        let raw_key = key.strip_prefix(&format!("{package_name}.")).unwrap_or(key);
        let ui_key = format!("ext:{package_name}:{raw_key}");
        if let Some(item) = prefs.get(&ui_key)
            && let Some(val) = item.value
            && let Some(raw_val) = val.value
        {
            return Ok(preference_value_to_struct(raw_val));
        }

        if let Ok(extension) = self
            .state_manager
            .get_extension_handler()
            .get_extension(package_name)
            && let Some(reg_pref) = extension.get_ui_preference(raw_key)
            && let Some(default_val) = reg_pref.default.or(reg_pref.value)
            && let Some(raw_val) = default_val.value
        {
            return Ok(preference_value_to_struct(raw_val));
        }

        Ok(None)
    }

    #[tracing::instrument(level = "debug", skip_all)]
    fn set_preference(
        &self,
        package_name: &str,
        key: &str,
        value: extensions_proto::struct_proto::google::protobuf::Value,
    ) -> Result<bool, ExtensionError> {
        let prefs = self.state_manager.get_preference_config();
        let scoped_key = format!("{}.{}", package_name, key);
        let bytes = value.encode_to_vec();
        let item = PreferenceItem {
            id: scoped_key,
            value: Some(preferences_proto::moosync::types::PreferenceValue {
                value: Some(
                    preferences_proto::moosync::types::preference_value::Value::StringValue(
                        hex::encode(bytes),
                    ),
                ),
            }),
            ..Default::default()
        };
        prefs
            .save(item)
            .map_err(|e| ExtensionError::Sanitize(e.to_string()))?;
        Ok(true)
    }

    // TODO: Tie secure preferences to extension signing keys so an extension
    // with a different key cannot read prior secrets.
    #[tracing::instrument(level = "debug", skip_all)]
    fn get_secure(
        &self,
        package_name: &str,
        key: &str,
    ) -> Result<Option<extensions_proto::struct_proto::google::protobuf::Value>, ExtensionError>
    {
        let prefs = self.state_manager.get_preference_config();
        let scoped_key = format!("{}.{}", package_name, key);
        match prefs
            .get_secure::<extensions_proto::struct_proto::google::protobuf::Value>(scoped_key)
        {
            Ok(val) => Ok(Some(val)),
            Err(preferences::error::PreferencesError::KeyNotFound(_)) => Ok(None),
            Err(e) => Err(ExtensionError::Sanitize(e.to_string())),
        }
    }

    #[tracing::instrument(level = "debug", skip_all)]
    fn set_secure(
        &self,
        package_name: &str,
        key: &str,
        value: extensions_proto::struct_proto::google::protobuf::Value,
    ) -> Result<bool, ExtensionError> {
        let prefs = self.state_manager.get_preference_config();
        let scoped_key = format!("{}.{}", package_name, key);
        prefs
            .set_secure(scoped_key, Some(value))
            .map_err(|e| ExtensionError::Sanitize(e.to_string()))?;
        Ok(true)
    }

    #[tracing::instrument(level = "debug", skip_all)]
    fn add_songs(
        &self,
        _package_name: &str,
        songs: Vec<Song>,
    ) -> Result<Vec<Song>, ExtensionError> {
        let db = self.state_manager.get_database();
        let songs = block_on_db(db.insert_songs(songs))
            .map_err(|e| ExtensionError::Sanitize(e.to_string()))?;
        Ok(songs)
    }

    #[tracing::instrument(level = "debug", skip_all)]
    fn remove_song(&self, _package_name: &str, song: Song) -> Result<bool, ExtensionError> {
        let db = self.state_manager.get_database();
        if let Some(id) = song.get_id() {
            let ids: &[_] = &[id.as_ref().to_string()];
            block_on_db(db.remove_songs(ids))
                .map_err(|e| ExtensionError::Sanitize(e.to_string()))?;
            return Ok(true);
        }
        Ok(false)
    }

    #[tracing::instrument(level = "debug", skip_all)]
    fn update_song(&self, _package_name: &str, song: Song) -> Result<Song, ExtensionError> {
        let db = self.state_manager.get_database();
        if let Some(inner) = &song.song {
            block_on_db(db.update_song(inner))
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
        let db = self.state_manager.get_database();
        let playlist_id = block_on_db(db.create_playlist(playlist))
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
        let db = self.state_manager.get_database();
        block_on_db(db.add_to_playlist(&playlist_id, &songs))
            .map_err(|e| ExtensionError::Sanitize(e.to_string()))?;
        Ok(true)
    }

    #[tracing::instrument(level = "debug", skip_all)]
    fn register_oauth(&self, package_name: &str, url: String) -> Result<bool, ExtensionError> {
        self.state_manager
            .get_oauth_manager()
            .register_path(package_name.to_string(), url);
        Ok(true)
    }

    #[tracing::instrument(level = "debug", skip_all)]
    fn open_external_url(&self, _package_name: &str, url: String) -> Result<bool, ExtensionError> {
        #[cfg(target_os = "android")]
        let success = self
            .state_manager
            .get_android_context()
            .open_url(&url)
            .is_ok();
        #[cfg(not(target_os = "android"))]
        let success = open::that_detached(&url).is_ok();
        Ok(success)
    }

    #[tracing::instrument(level = "debug", skip_all)]
    fn set_account(
        &self,
        package_name: &str,
        account: extensions_proto::moosync::types::ExtensionAccountDetail,
    ) -> Result<bool, ExtensionError> {
        let extensions = self.state_manager.get_extension_handler();
        tracing::debug!("Setting account {:?} for {}", account, package_name);
        if !account.logged_in {
            let database = self.state_manager.get_database();
            if let Err(err) = block_on_db(database.remove_extension_cache(package_name)) {
                tracing::error!(
                    "Failed to clear extension cache on logout for {}: {:?}",
                    package_name,
                    err
                );
            }
        }
        let extension = extensions.get_extension(package_name)?;
        extension.set_account(account.clone());
        extensions.trigger_accounts_updated(Some(account.id));
        Ok(true)
    }

    #[tracing::instrument(level = "debug", skip_all)]
    fn register_user_preference(
        &self,
        package_name: &str,
        prefs: Vec<PreferenceItem>,
    ) -> Result<bool, ExtensionError> {
        let extensions = self.state_manager.get_extension_handler();
        let extension = extensions.get_extension(package_name)?;
        extension.register_ui_preferences(prefs);
        extensions.trigger_preferences_updated(package_name.to_string());
        Ok(true)
    }

    #[tracing::instrument(level = "debug", skip_all)]
    fn unregister_user_preference(
        &self,
        package_name: &str,
        keys: Vec<String>,
    ) -> Result<bool, ExtensionError> {
        let extensions = self.state_manager.get_extension_handler();
        let extension = extensions.get_extension(package_name)?;
        extension.unregister_ui_preferences(keys);
        extensions.trigger_preferences_updated(package_name.to_string());
        Ok(true)
    }

    #[tracing::instrument(level = "debug", skip_all)]
    fn extensions_updated(&self, _package_name: &str) -> Result<(), ExtensionError> {
        let state_manager = self.state_manager.clone();
        self.runtime.spawn(
            async move {
                let extensions = state_manager.get_extension_handler();
                extensions.trigger_extensions_updated();
            }
            .in_current_span(),
        );
        Ok(())
    }

    #[tracing::instrument(level = "debug", skip_all)]
    fn get_app_version(&self, _package_name: &str) -> Result<String, ExtensionError> {
        let version = option_env!("CARGO_PKG_VERSION").unwrap_or("").to_string();
        Ok(version)
    }

    #[tracing::instrument(level = "debug", skip_all)]
    fn get_cache(&self, package_name: &str, key: &str) -> Result<Option<Vec<u8>>, ExtensionError> {
        let db = self.state_manager.get_database();
        block_on_db(db.get_extension_cache(package_name, key))
            .map_err(|e| ExtensionError::Cache(e.to_string()))
    }

    #[tracing::instrument(level = "debug", skip_all)]
    fn set_cache(
        &self,
        package_name: &str,
        key: &str,
        data: Vec<u8>,
        ttl_seconds: u64,
    ) -> Result<bool, ExtensionError> {
        let db = self.state_manager.get_database();
        block_on_db(db.set_extension_cache(package_name, key, &data, ttl_seconds))
            .map_err(|e| ExtensionError::Cache(e.to_string()))?;
        Ok(true)
    }

    #[tracing::instrument(level = "debug", skip_all)]
    fn spawn_process(
        &self,
        package_name: &str,
        req: SpawnProcessRequest,
    ) -> Result<SpawnProcessResponse, ExtensionError> {
        let extensions = self.state_manager.get_extension_handler();
        let extension = extensions.get_extension(package_name)?;
        extension.process_manager.spawn_process(req)
    }

    #[tracing::instrument(level = "debug", skip_all)]
    fn write_process_stdin(
        &self,
        package_name: &str,
        req: WriteProcessStdinRequest,
    ) -> Result<WriteProcessStdinResponse, ExtensionError> {
        let extensions = self.state_manager.get_extension_handler();
        let extension = extensions.get_extension(package_name)?;
        extension.process_manager.write_process_stdin(req)
    }

    #[tracing::instrument(level = "debug", skip_all)]
    fn read_process_stdout(
        &self,
        package_name: &str,
        req: ReadProcessStdoutRequest,
    ) -> Result<ReadProcessStdoutResponse, ExtensionError> {
        let extensions = self.state_manager.get_extension_handler();
        let extension = extensions.get_extension(package_name)?;
        extension.process_manager.read_process_stdout(req)
    }

    #[tracing::instrument(level = "debug", skip_all)]
    fn kill_process(
        &self,
        package_name: &str,
        req: KillProcessRequest,
    ) -> Result<KillProcessResponse, ExtensionError> {
        let extensions = self.state_manager.get_extension_handler();
        let extension = extensions.get_extension(package_name)?;
        extension.process_manager.kill_process(req)
    }

    #[tracing::instrument(level = "debug", skip_all)]
    fn get_process_status(
        &self,
        package_name: &str,
        req: GetProcessStatusRequest,
    ) -> Result<GetProcessStatusResponse, ExtensionError> {
        let extensions = self.state_manager.get_extension_handler();
        let extension = extensions.get_extension(package_name)?;
        extension.process_manager.get_process_status(req)
    }

    #[tracing::instrument(level = "debug", skip_all)]
    fn notify_player_event(
        &self,
        package_name: &str,
        req: NotifyPlayerEventRequest,
    ) -> Result<NotifyPlayerEventResponse, ExtensionError> {
        let extensions = self.state_manager.get_extension_handler();
        let extension = extensions.get_extension(package_name)?;
        extension.notify_player_event(req)
    }

    #[tracing::instrument(level = "debug", skip_all)]
    fn ws_connect(
        &self,
        package_name: &str,
        req: WsConnectRequest,
    ) -> Result<WsConnectResponse, ExtensionError> {
        let extensions = self.state_manager.get_extension_handler();
        let extension = extensions.get_extension(package_name)?;
        self.runtime.block_on(extension.ws_manager.ws_connect(req))
    }

    #[tracing::instrument(level = "debug", skip_all)]
    fn ws_send(
        &self,
        package_name: &str,
        req: WsSendRequest,
    ) -> Result<WsSendResponse, ExtensionError> {
        let extensions = self.state_manager.get_extension_handler();
        let extension = extensions.get_extension(package_name)?;
        self.runtime.block_on(extension.ws_manager.ws_send(req))
    }

    #[tracing::instrument(level = "debug", skip_all)]
    fn ws_close(
        &self,
        package_name: &str,
        req: WsCloseRequest,
    ) -> Result<WsCloseResponse, ExtensionError> {
        let extensions = self.state_manager.get_extension_handler();
        let extension = extensions.get_extension(package_name)?;
        self.runtime.block_on(extension.ws_manager.ws_close(req))
    }

    #[tracing::instrument(level = "debug", skip_all)]
    fn system_time(
        &self,
        _package_name: &str,
        _req: SystemTimeRequest,
    ) -> Result<SystemTimeResponse, ExtensionError> {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|error| ExtensionError::Sanitize(error.to_string()))?
            .as_secs();
        Ok(SystemTimeResponse { timestamp })
    }

    #[tracing::instrument(level = "debug", skip_all)]
    fn open_client_fd(
        &self,
        package_name: &str,
        req: OpenClientFdRequest,
    ) -> Result<OpenClientFdResponse, ExtensionError> {
        let extensions = self.state_manager.get_extension_handler();
        let extension = extensions.get_extension(package_name)?;
        extension.socket_manager.open_client_fd(req)
    }

    #[tracing::instrument(level = "debug", skip_all)]
    fn write_sock(
        &self,
        package_name: &str,
        req: WriteSockRequest,
    ) -> Result<WriteSockResponse, ExtensionError> {
        let extensions = self.state_manager.get_extension_handler();
        let extension = extensions.get_extension(package_name)?;
        extension.socket_manager.write_sock(req)
    }

    #[tracing::instrument(level = "debug", skip_all)]
    fn read_sock(
        &self,
        package_name: &str,
        req: ReadSockRequest,
    ) -> Result<ReadSockResponse, ExtensionError> {
        let extensions = self.state_manager.get_extension_handler();
        let extension = extensions.get_extension(package_name)?;
        extension.socket_manager.read_sock(req)
    }

    #[tracing::instrument(level = "debug", skip_all)]
    fn batch_http_request(
        &self,
        package_name: &str,
        req: BatchHttpRequest,
    ) -> Result<BatchHttpResponse, ExtensionError> {
        let extensions = self.state_manager.get_extension_handler();
        let extension = extensions.get_extension(package_name)?;
        self.runtime
            .block_on(extension.http_manager.batch_http_request(req))
    }

    #[tracing::instrument(level = "debug", skip_all)]
    fn get_runtime_handle(&self) -> Option<Handle> { Some(self.runtime.clone()) }
}
