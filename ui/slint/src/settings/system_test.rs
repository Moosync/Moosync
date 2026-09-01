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

use slint::ComponentHandle;
use state_manager::StateManager;
use tempdir::TempDir;
use types::plugin::PluginContext;

use crate::{
    MainWindow, PreferenceChange,
    pages::PageHandler,
    settings::{PreferenceHandler, system::SystemPageHandler},
    test_utils::run_async_test,
};

#[test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_system_page_handler_initialize() {
    run_async_test(|| async move {
        let tmp = TempDir::new("moosync_ui_system_init").unwrap();
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
        let handler = SystemPageHandler::new(main_window, sm);

        handler.initialize();
        handler.on_show();
        handler.on_hide();

        assert!(!main_window.get_playing());
    });
}

#[test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_system_page_handler_handle_change() {
    run_async_test(|| async move {
        let tmp = TempDir::new("moosync_ui_system_change").unwrap();
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
        let change = PreferenceChange {
            id: "system_key".into(),
            value_string: "".into(),
            value_bool: true,
            value_number: 0.0,
            value_list: slint::ModelRc::default(),
        };
        let mw_weak = main_window.as_weak();
        let handler = SystemPageHandler::new(main_window, sm);

        let handled = handler.handle_preference_change(&change, &mw_weak, sm);

        assert!(!handled);
    });
}
