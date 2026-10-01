#![no_main]

use std::{
    sync::atomic::{AtomicBool, Ordering},
    time::Duration,
};

use moosync_edk::{
    ContextMenuActionRequest, ContextMenuReturnType, CustomRequest, ExtensionAccountDetail,
    ExtensionProviderScope, Lyrics, OauthCallbackRequest, PerformAccountLoginRequest,
    PlaybackDetailsRequestedRequest, PlayerStateChangedRequest, Playlist, PlaylistAddedRequest,
    PlaylistRemovedRequest, PreferenceChangedRequest, RequestedAlbumSongsRequest,
    RequestedArtistSongsRequest, RequestedLyricsRequest, RequestedPlaylistContextMenuRequest,
    RequestedPlaylistFromUrlRequest, RequestedPlaylistSongsRequest, RequestedPlaylistsRequest,
    RequestedRecommendationsRequest, RequestedSearchResultRequest, RequestedSongContextMenuRequest,
    RequestedSongFromIdRequest, RequestedSongFromUrlRequest, ScrobbleRequest, SearchResult,
    SeekedRequest, Song, SongAddedRequest, SongChangedRequest, SongQueueChangedRequest,
    SongRemovedRequest, VolumeChangedRequest,
    api::{
        self, Accounts, ContextMenu, CustomRequestReturnType, DatabaseEvents, Extension,
        PlaybackDetailsReturnType, PlayerEvents, PreferenceEvents, Provider,
        SongsWithPageTokenReturnType,
        extension_api::{register_oauth, set_account},
    },
    info,
};

static LOGGED_IN: AtomicBool = AtomicBool::new(false);

fn sync_account() {
    let is_logged_in = LOGGED_IN.load(Ordering::SeqCst);
    let _ = set_account(ExtensionAccountDetail {
        id: "sample_spotify".to_string(),
        name: "Spotify".to_string(),
        bg_color: "#1ed760".to_string(),
        icon: "spotify".to_string(),
        logged_in: is_logged_in,
        username: if is_logged_in {
            Some("EmptyUser".to_string())
        } else {
            None
        },
        ..Default::default()
    });
}

struct EmptySampleExtension;

impl Extension for EmptySampleExtension {}

impl Accounts for EmptySampleExtension {
    fn perform_account_login(&self, req: PerformAccountLoginRequest) -> api::MoosyncResult<String> {
        info!("perform_account_login called: {:?}", req);
        if req.login_status {
            let _ = register_oauth("sample_callback".to_string());
            Ok("https://example.com/oauth/authorize".to_string())
        } else {
            LOGGED_IN.store(false, Ordering::SeqCst);
            sync_account();
            Ok("logged_out".to_string())
        }
    }

    fn oauth_callback(&self, req: OauthCallbackRequest) -> api::MoosyncResult<()> {
        info!("oauth_callback called with code: {}", req.callback_uri);
        LOGGED_IN.store(true, Ordering::SeqCst);
        sync_account();
        Ok(())
    }
}

impl DatabaseEvents for EmptySampleExtension {
    fn on_song_added(&self, _req: SongAddedRequest) -> api::MoosyncResult<()> { Ok(()) }

    fn on_song_removed(&self, _req: SongRemovedRequest) -> api::MoosyncResult<()> { Ok(()) }

    fn on_playlist_added(&self, _req: PlaylistAddedRequest) -> api::MoosyncResult<()> { Ok(()) }

    fn on_playlist_removed(&self, _req: PlaylistRemovedRequest) -> api::MoosyncResult<()> { Ok(()) }
}

impl PreferenceEvents for EmptySampleExtension {
    fn on_preferences_changed(&self, _req: PreferenceChangedRequest) -> api::MoosyncResult<()> {
        Ok(())
    }
}

impl PlayerEvents for EmptySampleExtension {
    fn on_queue_changed(&self, _req: SongQueueChangedRequest) -> api::MoosyncResult<()> { Ok(()) }

    fn on_volume_changed(&self, _req: VolumeChangedRequest) -> api::MoosyncResult<()> { Ok(()) }

    fn on_player_state_changed(&self, _req: PlayerStateChangedRequest) -> api::MoosyncResult<()> {
        Ok(())
    }

    fn on_song_changed(&self, _req: SongChangedRequest) -> api::MoosyncResult<()> { Ok(()) }

    fn on_seeked(&self, _req: SeekedRequest) -> api::MoosyncResult<()> { Ok(()) }
}

fn empty_songs_with_page_token() -> SongsWithPageTokenReturnType {
    SongsWithPageTokenReturnType {
        songs: vec![],
        next_page_token: None,
    }
}

impl Provider for EmptySampleExtension {
    fn get_provider_scopes(&self) -> api::MoosyncResult<Vec<ExtensionProviderScope>> {
        Ok(vec![
            ExtensionProviderScope::Accounts,
            ExtensionProviderScope::Search,
            ExtensionProviderScope::Playlists,
            ExtensionProviderScope::PlaylistSongs,
            ExtensionProviderScope::ArtistSongs,
            ExtensionProviderScope::AlbumSongs,
            ExtensionProviderScope::Lyrics,
        ])
    }

    fn get_playlists(&self, _req: RequestedPlaylistsRequest) -> api::MoosyncResult<Vec<Playlist>> {
        info!("get_playlists called: empty");
        Ok(vec![])
    }

    fn get_playlist_content(
        &self,
        _req: RequestedPlaylistSongsRequest,
    ) -> api::MoosyncResult<SongsWithPageTokenReturnType> {
        Ok(empty_songs_with_page_token())
    }

    fn get_playlist_from_url(
        &self,
        _req: RequestedPlaylistFromUrlRequest,
    ) -> api::MoosyncResult<Option<Playlist>> {
        Ok(None)
    }

    fn get_playback_details(
        &self,
        _req: PlaybackDetailsRequestedRequest,
    ) -> api::MoosyncResult<PlaybackDetailsReturnType> {
        Ok(PlaybackDetailsReturnType {
            duration: Duration::ZERO,
            url: String::new(),
        })
    }

    fn search(&self, req: RequestedSearchResultRequest) -> api::MoosyncResult<SearchResult> {
        info!("empty search called for query: {}", req.query);
        Ok(SearchResult::default())
    }

    fn get_recommendations(
        &self,
        _req: RequestedRecommendationsRequest,
    ) -> api::MoosyncResult<Vec<Song>> {
        Ok(vec![])
    }

    fn get_song_from_url(
        &self,
        _req: RequestedSongFromUrlRequest,
    ) -> api::MoosyncResult<Option<Song>> {
        Ok(None)
    }

    fn handle_custom_request(
        &self,
        _req: CustomRequest,
    ) -> api::MoosyncResult<CustomRequestReturnType> {
        Ok(CustomRequestReturnType {
            mime_type: None,
            data: None,
            redirect_url: None,
        })
    }

    fn get_artist_songs(
        &self,
        _req: RequestedArtistSongsRequest,
    ) -> api::MoosyncResult<SongsWithPageTokenReturnType> {
        Ok(empty_songs_with_page_token())
    }

    fn get_album_songs(
        &self,
        _req: RequestedAlbumSongsRequest,
    ) -> api::MoosyncResult<SongsWithPageTokenReturnType> {
        Ok(empty_songs_with_page_token())
    }

    fn get_song_from_id(
        &self,
        _req: RequestedSongFromIdRequest,
    ) -> api::MoosyncResult<Option<Song>> {
        Ok(None)
    }

    fn scrobble(&self, _req: ScrobbleRequest) -> api::MoosyncResult<()> { Ok(()) }

    fn get_lyrics(&self, _req: RequestedLyricsRequest) -> api::MoosyncResult<Option<Lyrics>> {
        Ok(None)
    }
}

impl ContextMenu for EmptySampleExtension {
    fn get_song_context_menu(
        &self,
        _req: RequestedSongContextMenuRequest,
    ) -> api::MoosyncResult<Vec<ContextMenuReturnType>> {
        Ok(vec![])
    }

    fn get_playlist_context_menu(
        &self,
        _req: RequestedPlaylistContextMenuRequest,
    ) -> api::MoosyncResult<Vec<ContextMenuReturnType>> {
        Ok(vec![])
    }

    fn on_context_menu_action(&self, _req: ContextMenuActionRequest) -> api::MoosyncResult<()> {
        Ok(())
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn init() {
    moosync_edk::handler::register_extension(Box::new(EmptySampleExtension)).unwrap();
    sync_account();
}
