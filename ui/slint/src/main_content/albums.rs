use std::sync::{Arc, Mutex};

use slint::{ComponentHandle, ModelRc, Weak};
use songs_proto::moosync::types::{Album, AlbumList, GetEntityOptions, entity_result};
use state_manager::StateManager;
use tracing::debug;
use types::{ScanProgress, errors::MoosyncError};

use crate::{MainWindow, Pages, pages::PageHandler, utils::LazySongVecModel};

pub struct AlbumsPageHandler<'a> {
    main_window: &'a MainWindow,
    state_manager: &'a StateManager,
    albums: Arc<Mutex<Vec<Album>>>,
}

impl<'a> AlbumsPageHandler<'a> {
    pub fn new(main_window: &'a MainWindow, state_manager: &'a StateManager) -> Self {
        Self {
            main_window,
            state_manager,
            albums: Arc::new(Mutex::new(Vec::new())),
        }
    }

    fn set_scanner_cb(&self) {
        let main_window_weak = self.main_window.as_weak();
        let state_manager = self.state_manager.clone();
        let albums_cache = Arc::clone(&self.albums);
        tokio::task::spawn(async move {
            run_scanner_loop(main_window_weak, state_manager, albums_cache).await;
        });
        debug!("Scanner callback set");
    }
}

async fn run_scanner_loop(
    main_window_weak: Weak<MainWindow>,
    state_manager: StateManager,
    albums_cache: Arc<Mutex<Vec<Album>>>,
) {
    let mut progress = {
        let scanner = state_manager.get_scanner_holder().await;
        scanner.add_subscriber()
    };

    while let Some(p) = progress.recv().await {
        if p == ScanProgress::STOPPED {
            fetch_and_cache_albums(
                main_window_weak.clone(),
                state_manager.clone(),
                albums_cache.clone(),
            )
            .await;
        }
    }
}

async fn get_albums_from_db(state_manager: &StateManager) -> Result<Vec<Album>, MoosyncError> {
    let database = state_manager.get_database().await;
    let albums_res = database.get_entity_by_options(GetEntityOptions {
        album: Some(Album::default()),
        ..Default::default()
    })?;

    match albums_res.result {
        Some(entity_result::Result::Albums(AlbumList { albums })) => Ok(albums),
        _ => Err(MoosyncError::String(
            "Failed to get albums from db".to_string(),
        )),
    }
}

async fn fetch_and_cache_albums(
    main_window_weak: Weak<MainWindow>,
    state_manager: StateManager,
    albums_cache: Arc<Mutex<Vec<Album>>>,
) {
    if let Ok(albums) = get_albums_from_db(&state_manager).await {
        *albums_cache.lock().unwrap() = albums.clone();
        let cache_dir = state_manager.get_cache_dir();
        let _ = slint::invoke_from_event_loop(move || {
            if let Some(main_window) = main_window_weak.upgrade() {
                if main_window.get_active_page() == Pages::Albums {
                    set_all_albums(&main_window, albums, cache_dir);
                }
            }
        });
    }
}

fn set_all_albums(main_window: &MainWindow, albums: Vec<Album>, cache_dir: std::path::PathBuf) {
    debug!("Setting albums");
    let album_model = albums
        .into_iter()
        .map(|album| crate::utils::to_album_model(&album))
        .collect::<Vec<_>>();

    let theme = main_window.global::<crate::Theme>();
    main_window.set_albums(ModelRc::new(LazySongVecModel::new(
        album_model,
        theme.get_cardHeight() as usize,
        theme.get_cardWidth() as usize,
        cache_dir,
    )));
}

impl<'a> PageHandler for AlbumsPageHandler<'a> {
    fn initialize(&self) {
        self.set_scanner_cb();
        let state_manager = self.state_manager.clone();
        let main_window_weak = self.main_window.as_weak();
        let albums_cache = Arc::clone(&self.albums);
        tokio::spawn(async move {
            fetch_and_cache_albums(main_window_weak, state_manager, albums_cache).await;
        });
    }

    fn on_show(&self) {
        let albums = self.albums.lock().unwrap().clone();
        let cache_dir = self.state_manager.get_cache_dir();
        set_all_albums(self.main_window, albums, cache_dir);
    }

    fn on_hide(&self) { self.main_window.set_albums(ModelRc::default()); }
}
