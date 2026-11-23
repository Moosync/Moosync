use preferences::keys::THEME_REGISTRIES;
use rstest::rstest;
use slint::{ComponentHandle, Model};
use themes_proto::moosync::types::{ThemeDetails, ThemeItem, ThemeManifest};
use tracing_test::traced_test;

use crate::{
    MainWindow, ThemesPageProps,
    pages::PageHandler,
    settings::themes::ThemesPageHandler,
    test_utils::{TestSlintSmContext, main_window, state_manager_fixture},
};

#[rstest]
#[tokio::test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_themes_page_handler_initialize(
    main_window: MainWindow,
    state_manager_fixture: TestSlintSmContext,
) {
    let TestSlintSmContext { sm, .. } = state_manager_fixture;
    let main_window = main_window;
    let state_manager = sm;
    let handler = ThemesPageHandler::new(&main_window, &state_manager);

    handler.initialize();

    assert_eq!(
        main_window
            .global::<ThemesPageProps>()
            .get_available_themes()
            .row_count(),
        0
    );
}

#[rstest]
#[tokio::test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_themes_page_handler_map_and_apply_theme(
    main_window: MainWindow,
    state_manager_fixture: TestSlintSmContext,
) {
    let TestSlintSmContext { sm, .. } = state_manager_fixture;
    let main_window = main_window;
    let state_manager = sm;
    let _handler = ThemesPageHandler::new(&main_window, &state_manager);

    let theme = ThemeDetails {
        id: "custom_test".to_string(),
        name: "Custom Test Theme".to_string(),
        author: Some("Tester".to_string()),
        description: Some("A test theme".to_string()),
        ..Default::default()
    };

    let config = ThemesPageHandler::map_theme_to_config(&theme);
    assert_eq!(config.id.as_str(), "custom_test");
    assert_eq!(config.name.as_str(), "Custom Test Theme");
    assert_eq!(config.author.as_str(), "Tester");

    ThemesPageHandler::apply_theme(&main_window, &theme);
    assert!(main_window.global::<crate::Theme>().get_borderRadiusLg() >= 0.0);
}

#[rstest]
#[tokio::test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_themes_page_handler_all_themes_list(state_manager_fixture: TestSlintSmContext) {
    let TestSlintSmContext { sm, .. } = state_manager_fixture;
    let theme_holder = sm.get_theme_holder();
    let themes = ThemesPageHandler::get_all_themes_list(&theme_holder.inner);
    assert!(!themes.is_empty());
    assert_eq!(themes[0].id, "default");
}

#[rstest]
#[tokio::test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_themes_page_handler_callbacks(
    main_window: MainWindow,
    state_manager_fixture: TestSlintSmContext,
) {
    let TestSlintSmContext { sm, .. } = state_manager_fixture;
    let main_window = main_window;
    let state_manager = sm;
    let handler = ThemesPageHandler::new(&main_window, &state_manager);
    handler.initialize();

    main_window
        .global::<crate::AppCallbacks>()
        .invoke_select_preset_theme("default".into());
    tokio::time::sleep(std::time::Duration::from_millis(50)).await;

    main_window
        .global::<crate::AppCallbacks>()
        .invoke_theme_constant_changed("primary".into(), "#ff0000".into());
    tokio::time::sleep(std::time::Duration::from_millis(50)).await;

    main_window
        .global::<crate::AppCallbacks>()
        .invoke_save_custom_theme("Saved Theme".into(), "Author".into(), "Description".into());
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
}

#[rstest]
#[tokio::test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_themes_page_handler_registry_and_remote(
    main_window: MainWindow,
    state_manager_fixture: TestSlintSmContext,
) {
    let TestSlintSmContext { sm, .. } = state_manager_fixture;
    let main_window = main_window;
    let state_manager = sm;
    let handler = ThemesPageHandler::new(&main_window, &state_manager);

    let prefs = ThemesPageHandler::get_preferences();
    assert_eq!(prefs.len(), 1);
    assert_eq!(prefs[0].id, THEME_REGISTRIES.id);

    handler.initialize();
    ThemesPageHandler::refresh_preferences(main_window.as_weak(), &state_manager);
    tokio::time::sleep(std::time::Duration::from_millis(50)).await;

    assert_eq!(
        main_window
            .global::<ThemesPageProps>()
            .get_static_preferences()
            .row_count(),
        1
    );

    let manifest = ThemeManifest {
        id: "remote_ocean".to_string(),
        name: "Ocean Deep".to_string(),
        author: Some("DeepSea".to_string()),
        description: Some("Deep blue ocean".to_string()),
        url: Some("http://localhost:8081/ocean.mstx".to_string()),
        registry: Some("Official".to_string()),
        theme: Some(ThemeItem {
            primary: "#001122".to_string(),
            accent: "#0088cc".to_string(),
            ..Default::default()
        }),
    };

    let config = ThemesPageHandler::map_manifest_to_config(&manifest);
    assert_eq!(config.id.as_str(), "remote_ocean");
    assert_eq!(config.name.as_str(), "Ocean Deep");
    assert_eq!(config.registry.as_str(), "Official");

    let theme_holder = state_manager.get_theme_holder();
    theme_holder.set_remote_manifests(vec![manifest]);

    let all_configs = ThemesPageHandler::get_all_theme_configs(&theme_holder.inner);
    assert!(
        all_configs
            .iter()
            .any(|c| c.id == "remote_ocean" && c.registry == "Official")
    );
}
