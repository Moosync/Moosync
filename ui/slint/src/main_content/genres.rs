use slint::{ComponentHandle, ModelRc};
use songs_proto::moosync::types::{Genre, GenreList, GetEntityOptions, entity_result};
use state_manager::StateManager;
use tracing::debug;

use crate::{MainWindow, Pages, error::UiError, pages::PageHandler, utils::LazySongVecModel};

pub struct GenresPageHandler<'a> {
    main_window: &'a MainWindow,
    state_manager: &'a StateManager,
}

impl<'a> GenresPageHandler<'a> {
    #[tracing::instrument(level = "debug", skip_all)]
    pub fn new(main_window: &'a MainWindow, state_manager: &'a StateManager) -> Self {
        Self {
            main_window,
            state_manager,
        }
    }

    #[tracing::instrument(level = "debug", skip_all)]
    async fn get_genres_from_db(state_manager: &StateManager) -> Result<Vec<Genre>, UiError> {
        let database = state_manager.get_database().await;
        let genres_res = database.get_entity_by_options(GetEntityOptions {
            genre: Some(Genre::default()),
            ..Default::default()
        })?;

        match genres_res.result {
            Some(entity_result::Result::Genres(GenreList { genres })) => Ok(genres),
            _ => Err(UiError::EntityParseFailed),
        }
    }

    #[tracing::instrument(level = "debug", skip_all)]
    async fn fetch_genres(state_manager: &StateManager) -> Result<Vec<Genre>, UiError> {
        Self::get_genres_from_db(state_manager).await
    }

    #[tracing::instrument(level = "debug", skip_all)]
    fn set_genres(main_window: &MainWindow, genres: Vec<Genre>, cache_dir: std::path::PathBuf) {
        debug!("Setting genres");
        let genre_model = genres
            .into_iter()
            .map(|genre| crate::utils::to_genre_model(&genre))
            .collect::<Vec<_>>();

        let theme = main_window.global::<crate::Theme>();
        main_window.set_genres(ModelRc::new(LazySongVecModel::new(
            genre_model,
            theme.get_cardHeight() as usize,
            theme.get_cardWidth() as usize,
            cache_dir,
        )));
    }
}

impl<'a> PageHandler for GenresPageHandler<'a> {
    #[tracing::instrument(level = "debug", skip_all)]
    fn initialize(&self) {}

    #[tracing::instrument(level = "debug", skip_all)]
    fn on_show(&self) {
        let state_manager = self.state_manager.clone();
        let main_window_weak = self.main_window.as_weak();
        tokio::spawn(async move {
            if let Ok(genres) = Self::fetch_genres(&state_manager).await {
                let cache_dir = state_manager.get_cache_dir();
                let _ = slint::invoke_from_event_loop(move || {
                    if let Some(main_window) = main_window_weak.upgrade() {
                        if main_window.get_active_page() == Pages::Genres {
                            Self::set_genres(&main_window, genres, cache_dir);
                        }
                    }
                });
            }
        });
    }

    #[tracing::instrument(level = "debug", skip_all)]
    fn on_hide(&self) { self.main_window.set_genres(ModelRc::default()); }
}
