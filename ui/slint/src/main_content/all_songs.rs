use std::path::Path;
use std::sync::{Arc, Mutex};

use slint::{ComponentHandle, Image, ModelRc, Weak};
use songs_proto::moosync::types::{GetSongOptions, SearchableSong, Song};
use state_manager::StateManager;
use tracing::debug;
use types::errors::MoosyncError;
use types::{ScanProgress, prelude::SongsExt};

use crate::utils::LazySongVecModel;
use crate::{MainWindow, Pages, SongModel, pages::PageHandler};

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
        debug!("Scanner callback set");
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
    if let Ok(songs) = get_songs_from_db(&state_manager).await {
        *songs_cache.lock().unwrap() = songs.clone();
        let _ = slint::invoke_from_event_loop(move || {
            if let Some(main_window) = main_window_weak.upgrade() {
                if main_window.get_active_page() == Pages::AllSongs {
                    set_all_songs(&main_window, songs);
                }
            }
        });
    }
}

fn get_extension_icon(song: &Song) -> Image {
    if let Some(icon_path) = song.song.as_ref().and_then(|s| s.icon.clone()) {
        if let Ok(image) = Image::load_from_path(Path::new(&icon_path)) {
            return image;
        }
    }
    Image::load_from_svg_data(include_bytes!("../icons/empty.svg")).unwrap()
}

fn set_all_songs(main_window: &MainWindow, songs: Vec<Song>) {
    debug!("Setting songs");
    let songs_view = songs
        .into_iter()
        .map(|song| SongModel {
            id: song.get_id().unwrap_or_default().into(),
            title: song.get_title().unwrap_or_default().into(),
            artist_name: song.get_artist_string().unwrap_or_default().into(),
            album_name: song.get_album_string().unwrap_or_default().into(),
            duration: song.format_duration().into(),
            coverPathHigh: Image::default(),
            coverPathLow: Image::default(),
            extensionIcon: get_extension_icon(&song),
            coverPathUrlHigh: song.get_cover_high().unwrap_or_default().into(),
            coverPathUrlLow: song.get_cover_low().unwrap_or_default().into(),
        })
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
