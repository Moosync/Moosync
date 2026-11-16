#![no_main]

use std::sync::atomic::{AtomicBool, Ordering};

use moosync_edk::{
    MoosyncResult,
    api::{
        Accounts, ContextMenu, DatabaseEvents, Extension, PlayerEvents, PlayerProvider,
        PreferenceEvents, Provider, add_playlist, add_songs, add_to_playlist, get_current_song,
        get_player_state, get_preference, get_queue, get_secure, get_time, get_volume,
        open_clientfd, open_external_url, read_sock, register_extension, register_oauth,
        register_user_preference, set_account, system_time, unregister_user_preference, write_sock,
    },
    info,
    types::{
        AddPlaylistRequest, AddSongsRequest, AddToPlaylistRequest, Album, Artist,
        ContextMenuActionRequest, ContextMenuActionResponse, ContextMenuReturnType, CustomRequest,
        CustomRequestResponse, Duration, ExtensionAccountDetail, GetCurrentSongRequest,
        GetPlayerStateRequest, GetPreferenceRequest, GetQueueRequest, GetSecureRequest,
        GetTimeRequest, GetVolumeRequest, InnerSong, LyricLine, Lyrics, OauthCallbackRequest,
        OauthCallbackResponse, OpenExternalUrlRequest, PerformAccountLoginRequest,
        PerformAccountLoginResponse, PlaybackDetailsRequestedRequest,
        PlaybackDetailsRequestedResponse, PlayerStateChangedRequest, Playlist,
        PlaylistAddedRequest, PlaylistRemovedRequest, PreferenceChangedRequest, PreferenceData,
        PreferenceItem, PreferenceType, RegisterOauthRequest, RegisterUserPreferenceRequest,
        RequestedAlbumSongsRequest, RequestedAlbumSongsResponse, RequestedArtistSongsRequest,
        RequestedArtistSongsResponse, RequestedCoverArtRequest, RequestedCoverArtResponse,
        RequestedLyricsRequest, RequestedLyricsResponse, RequestedPlaylistContextMenuRequest,
        RequestedPlaylistContextMenuResponse, RequestedPlaylistSongsRequest,
        RequestedPlaylistSongsResponse, RequestedPlaylistsRequest, RequestedPlaylistsResponse,
        RequestedRecommendationsRequest, RequestedRecommendationsResponse,
        RequestedSearchResultRequest, RequestedSearchResultResponse,
        RequestedSongContextMenuRequest, RequestedSongContextMenuResponse,
        RequestedSongFromIdRequest, RequestedSongFromIdResponse, ScrobbleRequest, SeekedRequest,
        SetAccountRequest, Song, SongAddedRequest, SongChangedRequest, SongQueueChangedRequest,
        SongRemovedRequest, UnregisterUserPreferenceRequest, VolumeChangedRequest,
        requested_cover_art_request,
    },
};

static LOGGED_IN: AtomicBool = AtomicBool::new(false);

fn sync_account() {
    let is_logged_in = LOGGED_IN.load(Ordering::SeqCst);
    let _ = set_account(SetAccountRequest {
        account: Some(ExtensionAccountDetail {
            id: "sample_spotify".to_string(),
            name: "Spotify".to_string(),
            bg_color: "#1ed760".to_string(),
            icon: "spotify".to_string(),
            logged_in: is_logged_in,
            username: if is_logged_in {
                Some("SampleUser".to_string())
            } else {
                None
            },
            ..Default::default()
        }),
    });
}

struct SampleExtension;

impl Extension for SampleExtension {}

impl PlayerProvider for SampleExtension {}

impl Accounts for SampleExtension {
    fn perform_account_login(
        &self,
        req: PerformAccountLoginRequest,
    ) -> MoosyncResult<PerformAccountLoginResponse> {
        info!("perform_account_login called: {:?}", req);
        if req.login_status {
            let _ = register_oauth(RegisterOauthRequest {
                url: "sample_callback".to_string(),
            });
            Ok(PerformAccountLoginResponse {
                status: "https://example.com/oauth/authorize".to_string(),
            })
        } else {
            LOGGED_IN.store(false, Ordering::SeqCst);
            sync_account();
            Ok(PerformAccountLoginResponse {
                status: "logged_out".to_string(),
            })
        }
    }

    fn oauth_callback(&self, req: OauthCallbackRequest) -> MoosyncResult<OauthCallbackResponse> {
        info!("oauth_callback called with code: {}", req.callback_uri);
        LOGGED_IN.store(true, Ordering::SeqCst);
        sync_account();
        Ok(OauthCallbackResponse {})
    }
}

impl DatabaseEvents for SampleExtension {
    fn song_added(&self, req: SongAddedRequest) -> MoosyncResult<()> {
        for song in req.songs {
            if let Some(inner) = &song.song {
                info!("song_added: {:?}", inner.title);
            }
        }
        Ok(())
    }

    fn song_removed(&self, req: SongRemovedRequest) -> MoosyncResult<()> {
        for song in req.songs {
            if let Some(inner) = &song.song {
                info!("song_removed: {:?}", inner.title);
            }
        }
        Ok(())
    }

    fn playlist_added(&self, req: PlaylistAddedRequest) -> MoosyncResult<()> {
        for playlist in req.playlists {
            info!("playlist_added: {:?}", playlist.playlist_name);
        }
        Ok(())
    }

    fn playlist_removed(&self, req: PlaylistRemovedRequest) -> MoosyncResult<()> {
        for playlist in req.playlists {
            info!("playlist_removed: {:?}", playlist.playlist_name);
        }
        Ok(())
    }
}

impl PreferenceEvents for SampleExtension {
    fn preference_changed(&self, req: PreferenceChangedRequest) -> MoosyncResult<()> {
        if let Some(args) = req.preference {
            info!("preference_changed: {:?}", args);
            let key = args.key;
            if !key.is_empty() {
                let _val = get_preference(GetPreferenceRequest {
                    data: Some(PreferenceData {
                        key: key.clone(),
                        ..Default::default()
                    }),
                });
                let _sec = get_secure(GetSecureRequest {
                    data: Some(PreferenceData {
                        key,
                        ..Default::default()
                    }),
                });
            }
        }
        Ok(())
    }
}

impl PlayerEvents for SampleExtension {
    fn song_queue_changed(&self, _req: SongQueueChangedRequest) -> MoosyncResult<()> {
        info!("song_queue_changed");
        let _q = get_queue(GetQueueRequest {});
        Ok(())
    }

    fn volume_changed(&self, _req: VolumeChangedRequest) -> MoosyncResult<()> {
        info!("volume_changed");
        let _vol = get_volume(GetVolumeRequest {});
        Ok(())
    }

    fn player_state_changed(&self, _req: PlayerStateChangedRequest) -> MoosyncResult<()> {
        info!("player_state_changed");
        let _state = get_player_state(GetPlayerStateRequest {});
        Ok(())
    }

    fn song_changed(&self, _req: SongChangedRequest) -> MoosyncResult<()> {
        info!("song_changed");
        let _song = get_current_song(GetCurrentSongRequest {});
        Ok(())
    }

    fn seeked(&self, req: SeekedRequest) -> MoosyncResult<()> {
        info!("seeked: {}", req.position);
        let _t = get_time(GetTimeRequest {});
        Ok(())
    }
}

fn mock_sample_song(id: &str, title: &str) -> Song {
    Song {
        song: Some(InnerSong {
            id: Some(id.to_string()),
            title: Some(title.to_string()),
            ..Default::default()
        }),
        artists: vec![Artist {
            artist_id: Some(format!("art-{}", id)),
            artist_name: Some(format!("Artist for {}", title)),
            ..Default::default()
        }],
        album: Some(Album {
            album_id: Some(format!("alb-{}", id)),
            album_name: Some(format!("Album for {}", title)),
            ..Default::default()
        }),
        ..Default::default()
    }
}

fn mock_sample_songs_page(
    id_prefix: &str,
    name_prefix: &str,
    start: usize,
    count: usize,
) -> Vec<Song> {
    (start..start + count)
        .map(|i| {
            mock_sample_song(
                &format!("{}-song-{}", id_prefix, i),
                &format!("{} Song {}", name_prefix, i),
            )
        })
        .collect()
}

fn get_paginated_sample_songs(
    id_prefix: &str,
    name_prefix: &str,
    page_token: Option<&str>,
) -> (Vec<Song>, Option<String>) {
    let page = page_token.unwrap_or("0");
    match page {
        "0" => (
            mock_sample_songs_page(id_prefix, name_prefix, 1, 25),
            Some("1".to_string()),
        ),
        "1" => (
            mock_sample_songs_page(id_prefix, name_prefix, 26, 25),
            Some("2".to_string()),
        ),
        "2" => (mock_sample_songs_page(id_prefix, name_prefix, 51, 10), None),
        _ => (vec![], None),
    }
}

impl Provider for SampleExtension {
    fn requested_playlists(
        &self,
        _req: RequestedPlaylistsRequest,
    ) -> MoosyncResult<RequestedPlaylistsResponse> {
        info!("requested_playlists called");
        Ok(RequestedPlaylistsResponse {
            playlists: vec![
                Playlist {
                    playlist_id: Some("ext-playlist-1".to_string()),
                    playlist_name: "Extension Playlist 1".to_string(),
                    ..Default::default()
                },
                Playlist {
                    playlist_id: Some("ext-playlist-2".to_string()),
                    playlist_name: "Extension Playlist 2".to_string(),
                    ..Default::default()
                },
            ],
        })
    }

    fn requested_playlist_songs(
        &self,
        req: RequestedPlaylistSongsRequest,
    ) -> MoosyncResult<RequestedPlaylistSongsResponse> {
        info!("requested_playlist_songs: {}", req.id);
        let (songs, next_page_token) =
            get_paginated_sample_songs("pl", "Playlist", req.page_token.as_deref());
        Ok(RequestedPlaylistSongsResponse {
            songs,
            next_page_token,
        })
    }

    fn playback_details_requested(
        &self,
        req: PlaybackDetailsRequestedRequest,
    ) -> MoosyncResult<PlaybackDetailsRequestedResponse> {
        if let Some(song) = req.song {
            if let Some(inner) = &song.song {
                info!("playback_details_requested: {:?}", inner.id);
            }
        }
        Ok(PlaybackDetailsRequestedResponse {
            duration: Some(Duration {
                seconds: 0,
                nanos: 0,
            }),
            url: "https://example.com/song.mp3".to_string(),
            decryption_key: None,
        })
    }

    fn requested_search_result(
        &self,
        req: RequestedSearchResultRequest,
    ) -> MoosyncResult<RequestedSearchResultResponse> {
        info!("requested_search_result: {}", req.query);
        let _time = system_time();
        let _ = open_external_url(OpenExternalUrlRequest {
            url: "https://google.com".to_string(),
        });
        Ok(RequestedSearchResultResponse {
            songs: vec![
                mock_sample_song("search-song-1", "Search Song 1"),
                mock_sample_song("search-song-2", "Search Song 2"),
            ],
            playlists: vec![
                Playlist {
                    playlist_id: Some("search-pl-1".to_string()),
                    playlist_name: "Search Playlist 1".to_string(),
                    ..Default::default()
                },
                Playlist {
                    playlist_id: Some("search-pl-2".to_string()),
                    playlist_name: "Search Playlist 2".to_string(),
                    ..Default::default()
                },
            ],
            artists: vec![
                Artist {
                    artist_id: Some("search-art-1".to_string()),
                    artist_name: Some("Search Artist 1".to_string()),
                    ..Default::default()
                },
                Artist {
                    artist_id: Some("search-art-2".to_string()),
                    artist_name: Some("Search Artist 2".to_string()),
                    ..Default::default()
                },
            ],
            albums: vec![
                Album {
                    album_id: Some("search-alb-1".to_string()),
                    album_name: Some("Search Album 1".to_string()),
                    ..Default::default()
                },
                Album {
                    album_id: Some("search-alb-2".to_string()),
                    album_name: Some("Search Album 2".to_string()),
                    ..Default::default()
                },
            ],
        })
    }

    fn requested_recommendations(
        &self,
        _req: RequestedRecommendationsRequest,
    ) -> MoosyncResult<RequestedRecommendationsResponse> {
        info!("requested_recommendations");
        Ok(RequestedRecommendationsResponse { songs: vec![] })
    }

    fn custom_request(&self, req: CustomRequest) -> MoosyncResult<CustomRequestResponse> {
        let request_id = req.request_id;
        info!("custom_request: {}", request_id);

        if request_id == "socket_test" {
            if let Ok(fd) = open_clientfd("/tmp/test.sock".to_string()) {
                let _ = write_sock(fd, vec![1, 2, 3]);
                let _ = read_sock(fd, 3);
            }
        }

        if request_id == "preferences_test" {
            let _ = register_user_preference(RegisterUserPreferenceRequest { prefs: vec![] });
            let _ = unregister_user_preference(UnregisterUserPreferenceRequest { keys: vec![] });
        }

        if request_id == "http_get_test" {
            let _ = moosync_edk::http::get("https://example.com", None);
        }

        if request_id == "http_request_test" {
            let req = moosync_edk::http::HttpRequest::post("https://example.com")
                .header("X-Test", "Value")
                .body(vec![1, 2, 3])
                .timeout_ms(5000);
            let _ = moosync_edk::http::request(&req);
        }

        if request_id == "http_batch_get_test" {
            let _ = moosync_edk::http::batch_get(
                &["https://example.com/1", "https://example.com/2"],
                None,
            );
        }

        if request_id == "http_batch_request_test" {
            let reqs = vec![
                moosync_edk::http::HttpRequest::get("https://example.com/1"),
                moosync_edk::http::HttpRequest::post("https://example.com/2").body(vec![4, 5, 6]),
            ];
            let _ = moosync_edk::http::batch_request(&reqs);
        }

        Ok(CustomRequestResponse {
            mime_type: None,
            data: None,
            redirect_url: None,
        })
    }

    fn requested_artist_songs(
        &self,
        req: RequestedArtistSongsRequest,
    ) -> MoosyncResult<RequestedArtistSongsResponse> {
        if let Some(artist) = &req.artist {
            info!("requested_artist_songs: {:?}", artist.artist_name);
        }
        let (songs, next_page_token) =
            get_paginated_sample_songs("artist", "Artist", req.page_token.as_deref());
        Ok(RequestedArtistSongsResponse {
            songs,
            next_page_token,
        })
    }

    fn requested_album_songs(
        &self,
        req: RequestedAlbumSongsRequest,
    ) -> MoosyncResult<RequestedAlbumSongsResponse> {
        if let Some(album) = &req.album {
            info!("requested_album_songs: {:?}", album.album_name);
        }
        let (songs, next_page_token) =
            get_paginated_sample_songs("album", "Album", req.page_token.as_deref());
        Ok(RequestedAlbumSongsResponse {
            songs,
            next_page_token,
        })
    }

    fn requested_song_from_id(
        &self,
        req: RequestedSongFromIdRequest,
    ) -> MoosyncResult<RequestedSongFromIdResponse> {
        info!("requested_song_from_id: {}", req.id);
        Ok(RequestedSongFromIdResponse { song: None })
    }

    fn scrobble(&self, req: ScrobbleRequest) -> MoosyncResult<()> {
        if let Some(song) = req.song {
            if let Some(inner) = &song.song {
                info!("scrobble: {:?}", inner.title);
            }
        }
        Ok(())
    }

    fn requested_lyrics(
        &self,
        req: RequestedLyricsRequest,
    ) -> MoosyncResult<RequestedLyricsResponse> {
        if let Some(song) = req.song {
            if let Some(inner) = &song.song {
                info!("requested_lyrics: {:?}", inner.title);
            }
        }
        let lines = (1..=100)
            .map(|i| LyricLine {
                text: format!("Sample lyric line {i}"),
                time_ms: ((i - 1) * 500) as i64,
            })
            .collect();
        Ok(RequestedLyricsResponse {
            lyrics: Some(Lyrics {
                lines,
                is_synced: true,
            }),
        })
    }

    fn requested_cover_art(
        &self,
        req: RequestedCoverArtRequest,
    ) -> MoosyncResult<RequestedCoverArtResponse> {
        info!("requested_cover_art called");
        let url = match req.item {
            Some(requested_cover_art_request::Item::Song(song)) => {
                if let Some(inner) = &song.song
                    && let Some(playback_url) = &inner.playback_url
                    && playback_url.starts_with("http")
                {
                    playback_url.clone()
                } else {
                    let id = song
                        .song
                        .as_ref()
                        .and_then(|s| s.id.as_deref())
                        .unwrap_or("sample_song");
                    format!("http://127.0.0.1:8080/covers/song_{id}.png")
                }
            }
            Some(requested_cover_art_request::Item::Album(album)) => {
                if let Some(artist) = &album.album_artist
                    && artist.starts_with("http")
                {
                    artist.clone()
                } else {
                    let id = album.album_id.as_deref().unwrap_or("sample_album");
                    format!("http://127.0.0.1:8080/covers/album_{id}.png")
                }
            }
            Some(requested_cover_art_request::Item::Artist(artist)) => {
                if let Some(id) = &artist.artist_id
                    && id.starts_with("http")
                {
                    id.clone()
                } else if let Some(name) = &artist.artist_name
                    && name.starts_with("http")
                {
                    name.clone()
                } else {
                    let id = artist.artist_id.as_deref().unwrap_or("sample_artist");
                    format!("http://127.0.0.1:8080/covers/artist_{id}.png")
                }
            }
            None => return Ok(RequestedCoverArtResponse { cover_urls: vec![] }),
        };

        Ok(RequestedCoverArtResponse {
            cover_urls: vec![url],
        })
    }
}

impl ContextMenu for SampleExtension {
    fn requested_song_context_menu(
        &self,
        _req: RequestedSongContextMenuRequest,
    ) -> MoosyncResult<RequestedSongContextMenuResponse> {
        info!("requested_song_context_menu");
        Ok(RequestedSongContextMenuResponse {
            menu: Some(ContextMenuReturnType {
                action_id: "sample_song_action".to_string(),
                name: "Sample Song Action".to_string(),
                icon: String::new(),
            }),
        })
    }

    fn requested_playlist_context_menu(
        &self,
        _req: RequestedPlaylistContextMenuRequest,
    ) -> MoosyncResult<RequestedPlaylistContextMenuResponse> {
        info!("requested_playlist_context_menu");
        Ok(RequestedPlaylistContextMenuResponse {
            menu: Some(ContextMenuReturnType {
                action_id: "sample_playlist_action".to_string(),
                name: "Sample Playlist Action".to_string(),
                icon: String::new(),
            }),
        })
    }

    fn context_menu_action(
        &self,
        req: ContextMenuActionRequest,
    ) -> MoosyncResult<ContextMenuActionResponse> {
        let action = req.action_id;
        info!("context_menu_action: {}", action);
        if action == "sample_song_action" {
            let _ = add_songs(AddSongsRequest {
                songs: vec![mock_sample_song("ctx_action_song", "Context Action Song")],
            });
        } else if action == "sample_playlist_action" {
            let _ = add_playlist(AddPlaylistRequest {
                playlist: Some(Playlist {
                    playlist_id: Some("ctx_action_playlist".to_string()),
                    playlist_name: "Context Action Playlist".to_string(),
                    ..Default::default()
                }),
            });
        } else if action == "add_test" {
            let _ = add_playlist(AddPlaylistRequest {
                playlist: Some(Playlist::default()),
            });
            let _ = add_songs(AddSongsRequest { songs: vec![] });
            let _ = add_to_playlist(AddToPlaylistRequest::default());
        }
        Ok(ContextMenuActionResponse {})
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn init() {
    register_extension(Box::new(SampleExtension)).unwrap();
    sync_account();
    let _ = register_user_preference(RegisterUserPreferenceRequest {
        prefs: vec![
            PreferenceItem {
                id: "api_key".to_string(),
                pref_type: PreferenceType::TextInputGroup as i32,
                title: "API Key".to_string(),
                subtitle: "API Key for Sample Extension".to_string(),
                ..Default::default()
            },
            PreferenceItem {
                id: "enable_feature".to_string(),
                pref_type: PreferenceType::ToggleGroup as i32,
                title: "Enable Feature".to_string(),
                subtitle: "Toggle sample feature".to_string(),
                ..Default::default()
            },
        ],
    });
}
