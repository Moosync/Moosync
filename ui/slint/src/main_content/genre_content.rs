use slint::{ComponentHandle, ModelRc};
use songs_proto::moosync::types::{Genre, GetSongOptions, Song};
use state_manager::StateManager;
use tracing::debug;

use crate::{
    GenreContentPageProps, GenresPageProps, MainWindow, Theme, error::UiError, pages::PageHandler,
    utils::LazySongVecModel,
};

pub struct GenreContentPageHandler<'a> {
    main_window: &'a MainWindow,
    state_manager: &'a StateManager,
}

impl<'a> GenreContentPageHandler<'a> {
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
        genre_id: String,
    ) -> Result<Vec<Song>, UiError> {
        debug!("Fetching songs for genre ID: {}", genre_id);
        let database = state_manager.get_database().await;
        let options = GetSongOptions {
            genre: Some(Genre {
                genre_id: Some(genre_id),
                ..Default::default()
            }),
            ..Default::default()
        };
        database.get_songs_by_options(options).map_err(|e| e.into())
    }

    #[tracing::instrument(level = "debug", skip_all)]
    fn set_songs(main_window: &MainWindow, state_manager: &StateManager, songs: Vec<Song>) {
        let songs_view = songs.into_iter().map(Into::into).collect::<Vec<_>>();
        let theme = main_window.global::<Theme>();
        let cache_dir = state_manager.get_cache_dir();
        main_window
            .global::<GenreContentPageProps>()
            .set_songs(ModelRc::new(LazySongVecModel::new(
                songs_view,
                theme.get_songListItemHeight() as usize,
                theme.get_songListItemWidth() as usize,
                cache_dir,
            )));
    }
}

impl<'a> PageHandler for GenreContentPageHandler<'a> {
    #[tracing::instrument(level = "debug", skip_all)]
    fn initialize(&self) {}

    #[tracing::instrument(level = "debug", skip_all)]
    fn on_show(&self) {
        let selected = self
            .main_window
            .global::<GenresPageProps>()
            .get_selected_genre();
        let genre_id = selected.id.to_string();

        tokio::spawn({
            let state_manager = self.state_manager.clone();
            let main_window_weak = self.main_window.as_weak();
            async move {
                match Self::fetch_songs(&state_manager, genre_id).await {
                    Ok(songs) => {
                        let _ = main_window_weak.upgrade_in_event_loop(move |main_window| {
                            Self::set_songs(&main_window, &state_manager, songs);
                        });
                    }
                    Err(e) => {
                        tracing::error!("Failed to fetch genre songs: {:?}", e)
                    }
                }
            }
        });
    }

    #[tracing::instrument(level = "debug", skip_all)]
    fn on_hide(&self) {
        self.main_window
            .global::<GenreContentPageProps>()
            .set_songs(ModelRc::default());
    }
}
