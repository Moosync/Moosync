use slint::{ComponentHandle, ModelRc};
use songs_proto::moosync::types::{Genre, GenreList, GetEntityOptions, entity_result};
use state_manager::StateManager;

use crate::{
    ContextMenuCallbacks, GenreModel, GenresPageProps, MainWindow,
    error::UiError,
    pages::PageHandler,
    utils::{
        EntityListCoordinator, EntityListProvider, build_genre_context_menu_items,
        dispatch_genre_context_action,
    },
};

pub struct GenreListProvider;

#[async_trait::async_trait]
impl EntityListProvider for GenreListProvider {
    type Entity = Genre;
    type EntityModel = GenreModel;

    #[tracing::instrument(level = "debug", skip_all)]
    fn name() -> &'static str { "Genres" }

    #[tracing::instrument(level = "debug", skip_all)]
    fn set_models(main_window: &MainWindow, model: ModelRc<GenreModel>) {
        main_window.global::<GenresPageProps>().set_genres(model);
    }

    #[tracing::instrument(level = "debug", skip_all)]
    async fn fetch_entities(state_manager: &StateManager) -> Result<Vec<Genre>, UiError> {
        tracing::debug!("Fetching genres from database");
        let database = state_manager.get_database();
        let genres_res = database.get_entity_by_options(GetEntityOptions {
            genre: Some(Genre::default()),
            ..Default::default()
        })?;

        match genres_res.result {
            Some(entity_result::Result::Genres(GenreList { genres })) => Ok(genres),
            _ => Err(UiError::EntityParseFailed),
        }
    }
}

pub struct GenresPageHandler {
    _main_window: &'static MainWindow,
    coordinator: EntityListCoordinator<GenreListProvider>,
}

impl GenresPageHandler {
    #[tracing::instrument(level = "debug", skip_all)]
    pub fn new(main_window: &'static MainWindow, state_manager: &'static StateManager) -> Self {
        main_window
            .global::<ContextMenuCallbacks>()
            .on_get_genre_menu_items(move |genre_models| {
                build_genre_context_menu_items(main_window, state_manager, &genre_models)
            });

        let main_window_weak = main_window.as_weak();
        main_window
            .global::<ContextMenuCallbacks>()
            .on_genre_action(move |genre_models, action_id| {
                dispatch_genre_context_action(
                    &main_window_weak,
                    state_manager,
                    &genre_models,
                    &action_id,
                );
            });

        Self {
            _main_window: main_window,
            coordinator: EntityListCoordinator::new(main_window, state_manager),
        }
    }
}

impl PageHandler for GenresPageHandler {
    #[tracing::instrument(level = "debug", skip_all)]
    fn on_show(&self) { self.coordinator.on_show(); }

    #[tracing::instrument(level = "debug", skip_all)]
    fn on_hide(&self) { self.coordinator.on_hide(); }
}
