use slint::{ComponentHandle, ModelRc};
use songs_proto::moosync::types::Artist;
use state_manager::StateManager;

use crate::{
    ArtistModel, ArtistsPageProps, ContextMenuCallbacks, MainWindow,
    error::UiError,
    pages::PageHandler,
    utils::{
        EntityListCoordinator, EntityListProvider, build_artist_context_menu_items,
        dispatch_artist_context_action,
    },
};

pub struct ArtistListProvider;

#[async_trait::async_trait]
impl EntityListProvider for ArtistListProvider {
    type Entity = Artist;
    type EntityModel = ArtistModel;

    #[tracing::instrument(level = "debug", skip_all)]
    fn name() -> &'static str { "Artists" }

    #[tracing::instrument(level = "debug", skip_all)]
    fn set_models(main_window: &MainWindow, model: ModelRc<ArtistModel>) {
        main_window.global::<ArtistsPageProps>().set_artists(model);
    }

    #[tracing::instrument(level = "debug", skip_all)]
    async fn fetch_entities(state_manager: &StateManager) -> Result<Vec<Artist>, UiError> {
        tracing::debug!("Fetching artists from database");
        let database = state_manager.get_database();
        Ok(database.get_artists(Artist::default(), false).await?)
    }
}

pub struct ArtistsPageHandler {
    _main_window: &'static MainWindow,
    coordinator: EntityListCoordinator<ArtistListProvider>,
}

impl ArtistsPageHandler {
    #[tracing::instrument(level = "debug", skip_all)]
    pub fn new(main_window: &'static MainWindow, state_manager: &'static StateManager) -> Self {
        main_window
            .global::<ContextMenuCallbacks>()
            .on_get_artist_menu_items(move |artist_models| {
                build_artist_context_menu_items(main_window, state_manager, &artist_models)
            });

        let main_window_weak = main_window.as_weak();
        main_window
            .global::<ContextMenuCallbacks>()
            .on_artist_action(move |artist_models, action_id| {
                dispatch_artist_context_action(
                    &main_window_weak,
                    state_manager,
                    &artist_models,
                    &action_id,
                );
            });

        Self {
            _main_window: main_window,
            coordinator: EntityListCoordinator::new(main_window, state_manager),
        }
    }
}

impl PageHandler for ArtistsPageHandler {
    #[tracing::instrument(level = "debug", skip_all)]
    fn on_show(&self) { self.coordinator.on_show(); }

    #[tracing::instrument(level = "debug", skip_all)]
    fn on_hide(&self) { self.coordinator.on_hide(); }
}
