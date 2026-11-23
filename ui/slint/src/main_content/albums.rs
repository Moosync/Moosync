use slint::{ComponentHandle, ModelRc};
use songs_proto::moosync::types::Album;
use state_manager::StateManager;

use crate::{
    AlbumModel, AlbumsPageProps, ContextMenuCallbacks, MainWindow,
    error::UiError,
    pages::PageHandler,
    utils::{
        EntityListCoordinator, EntityListProvider, build_album_context_menu_items,
        dispatch_album_context_action,
    },
};

pub struct AlbumListProvider;

#[async_trait::async_trait]
impl EntityListProvider for AlbumListProvider {
    type Entity = Album;
    type EntityModel = AlbumModel;

    #[tracing::instrument(level = "debug", skip_all)]
    fn name() -> &'static str { "Albums" }

    #[tracing::instrument(level = "debug", skip_all)]
    fn set_models(main_window: &MainWindow, model: ModelRc<AlbumModel>) {
        main_window.global::<AlbumsPageProps>().set_albums(model);
    }

    #[tracing::instrument(level = "debug", skip_all)]
    async fn fetch_entities(state_manager: &StateManager) -> Result<Vec<Album>, UiError> {
        tracing::debug!("Fetching albums from database");
        let database = state_manager.get_database();
        Ok(database.get_albums(Album::default(), false).await?)
    }
}

pub struct AlbumsPageHandler {
    _main_window: MainWindow,
    coordinator: EntityListCoordinator<AlbumListProvider>,
}

impl AlbumsPageHandler {
    #[tracing::instrument(level = "debug", skip_all)]
    pub fn new(main_window: &MainWindow, state_manager: &StateManager) -> Self {
        let main_window_weak = main_window.as_weak();
        let state_manager_menu = state_manager.clone();
        main_window
            .global::<ContextMenuCallbacks>()
            .on_get_album_menu_items(move |album_models| {
                let Some(main_window) = main_window_weak.upgrade() else {
                    return slint::ModelRc::default();
                };
                build_album_context_menu_items(&main_window, &state_manager_menu, &album_models)
            });

        let main_window_weak = main_window.as_weak();
        let state_manager_action = state_manager.clone();
        main_window
            .global::<ContextMenuCallbacks>()
            .on_album_action(move |album_models, action_id| {
                dispatch_album_context_action(
                    &main_window_weak,
                    &state_manager_action,
                    &album_models,
                    &action_id,
                );
            });

        Self {
            _main_window: main_window.clone_strong(),
            coordinator: EntityListCoordinator::new(main_window, state_manager),
        }
    }
}

impl PageHandler for AlbumsPageHandler {
    #[tracing::instrument(level = "debug", skip_all)]
    fn on_show(&self) { self.coordinator.on_show(); }

    #[tracing::instrument(level = "debug", skip_all)]
    fn on_hide(&self) { self.coordinator.on_hide(); }
}
