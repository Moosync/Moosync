use slint::{ComponentHandle, ModelRc, Weak};
use songs_proto::moosync::types::{GetSongOptions, Playlist};
use state_manager::StateManager;
use tracing::debug;

use crate::{MainWindow, Pages, pages::PageHandler, utils::LazySongVecModel};

pub struct PlaylistContentPageHandler<'a> {
    main_window: &'a MainWindow,
    state_manager: &'a StateManager,
}

impl<'a> PlaylistContentPageHandler<'a> {
    #[tracing::instrument(level = "debug", skip_all)]
    pub fn new(main_window: &'a MainWindow, state_manager: &'a StateManager) -> Self {
        Self {
            main_window,
            state_manager,
        }
    }
}

#[tracing::instrument(level = "debug", skip_all)]
async fn fetch_and_set_songs(
    main_window_weak: Weak<MainWindow>,
    state_manager: StateManager,
    playlist_id: String,
) {
    debug!("Fetching songs for playlist ID: {}", playlist_id);
    let database = state_manager.get_database().await;
    let options = GetSongOptions {
        playlist: Some(Playlist {
            playlist_id: Some(playlist_id),
            ..Default::default()
        }),
        ..Default::default()
    };
    match database.get_songs_by_options(options) {
        Ok(songs) => {
            debug!("Fetched {} songs for playlist", songs.len());
            let cache_dir = state_manager.get_cache_dir();
            let _ = slint::invoke_from_event_loop(move || {
                if let Some(main_window) = main_window_weak.upgrade() {
                    if main_window.get_active_page() == Pages::PlaylistContent {
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
            });
        }
        Err(e) => {
            tracing::error!("Failed to fetch playlist songs: {:?}", e)
        }
    }
}

impl<'a> PageHandler for PlaylistContentPageHandler<'a> {
    #[tracing::instrument(level = "debug", skip_all)]
    fn initialize(&self) {}

    #[tracing::instrument(level = "debug", skip_all)]
    fn on_show(&self) {
        let selected = self.main_window.get_selected_entity();
        let playlist_id = selected.id.to_string();
        let state_manager = self.state_manager.clone();
        let main_window_weak = self.main_window.as_weak();
        tokio::spawn(async move {
            fetch_and_set_songs(main_window_weak, state_manager, playlist_id).await;
        });
    }

    #[tracing::instrument(level = "debug", skip_all)]
    fn on_hide(&self) { self.main_window.set_content_songs(ModelRc::default()); }
}
