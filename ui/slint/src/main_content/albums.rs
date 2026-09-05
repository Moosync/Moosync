use slint::{ComponentHandle, ModelRc};
use songs_proto::moosync::types::{Album, AlbumList, GetEntityOptions, entity_result};
use state_manager::StateManager;

use crate::{
    AlbumModel, AlbumsPageProps, MainWindow,
    error::UiError,
    pages::PageHandler,
    utils::{EntityListCoordinator, EntityListProvider},
};

pub struct AlbumListProvider;

impl EntityListProvider for AlbumListProvider {
    type Entity = Album;
    type EntityModel = AlbumModel;

    #[tracing::instrument(level = "debug", skip_all)]
    fn name() -> &'static str { "Albums" }

    #[tracing::instrument(level = "debug", skip_all)]
    fn to_model(entity: Album) -> AlbumModel { AlbumModel::from(entity) }

    #[tracing::instrument(level = "debug", skip_all)]
    fn set_models(main_window: &MainWindow, model: ModelRc<AlbumModel>) {
        main_window.global::<AlbumsPageProps>().set_albums(model);
    }

    #[tracing::instrument(level = "debug", skip_all)]
    async fn fetch_entities(state_manager: &StateManager) -> Result<Vec<Album>, UiError> {
        tracing::debug!("Fetching albums from database");
        let database = state_manager.get_database().await;
        let albums_res = database.get_entity_by_options(GetEntityOptions {
            album: Some(Album::default()),
            ..Default::default()
        })?;

        match albums_res.result {
            Some(entity_result::Result::Albums(AlbumList { albums })) => Ok(albums),
            _ => Err(UiError::EntityParseFailed),
        }
    }
}

pub struct AlbumsPageHandler<'a> {
    _main_window: &'a MainWindow,
    coordinator: EntityListCoordinator<AlbumListProvider>,
}

impl<'a> AlbumsPageHandler<'a> {
    #[tracing::instrument(level = "debug", skip_all)]
    pub fn new(main_window: &'a MainWindow, state_manager: &'a StateManager) -> Self {
        Self {
            _main_window: main_window,
            coordinator: EntityListCoordinator::new(main_window, state_manager),
        }
    }
}

impl<'a> PageHandler for AlbumsPageHandler<'a> {
    #[tracing::instrument(level = "debug", skip_all)]
    fn initialize(&self) {}

    #[tracing::instrument(level = "debug", skip_all)]
    fn on_show(&self) { self.coordinator.on_show(); }

    #[tracing::instrument(level = "debug", skip_all)]
    fn on_hide(&self) { self.coordinator.on_hide(); }
}
