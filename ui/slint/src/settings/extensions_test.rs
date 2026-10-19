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
use slint::{ComponentHandle, Model, ModelRc};
use tracing_test::traced_test;

use crate::{
    AppCallbacks, ExtensionDetailsState, ExtensionItem, ExtensionsPageProps, MainWindow,
    pages::PageHandler,
    settings::{extensions::ExtensionsPageHandler, handle_preference_change},
    test_utils::{TestSlintSmContext, main_window, state_manager_fixture},
};

#[rstest]
#[tokio::test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_extensions_page_handler_on_show(
    main_window: MainWindow,
    state_manager_fixture: TestSlintSmContext,
) {
    let TestSlintSmContext { sm, .. } = state_manager_fixture;
    let main_window = Box::leak(Box::new(main_window));
    let state_manager = Box::leak(Box::new(sm));
    let props = main_window.global::<ExtensionsPageProps>();
    props.set_extensions(ModelRc::default());
    let handler = ExtensionsPageHandler::new(main_window, state_manager);

    handler.on_show();

    assert_eq!(props.get_extensions().row_count(), 0);
}

#[rstest]
#[tokio::test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_extensions_page_handler_on_hide(
    main_window: MainWindow,
    state_manager_fixture: TestSlintSmContext,
) {
    let TestSlintSmContext { sm, .. } = state_manager_fixture;
    let main_window = Box::leak(Box::new(main_window));
    let state_manager = Box::leak(Box::new(sm));
    let props = main_window.global::<ExtensionsPageProps>();
    let dummy_exts = vec![ExtensionItem::default(), ExtensionItem::default()];
    props.set_extensions(ModelRc::new(slint::VecModel::from(dummy_exts)));
    let handler = ExtensionsPageHandler::new(main_window, state_manager);

    handler.on_hide();

    assert_eq!(props.get_extensions().row_count(), 0);
}

#[rstest]
#[tokio::test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_extensions_page_handler_initialize(
    main_window: MainWindow,
    state_manager_fixture: TestSlintSmContext,
) {
    let TestSlintSmContext { sm, .. } = state_manager_fixture;
    let main_window = Box::leak(Box::new(main_window));
    let state_manager = Box::leak(Box::new(sm));
    let props = main_window.global::<ExtensionsPageProps>();
    props.set_extensions(ModelRc::default());
    let handler = ExtensionsPageHandler::new(main_window, state_manager);

    handler.initialize();

    assert_eq!(props.get_extensions().row_count(), 0);
}

#[rstest]
#[tokio::test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_extensions_page_handler_registries(
    main_window: MainWindow,
    state_manager_fixture: TestSlintSmContext,
) {
    use preferences::keys::{EXTENSION_REGISTRIES, PreferenceItemExt};

    let TestSlintSmContext { sm, .. } = state_manager_fixture;
    let main_window = Box::leak(Box::new(main_window));
    let state_manager = Box::leak(Box::new(sm));
    let handler = ExtensionsPageHandler::new(main_window, state_manager);
    handler.initialize();
    handle_preference_change(
        "extension_registries".to_string(),
        "".to_string(),
        false,
        0.0,
        vec!["https://new-registry.org/manifest.json".to_string()],
        state_manager,
        main_window.as_weak(),
    )
    .await;

    let pref = state_manager.get_preference_config();
    let saved: Vec<String> = pref
        .load(&EXTENSION_REGISTRIES)
        .value::<Vec<String>>()
        .unwrap();
    assert!(saved.contains(&"https://new-registry.org/manifest.json".to_string()));
}

#[rstest]
#[tokio::test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_extensions_page_handler_props(
    main_window: MainWindow,
    state_manager_fixture: TestSlintSmContext,
) {
    let TestSlintSmContext { sm, .. } = state_manager_fixture;
    let main_window = Box::leak(Box::new(main_window));
    let state_manager = Box::leak(Box::new(sm));
    let handler = ExtensionsPageHandler::new(main_window, state_manager);
    handler.initialize();
    let props = main_window.global::<ExtensionsPageProps>();

    props.set_has_updates(true);

    assert!(props.get_has_updates());
}

#[rstest]
#[tokio::test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_extensions_page_handler_update_all_callback(
    main_window: MainWindow,
    state_manager_fixture: TestSlintSmContext,
) {
    let TestSlintSmContext { sm, .. } = state_manager_fixture;
    let main_window = Box::leak(Box::new(main_window));
    let state_manager = Box::leak(Box::new(sm));
    let props = main_window.global::<ExtensionsPageProps>();
    props.set_has_updates(true);
    let handler = ExtensionsPageHandler::new(main_window, state_manager);
    handler.initialize();

    main_window
        .global::<AppCallbacks>()
        .invoke_update_all_extensions();

    assert!(!props.get_has_updates());
}

#[rstest]
#[tokio::test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_extension_details_modal_state(main_window: MainWindow) {
    let state = main_window.global::<ExtensionDetailsState>();
    let test_item = ExtensionItem {
        name: "Test Extension".into(),
        package_name: "test.ext".into(),
        version: "1.0.0".into(),
        description: "Modal test description".into(),
        author: "Tester".into(),
        ..Default::default()
    };

    state.set_item(test_item);
    state.set_show_modal(true);

    assert!(state.get_show_modal());
    assert_eq!(state.get_item().name, "Test Extension");
    assert_eq!(state.get_item().package_name, "test.ext");
    assert_eq!(state.get_item().version, "1.0.0");
    assert_eq!(state.get_item().description, "Modal test description");
    assert_eq!(state.get_item().author, "Tester");
}

#[rstest]
#[tokio::test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_extension_item_with_error(main_window: MainWindow) {
    let state = main_window.global::<ExtensionDetailsState>();
    let test_item = ExtensionItem {
        name: "Broken Extension".into(),
        package_name: "broken.ext".into(),
        version: "1.0.0".into(),
        error: "failed to load wasm".into(),
        is_installed: true,
        loading: false,
        ..Default::default()
    };

    state.set_item(test_item);
    state.set_show_modal(true);

    assert_eq!(state.get_item().error, "failed to load wasm");
    assert!(!state.get_item().loading);
}

#[rstest]
#[tokio::test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_extensions_page_handler_render_and_preferences(
    main_window: MainWindow,
    state_manager_fixture: TestSlintSmContext,
) {
    let TestSlintSmContext { sm, .. } = state_manager_fixture;
    let prefs = ExtensionsPageHandler::get_preferences();
    assert!(!prefs.is_empty());

    let detail = extensions_proto::moosync::types::ExtensionDetail {
        name: "Mock Ext".to_string(),
        package_name: "mock.ext".to_string(),
        version: "1.0.0".to_string(),
        author: Some("Dev".to_string()),
        desc: Some("Test".to_string()),
        ..Default::default()
    };
    let extensions = vec![extensions::ExtensionInfo::Local(detail)];
    let cache_dir = sm.get_cache_dir();

    ExtensionsPageHandler::render_extensions(&main_window, extensions, cache_dir);
    tokio::time::sleep(std::time::Duration::from_millis(50)).await;

    let props = main_window.global::<ExtensionsPageProps>();
    assert_eq!(props.get_extensions().row_count(), 1);
}

#[rstest]
#[tokio::test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_extensions_page_handler_action_callbacks(
    main_window: MainWindow,
    state_manager_fixture: TestSlintSmContext,
) {
    let TestSlintSmContext { sm, .. } = state_manager_fixture;
    let main_window = Box::leak(Box::new(main_window));
    let state_manager = Box::leak(Box::new(sm));
    let handler = ExtensionsPageHandler::new(main_window, state_manager);
    handler.initialize();

    let callbacks = main_window.global::<AppCallbacks>();
    callbacks.invoke_toggle_extension("test.pkg".into());
    callbacks.invoke_install_extension("test.pkg".into());
    callbacks.invoke_update_all_extensions();

    tokio::time::sleep(std::time::Duration::from_millis(50)).await;
}
