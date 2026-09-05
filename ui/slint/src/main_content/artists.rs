use slint::{ComponentHandle, ModelRc};
use songs_proto::moosync::types::{Artist, ArtistList, GetEntityOptions, entity_result};
use state_manager::StateManager;

use crate::{
    ArtistModel, ArtistsPageProps, MainWindow,
    error::UiError,
    pages::PageHandler,
    utils::{EntityListCoordinator, EntityListProvider},
};

pub struct ArtistListProvider;

impl EntityListProvider for ArtistListProvider {
    type Entity = Artist;
    type EntityModel = ArtistModel;

    #[tracing::instrument(level = "debug", skip_all)]
    fn name() -> &'static str { "Artists" }

    #[tracing::instrument(level = "debug", skip_all)]
    fn to_model(entity: Artist) -> ArtistModel { ArtistModel::from(entity) }

    #[tracing::instrument(level = "debug", skip_all)]
    fn set_models(main_window: &MainWindow, model: ModelRc<ArtistModel>) {
        main_window.global::<ArtistsPageProps>().set_artists(model);
    }

    #[tracing::instrument(level = "debug", skip_all)]
    async fn fetch_entities(state_manager: &StateManager) -> Result<Vec<Artist>, UiError> {
        tracing::debug!("Fetching artists from database");
        let database = state_manager.get_database().await;
        let artists_res = database.get_entity_by_options(GetEntityOptions {
            artist: Some(Artist::default()),
            ..Default::default()
        })?;

        match artists_res.result {
            Some(entity_result::Result::Artists(ArtistList { artists })) => Ok(artists),
            _ => Err(UiError::EntityParseFailed),
        }
    }
}

pub struct ArtistsPageHandler<'a> {
    _main_window: &'a MainWindow,
    coordinator: EntityListCoordinator<ArtistListProvider>,
}

impl<'a> ArtistsPageHandler<'a> {
    #[tracing::instrument(level = "debug", skip_all)]
    pub fn new(main_window: &'a MainWindow, state_manager: &'a StateManager) -> Self {
        Self {
            _main_window: main_window,
            coordinator: EntityListCoordinator::new(main_window, state_manager),
        }
    }
}

impl<'a> PageHandler for ArtistsPageHandler<'a> {
    #[tracing::instrument(level = "debug", skip_all)]
    fn initialize(&self) {}

    #[tracing::instrument(level = "debug", skip_all)]
    fn on_show(&self) { self.coordinator.on_show(); }

    #[tracing::instrument(level = "debug", skip_all)]
    fn on_hide(&self) { self.coordinator.on_hide(); }
}
