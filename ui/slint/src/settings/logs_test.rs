use std::time::Duration;

use preferences::keys::{LOG_COMPONENT_PLAYER, LOG_LEVEL, PreferenceItemExt};
use rstest::rstest;
use slint::{ComponentHandle, Model};
use tracing_test::traced_test;

use crate::{
    AppPreferences, MainWindow,
    pages::PageHandler,
    settings::{handle_preference_change, logs::LogsPageHandler},
    test_utils::{TestSlintSmContext, main_window, state_manager_fixture},
};

#[rstest]
#[tokio::test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_logs_page_handler_populates_preferences(
    main_window: MainWindow,
    state_manager_fixture: TestSlintSmContext,
) {
    let TestSlintSmContext { sm, .. } = state_manager_fixture;
    let handler = LogsPageHandler::new(&main_window, &sm);
    handler.initialize();
    tokio::time::sleep(Duration::from_millis(50)).await;

    let items = main_window.global::<AppPreferences>().get_logs_items();
    assert_eq!(items.row_count(), 17);
    assert_eq!(items.row_data(0).unwrap().id.as_str(), LOG_LEVEL.id);
}

#[rstest]
#[tokio::test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_logs_preferences_persist_and_apply_per_component_levels(
    main_window: MainWindow,
    state_manager_fixture: TestSlintSmContext,
) {
    let TestSlintSmContext { sm, .. } = state_manager_fixture;
    let weak = main_window.as_weak();

    handle_preference_change(
        LOG_LEVEL.id.to_string(),
        "warn".to_string(),
        false,
        0.0,
        vec![],
        sm.clone(),
        weak.clone(),
    )
    .await;

    handle_preference_change(
        LOG_COMPONENT_PLAYER.id.to_string(),
        "debug".to_string(),
        false,
        0.0,
        vec![],
        sm.clone(),
        weak,
    )
    .await;

    let config = sm.get_preference_config();
    assert_eq!(
        config.load(&LOG_LEVEL).value::<String>().as_deref(),
        Some("warn")
    );
    assert_eq!(
        config
            .load(&LOG_COMPONENT_PLAYER)
            .value::<String>()
            .as_deref(),
        Some("debug")
    );
}
