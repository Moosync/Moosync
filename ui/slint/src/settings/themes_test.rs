// Moosync
// Copyright (C) 2024, 2025  Moosync <support@moosync.app>
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <http://www.gnu.org/licenses/>.

use rstest::rstest;
use slint::{ComponentHandle, Model};
use themes_proto::moosync::types::ThemeDetails;
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
    let main_window = Box::leak(Box::new(main_window));
    let state_manager = Box::leak(Box::new(sm));
    let handler = ThemesPageHandler::new(main_window, state_manager);

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
    let main_window = Box::leak(Box::new(main_window));
    let state_manager = Box::leak(Box::new(sm));
    let _handler = ThemesPageHandler::new(main_window, state_manager);

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

    ThemesPageHandler::apply_theme(main_window, &theme);
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
    let main_window = Box::leak(Box::new(main_window));
    let state_manager = Box::leak(Box::new(sm));
    let handler = ThemesPageHandler::new(main_window, state_manager);
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
