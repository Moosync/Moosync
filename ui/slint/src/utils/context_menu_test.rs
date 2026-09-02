use assertables::{assert_gt, assert_len_eq_x};
use rstest::rstest;
use slint::{Model, ModelRc, VecModel};
use tracing_test::traced_test;

use super::{
    build_queue_context_menu_items, build_song_context_menu_items, default_empty_icon,
    make_context_menu_item, models::IntoVec,
};
use crate::{
    ContextMenuItem, MainWindow,
    test_utils::{TestSlintSmContext, main_window, state_manager_fixture},
};

#[test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_context_menu_item_vec_from_model_rc() {
    let items = vec![
        make_context_menu_item("play_now", "Play Now", default_empty_icon()),
        make_context_menu_item("add_to_queue", "Add to Queue", default_empty_icon()),
    ];

    let model_rc = ModelRc::new(VecModel::from(items));
    let converted: Vec<ContextMenuItem> = model_rc.into_vec();

    assert_len_eq_x!(&converted, 2);
    assert_eq!(converted[0].action_id, "play_now");
    assert_eq!(converted[0].title, "Play Now");
    assert_eq!(converted[1].action_id, "add_to_queue");
    assert_eq!(converted[1].title, "Add to Queue");
}

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

    assert_gt!(items.row_count(), 0);
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

    assert_gt!(items.row_count(), 0);
}
