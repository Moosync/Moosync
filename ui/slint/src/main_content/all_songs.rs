use std::sync::{Arc, Mutex};

use slint::{ComponentHandle, ModelRc, Weak};
use songs_proto::moosync::types::{GetSongOptions, SearchableSong, Song};
use state_manager::StateManager;
use tracing::debug;
use types::ScanProgress;
use types::errors::MoosyncError;

use crate::utils::LazySongVecModel;
use crate::{MainWindow, Pages, pages::PageHandler};

pub struct AllSongsPageHandler<'a> {
    main_window: &'a MainWindow,
    state_manager: &'a StateManager,
    songs: Arc<Mutex<Vec<Song>>>,
}

impl<'a> AllSongsPageHandler<'a> {
    pub fn new(main_window: &'a MainWindow, state_manager: &'a StateManager) -> Self {
        Self {
            main_window,
            state_manager,
            songs: Arc::new(Mutex::new(Vec::new())),
        }
    }

    fn set_scanner_cb(&self) {
        let main_window_weak = self.main_window.as_weak();
        let state_manager = self.state_manager.clone();
        let songs_cache = Arc::clone(&self.songs);
        tokio::task::spawn(async move {
            run_scanner_loop(main_window_weak, state_manager, songs_cache).await;
        });
    }
}

async fn run_scanner_loop(
    main_window_weak: Weak<MainWindow>,
    state_manager: StateManager,
    songs_cache: Arc<Mutex<Vec<Song>>>,
) {
    let mut progress = {
        let scanner = state_manager.get_scanner_holder().await;
        scanner.add_subscriber()
    };

    while let Some(p) = progress.recv().await {
        if p == ScanProgress::STOPPED {
            fetch_and_cache_songs(
                main_window_weak.clone(),
                state_manager.clone(),
                songs_cache.clone(),
            )
            .await;
        }
    }
}

async fn get_songs_from_db(state_manager: &StateManager) -> Result<Vec<Song>, MoosyncError> {
    let database = state_manager.get_database().await;
    database.get_songs_by_options(GetSongOptions {
        song: Some(SearchableSong::default()),
        ..Default::default()
    })
}

async fn fetch_and_cache_songs(
    main_window_weak: Weak<MainWindow>,
    state_manager: StateManager,
    songs_cache: Arc<Mutex<Vec<Song>>>,
) {
    match get_songs_from_db(&state_manager).await {
        Ok(songs) => {
            tracing::trace!("got songs {:?}", songs.len());
            *songs_cache.lock().unwrap() = songs.clone();
            let _ = slint::invoke_from_event_loop(move || {
                if let Some(main_window) = main_window_weak.upgrade() {
                    if main_window.get_active_page() == Pages::AllSongs {
                        set_all_songs(&main_window, songs);
                    }
                }
            });
        }
        Err(e) => tracing::error!("Failed to fetch songs: {:?}", e),
    }
}

fn set_all_songs(main_window: &MainWindow, songs: Vec<Song>) {
    debug!("Setting songs");
    let songs_view = songs
        .into_iter()
        .map(|song| crate::utils::to_song_model(Some(&song)))
        .collect::<Vec<_>>();

    main_window.set_songs(ModelRc::new(LazySongVecModel::new(songs_view, 60, 0)));
}

impl<'a> PageHandler for AllSongsPageHandler<'a> {
    fn initialize(&self) {
        self.set_scanner_cb();
        let state_manager = self.state_manager.clone();
        let main_window_weak = self.main_window.as_weak();
        let songs_cache = Arc::clone(&self.songs);
        tokio::spawn(async move {
            fetch_and_cache_songs(main_window_weak, state_manager, songs_cache).await;
        });
    }

    fn on_show(&self) {
        let songs = self.songs.lock().unwrap().clone();
        set_all_songs(self.main_window, songs);
    }

    fn on_hide(&self) {
        self.main_window.set_songs(ModelRc::default());
    }
}
