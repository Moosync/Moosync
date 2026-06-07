use slint::{ComponentHandle, ModelRc};
use state_manager::StateManager;

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
                    match database.search_all(&term) {
                        Ok(search_res) => {
                            let _ = main_window_weak.upgrade_in_event_loop(move |main_window| {
                                main_window.set_search_results(SearchResult {
                                    albums: ModelRc::new(LazySongVecModel::new(
                                        search_res
                                            .albums
                                            .iter()
                                            .map(|a| to_album_model(a))
                                            .collect(),
                                        230,
                                        200,
                                        state_manager.get_cache_dir(),
                                    )),
                                    artists: ModelRc::new(LazySongVecModel::new(
                                        search_res
                                            .artists
                                            .iter()
                                            .map(|a| to_artist_model(a))
                                            .collect(),
                                        230,
                                        200,
                                        state_manager.get_cache_dir(),
                                    )),
                                    genres: ModelRc::new(LazySongVecModel::new(
                                        search_res
                                            .genres
                                            .iter()
                                            .map(|g| to_genre_model(g))
                                            .collect(),
                                        230,
                                        200,
                                        state_manager.get_cache_dir(),
                                    )),
                                    playlists: ModelRc::new(LazySongVecModel::new(
                                        search_res
                                            .playlists
                                            .iter()
                                            .map(|p| to_playlist_model(p))
                                            .collect(),
                                        230,
                                        200,
                                        state_manager.get_cache_dir(),
                                    )),
                                    songs: ModelRc::new(LazySongVecModel::new(
                                        search_res.songs.iter().map(|s| to_song_model(s)).collect(),
                                        60,
                                        0,
                                        state_manager.get_cache_dir(),
                                    )),
                                });
                            });
                        }
                        Err(e) => tracing::error!("Search failed for term '{}': {:?}", term, e),
                    }
                });
            });
    }
    fn on_show(&self) {}
    fn on_hide(&self) {}
}
