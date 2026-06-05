use slint::{ComponentHandle, ModelRc};
use songs_proto::moosync::types::{
    Album, Artist, Genre, GetEntityOptions, GetSongOptions, Playlist, SearchableSong,
};
use state_manager::StateManager;
use types::prelude::EntityResultExt;

use crate::{
    AppCallbacks, MainWindow, SearchResult,
    pages::PageHandler,
    utils::{
        LazySongVecModel, to_album_model, to_artist_model, to_genre_model, to_playlist_model,
        to_song_model,
    },
};

pub struct SearchPageHandler<'a> {
    main_window: &'a MainWindow,
    state_manager: &'a StateManager,
}

impl<'a> SearchPageHandler<'a> {
    pub fn new(main_window: &'a MainWindow, state_manager: &'a StateManager) -> Self {
        Self {
            main_window,
            state_manager,
        }
    }
}

impl<'a> PageHandler for SearchPageHandler<'a> {
    fn initialize(&self) {
        let state_manager = self.state_manager.clone();
        let main_window_weak = self.main_window.as_weak();
        self.main_window
            .global::<AppCallbacks>()
            .on_search_term_changed(move |term| {
                let state_manager = state_manager.clone();
                let main_window_weak = main_window_weak.clone();
                tokio::spawn(async move {
                    let database = state_manager.get_database().await;
                    let songs = database
                        .get_songs_by_options(GetSongOptions {
                            song: Some(SearchableSong {
                                title: Some(format!("%{}%", term)),
                                ..Default::default()
                            }),
                            ..Default::default()
                        })
                        .unwrap_or_default();

                    let albums = database
                        .get_entity_by_options(GetEntityOptions {
                            album: Some(Album {
                                album_name: Some(format!("%{}%", term)),
                                ..Default::default()
                            }),
                            ..Default::default()
                        })
                        .unwrap_or_default()
                        .get_albums()
                        .unwrap_or_default();

                    let artists = database
                        .get_entity_by_options(GetEntityOptions {
                            artist: Some(Artist {
                                artist_name: Some(format!("%{}%", term)),
                                ..Default::default()
                            }),
                            ..Default::default()
                        })
                        .unwrap_or_default()
                        .get_artists()
                        .unwrap_or_default();

                    let playlists = database
                        .get_entity_by_options(GetEntityOptions {
                            playlist: Some(Playlist {
                                playlist_name: format!("%{}%", term),
                                ..Default::default()
                            }),
                            ..Default::default()
                        })
                        .unwrap_or_default()
                        .get_playlists()
                        .unwrap_or_default();

                    let genres = database
                        .get_entity_by_options(GetEntityOptions {
                            genre: Some(Genre {
                                genre_name: Some(format!("%{}%", term)),
                                ..Default::default()
                            }),
                            ..Default::default()
                        })
                        .unwrap_or_default()
                        .get_genres()
                        .unwrap_or_default();

                    main_window_weak.upgrade_in_event_loop(move |main_window| {
                        main_window.set_search_results(SearchResult {
                            albums: ModelRc::new(LazySongVecModel::new(
                                albums.iter().map(|a| to_album_model(a)).collect(),
                                230,
                                200,
                                state_manager.get_cache_dir(),
                            )),
                            artists: ModelRc::new(LazySongVecModel::new(
                                artists.iter().map(|a| to_artist_model(a)).collect(),
                                230,
                                200,
                                state_manager.get_cache_dir(),
                            )),
                            genres: ModelRc::new(LazySongVecModel::new(
                                genres.iter().map(|g| to_genre_model(g)).collect(),
                                230,
                                200,
                                state_manager.get_cache_dir(),
                            )),
                            playlists: ModelRc::new(LazySongVecModel::new(
                                playlists.iter().map(|p| to_playlist_model(p)).collect(),
                                230,
                                200,
                                state_manager.get_cache_dir(),
                            )),
                            songs: ModelRc::new(LazySongVecModel::new(
                                songs.iter().map(|s| to_song_model(s)).collect(),
                                60,
                                0,
                                state_manager.get_cache_dir(),
                            )),
                        });
                    });
                });
            });
    }
    fn on_show(&self) {}
    fn on_hide(&self) {}
}
