use assertables::{assert_err, assert_ok};
use rstest::rstest;
use slint::{ComponentHandle, Model, ModelRc, VecModel};
use songs_proto::moosync::types::Album;
use state_manager::StateManager;
use tracing_test::traced_test;

use crate::{
    AlbumModel, AlbumsPageProps, MainWindow,
    error::UiError,
    test_utils::{TestSlintSmContext, main_window, state_manager_fixture},
    utils::{EntityListCoordinator, EntityListProvider},
};

struct MockSuccessListProvider;

#[async_trait::async_trait]
impl EntityListProvider for MockSuccessListProvider {
    type Entity = Album;
    type EntityModel = AlbumModel;

    #[tracing::instrument(level = "debug", skip_all)]
    fn name() -> &'static str { "MockSuccessListProvider" }

    #[tracing::instrument(level = "debug", skip_all)]
    fn set_models(main_window: &MainWindow, model: ModelRc<Self::EntityModel>) {
        main_window.global::<AlbumsPageProps>().set_albums(model);
    }

    #[tracing::instrument(level = "debug", skip_all)]
    async fn fetch_entities(_state_manager: &StateManager) -> Result<Vec<Self::Entity>, UiError> {
        Ok(vec![
            Album {
                album_name: Some("Item A".to_string()),
                ..Default::default()
            },
            Album {
                album_name: Some("Item B".to_string()),
                ..Default::default()
            },
        ])
    }
}

struct MockErrorListProvider;

#[async_trait::async_trait]
impl EntityListProvider for MockErrorListProvider {
    type Entity = Album;
    type EntityModel = AlbumModel;

    #[tracing::instrument(level = "debug", skip_all)]
    fn name() -> &'static str { "MockErrorListProvider" }

    #[tracing::instrument(level = "debug", skip_all)]
    fn set_models(main_window: &MainWindow, model: ModelRc<Self::EntityModel>) {
        main_window.global::<AlbumsPageProps>().set_albums(model);
    }

    #[tracing::instrument(level = "debug", skip_all)]
    async fn fetch_entities(_state_manager: &StateManager) -> Result<Vec<Self::Entity>, UiError> {
        Err(UiError::EntityParseFailed)
    }
}

#[rstest]
#[tokio::test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_entity_list_coordinator_on_hide(
    main_window: MainWindow,
    state_manager_fixture: TestSlintSmContext,
) {
    let TestSlintSmContext { sm, .. } = state_manager_fixture;
    let dummy_albums = vec![AlbumModel::default(), AlbumModel::default()];
    main_window
        .global::<AlbumsPageProps>()
        .set_albums(ModelRc::new(VecModel::from(dummy_albums)));
    let coordinator = EntityListCoordinator::<MockSuccessListProvider>::new(&main_window, &sm);

    coordinator.on_hide();

    assert_eq!(
        main_window
            .global::<AlbumsPageProps>()
            .get_albums()
            .row_count(),
        0
    );
}

#[rstest]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_entity_list_provider_clear_models_default(main_window: MainWindow) {
    let dummy_albums = vec![AlbumModel::default(), AlbumModel::default()];
    main_window
        .global::<AlbumsPageProps>()
        .set_albums(ModelRc::new(VecModel::from(dummy_albums)));

    MockSuccessListProvider::clear_models(&main_window);

    assert_eq!(
        main_window
            .global::<AlbumsPageProps>()
            .get_albums()
            .row_count(),
        0
    );
}

#[rstest]
#[tokio::test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_entity_list_provider_fetch_entities_success(
    state_manager_fixture: TestSlintSmContext,
) {
    let TestSlintSmContext { sm, .. } = state_manager_fixture;

    let res_success = MockSuccessListProvider::fetch_entities(&sm).await;

    assert_ok!(&res_success);
    assert_eq!(res_success.unwrap().len(), 2);
}

#[rstest]
#[tokio::test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_entity_list_provider_fetch_entities_error(state_manager_fixture: TestSlintSmContext) {
    let TestSlintSmContext { sm, .. } = state_manager_fixture;

    let res_error = MockErrorListProvider::fetch_entities(&sm).await;

    assert_err!(&res_error);
}
