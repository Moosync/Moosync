use std::sync::{Arc, Mutex};
use slint::{ComponentHandle, Image, ModelRc, Weak};
use songs_proto::moosync::types::{
    Genre, GenreList, GetEntityOptions, entity_result,
};
use state_manager::StateManager;
use tracing::debug;
use types::ScanProgress;
use types::errors::MoosyncError;

use crate::GenreModel;
use crate::utils::LazySongVecModel;
use crate::{MainWindow, Pages, pages::PageHandler};

pub struct GenresPageHandler<'a> {
    main_window: &'a MainWindow,
    state_manager: &'a StateManager,
    genres: Arc<Mutex<Vec<Genre>>>,
}

impl<'a> GenresPageHandler<'a> {
    pub fn new(main_window: &'a MainWindow, state_manager: &'a StateManager) -> Self {
        Self {
            main_window,
            state_manager,
            genres: Arc::new(Mutex::new(Vec::new())),
        }
    }

    fn set_scanner_cb(&self) {
        let main_window_weak = self.main_window.as_weak();
        let state_manager = self.state_manager.clone();
        let genres_cache = Arc::clone(&self.genres);
        tokio::task::spawn(async move {
            run_scanner_loop(main_window_weak, state_manager, genres_cache).await;
        });
        debug!("Scanner callback set");
    }
}

async fn run_scanner_loop(
    main_window_weak: Weak<MainWindow>,
    state_manager: StateManager,
    genres_cache: Arc<Mutex<Vec<Genre>>>,
) {
    let mut progress = {
        let scanner = state_manager.get_scanner().await;
        scanner.add_subscriber()
    };

    while let Some(p) = progress.recv().await {
        if p == ScanProgress::STOPPED {
            fetch_and_cache_genres(main_window_weak.clone(), state_manager.clone(), genres_cache.clone()).await;
        }
    }
}

async fn get_genres_from_db(
    state_manager: &StateManager,
) -> Result<Vec<Genre>, MoosyncError> {
    let database = state_manager.get_database().await;
    let genres_res = database.get_entity_by_options(GetEntityOptions {
        genre: Some(Genre::default()),
        ..Default::default()
    })?;

    match genres_res.result {
        Some(entity_result::Result::Genres(GenreList { genres })) => Ok(genres),
        _ => Err(MoosyncError::String("Failed to get genres from db".to_string())),
    }
}

async fn fetch_and_cache_genres(
    main_window_weak: Weak<MainWindow>,
    state_manager: StateManager,
    genres_cache: Arc<Mutex<Vec<Genre>>>,
) {
    if let Ok(genres) = get_genres_from_db(&state_manager).await {
        *genres_cache.lock().unwrap() = genres.clone();
        let _ = slint::invoke_from_event_loop(move || {
            if let Some(main_window) = main_window_weak.upgrade() {
                if main_window.get_active_page() == Pages::Genres {
                    set_all_genres(&main_window, genres);
                }
            }
        });
    }
}

fn set_all_genres(main_window: &MainWindow, genres: Vec<Genre>) {
    debug!("Setting genres");
    let genre_model = genres
        .into_iter()
        .map(|genre| GenreModel {
            coverPath: Image::default(),
            coverPathUrl: "".into(),
            id: genre.genre_id.clone().unwrap_or_default().into(),
            songs_count: genre.genre_song_count as i32,
            title: genre.genre_name.clone().unwrap_or_default().into(),
        })
        .collect::<Vec<_>>();

    main_window.set_genres(ModelRc::new(LazySongVecModel::new(
        genre_model,
        230,
        200,
    )));
}

impl<'a> PageHandler for GenresPageHandler<'a> {
    fn initialize(&self) {
        self.set_scanner_cb();
        let state_manager = self.state_manager.clone();
        let main_window_weak = self.main_window.as_weak();
        let genres_cache = Arc::clone(&self.genres);
        tokio::spawn(async move {
            fetch_and_cache_genres(main_window_weak, state_manager, genres_cache).await;
        });
    }

    fn on_show(&self) {
        let genres = self.genres.lock().unwrap().clone();
        set_all_genres(self.main_window, genres);
    }

    fn on_hide(&self) {
        self.main_window.set_genres(ModelRc::default());
    }
}
