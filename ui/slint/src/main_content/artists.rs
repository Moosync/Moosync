use std::sync::{Arc, Mutex};

use slint::{ComponentHandle, ModelRc, Weak};
use songs_proto::moosync::types::{Artist, ArtistList, GetEntityOptions, entity_result};
use state_manager::StateManager;
use tracing::debug;
use types::{ScanProgress, errors::MoosyncError};

use crate::{MainWindow, Pages, pages::PageHandler, utils::LazySongVecModel};

pub struct ArtistsPageHandler<'a> {
    main_window: &'a MainWindow,
    state_manager: &'a StateManager,
    artists: Arc<Mutex<Vec<Artist>>>,
}

impl<'a> ArtistsPageHandler<'a> {
    pub fn new(main_window: &'a MainWindow, state_manager: &'a StateManager) -> Self {
        Self {
            main_window,
            state_manager,
            artists: Arc::new(Mutex::new(Vec::new())),
        }
    }

    fn set_scanner_cb(&self) {
        let main_window_weak = self.main_window.as_weak();
        let state_manager = self.state_manager.clone();
        let artists_cache = Arc::clone(&self.artists);
        tokio::task::spawn(async move {
            run_scanner_loop(main_window_weak, state_manager, artists_cache).await;
        });
        debug!("Scanner callback set");
    }
}

async fn run_scanner_loop(
    main_window_weak: Weak<MainWindow>,
    state_manager: StateManager,
    artists_cache: Arc<Mutex<Vec<Artist>>>,
) {
    let mut progress = {
        let scanner = state_manager.get_scanner_holder().await;
        scanner.add_subscriber()
    };

    while let Some(p) = progress.recv().await {
        if p == ScanProgress::STOPPED {
            fetch_and_cache_artists(
                main_window_weak.clone(),
                state_manager.clone(),
                artists_cache.clone(),
            )
            .await;
        }
    }
}

async fn get_artists_from_db(state_manager: &StateManager) -> Result<Vec<Artist>, MoosyncError> {
    let database = state_manager.get_database().await;
    let artists_res = database.get_entity_by_options(GetEntityOptions {
        artist: Some(Artist::default()),
        ..Default::default()
    })?;

    match artists_res.result {
        Some(entity_result::Result::Artists(ArtistList { artists })) => Ok(artists),
        _ => Err(MoosyncError::String(
            "Failed to get artists from db".to_string(),
        )),
    }
}

async fn fetch_and_cache_artists(
    main_window_weak: Weak<MainWindow>,
    state_manager: StateManager,
    artists_cache: Arc<Mutex<Vec<Artist>>>,
) {
    if let Ok(artists) = get_artists_from_db(&state_manager).await {
        *artists_cache.lock().unwrap() = artists.clone();
        let cache_dir = state_manager.get_cache_dir();
        let _ = slint::invoke_from_event_loop(move || {
            if let Some(main_window) = main_window_weak.upgrade() {
                if main_window.get_active_page() == Pages::Artists {
                    set_all_artists(&main_window, artists, cache_dir);
                }
            }
        });
    }
}

fn set_all_artists(main_window: &MainWindow, artists: Vec<Artist>, cache_dir: std::path::PathBuf) {
    debug!("Setting artists");
    let artist_model = artists
        .into_iter()
        .map(|artist| crate::utils::to_artist_model(&artist))
        .collect::<Vec<_>>();

    main_window.set_artists(ModelRc::new(LazySongVecModel::new(
        artist_model,
        230,
        200,
        cache_dir,
    )));
}

impl<'a> PageHandler for ArtistsPageHandler<'a> {
    fn initialize(&self) {
        self.set_scanner_cb();
        let state_manager = self.state_manager.clone();
        let main_window_weak = self.main_window.as_weak();
        let artists_cache = Arc::clone(&self.artists);
        tokio::spawn(async move {
            fetch_and_cache_artists(main_window_weak, state_manager, artists_cache).await;
        });
    }

    fn on_show(&self) {
        let artists = self.artists.lock().unwrap().clone();
        let cache_dir = self.state_manager.get_cache_dir();
        set_all_artists(self.main_window, artists, cache_dir);
    }

    fn on_hide(&self) { self.main_window.set_artists(ModelRc::default()); }
}
