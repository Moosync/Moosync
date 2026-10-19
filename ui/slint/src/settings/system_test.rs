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

use preferences::keys::{AUTO_STARTUP, PreferenceItemExt};
use preferences_proto::moosync::types::{
    PreferenceItem as ProtoPreferenceItem, PreferenceType as ProtoPreferenceType, preference_value,
};
use rstest::rstest;
use slint::{ComponentHandle, Model};
use tracing_test::traced_test;

use crate::{
    AppCallbacks, AppPreferences, MainWindow, PreferenceChange,
    pages::PageHandler,
    settings::{
        compute_list_value, compute_updated_preference_value, find_preference_item,
        get_all_static_preferences, handle_preference_change, setup_settings,
        system::SystemPageHandler,
    },
    test_utils::{TestSlintSmContext, main_window, state_manager_fixture},
};

#[rstest]
#[tokio::test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_system_page_handler_initialize(
    main_window: MainWindow,
    state_manager_fixture: TestSlintSmContext,
) {
    let TestSlintSmContext { sm, .. } = state_manager_fixture;
    let main_window = Box::leak(Box::new(main_window));
    let state_manager = Box::leak(Box::new(sm));
    let handler = SystemPageHandler::new(main_window, state_manager);

    handler.initialize();

    assert_eq!(SystemPageHandler::get_preferences().len(), 6);
}

#[rstest]
#[tokio::test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_system_page_handler_on_show(
    main_window: MainWindow,
    state_manager_fixture: TestSlintSmContext,
) {
    let TestSlintSmContext { sm, .. } = state_manager_fixture;
    let main_window = Box::leak(Box::new(main_window));
    let state_manager = Box::leak(Box::new(sm));
    let handler = SystemPageHandler::new(main_window, state_manager);

    handler.on_show();

    assert_eq!(SystemPageHandler::get_preferences().len(), 6);
}

#[rstest]
#[tokio::test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_system_page_handler_handle_change_auto_startup(
    _main_window: MainWindow,
    state_manager_fixture: TestSlintSmContext,
) {
    let TestSlintSmContext { sm, .. } = state_manager_fixture;
    let state_manager = Box::leak(Box::new(sm.clone()));
    handle_preference_change(
        "auto_startup".to_string(),
        "".to_string(),
        true,
        0.0,
        vec![],
        state_manager,
        _main_window.as_weak(),
    )
    .await;

    let config = sm.get_preference_config();
    let auto_startup = config.load(&AUTO_STARTUP).value::<bool>().unwrap();
    assert!(auto_startup);
}

#[rstest]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_system_page_handler_render_system(main_window: MainWindow) {
    let prefs = SystemPageHandler::get_preferences();

    SystemPageHandler::render_system(&main_window, prefs);

    let items = main_window.global::<AppPreferences>().get_system_items();
    assert_eq!(items.row_count(), 6);
}

#[rstest]
#[tokio::test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_settings_static_prefs_and_find(state_manager_fixture: TestSlintSmContext) {
    let TestSlintSmContext { sm, .. } = state_manager_fixture;
    let state_manager = Box::leak(Box::new(sm));

    let all_static = get_all_static_preferences();
    assert!(!all_static.is_empty());

    let found_auto = find_preference_item("auto_startup", state_manager);
    assert!(found_auto.is_some());
    assert_eq!(found_auto.unwrap().id, "auto_startup");

    let found_none = find_preference_item("nonexistent_pref_key", state_manager);
    assert!(found_none.is_none());
}

#[test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_settings_compute_values() {
    let toggle_item = ProtoPreferenceItem {
        id: "t_item".to_string(),
        pref_type: ProtoPreferenceType::ToggleGroup as i32,
        ..Default::default()
    };
    let val_bool = compute_updated_preference_value(&toggle_item, "".into(), true, 0.0, vec![]);
    match val_bool {
        preference_value::Value::BoolValue(b) => assert!(b),
        _ => panic!("Expected BoolValue"),
    }

    let num_item = ProtoPreferenceItem {
        id: "n_item".to_string(),
        pref_type: ProtoPreferenceType::NumberInputGroup as i32,
        ..Default::default()
    };
    let val_num = compute_updated_preference_value(&num_item, "".into(), false, 7.5, vec![]);
    match val_num {
        preference_value::Value::NumberValue(n) => assert_eq!(n, 7.5),
        _ => panic!("Expected NumberValue"),
    }

    let path_item = ProtoPreferenceItem {
        id: "p_item".to_string(),
        pref_type: ProtoPreferenceType::PathSelector as i32,
        ..Default::default()
    };
    let val_list = compute_list_value(&path_item, "".into(), vec!["/dir1".into(), "/dir2".into()]);
    match val_list {
        preference_value::Value::ListValue(list) => assert_eq!(list.values.len(), 2),
        _ => panic!("Expected ListValue"),
    }

    let val_toggle_in = compute_list_value(&path_item, "/dir_toggle".into(), vec![]);
    match val_toggle_in {
        preference_value::Value::ListValue(list) => {
            assert_eq!(list.values.len(), 1);
            assert_eq!(list.values[0], "/dir_toggle");
        }
        _ => panic!("Expected ListValue"),
    }
}

#[rstest]
#[tokio::test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_settings_setup_and_change_callback(
    main_window: MainWindow,
    state_manager_fixture: TestSlintSmContext,
) {
    let TestSlintSmContext { sm, .. } = state_manager_fixture;
    let main_window = Box::leak(Box::new(main_window));
    let state_manager = Box::leak(Box::new(sm));

    setup_settings(main_window, state_manager);

    let change = PreferenceChange {
        id: "auto_startup".into(),
        value_bool: true,
        value_number: 0.0,
        value_string: "".into(),
        value_list: slint::ModelRc::default(),
    };
    main_window
        .global::<AppCallbacks>()
        .invoke_preference_changed(change);
    tokio::time::sleep(std::time::Duration::from_millis(50)).await;

    let dir = main_window
        .global::<AppCallbacks>()
        .invoke_open_directory_picker();
    let _ = dir;

    let file = main_window
        .global::<AppCallbacks>()
        .invoke_open_file_picker("mp3".into());
    let _ = file;
}
