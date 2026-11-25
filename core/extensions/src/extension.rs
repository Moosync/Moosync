use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
    sync::{
        Arc, Mutex, OnceLock, RwLock,
        atomic::{AtomicU8, Ordering},
    },
    time::{Duration, Instant},
};

use async_trait::async_trait;
use extensions_proto::moosync::types::{
    AppExitRequest, AppExitResponse, CanPlayRequest, CanPlayResponse, ContextMenuActionRequest,
    ContextMenuActionResponse, CustomRequest, CustomRequestResponse, ExtensionAccountDetail,
    ExtensionCommand, ExtensionCommandResponse, ExtensionDetail, ExtensionLockData,
    ExtensionManifest, ExtensionProviderScope, GetRemoteUrlRequest, GetRemoteUrlResponse,
    NotifyPlayerEventRequest, NotifyPlayerEventResponse, OauthCallbackRequest,
    OauthCallbackResponse, PerformAccountLoginRequest, PerformAccountLoginResponse,
    PlaybackDetailsRequestedRequest, PlaybackDetailsRequestedResponse, PlayerLoadTrackRequest,
    PlayerLoadTrackResponse, PlayerPauseRequest, PlayerPauseResponse, PlayerPlayRequest,
    PlayerPlayResponse, PlayerSeekRequest, PlayerSeekResponse, PlayerSetVolumeRequest,
    PlayerSetVolumeResponse, PlayerState, PlayerStateChangedRequest, PlayerStateChangedResponse,
    PlayerStopRequest, PlayerStopResponse, PlaylistAddedRequest, PlaylistAddedResponse,
    PlaylistRemovedRequest, PlaylistRemovedResponse, PreferenceChangedRequest,
    PreferenceChangedResponse, RequestedAlbumSongsRequest, RequestedAlbumSongsResponse,
    RequestedArtistSongsRequest, RequestedArtistSongsResponse, RequestedCoverArtRequest,
    RequestedCoverArtResponse, RequestedLyricsRequest, RequestedLyricsResponse,
    RequestedPlaylistContextMenuRequest, RequestedPlaylistContextMenuResponse,
    RequestedPlaylistSongsRequest, RequestedPlaylistSongsResponse, RequestedPlaylistsRequest,
    RequestedPlaylistsResponse, RequestedRecommendationsRequest, RequestedRecommendationsResponse,
    RequestedSearchResultRequest, RequestedSearchResultResponse, RequestedSongContextMenuRequest,
    RequestedSongContextMenuResponse, RequestedSongFromIdRequest, RequestedSongFromIdResponse,
    ScrobbleRequest, ScrobbleResponse, SeekedRequest, SeekedResponse, SongAddedRequest,
    SongAddedResponse, SongChangedRequest, SongChangedResponse, SongQueueChangedRequest,
    SongQueueChangedResponse, SongRemovedRequest, SongRemovedResponse, VolumeChangedRequest,
    VolumeChangedResponse, extension_command, extension_command_response,
    player_event::Event as PlayerEvent,
};
use preferences_proto::moosync::types::PreferenceItem;
use prost::Message;
use tokio::sync::{mpsc::UnboundedSender, watch};
use types::{
    player::{AudioPlayerContext, PlayerError, ValidSrc},
    prelude::{ScopeListExt, core_to_proto_duration},
};

use crate::{
    ReplyHandler,
    cache::ExtensionMemoryCache,
    context::{ExtensionContext, ExtismContext},
    errors::ExtensionError,
    http::HttpManager,
    process::ExtensionProcessManager,
    sanitize::Sanitize,
    socket::SocketManager,
    websocket::WsManager,
};

pub(crate) struct AsyncContext {
    tx: watch::Sender<Option<Result<Arc<dyn ExtensionContext>, String>>>,
    rx: watch::Receiver<Option<Result<Arc<dyn ExtensionContext>, String>>>,
}

impl AsyncContext {
    #[tracing::instrument(level = "trace", skip_all)]
    pub fn new() -> Self {
        let (tx, rx) = watch::channel(None);
        Self { tx, rx }
    }

    #[tracing::instrument(level = "trace", skip_all)]
    pub fn set(&self, result: Result<Arc<dyn ExtensionContext>, String>) {
        let _ = self.tx.send(Some(result));
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub async fn wait(&self) -> Result<Arc<dyn ExtensionContext>, ExtensionError> {
        if let Some(res) = self.rx.borrow().as_ref() {
            return match res {
                Ok(ctx) => Ok(ctx.clone()),
                Err(e) => Err(ExtensionError::Extism(extism::Error::msg(e.clone()))),
            };
        }
        let mut rx = self.rx.clone();
        let _ = rx
            .wait_for(|val| val.is_some())
            .await
            .map_err(|_| ExtensionError::NoContext)?;
        match self.rx.borrow().as_ref() {
            Some(Ok(ctx)) => Ok(ctx.clone()),
            Some(Err(e)) => Err(ExtensionError::Extism(extism::Error::msg(e.clone()))),
            None => Err(ExtensionError::NoContext),
        }
    }
}

#[derive(Clone)]
pub(crate) struct ExtensionPlayerData {
    pub(crate) player_id: String,
    pub(crate) state: Arc<RwLock<PlayerState>>,
    pub(crate) current_pos: Arc<RwLock<Duration>>,
    pub(crate) pos_updated_at: Arc<RwLock<Option<Instant>>>,
    pub(crate) volume: Arc<AtomicU8>,
    pub(crate) events_tx: Arc<RwLock<Option<UnboundedSender<PlayerEvent>>>>,
}

impl ExtensionPlayerData {
    #[tracing::instrument(level = "debug", skip_all)]
    pub(crate) fn new(package_name: &str) -> Self {
        Self {
            player_id: format!("extension:{package_name}"),
            state: Arc::new(RwLock::new(PlayerState::Stopped)),
            current_pos: Arc::new(RwLock::new(Duration::default())),
            pos_updated_at: Arc::new(RwLock::new(None)),
            volume: Arc::new(AtomicU8::new(100)),
            events_tx: Arc::new(RwLock::new(None)),
        }
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub(crate) fn current_position(&self) -> Duration {
        let base = *self.current_pos.read().unwrap();
        let elapsed = self
            .pos_updated_at
            .read()
            .unwrap()
            .map(|since| since.elapsed())
            .unwrap_or_default();
        base + elapsed
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub(crate) fn set_playing(&self) {
        *self.state.write().unwrap() = PlayerState::Playing;
        let mut updated_at = self.pos_updated_at.write().unwrap();
        if updated_at.is_none() {
            *updated_at = Some(Instant::now());
        }
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub(crate) fn set_paused(&self, next_state: PlayerState) {
        let elapsed = self
            .pos_updated_at
            .write()
            .unwrap()
            .take()
            .map(|since| since.elapsed());
        if let Some(delta) = elapsed {
            *self.current_pos.write().unwrap() += delta;
        }
        *self.state.write().unwrap() = next_state;
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub(crate) fn set_position(&self, pos: Duration) {
        *self.current_pos.write().unwrap() = pos;
        let is_playing = *self.state.read().unwrap() == PlayerState::Playing;
        *self.pos_updated_at.write().unwrap() = is_playing.then(Instant::now);
    }
}

pub struct Extension {
    pub(crate) context: Arc<AsyncContext>,
    pub manifest: ExtensionManifest,
    preferences: RwLock<Vec<PreferenceItem>>,
    accounts: RwLock<HashMap<String, ExtensionAccountDetail>>,
    pub(crate) has_started: Arc<std::sync::atomic::AtomicBool>,
    pub(crate) cache_path: PathBuf,
    extension_path: PathBuf,
    pub(crate) reply_handler: Arc<dyn ReplyHandler>,
    pub(crate) error: Arc<OnceLock<String>>,
    pub(crate) memory_cache: Arc<Mutex<ExtensionMemoryCache>>,
    pub process_manager: Arc<ExtensionProcessManager>,
    pub socket_manager: Arc<SocketManager>,
    pub http_manager: Arc<HttpManager>,
    pub ws_manager: Arc<WsManager>,
    pub(crate) player_data: ExtensionPlayerData,
}

impl Extension {
    #[tracing::instrument(level = "debug", skip_all)]
    pub(crate) fn read_manifest(
        manifest_path: &std::path::Path,
    ) -> Result<ExtensionManifest, ExtensionError> {
        let contents = std::fs::read(manifest_path)?;
        let mut manifest = ExtensionManifest::decode(&contents[..])?;

        let parent = manifest_path.parent().unwrap();
        let extension_entry_path = parent.join(&manifest.extension_entry);
        manifest.extension_entry = extension_entry_path.to_string_lossy().to_string();

        if !manifest.icon.is_empty() {
            let icon_path = parent.join(&manifest.icon);
            manifest.icon = icon_path.to_string_lossy().to_string();
        }

        Ok(manifest)
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub fn new(
        manifest_path: &std::path::Path,
        reply_handler: Arc<dyn ReplyHandler>,
        cache_path: PathBuf,
        has_started: Arc<std::sync::atomic::AtomicBool>,
    ) -> Result<Self, ExtensionError> {
        let extension_dir = manifest_path
            .parent()
            .expect("manifest path cannot be root")
            .to_path_buf();

        let manifest = Self::read_manifest(manifest_path)?;

        let ext_cache_dir = cache_path.join("extensions").join(&manifest.name);
        let ext_cache_dir_str = ext_cache_dir.to_string_lossy();
        let allowed_paths = manifest
            .permissions
            .as_ref()
            .map(|p| ExtismContext::get_allowed_paths(p, &ext_cache_dir));
        let allowed_binaries = manifest
            .permissions
            .as_ref()
            .map(|p| {
                p.binaries
                    .iter()
                    .map(|b| {
                        let mut b = b.clone();
                        b.name = b.name.replace("{CACHE_DIR}", &ext_cache_dir_str);
                        b
                    })
                    .collect()
            })
            .unwrap_or_default();
        let context = Arc::new(AsyncContext::new());
        let process_manager = Arc::new(ExtensionProcessManager::new_with_context(
            allowed_paths.clone(),
            allowed_binaries,
            Arc::downgrade(&context),
        ));
        let socket_manager = Arc::new(SocketManager::new(allowed_paths));

        let allowed_hosts = manifest
            .permissions
            .as_ref()
            .map(|p| p.hosts.iter().map(|h| h.host.clone()).collect::<Vec<_>>());
        let package_name = manifest.name.clone();
        let http_manager = Arc::new(HttpManager::new(allowed_hosts.clone()));
        let ws_manager = Arc::new(WsManager::new(
            allowed_hosts,
            package_name.clone(),
            Arc::downgrade(&context),
        ));

        let ext = Self {
            context,
            preferences: Default::default(),
            accounts: Default::default(),
            has_started,
            manifest,
            cache_path,
            extension_path: extension_dir,
            reply_handler,
            error: Arc::new(OnceLock::new()),
            memory_cache: Arc::new(Mutex::new(ExtensionMemoryCache::default())),
            process_manager,
            socket_manager,
            http_manager,
            ws_manager,
            player_data: ExtensionPlayerData::new(&package_name),
        };

        ext.spawn_extension();

        Ok(ext)
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub fn extension_path(&self) -> &Path { &self.extension_path }

    #[tracing::instrument(level = "debug", skip_all)]
    fn spawn_extension(&self) { let _ = Arc::<ExtismContext>::try_from(self); }

    #[tracing::instrument(level = "debug", skip_all)]
    pub(crate) async fn wait_context(&self) -> Result<Arc<dyn ExtensionContext>, ExtensionError> {
        tokio::time::timeout(std::time::Duration::from_secs(15), self.context.wait())
            .await
            .map_err(|_| {
                ExtensionError::Io(std::io::Error::new(
                    std::io::ErrorKind::TimedOut,
                    format!(
                        "Timed out waiting for extension {}",
                        self.get_package_name()
                    ),
                ))
            })?
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub fn get_lock_data(&self) -> ExtensionLockData {
        let lock_path = self.extension_path.join("extension.lock");
        if let Ok(bytes) = fs::read(&lock_path)
            && let Ok(data) = ExtensionLockData::decode(&bytes[..])
        {
            return data;
        }

        ExtensionLockData {
            registry: "local".to_string(),
        }
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub fn get_extension_detail(&self) -> ExtensionDetail { self.into() }
}

macro_rules! delegate_command {
    ($fn_name:ident, $variant:ident, $req_type:ty, $resp_type:ty) => {
        pub async fn $fn_name(&self, req: $req_type) -> Result<$resp_type, ExtensionError> {
            let cmd = ExtensionCommand {
                package_name: self.get_package_name().to_string(),
                event: Some(extension_command::Event::$variant(req)),
            }
            .sanitize(self.get_package_name());
            self.execute_command(cmd)
                .await?
                .sanitize(self.get_package_name())
                .response
                .and_then(|r| match r {
                    extension_command_response::Response::$variant(resp) => Some(resp),
                    _ => None,
                })
                .ok_or(ExtensionError::InvalidResponse)
        }
    };
}

impl Extension {
    #[tracing::instrument(level = "debug", skip_all)]
    pub fn get_package_name(&self) -> &str { &self.manifest.name }

    #[tracing::instrument(level = "debug", skip_all)]
    pub fn register_ui_preferences(&self, preferences: Vec<PreferenceItem>) {
        let mut preferences_list = self.preferences.write().unwrap();
        for preference in preferences {
            if let Some(existing) = preferences_list.iter_mut().find(|p| p.id == preference.id) {
                *existing = preference;
            } else {
                preferences_list.push(preference);
            }
        }
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub fn unregister_ui_preferences(&self, keys: Vec<String>) {
        let mut preferences_list = self.preferences.write().unwrap();
        preferences_list.retain(|p| !keys.contains(&p.id));
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub fn get_ui_preference(&self, key: &str) -> Option<PreferenceItem> {
        self.preferences
            .read()
            .unwrap()
            .iter()
            .find(|p| p.id == key)
            .cloned()
    }

    #[tracing::instrument(level = "debug", skip_all)]
    async fn execute_command(
        &self,
        cmd: ExtensionCommand,
    ) -> Result<ExtensionCommandResponse, ExtensionError> {
        let context = self.wait_context().await?;
        context.execute_command(cmd).await
    }

    delegate_command!(
        get_playlists,
        RequestedPlaylists,
        RequestedPlaylistsRequest,
        RequestedPlaylistsResponse
    );
    delegate_command!(
        get_playlist_songs,
        RequestedPlaylistSongs,
        RequestedPlaylistSongsRequest,
        RequestedPlaylistSongsResponse
    );
    delegate_command!(
        oauth_callback,
        OauthCallback,
        OauthCallbackRequest,
        OauthCallbackResponse
    );
    delegate_command!(
        song_queue_changed,
        SongQueueChanged,
        SongQueueChangedRequest,
        SongQueueChangedResponse
    );
    delegate_command!(seeked, Seeked, SeekedRequest, SeekedResponse);
    delegate_command!(
        volume_changed,
        VolumeChanged,
        VolumeChangedRequest,
        VolumeChangedResponse
    );
    delegate_command!(
        player_state_changed,
        PlayerStateChanged,
        PlayerStateChangedRequest,
        PlayerStateChangedResponse
    );
    delegate_command!(
        song_changed,
        SongChanged,
        SongChangedRequest,
        SongChangedResponse
    );
    delegate_command!(
        preference_changed,
        PreferenceChanged,
        PreferenceChangedRequest,
        PreferenceChangedResponse
    );
    delegate_command!(
        playback_details_requested,
        PlaybackDetailsRequested,
        PlaybackDetailsRequestedRequest,
        PlaybackDetailsRequestedResponse
    );
    delegate_command!(
        custom_request,
        CustomRequest,
        CustomRequest,
        CustomRequestResponse
    );
    delegate_command!(
        get_search_result,
        RequestedSearchResult,
        RequestedSearchResultRequest,
        RequestedSearchResultResponse
    );
    delegate_command!(
        get_recommendations,
        RequestedRecommendations,
        RequestedRecommendationsRequest,
        RequestedRecommendationsResponse
    );
    delegate_command!(
        get_lyrics,
        RequestedLyrics,
        RequestedLyricsRequest,
        RequestedLyricsResponse
    );
    delegate_command!(
        get_artist_songs,
        RequestedArtistSongs,
        RequestedArtistSongsRequest,
        RequestedArtistSongsResponse
    );
    delegate_command!(
        get_album_songs,
        RequestedAlbumSongs,
        RequestedAlbumSongsRequest,
        RequestedAlbumSongsResponse
    );
    delegate_command!(song_added, SongAdded, SongAddedRequest, SongAddedResponse);
    delegate_command!(
        song_removed,
        SongRemoved,
        SongRemovedRequest,
        SongRemovedResponse
    );
    delegate_command!(
        playlist_added,
        PlaylistAdded,
        PlaylistAddedRequest,
        PlaylistAddedResponse
    );
    delegate_command!(
        playlist_removed,
        PlaylistRemoved,
        PlaylistRemovedRequest,
        PlaylistRemovedResponse
    );
    delegate_command!(
        get_song_from_id,
        RequestedSongFromId,
        RequestedSongFromIdRequest,
        RequestedSongFromIdResponse
    );
    delegate_command!(
        get_remote_url,
        GetRemoteUrl,
        GetRemoteUrlRequest,
        GetRemoteUrlResponse
    );
    delegate_command!(scrobble, Scrobble, ScrobbleRequest, ScrobbleResponse);
    delegate_command!(
        get_song_context_menu,
        RequestedSongContextMenu,
        RequestedSongContextMenuRequest,
        RequestedSongContextMenuResponse
    );
    delegate_command!(
        get_playlist_context_menu,
        RequestedPlaylistContextMenu,
        RequestedPlaylistContextMenuRequest,
        RequestedPlaylistContextMenuResponse
    );
    delegate_command!(
        context_menu_action,
        ContextMenuAction,
        ContextMenuActionRequest,
        ContextMenuActionResponse
    );
    delegate_command!(
        perform_account_login,
        PerformAccountLogin,
        PerformAccountLoginRequest,
        PerformAccountLoginResponse
    );
    delegate_command!(
        get_cover_art,
        RequestedCoverArt,
        RequestedCoverArtRequest,
        RequestedCoverArtResponse
    );
    delegate_command!(can_play_command, CanPlay, CanPlayRequest, CanPlayResponse);
    delegate_command!(
        player_play,
        PlayerPlay,
        PlayerPlayRequest,
        PlayerPlayResponse
    );
    delegate_command!(
        player_pause,
        PlayerPause,
        PlayerPauseRequest,
        PlayerPauseResponse
    );
    delegate_command!(
        player_stop,
        PlayerStop,
        PlayerStopRequest,
        PlayerStopResponse
    );
    delegate_command!(
        player_seek,
        PlayerSeek,
        PlayerSeekRequest,
        PlayerSeekResponse
    );
    delegate_command!(
        player_set_volume,
        PlayerSetVolume,
        PlayerSetVolumeRequest,
        PlayerSetVolumeResponse
    );
    delegate_command!(
        player_load_track,
        PlayerLoadTrack,
        PlayerLoadTrackRequest,
        PlayerLoadTrackResponse
    );
    delegate_command!(app_exit, AppExit, AppExitRequest, AppExitResponse);

    #[tracing::instrument(level = "debug", skip_all)]
    pub fn notify_player_event(
        &self,
        req: NotifyPlayerEventRequest,
    ) -> Result<NotifyPlayerEventResponse, ExtensionError> {
        if let Some(player_event) = req.event {
            if let Some(event) = &player_event.event {
                match event {
                    PlayerEvent::Play(_) => self.player_data.set_playing(),
                    PlayerEvent::Pause(_) => self.player_data.set_paused(PlayerState::Paused),
                    PlayerEvent::Ended(_) | PlayerEvent::Error(_) => {
                        self.player_data.set_paused(PlayerState::Stopped);
                    }
                    PlayerEvent::TimeUpdate(pos) => {
                        let dur = Duration::new(pos.seconds.max(0) as u64, pos.nanos.max(0) as u32);
                        self.player_data.set_position(dur);
                    }
                    PlayerEvent::Loading(_) => {
                        self.player_data.set_paused(PlayerState::Loading);
                    }
                }
            }
            if let (Some(tx), Some(event)) = (
                self.player_data.events_tx.read().unwrap().as_ref(),
                player_event.event,
            ) {
                let _ = tx.send(event);
            }
        }
        Ok(NotifyPlayerEventResponse {})
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub fn set_account(&self, mut account: ExtensionAccountDetail) {
        account.package_name = self.get_package_name().to_string();
        let mut accounts = self.accounts.write().unwrap();
        accounts.insert(account.id.clone(), account);
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub fn get_accounts(&self) -> Vec<ExtensionAccountDetail> {
        let accounts = self.accounts.read().unwrap();
        accounts.values().cloned().collect()
    }
}

impl Drop for Extension {
    #[tracing::instrument(level = "debug", skip_all)]
    fn drop(&mut self) {
        self.process_manager.kill_all();
        self.socket_manager.close_all();
        self.ws_manager.close_all();
    }
}

impl From<&Extension> for ExtensionDetail {
    #[tracing::instrument(level = "debug", skip_all)]
    fn from(val: &Extension) -> Self {
        let lock_data = val.get_lock_data();
        ExtensionDetail {
            name: val.manifest.display_name.clone(),
            package_name: val.manifest.name.clone(),
            desc: None,
            author: val.manifest.author.clone(),
            version: val.manifest.version.clone(),
            has_started: val.has_started.load(std::sync::atomic::Ordering::SeqCst),
            preferences: val.preferences.read().unwrap().clone(),
            extension_icon: Some(val.manifest.icon.clone()),
            registry: Some(lock_data.registry),
            permissions: val.manifest.permissions.clone(),
            scopes: val.manifest.scopes.clone(),
            error: val.error.get().cloned(),
        }
    }
}

pub trait ExtensionGroupExt {
    fn with<F, Fut, R>(&self, f: F) -> futures::stream::FuturesUnordered<Fut>
    where
        F: Fn(Arc<Extension>) -> Fut,
        Fut: std::future::Future<Output = R>;
}

impl ExtensionGroupExt for [Arc<Extension>] {
    #[tracing::instrument(level = "trace", skip_all)]
    fn with<F, Fut, R>(&self, f: F) -> futures::stream::FuturesUnordered<Fut>
    where
        F: Fn(Arc<Extension>) -> Fut,
        Fut: std::future::Future<Output = R>,
    {
        self.iter().map(|ext| f(Arc::clone(ext))).collect()
    }
}

#[async_trait]
impl AudioPlayerContext for Extension {
    #[tracing::instrument(level = "debug", skip_all)]
    fn get_id(&self) -> &str { &self.player_data.player_id }

    #[tracing::instrument(level = "debug", skip_all)]
    async fn play(&self) -> Result<(), PlayerError> {
        self.player_play(PlayerPlayRequest {})
            .await
            .map_err(|e| PlayerError::Custom(e.to_string()))?;
        self.player_data.set_playing();
        Ok(())
    }

    #[tracing::instrument(level = "debug", skip_all)]
    async fn pause(&self) -> Result<(), PlayerError> {
        self.player_pause(PlayerPauseRequest {})
            .await
            .map_err(|e| PlayerError::Custom(e.to_string()))?;
        self.player_data.set_paused(PlayerState::Paused);
        Ok(())
    }

    #[tracing::instrument(level = "debug", skip_all)]
    async fn stop(&self) -> Result<(), PlayerError> {
        self.player_stop(PlayerStopRequest {})
            .await
            .map_err(|e| PlayerError::Custom(e.to_string()))?;
        self.player_data.set_paused(PlayerState::Stopped);
        self.player_data.set_position(Duration::ZERO);
        Ok(())
    }

    #[tracing::instrument(level = "debug", skip_all)]
    async fn set_volume(&self, volume: u8) -> Result<(), PlayerError> {
        self.player_data.volume.store(volume, Ordering::SeqCst);
        self.player_set_volume(PlayerSetVolumeRequest {
            volume: volume as u32,
        })
        .await
        .map_err(|e| PlayerError::Custom(e.to_string()))?;
        Ok(())
    }

    #[tracing::instrument(level = "debug", skip_all)]
    fn get_volume(&self) -> Result<u8, PlayerError> {
        Ok(self.player_data.volume.load(Ordering::SeqCst))
    }

    #[tracing::instrument(level = "debug", skip_all)]
    async fn seek(&self, pos: Duration) -> Result<(), PlayerError> {
        self.player_seek(PlayerSeekRequest {
            position: Some(core_to_proto_duration(pos)),
        })
        .await
        .map_err(|e| PlayerError::Custom(e.to_string()))?;
        self.player_data.set_position(pos);
        Ok(())
    }

    #[tracing::instrument(level = "debug", skip_all)]
    fn get_current_pos(&self) -> Result<Duration, PlayerError> {
        Ok(self.player_data.current_position())
    }

    #[tracing::instrument(level = "debug", skip_all)]
    fn get_player_state(&self) -> Result<PlayerState, PlayerError> {
        Ok(*self.player_data.state.read().unwrap())
    }

    #[tracing::instrument(level = "debug", skip_all)]
    async fn can_play(&self, src: &ValidSrc<'_>) -> bool {
        if !self
            .manifest
            .contains_scope(ExtensionProviderScope::PlayerProvider)
        {
            return false;
        }

        match self
            .can_play_command(CanPlayRequest {
                src: src.inner().to_string(),
                song: None,
            })
            .await
        {
            Ok(resp) => resp.can_play,
            Err(e) => {
                tracing::error!("Extension can_play check failed: {:?}", e);
                false
            }
        }
    }

    #[tracing::instrument(level = "debug", skip_all)]
    async fn set_src<'a>(
        &self,
        src: ValidSrc<'a>,
        events_tx: UnboundedSender<PlayerEvent>,
    ) -> Result<(), PlayerError> {
        *self.player_data.events_tx.write().unwrap() = Some(events_tx);
        self.player_data.set_paused(PlayerState::Paused);
        self.player_data.set_position(Duration::ZERO);

        self.player_load_track(PlayerLoadTrackRequest {
            src: src.inner().to_string(),
            song: None,
        })
        .await
        .map_err(|e| PlayerError::Custom(e.to_string()))?;

        Ok(())
    }
}
