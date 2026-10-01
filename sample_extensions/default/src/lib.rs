#![no_main]

use std::{
    sync::atomic::{AtomicBool, Ordering},
    time::Duration,
};

use moosync_edk::{
    AddToPlaylistRequest, Album, Artist, ContextMenuActionRequest, ContextMenuReturnType,
    CustomRequest, ExtensionAccountDetail, ExtensionProviderScope, InnerSong, LyricLine, Lyrics,
    OauthCallbackRequest, PerformAccountLoginRequest, PlaybackDetailsRequestedRequest,
    PlayerStateChangedRequest, Playlist, PlaylistAddedRequest, PlaylistRemovedRequest,
    PreferenceChangedRequest, PreferenceData, RequestedAlbumSongsRequest,
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
        extension_api::{
            add_playlist, add_songs, add_to_playlist, get_current_song, get_player_state,
            get_preference, get_queue, get_secure, get_system_time, get_time, get_volume,
            open_external_url, open_sock, read_sock, register_oauth, register_user_preferences,
            set_account, unregister_user_preferences, write_sock,
        },
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
            Some("SampleUser".to_string())
        } else {
            None
        },
        ..Default::default()
    });
}

struct SampleExtension;

impl Extension for SampleExtension {}

impl Accounts for SampleExtension {
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

impl DatabaseEvents for SampleExtension {
    fn on_song_added(&self, req: SongAddedRequest) -> api::MoosyncResult<()> {
        for song in req.songs {
            if let Some(inner) = &song.song {
                info!("on_song_added: {:?}", inner.title);
            }
        }
        Ok(())
    }

    fn on_song_removed(&self, req: SongRemovedRequest) -> api::MoosyncResult<()> {
        for song in req.songs {
            if let Some(inner) = &song.song {
                info!("on_song_removed: {:?}", inner.title);
            }
        }
        Ok(())
    }

    fn on_playlist_added(&self, req: PlaylistAddedRequest) -> api::MoosyncResult<()> {
        for playlist in req.playlists {
            info!("on_playlist_added: {:?}", playlist.playlist_name);
        }
        Ok(())
    }

    fn on_playlist_removed(&self, req: PlaylistRemovedRequest) -> api::MoosyncResult<()> {
        for playlist in req.playlists {
            info!("on_playlist_removed: {:?}", playlist.playlist_name);
        }
        Ok(())
    }
}

impl PreferenceEvents for SampleExtension {
    fn on_preferences_changed(&self, req: PreferenceChangedRequest) -> api::MoosyncResult<()> {
        if let Some(args) = req.preference {
            info!("on_preferences_changed: {:?}", args);
            let key = args.key;
            if !key.is_empty() {
                let _val = get_preference(PreferenceData {
                    key: key.clone(),
                    ..Default::default()
                });
                let _sec = get_secure(PreferenceData {
                    key,
                    ..Default::default()
                });
            }
        }
        Ok(())
    }
}

impl PlayerEvents for SampleExtension {
    fn on_queue_changed(&self, _req: SongQueueChangedRequest) -> api::MoosyncResult<()> {
        info!("on_queue_changed");
        let _q = get_queue();
        Ok(())
    }

    fn on_volume_changed(&self, _req: VolumeChangedRequest) -> api::MoosyncResult<()> {
        info!("on_volume_changed");
        let _vol = get_volume();
        Ok(())
    }

    fn on_player_state_changed(&self, _req: PlayerStateChangedRequest) -> api::MoosyncResult<()> {
        info!("on_player_state_changed");
        let _state = get_player_state();
        Ok(())
    }

    fn on_song_changed(&self, _req: SongChangedRequest) -> api::MoosyncResult<()> {
        info!("on_song_changed");
        let _song = get_current_song();
        Ok(())
    }

    fn on_seeked(&self, req: SeekedRequest) -> api::MoosyncResult<()> {
        info!("on_seeked: {}", req.position);
        let _t = get_time();
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
        info!("get_playlists called");
        Ok(vec![
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
        ])
    }

    fn get_playlist_content(
        &self,
        req: RequestedPlaylistSongsRequest,
    ) -> api::MoosyncResult<SongsWithPageTokenReturnType> {
        info!("get_playlist_content: {}", req.id);
        let (songs, next_page_token) =
            get_paginated_sample_songs("pl", "Playlist", req.page_token.as_deref());
        Ok(SongsWithPageTokenReturnType {
            songs,
            next_page_token,
        })
    }

    fn get_playlist_from_url(
        &self,
        req: RequestedPlaylistFromUrlRequest,
    ) -> api::MoosyncResult<Option<Playlist>> {
        info!("get_playlist_from_url: {}", req.url);
        Ok(None)
    }

    fn get_playback_details(
        &self,
        req: PlaybackDetailsRequestedRequest,
    ) -> api::MoosyncResult<PlaybackDetailsReturnType> {
        if let Some(song) = req.song {
            if let Some(inner) = &song.song {
                info!("get_playback_details: {:?}", inner.id);
            }
        }
        Ok(PlaybackDetailsReturnType {
            duration: Duration::ZERO,
            url: "https://example.com/song.mp3".to_string(),
        })
    }

    fn search(&self, req: RequestedSearchResultRequest) -> api::MoosyncResult<SearchResult> {
        info!("search: {}", req.query);
        let _time = get_system_time();
        let _ = open_external_url("https://google.com".to_string());
        Ok(SearchResult {
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
            genres: vec![],
        })
    }

    fn get_recommendations(
        &self,
        _req: RequestedRecommendationsRequest,
    ) -> api::MoosyncResult<Vec<Song>> {
        info!("get_recommendations");
        Ok(vec![])
    }

    fn get_song_from_url(
        &self,
        req: RequestedSongFromUrlRequest,
    ) -> api::MoosyncResult<Option<Song>> {
        info!("get_song_from_url: {}", req.url);
        Ok(None)
    }

    fn handle_custom_request(
        &self,
        req: CustomRequest,
    ) -> api::MoosyncResult<CustomRequestReturnType> {
        let request_id = req.request_id;
        info!("handle_custom_request: {}", request_id);

        if request_id == "socket_test" {
            if let Ok(fd) = open_sock("/tmp/test.sock".to_string()) {
                let _ = write_sock(fd, vec![1, 2, 3]);
                let _ = read_sock(fd, 3);
            }
        }

        if request_id == "preferences_test" {
            let _ = register_user_preferences(vec![]);
            let _ = unregister_user_preferences(vec![]);
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

        Ok(CustomRequestReturnType {
            mime_type: None,
            data: None,
            redirect_url: None,
        })
    }

    fn get_artist_songs(
        &self,
        req: RequestedArtistSongsRequest,
    ) -> api::MoosyncResult<SongsWithPageTokenReturnType> {
        if let Some(artist) = &req.artist {
            info!("get_artist_songs: {:?}", artist.artist_name);
        }
        let (songs, next_page_token) =
            get_paginated_sample_songs("artist", "Artist", req.page_token.as_deref());
        Ok(SongsWithPageTokenReturnType {
            songs,
            next_page_token,
        })
    }

    fn get_album_songs(
        &self,
        req: RequestedAlbumSongsRequest,
    ) -> api::MoosyncResult<SongsWithPageTokenReturnType> {
        if let Some(album) = &req.album {
            info!("get_album_songs: {:?}", album.album_name);
        }
        let (songs, next_page_token) =
            get_paginated_sample_songs("album", "Album", req.page_token.as_deref());
        Ok(SongsWithPageTokenReturnType {
            songs,
            next_page_token,
        })
    }

    fn get_song_from_id(
        &self,
        req: RequestedSongFromIdRequest,
    ) -> api::MoosyncResult<Option<Song>> {
        info!("get_song_from_id: {}", req.id);
        Ok(None)
    }

    fn scrobble(&self, req: ScrobbleRequest) -> api::MoosyncResult<()> {
        if let Some(song) = req.song {
            if let Some(inner) = &song.song {
                info!("scrobble: {:?}", inner.title);
            }
        }
        Ok(())
    }

    fn get_lyrics(&self, req: RequestedLyricsRequest) -> api::MoosyncResult<Option<Lyrics>> {
        if let Some(song) = req.song {
            if let Some(inner) = &song.song {
                info!("get_lyrics: {:?}", inner.title);
            }
        }
        let lines = (1..=100)
            .map(|i| LyricLine {
                text: format!("Sample lyric line {i}"),
                time_ms: ((i - 1) * 500) as i64,
            })
            .collect();
        Ok(Some(Lyrics {
            lines,
            is_synced: true,
        }))
    }
}

impl ContextMenu for SampleExtension {
    fn get_song_context_menu(
        &self,
        _req: RequestedSongContextMenuRequest,
    ) -> api::MoosyncResult<Vec<ContextMenuReturnType>> {
        info!("get_song_context_menu");
        Ok(vec![])
    }

    fn get_playlist_context_menu(
        &self,
        _req: RequestedPlaylistContextMenuRequest,
    ) -> api::MoosyncResult<Vec<ContextMenuReturnType>> {
        info!("get_playlist_context_menu");
        Ok(vec![])
    }

    fn on_context_menu_action(&self, req: ContextMenuActionRequest) -> api::MoosyncResult<()> {
        let action = req.action_id;
        info!("on_context_menu_action: {}", action);
        if action == "add_test" {
            let _ = add_playlist(Playlist::default());
            let _ = add_songs(vec![]);
            let _ = add_to_playlist(AddToPlaylistRequest::default());
        }
        Ok(())
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn init() {
    moosync_edk::handler::register_extension(Box::new(SampleExtension)).unwrap();
    sync_account();
    let _ = register_user_preferences(vec![
        moosync_edk::PreferenceItem {
            id: "api_key".to_string(),
            pref_type: moosync_edk::PreferenceType::TextInputGroup as i32,
            title: "API Key".to_string(),
            subtitle: "API Key for Sample Extension".to_string(),
            ..Default::default()
        },
        moosync_edk::PreferenceItem {
            id: "enable_feature".to_string(),
            pref_type: moosync_edk::PreferenceType::ToggleGroup as i32,
            title: "Enable Feature".to_string(),
            subtitle: "Toggle sample feature".to_string(),
            ..Default::default()
        },
    ]);
}
