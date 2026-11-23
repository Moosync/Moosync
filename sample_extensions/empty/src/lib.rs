#![no_main]

use moosync_edk::{
    MoosyncResult,
    api::{
        Accounts, ContextMenu, DatabaseEvents, Extension, PlayerEvents, PlayerProvider,
        PreferenceEvents, Provider, register_extension, register_oauth, set_account,
    },
    info,
    types::{
        ContextMenuActionRequest, ContextMenuActionResponse, CustomRequest, CustomRequestResponse,
        Duration, ExtensionAccountDetail, OauthCallbackRequest, OauthCallbackResponse,
        PerformAccountLoginRequest, PerformAccountLoginResponse, PlaybackDetailsRequestedRequest,
        PlaybackDetailsRequestedResponse, RegisterOauthRequest, RequestedAlbumSongsRequest,
        RequestedAlbumSongsResponse, RequestedArtistSongsRequest, RequestedArtistSongsResponse,
        RequestedCoverArtRequest, RequestedCoverArtResponse, RequestedLyricsRequest,
        RequestedLyricsResponse, RequestedPlaylistContextMenuRequest,
        RequestedPlaylistContextMenuResponse, RequestedPlaylistSongsRequest,
        RequestedPlaylistSongsResponse, RequestedPlaylistsRequest, RequestedPlaylistsResponse,
        RequestedRecommendationsRequest, RequestedRecommendationsResponse,
        RequestedSearchResultRequest, RequestedSearchResultResponse,
        RequestedSongContextMenuRequest, RequestedSongContextMenuResponse,
        RequestedSongFromIdRequest, RequestedSongFromIdResponse, SetAccountRequest,
    },
};

fn update_spotify_account(logged_in: bool) {
    let username = logged_in.then(|| "EmptyUser".to_string());
    let detail = ExtensionAccountDetail {
        id: "sample_spotify".to_string(),
        name: "Spotify".to_string(),
        bg_color: "#1ed760".to_string(),
        icon: "spotify".to_string(),
        logged_in,
        username,
        ..Default::default()
    };
    let _ = set_account(SetAccountRequest {
        account: Some(detail),
    });
}

struct EmptySampleExtension;

impl PlayerProvider for EmptySampleExtension {}
impl Extension for EmptySampleExtension {}
impl DatabaseEvents for EmptySampleExtension {}
impl PreferenceEvents for EmptySampleExtension {}
impl PlayerEvents for EmptySampleExtension {}

impl Accounts for EmptySampleExtension {
    fn perform_account_login(
        &self,
        req: PerformAccountLoginRequest,
    ) -> MoosyncResult<PerformAccountLoginResponse> {
        info!("perform_account_login called: {:?}", req);
        match req.login_status {
            true => {
                let _ = register_oauth(RegisterOauthRequest {
                    url: "sample_callback".to_string(),
                });
                Ok(PerformAccountLoginResponse {
                    url: "https://example.com/oauth/authorize".to_string(),
                    device_code: None,
                })
            }
            false => {
                update_spotify_account(false);
                Ok(PerformAccountLoginResponse {
                    url: "logged_out".to_string(),
                    device_code: None,
                })
            }
        }
    }

    fn oauth_callback(&self, req: OauthCallbackRequest) -> MoosyncResult<OauthCallbackResponse> {
        info!("oauth_callback called with code: {}", req.callback_uri);
        update_spotify_account(true);
        Ok(OauthCallbackResponse {
            extra_content: None,
        })
    }
}

impl Provider for EmptySampleExtension {
    fn requested_playlists(
        &self,
        _req: RequestedPlaylistsRequest,
    ) -> MoosyncResult<RequestedPlaylistsResponse> {
        info!("requested_playlists called: empty");
        Ok(RequestedPlaylistsResponse { playlists: vec![] })
    }

    fn requested_playlist_songs(
        &self,
        _req: RequestedPlaylistSongsRequest,
    ) -> MoosyncResult<RequestedPlaylistSongsResponse> {
        Ok(RequestedPlaylistSongsResponse {
            songs: vec![],
            next_page_token: None,
        })
    }

    fn playback_details_requested(
        &self,
        _req: PlaybackDetailsRequestedRequest,
    ) -> MoosyncResult<PlaybackDetailsRequestedResponse> {
        Ok(PlaybackDetailsRequestedResponse {
            duration: Some(Duration {
                seconds: 0,
                nanos: 0,
            }),
            url: String::new(),
            decryption_key: None,
            ..Default::default()
        })
    }

    fn requested_search_result(
        &self,
        req: RequestedSearchResultRequest,
    ) -> MoosyncResult<RequestedSearchResultResponse> {
        info!("empty search called for query: {}", req.query);
        Ok(RequestedSearchResultResponse {
            songs: vec![],
            playlists: vec![],
            artists: vec![],
            albums: vec![],
        })
    }

    fn requested_recommendations(
        &self,
        _req: RequestedRecommendationsRequest,
    ) -> MoosyncResult<RequestedRecommendationsResponse> {
        Ok(RequestedRecommendationsResponse { songs: vec![] })
    }

    fn custom_request(&self, _req: CustomRequest) -> MoosyncResult<CustomRequestResponse> {
        Ok(CustomRequestResponse {
            mime_type: None,
            data: None,
            redirect_url: None,
        })
    }

    fn requested_artist_songs(
        &self,
        _req: RequestedArtistSongsRequest,
    ) -> MoosyncResult<RequestedArtistSongsResponse> {
        Ok(RequestedArtistSongsResponse {
            songs: vec![],
            next_page_token: None,
        })
    }

    fn requested_album_songs(
        &self,
        _req: RequestedAlbumSongsRequest,
    ) -> MoosyncResult<RequestedAlbumSongsResponse> {
        Ok(RequestedAlbumSongsResponse {
            songs: vec![],
            next_page_token: None,
        })
    }

    fn requested_song_from_id(
        &self,
        _req: RequestedSongFromIdRequest,
    ) -> MoosyncResult<RequestedSongFromIdResponse> {
        Ok(RequestedSongFromIdResponse { song: None })
    }

    fn requested_lyrics(
        &self,
        _req: RequestedLyricsRequest,
    ) -> MoosyncResult<RequestedLyricsResponse> {
        Ok(RequestedLyricsResponse { lyrics: None })
    }

    fn requested_cover_art(
        &self,
        _req: RequestedCoverArtRequest,
    ) -> MoosyncResult<RequestedCoverArtResponse> {
        Ok(RequestedCoverArtResponse { cover_urls: vec![] })
    }
}

impl ContextMenu for EmptySampleExtension {
    fn requested_song_context_menu(
        &self,
        _req: RequestedSongContextMenuRequest,
    ) -> MoosyncResult<RequestedSongContextMenuResponse> {
        Ok(RequestedSongContextMenuResponse { menu: None })
    }

    fn requested_playlist_context_menu(
        &self,
        _req: RequestedPlaylistContextMenuRequest,
    ) -> MoosyncResult<RequestedPlaylistContextMenuResponse> {
        Ok(RequestedPlaylistContextMenuResponse { menu: None })
    }

    fn context_menu_action(
        &self,
        _req: ContextMenuActionRequest,
    ) -> MoosyncResult<ContextMenuActionResponse> {
        Ok(ContextMenuActionResponse {})
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn init() {
    register_extension(Box::new(EmptySampleExtension)).unwrap();
    update_spotify_account(false);
}
