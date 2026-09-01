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

use slint::{ComponentHandle, Model, ModelRc};
use state_manager::StateManager;
use tempdir::TempDir;
use types::plugin::PluginContext;

use crate::{
    MainWindow, PreferenceChange,
    pages::PageHandler,
    settings::{PreferenceHandler, extensions::ExtensionsPageHandler},
    test_utils::run_async_test,
};

#[test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_extensions_page_handler_on_show() {
    run_async_test(|| async move {
        let tmp = TempDir::new("moosync_ui_ext_show").unwrap();
        let test_dir = tmp.path().to_path_buf();
        let context = PluginContext {
            data_dir: test_dir.clone(),
            cache_dir: test_dir.clone(),
            tmp_dir: test_dir.clone(),
            #[cfg(target_os = "android")]
            android_context: types::android::AndroidJNIContext::default(),
        };
        let sm: &'static StateManager =
            Box::leak(Box::new(StateManager::new_with_context(context).unwrap()));
        let main_window = Box::leak(Box::new(MainWindow::new().unwrap()));
        main_window.set_extensions(ModelRc::default());
        let handler = ExtensionsPageHandler::new(main_window, sm);

        handler.on_show();

        assert_eq!(main_window.get_extensions().row_count(), 0);
    });
}

#[test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_extensions_page_handler_on_hide() {
    run_async_test(|| async move {
        let tmp = TempDir::new("moosync_ui_ext_hide").unwrap();
        let test_dir = tmp.path().to_path_buf();
        let context = PluginContext {
            data_dir: test_dir.clone(),
            cache_dir: test_dir.clone(),
            tmp_dir: test_dir.clone(),
            #[cfg(target_os = "android")]
            android_context: types::android::AndroidJNIContext::default(),
        };
        let sm: &'static StateManager =
            Box::leak(Box::new(StateManager::new_with_context(context).unwrap()));
        let main_window = Box::leak(Box::new(MainWindow::new().unwrap()));
        let handler = ExtensionsPageHandler::new(main_window, sm);

        handler.on_hide();

        assert_eq!(main_window.get_extensions().row_count(), 0);
    });
}

#[test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_extensions_page_handler_initialize() {
    run_async_test(|| async move {
        let tmp = TempDir::new("moosync_ui_ext_init").unwrap();
        let test_dir = tmp.path().to_path_buf();
        let context = PluginContext {
            data_dir: test_dir.clone(),
            cache_dir: test_dir.clone(),
            tmp_dir: test_dir.clone(),
            #[cfg(target_os = "android")]
            android_context: types::android::AndroidJNIContext::default(),
        };
        let sm: &'static StateManager =
            Box::leak(Box::new(StateManager::new_with_context(context).unwrap()));
        let main_window = Box::leak(Box::new(MainWindow::new().unwrap()));
        main_window.set_extensions(ModelRc::default());
        let handler = ExtensionsPageHandler::new(main_window, sm);

        handler.initialize();

        assert_eq!(main_window.get_extensions().row_count(), 0);
    });
}

#[test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_extensions_page_handler_registries() {
    run_async_test(|| async move {
        let tmp = TempDir::new("moosync_ui_ext_registries").unwrap();
        let test_dir = tmp.path().to_path_buf();
        let context = PluginContext {
            data_dir: test_dir.clone(),
            cache_dir: test_dir.clone(),
            tmp_dir: test_dir.clone(),
            #[cfg(target_os = "android")]
            android_context: types::android::AndroidJNIContext::default(),
        };
        let sm: &'static StateManager =
            Box::leak(Box::new(StateManager::new_with_context(context).unwrap()));
        let main_window = Box::leak(Box::new(MainWindow::new().unwrap()));
        let handler = ExtensionsPageHandler::new(main_window, sm);
        handler.initialize();

        let change = PreferenceChange {
            id: "ExtensionRegistries".into(),
            value_string: "https://new-registry.org/manifest.json".into(),
            value_bool: false,
            value_number: 0.0,
            value_list: slint::ModelRc::default(),
        };
        let mw_weak = main_window.as_weak();
        let handled = handler.handle_preference_change(&change, &mw_weak, sm);
        assert!(handled);

        tokio::time::sleep(std::time::Duration::from_millis(50)).await;

        let pref = sm.get_preference_config().await;
        let saved: Vec<String> = pref.load(preferences::keys::ExtensionRegistries).unwrap();
        assert!(saved.contains(&"https://new-registry.org/manifest.json".to_string()));
    });
}
