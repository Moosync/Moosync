use rstest::rstest;
use slint::{Model, ModelRc};
use tracing_test::traced_test;

use super::{build_queue_context_menu_items, build_song_context_menu_items};
use crate::{
    MainWindow,
    test_utils::{TestSlintSmContext, main_window, state_manager_fixture},
};

#[rstest]
#[tokio::test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_build_song_context_menu_items(
    main_window: MainWindow,
    state_manager_fixture: TestSlintSmContext,
) {
    let TestSlintSmContext { sm, .. } = state_manager_fixture;
    let song_models = ModelRc::default();

    let items = build_song_context_menu_items(&main_window, &sm, &song_models);

    assert_eq!(items.row_count(), 4);
}

#[rstest]
#[tokio::test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_build_queue_context_menu_items(
    main_window: MainWindow,
    state_manager_fixture: TestSlintSmContext,
) {
    let TestSlintSmContext { sm, .. } = state_manager_fixture;
    let song_models = ModelRc::default();

    let items = build_queue_context_menu_items(&main_window, &sm, &song_models, 0);

    assert_eq!(items.row_count(), 2);
}
