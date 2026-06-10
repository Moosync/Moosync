use slint::{ComponentHandle, ModelRc, Weak};
use songs_proto::moosync::types::{Artist, GetSongOptions};
use state_manager::StateManager;
use tracing::debug;

use crate::{MainWindow, Pages, pages::PageHandler, utils::LazySongVecModel};

pub struct ArtistContentPageHandler<'a> {
    main_window: &'a MainWindow,
    state_manager: &'a StateManager,
}

impl<'a> ArtistContentPageHandler<'a> {
    pub fn new(main_window: &'a MainWindow, state_manager: &'a StateManager) -> Self {
        Self {
            main_window,
            state_manager,
        }
    }
}

async fn fetch_and_set_songs(
    main_window_weak: Weak<MainWindow>,
    state_manager: StateManager,
    artist_id: String,
) {
    debug!("Fetching songs for artist ID: {}", artist_id);
    let database = state_manager.get_database().await;
    let options = GetSongOptions {
        artist: Some(Artist {
            artist_id: Some(artist_id),
            ..Default::default()
        }),
        ..Default::default()
    };
    match database.get_songs_by_options(options) {
        Ok(songs) => {
            debug!("Fetched {} songs for artist", songs.len());
            let cache_dir = state_manager.get_cache_dir();
            let _ = slint::invoke_from_event_loop(move || {
                if let Some(main_window) = main_window_weak.upgrade() {
                    if main_window.get_active_page() == Pages::ArtistContent {
                        let songs_view = songs
                            .iter()
                            .map(crate::utils::to_song_model)
                            .collect::<Vec<_>>();
                        main_window.set_content_songs(ModelRc::new(LazySongVecModel::new(
                            songs_view, 60, 0, cache_dir,
                        )));
                    }
                }
            });
        }
        Err(e) => {
            tracing::error!("Failed to fetch artist songs: {:?}", e)
        }
    }
}

impl<'a> PageHandler for ArtistContentPageHandler<'a> {
    fn initialize(&self) {}

    fn on_show(&self) {
        let selected = self.main_window.get_selected_entity();
        let artist_id = selected.id.to_string();
        let state_manager = self.state_manager.clone();
        let main_window_weak = self.main_window.as_weak();
        tokio::spawn(async move {
            fetch_and_set_songs(main_window_weak, state_manager, artist_id).await;
        });
    }

    fn on_hide(&self) { self.main_window.set_content_songs(ModelRc::default()); }
}
