use slint::{ComponentHandle, Image, ModelRc, Weak};
use songs_proto::moosync::types::{GetEntityOptions, Playlist, PlaylistList, entity_result};
use state_manager::StateManager;
use std::sync::{Arc, Mutex};
use tracing::debug;
use types::ScanProgress;
use types::errors::MoosyncError;

use crate::PlaylistModel;
use crate::utils::LazySongVecModel;
use crate::{MainWindow, Pages, pages::PageHandler};

pub struct PlaylistsPageHandler<'a> {
    main_window: &'a MainWindow,
    state_manager: &'a StateManager,
    playlists: Arc<Mutex<Vec<Playlist>>>,
}

impl<'a> PlaylistsPageHandler<'a> {
    pub fn new(main_window: &'a MainWindow, state_manager: &'a StateManager) -> Self {
        Self {
            main_window,
            state_manager,
            playlists: Arc::new(Mutex::new(Vec::new())),
        }
    }

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

async fn get_playlists_from_db(
    state_manager: &StateManager,
) -> Result<Vec<Playlist>, MoosyncError> {
    let database = state_manager.get_database().await;
    let playlists_res = database.get_entity_by_options(GetEntityOptions {
        playlist: Some(Playlist::default()),
        ..Default::default()
    })?;

    match playlists_res.result {
        Some(entity_result::Result::Playlists(PlaylistList { playlists })) => Ok(playlists),
        _ => Err(MoosyncError::String(
            "Failed to get playlists from db".to_string(),
        )),
    }
}

async fn fetch_and_cache_playlists(
    main_window_weak: Weak<MainWindow>,
    state_manager: StateManager,
    playlists_cache: Arc<Mutex<Vec<Playlist>>>,
) {
    if let Ok(playlists) = get_playlists_from_db(&state_manager).await {
        *playlists_cache.lock().unwrap() = playlists.clone();
        let _ = slint::invoke_from_event_loop(move || {
            if let Some(main_window) = main_window_weak.upgrade() {
                if main_window.get_active_page() == Pages::Playlists {
                    set_all_playlists(&main_window, playlists);
                }
            }
        });
    }
}

fn set_all_playlists(main_window: &MainWindow, playlists: Vec<Playlist>) {
    debug!("Setting playlists");
    let playlist_model = playlists
        .into_iter()
        .map(|playlist| PlaylistModel {
            coverPath: Image::default(),
            coverPathUrl: playlist
                .playlist_coverpath
                .clone()
                .unwrap_or_default()
                .into(),
            id: playlist.playlist_id.clone().unwrap_or_default().into(),
            songs_count: playlist.playlist_song_count as i32,
            title: playlist.playlist_name.clone().into(),
        })
        .collect::<Vec<_>>();

    main_window.set_playlists(ModelRc::new(LazySongVecModel::new(
        playlist_model,
        230,
        200,
    )));
}

impl<'a> PageHandler for PlaylistsPageHandler<'a> {
    fn initialize(&self) {
        self.set_scanner_cb();
        let state_manager = self.state_manager.clone();
        let main_window_weak = self.main_window.as_weak();
        let playlists_cache = Arc::clone(&self.playlists);
        tokio::spawn(async move {
            fetch_and_cache_playlists(main_window_weak, state_manager, playlists_cache).await;
        });
    }

    fn on_show(&self) {
        let playlists = self.playlists.lock().unwrap().clone();
        set_all_playlists(self.main_window, playlists);
    }

    fn on_hide(&self) {
        self.main_window.set_playlists(ModelRc::default());
    }
}
