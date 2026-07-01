use std::sync::{Arc, Mutex};

use slint::{ComponentHandle, ModelRc, Weak};
use songs_proto::moosync::types::{GetEntityOptions, Playlist, PlaylistList, entity_result};
use state_manager::StateManager;
use tracing::debug;
use types::ScanProgress;

use crate::{MainWindow, Pages, error::UiError, pages::PageHandler, utils::LazySongVecModel};

pub struct PlaylistsPageHandler<'a> {
    main_window: &'a MainWindow,
    state_manager: &'a StateManager,
    playlists: Arc<Mutex<Vec<Playlist>>>,
}

impl<'a> PlaylistsPageHandler<'a> {
    #[tracing::instrument(level = "debug", skip_all)]
    pub fn new(main_window: &'a MainWindow, state_manager: &'a StateManager) -> Self {
        Self {
            main_window,
            state_manager,
            playlists: Arc::new(Mutex::new(Vec::new())),
        }
    }

    #[tracing::instrument(level = "debug", skip_all)]
    fn set_scanner_cb(&self) {
        let main_window_weak = self.main_window.as_weak();
        let state_manager = self.state_manager.clone();
        let playlists_cache = Arc::clone(&self.playlists);
        tokio::task::spawn(async move {
            run_scanner_loop(main_window_weak, state_manager, playlists_cache).await;
        });
        debug!("Scanner callback set");
    }
}

#[tracing::instrument(level = "debug", skip_all)]
async fn run_scanner_loop(
    main_window_weak: Weak<MainWindow>,
    state_manager: StateManager,
    playlists_cache: Arc<Mutex<Vec<Playlist>>>,
) {
    let mut progress = {
        let scanner = state_manager.get_scanner_holder().await;
        scanner.add_subscriber()
    };

    while let Some(p) = progress.recv().await {
        if p == ScanProgress::STOPPED {
            fetch_and_cache_playlists(
                main_window_weak.clone(),
                state_manager.clone(),
                playlists_cache.clone(),
            )
            .await;
        }
    }
}

#[tracing::instrument(level = "debug", skip_all)]
async fn get_playlists_from_db(state_manager: &StateManager) -> Result<Vec<Playlist>, UiError> {
    let database = state_manager.get_database().await;
    let playlists_res = database.get_entity_by_options(GetEntityOptions {
        playlist: Some(Playlist::default()),
        ..Default::default()
    })?;

    match playlists_res.result {
        Some(entity_result::Result::Playlists(PlaylistList { playlists })) => Ok(playlists),
        _ => Err(UiError::EntityParseFailed),
    }
}

#[tracing::instrument(level = "debug", skip_all)]
async fn fetch_and_cache_playlists(
    main_window_weak: Weak<MainWindow>,
    state_manager: StateManager,
    playlists_cache: Arc<Mutex<Vec<Playlist>>>,
) {
    if let Ok(playlists) = get_playlists_from_db(&state_manager).await {
        *playlists_cache.lock().unwrap() = playlists.clone();
        let cache_dir = state_manager.get_cache_dir();
        let _ = slint::invoke_from_event_loop(move || {
            if let Some(main_window) = main_window_weak.upgrade() {
                if main_window.get_active_page() == Pages::Playlists {
                    set_all_playlists(&main_window, playlists, cache_dir);
                }
            }
        });
    }
}

#[tracing::instrument(level = "debug", skip_all)]
fn set_all_playlists(
    main_window: &MainWindow,
    playlists: Vec<Playlist>,
    cache_dir: std::path::PathBuf,
) {
    debug!("Setting playlists");
    let playlist_model = playlists
        .into_iter()
        .map(|playlist| crate::utils::to_playlist_model(&playlist))
        .collect::<Vec<_>>();

    let theme = main_window.global::<crate::Theme>();
    main_window.set_playlists(ModelRc::new(LazySongVecModel::new(
        playlist_model,
        theme.get_cardHeight() as usize,
        theme.get_cardWidth() as usize,
        cache_dir,
    )));
}

impl<'a> PageHandler for PlaylistsPageHandler<'a> {
    #[tracing::instrument(level = "debug", skip_all)]
    fn initialize(&self) {
        self.set_scanner_cb();
        let state_manager = self.state_manager.clone();
        let main_window_weak = self.main_window.as_weak();
        let playlists_cache = Arc::clone(&self.playlists);
        tokio::spawn(async move {
            fetch_and_cache_playlists(main_window_weak, state_manager, playlists_cache).await;
        });
    }

    #[tracing::instrument(level = "debug", skip_all)]
    fn on_show(&self) {
        let playlists = self.playlists.lock().unwrap().clone();
        let cache_dir = self.state_manager.get_cache_dir();
        set_all_playlists(self.main_window, playlists, cache_dir);
    }

    #[tracing::instrument(level = "debug", skip_all)]
    fn on_hide(&self) { self.main_window.set_playlists(ModelRc::default()); }
}
