use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
    sync::{Arc, Mutex, RwLock},
};

use extensions_proto::moosync::types::{
    ContextMenuActionRequest, ContextMenuActionResponse, ContextMenuReturnType, CustomRequest,
    CustomRequestResponse, ExtensionAccountDetail, ExtensionCommand, ExtensionCommandResponse,
    ExtensionDetail, ExtensionManifest, ExtensionProviderScope, GetAccountsRequest,
    GetAccountsResponse, GetProviderScopesRequest, GetProviderScopesResponse, GetRemoteUrlRequest,
    GetRemoteUrlResponse, OauthCallbackRequest, OauthCallbackResponse, PerformAccountLoginRequest,
    PerformAccountLoginResponse, PlaybackDetailsRequestedRequest, PlaybackDetailsRequestedResponse,
    PlayerStateChangedRequest, PlayerStateChangedResponse, PlaylistAddedRequest,
    PlaylistAddedResponse, PlaylistRemovedRequest, PlaylistRemovedResponse,
    PreferenceChangedRequest, PreferenceChangedResponse, RequestedAlbumSongsRequest,
    RequestedAlbumSongsResponse, RequestedArtistSongsRequest, RequestedArtistSongsResponse,
    RequestedLyricsRequest, RequestedLyricsResponse, RequestedPlaylistContextMenuRequest,
    RequestedPlaylistContextMenuResponse, RequestedPlaylistFromUrlRequest,
    RequestedPlaylistFromUrlResponse, RequestedPlaylistSongsRequest,
    RequestedPlaylistSongsResponse, RequestedPlaylistsRequest, RequestedPlaylistsResponse,
    RequestedRecommendationsRequest, RequestedRecommendationsResponse,
    RequestedSearchResultRequest, RequestedSearchResultResponse, RequestedSongContextMenuRequest,
    RequestedSongContextMenuResponse, RequestedSongFromIdRequest, RequestedSongFromIdResponse,
    RequestedSongFromUrlRequest, RequestedSongFromUrlResponse, ScrobbleRequest, ScrobbleResponse,
    SeekedRequest, SeekedResponse, SongAddedRequest, SongAddedResponse, SongChangedRequest,
    SongChangedResponse, SongQueueChangedRequest, SongQueueChangedResponse, SongRemovedRequest,
    SongRemovedResponse, VolumeChangedRequest, VolumeChangedResponse, extension_command,
    extension_command_response,
};
use songs_proto::moosync::types::{Album, Artist, Playlist, Song};
use ui_proto::moosync::types::PreferenceUiData;

use crate::{
    ReplyHandler,
    context::{ExtensionContext, ExtismContext},
    errors::ExtensionError,
};

pub struct Extension {
    context: Mutex<Option<Arc<dyn ExtensionContext>>>,
    manifest: ExtensionManifest,
    preferences: RwLock<HashMap<String, PreferenceUiData>>,
    has_started: Arc<std::sync::atomic::AtomicBool>,
    provider_scopes: Mutex<Option<GetProviderScopesResponse>>,
    cache_path: PathBuf,
    extension_path: PathBuf,
    reply_handler: Arc<dyn ReplyHandler>,
}

impl Extension {
    #[tracing::instrument(level = "debug", skip_all)]
    fn read_manifest(manifest_path: &std::path::Path) -> Result<ExtensionManifest, ExtensionError> {
        let contents = std::fs::read(manifest_path)?;
        let mut manifest = serde_json::from_slice::<ExtensionManifest>(&contents)?;

        let parent = manifest_path.parent().unwrap();
        let extension_entry_path = parent.join(&manifest.extension_entry);
        manifest.extension_entry = extension_entry_path.to_string_lossy().to_string();

        let icon_path = parent.join(&manifest.icon);
        manifest.icon = icon_path.to_string_lossy().to_string();

        Ok(manifest)
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub fn new(
        manifest_path: &std::path::Path,
        reply_handler: Arc<dyn ReplyHandler>,
        cache_path: PathBuf,
        has_started: Arc<std::sync::atomic::AtomicBool>,
    ) -> Result<Self, ExtensionError> {
        let manifest = Self::read_manifest(manifest_path)?;
        let extension_dir = manifest_path
            .parent()
            .expect("manifest path cannot be root")
            .to_path_buf();

        let ext = Self {
            context: Mutex::new(None),
            preferences: Default::default(),
            has_started,
            manifest,
            provider_scopes: Mutex::new(None),
            cache_path,
            extension_path: extension_dir,
            reply_handler,
        };

        if ext.is_active() {
            ext.spawn_extension();
        }

        Ok(ext)
    }

    #[tracing::instrument(level = "debug", skip_all)]
    fn spawn_extension(&self) {
        let mut context = self.context.lock().unwrap();
        *context = Some(Arc::new(ExtismContext::new(
            &self.manifest,
            self.has_started.clone(),
            &self.cache_path,
            self.reply_handler.clone(),
        )));
    }

    #[tracing::instrument(level = "debug", skip_all)]
    fn kill_extension(&self) {
        if let Some(context) = self.context.lock().unwrap().take() {
            if let Err(e) = context.kill() {
                tracing::error!("Failed to kill extension: {}", e);
            }
        }
        self.has_started
            .store(false, std::sync::atomic::Ordering::SeqCst);
        *self.provider_scopes.lock().unwrap() = None;
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub fn is_active(&self) -> bool { !self.extension_path.join(".disabled").exists() }

    #[tracing::instrument(level = "debug", skip_all)]
    pub fn set_active(&self, active: bool) -> Result<(), ExtensionError> {
        self.kill_extension();
        if active {
            self.spawn_extension();
        }
        self.set_extension_disabled_file(!active)?;
        Ok(())
    }

    #[tracing::instrument(level = "debug", skip_all)]
    fn set_extension_disabled_file(&self, disabled: bool) -> Result<(), ExtensionError> {
        if !self.extension_path.exists() {
            tracing::error!("Extension path does not exist: {:?}", self.extension_path);
            return Err(ExtensionError::NoExtensionFound);
        }
        let disabled_file = self.extension_path.join(".disabled");
        if disabled {
            fs::write(disabled_file, "")?;
        } else if disabled_file.exists() {
            fs::remove_file(disabled_file)?;
        }
        Ok(())
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
            };
            self.execute_command(cmd)
                .await?
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
    pub fn register_ui_preferences(&self, preferences: Vec<PreferenceUiData>) {
        let mut preferences_map = self.preferences.write().unwrap();
        for preference in preferences {
            preferences_map.insert(preference.key.clone(), preference);
        }
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub fn unregister_ui_preferences(&self, keys: Vec<String>) {
        let mut preferences_map = self.preferences.write().unwrap();
        for key in keys {
            preferences_map.remove(&key);
        }
    }

    #[tracing::instrument(level = "debug", skip_all)]
    async fn execute_command(
        &self,
        cmd: ExtensionCommand,
    ) -> Result<ExtensionCommandResponse, ExtensionError> {
        let context = self.context.lock().unwrap().clone();
        if let Some(context) = context {
            context.execute_command(cmd).await
        } else {
            Err(ExtensionError::NoContext)
        }
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub async fn get_provider_scopes(
        &self,
        req: GetProviderScopesRequest,
    ) -> Result<GetProviderScopesResponse, ExtensionError> {
        let cached = self.provider_scopes.lock().unwrap().clone();
        if let Some(scopes) = cached {
            return Ok(scopes);
        }

        let cmd = ExtensionCommand {
            package_name: self.get_package_name().to_string(),
            event: Some(extension_command::Event::GetProviderScopes(req)),
        };
        let res = self.execute_command(cmd).await?;
        let resp = res
            .response
            .and_then(|r| match r {
                extension_command_response::Response::GetProviderScopes(resp) => Some(resp),
                _ => None,
            })
            .ok_or(ExtensionError::InvalidResponse)?;

        let mut lock = self.provider_scopes.lock().unwrap();
        *lock = Some(resp.clone());

        Ok(resp)
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
        get_song_from_url,
        RequestedSongFromUrl,
        RequestedSongFromUrlRequest,
        RequestedSongFromUrlResponse
    );
    delegate_command!(
        get_playlist_from_url,
        RequestedPlaylistFromUrl,
        RequestedPlaylistFromUrlRequest,
        RequestedPlaylistFromUrlResponse
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
        get_accounts,
        GetAccounts,
        GetAccountsRequest,
        GetAccountsResponse
    );
    delegate_command!(
        perform_account_login,
        PerformAccountLogin,
        PerformAccountLoginRequest,
        PerformAccountLoginResponse
    );
}

impl Into<ExtensionDetail> for &Extension {
    fn into(self) -> ExtensionDetail {
        ExtensionDetail {
            name: self.manifest.display_name.clone(),
            package_name: self.manifest.name.clone(),
            desc: None,
            author: self.manifest.author.clone(),
            version: self.manifest.version.clone(),
            has_started: self.has_started.load(std::sync::atomic::Ordering::SeqCst),
            preferences: self
                .preferences
                .read()
                .unwrap()
                .clone()
                .into_values()
                .collect(),
            extension_icon: Some(self.manifest.icon.clone()),
            active: self.is_active(),
        }
    }
}
