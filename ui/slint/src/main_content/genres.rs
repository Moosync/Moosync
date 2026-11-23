use slint::{ComponentHandle, ModelRc};
use songs_proto::moosync::types::Genre;
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
        Ok(database.get_genres(Genre::default(), false).await?)
    }
}

pub struct GenresPageHandler {
    _main_window: MainWindow,
    coordinator: EntityListCoordinator<GenreListProvider>,
}

impl GenresPageHandler {
    #[tracing::instrument(level = "debug", skip_all)]
    pub fn new(main_window: &MainWindow, state_manager: &StateManager) -> Self {
        let main_window_weak = main_window.as_weak();
        let state_manager_menu = state_manager.clone();
        main_window
            .global::<ContextMenuCallbacks>()
            .on_get_genre_menu_items(move |genre_models| {
                let Some(main_window) = main_window_weak.upgrade() else {
                    return slint::ModelRc::default();
                };
                build_genre_context_menu_items(&main_window, &state_manager_menu, &genre_models)
            });

        let main_window_weak = main_window.as_weak();
        let state_manager_action = state_manager.clone();
        main_window
            .global::<ContextMenuCallbacks>()
            .on_genre_action(move |genre_models, action_id| {
                dispatch_genre_context_action(
                    &main_window_weak,
                    &state_manager_action,
                    &genre_models,
                    &action_id,
                );
            });

        Self {
            _main_window: main_window.clone_strong(),
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
