use slint::{ComponentHandle, ModelRc};
use songs_proto::moosync::types::{Artist, GetSongOptions, Song};
use state_manager::StateManager;
use tracing::debug;

use crate::{MainWindow, Pages, error::UiError, pages::PageHandler, utils::LazySongVecModel};

pub struct ArtistContentPageHandler<'a> {
    main_window: &'a MainWindow,
    state_manager: &'a StateManager,
}

impl<'a> ArtistContentPageHandler<'a> {
    #[tracing::instrument(level = "debug", skip_all)]
    pub fn new(main_window: &'a MainWindow, state_manager: &'a StateManager) -> Self {
        Self {
            main_window,
            state_manager,
        }
    }

    #[tracing::instrument(level = "debug", skip_all)]
    async fn fetch_songs(
        state_manager: &StateManager,
        artist_id: String,
    ) -> Result<Vec<Song>, UiError> {
        debug!("Fetching songs for artist ID: {}", artist_id);
        let database = state_manager.get_database().await;
        let options = GetSongOptions {
            artist: Some(Artist {
                artist_id: Some(artist_id),
                ..Default::default()
            }),
            ..Default::default()
        };
        database.get_songs_by_options(options).map_err(|e| e.into())
    }

    #[tracing::instrument(level = "debug", skip_all)]
    fn set_songs(main_window: &MainWindow, songs: Vec<Song>, cache_dir: std::path::PathBuf) {
        debug!("Fetched {} songs for artist", songs.len());
        let songs_view = songs
            .iter()
            .map(crate::utils::to_song_model)
            .collect::<Vec<_>>();
        let theme = main_window.global::<crate::Theme>();
        main_window.set_content_songs(ModelRc::new(LazySongVecModel::new(
            songs_view,
            theme.get_songListItemHeight() as usize,
            theme.get_songListItemWidth() as usize,
            cache_dir,
        )));
    }
}

impl<'a> PageHandler for ArtistContentPageHandler<'a> {
    #[tracing::instrument(level = "debug", skip_all)]
    fn initialize(&self) {}

    #[tracing::instrument(level = "debug", skip_all)]
    fn on_show(&self) {
        let selected = self.main_window.get_selected_entity();
        let artist_id = selected.id.to_string();
        let state_manager = self.state_manager.clone();
        let main_window_weak = self.main_window.as_weak();
        tokio::spawn(async move {
            match Self::fetch_songs(&state_manager, artist_id).await {
                Ok(songs) => {
                    let cache_dir = state_manager.get_cache_dir();
                    let _ = slint::invoke_from_event_loop(move || {
                        if let Some(main_window) = main_window_weak.upgrade() {
                            if main_window.get_active_page() == Pages::ArtistContent {
                                Self::set_songs(&main_window, songs, cache_dir);
                            }
                        }
                    });
                }
                Err(e) => {
                    tracing::error!("Failed to fetch artist songs: {:?}", e)
                }
            }
        });
    }

    #[tracing::instrument(level = "debug", skip_all)]
    fn on_hide(&self) { self.main_window.set_content_songs(ModelRc::default()); }
}
