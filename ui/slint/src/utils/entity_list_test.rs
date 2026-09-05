use std::future::Future;

use slint::{ComponentHandle, Model, ModelRc, VecModel};
use state_manager::StateManager;
use tracing_test::traced_test;

use crate::{
    AlbumModel, AlbumsPageProps, MainWindow,
    error::UiError,
    test_utils::{TestSlintSmContext, run_slint_test, wait_until},
    utils::{EntityListCoordinator, EntityListProvider},
};

struct MockSuccessListProvider;

impl EntityListProvider for MockSuccessListProvider {
    type Entity = String;
    type EntityModel = AlbumModel;

    #[tracing::instrument(level = "debug", skip_all)]
    fn name() -> &'static str { "MockSuccessListProvider" }

    #[tracing::instrument(level = "debug", skip_all)]
    fn to_model(entity: Self::Entity) -> Self::EntityModel {
        AlbumModel {
            title: entity.into(),
            ..Default::default()
        }
    }

    #[tracing::instrument(level = "debug", skip_all)]
    fn set_models(main_window: &MainWindow, model: ModelRc<Self::EntityModel>) {
        main_window.global::<AlbumsPageProps>().set_albums(model);
    }

    #[tracing::instrument(level = "debug", skip_all)]
    fn fetch_entities(
        _state_manager: &StateManager,
    ) -> impl Future<Output = Result<Vec<Self::Entity>, UiError>> + Send {
        async { Ok(vec!["Item A".to_string(), "Item B".to_string()]) }
    }
}

struct MockErrorListProvider;

impl EntityListProvider for MockErrorListProvider {
    type Entity = String;
    type EntityModel = AlbumModel;

    #[tracing::instrument(level = "debug", skip_all)]
    fn name() -> &'static str { "MockErrorListProvider" }

    #[tracing::instrument(level = "debug", skip_all)]
    fn to_model(entity: Self::Entity) -> Self::EntityModel {
        AlbumModel {
            title: entity.into(),
            ..Default::default()
        }
    }

    #[tracing::instrument(level = "debug", skip_all)]
    fn set_models(main_window: &MainWindow, model: ModelRc<Self::EntityModel>) {
        main_window.global::<AlbumsPageProps>().set_albums(model);
    }

    #[tracing::instrument(level = "debug", skip_all)]
    fn fetch_entities(
        _state_manager: &StateManager,
    ) -> impl Future<Output = Result<Vec<Self::Entity>, UiError>> + Send {
        async { Err(UiError::EntityParseFailed) }
    }
}

#[test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_entity_list_coordinator_success_flow() {
    run_slint_test(do_test_entity_list_coordinator_success_flow);
}

#[tracing::instrument(level = "debug", skip_all)]
async fn do_test_entity_list_coordinator_success_flow(
    main_window: &'static MainWindow,
    state_manager_fixture: TestSlintSmContext,
) {
    let TestSlintSmContext { sm, .. } = state_manager_fixture;
    let coordinator = EntityListCoordinator::<MockSuccessListProvider>::new(main_window, &sm);
    let cloned_coordinator = coordinator.clone();

    cloned_coordinator.on_show();

    let loaded = wait_until(|| {
        main_window
            .global::<AlbumsPageProps>()
            .get_albums()
            .row_count()
            == 2
    })
    .await;

    assert!(loaded);
    assert_eq!(
        main_window
            .global::<AlbumsPageProps>()
            .get_albums()
            .row_count(),
        2
    );

    coordinator.on_hide();

    assert_eq!(
        main_window
            .global::<AlbumsPageProps>()
            .get_albums()
            .row_count(),
        0
    );
}

#[test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_entity_list_coordinator_error_flow() {
    run_slint_test(do_test_entity_list_coordinator_error_flow);
}

#[tracing::instrument(level = "debug", skip_all)]
async fn do_test_entity_list_coordinator_error_flow(
    main_window: &'static MainWindow,
    state_manager_fixture: TestSlintSmContext,
) {
    let TestSlintSmContext { sm, .. } = state_manager_fixture;
    main_window
        .global::<AlbumsPageProps>()
        .set_albums(ModelRc::default());

    let coordinator = EntityListCoordinator::<MockErrorListProvider>::new(main_window, &sm);
    coordinator.on_show();

    tokio::time::sleep(std::time::Duration::from_millis(50)).await;

    assert_eq!(
        main_window
            .global::<AlbumsPageProps>()
            .get_albums()
            .row_count(),
        0
    );
}

#[test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_entity_list_provider_clear_models_default() {
    run_slint_test(do_test_entity_list_provider_clear_models_default);
}

#[tracing::instrument(level = "debug", skip_all)]
async fn do_test_entity_list_provider_clear_models_default(
    main_window: &'static MainWindow,
    _state_manager_fixture: TestSlintSmContext,
) {
    let dummy_albums = vec![AlbumModel::default(), AlbumModel::default()];
    main_window
        .global::<AlbumsPageProps>()
        .set_albums(ModelRc::new(VecModel::from(dummy_albums)));
    assert_eq!(
        main_window
            .global::<AlbumsPageProps>()
            .get_albums()
            .row_count(),
        2
    );

    MockSuccessListProvider::clear_models(main_window);

    assert_eq!(
        main_window
            .global::<AlbumsPageProps>()
            .get_albums()
            .row_count(),
        0
    );
}
