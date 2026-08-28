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
    AllSongsPageProps, MainWindow, main_content::all_songs::AllSongsPageHandler,
    pages::PageHandler, test_utils::run_async_test, utils::IntoVec,
};

#[test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_all_songs_page_handler_on_show() {
    run_async_test(|| async move {
        let tmp = TempDir::new("moosync_ui_all_songs_show").unwrap();
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
        main_window
            .global::<AllSongsPageProps>()
            .set_songs(ModelRc::default());
        let handler = AllSongsPageHandler::new(main_window, sm);

        handler.on_show();

        assert_eq!(
            main_window
                .global::<AllSongsPageProps>()
                .get_songs()
                .row_count(),
            0
        );
    });
}

#[test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_all_songs_page_handler_on_hide() {
    run_async_test(|| async move {
        let tmp = TempDir::new("moosync_ui_all_songs_hide").unwrap();
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
        let handler = AllSongsPageHandler::new(main_window, sm);

        handler.on_hide();

        assert_eq!(
            main_window
                .global::<AllSongsPageProps>()
                .get_songs()
                .row_count(),
            0
        );
    });
}

#[test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_all_songs_context_menu_items() {
    run_async_test(|| async move {
        let tmp = TempDir::new("moosync_ui_all_songs_cm").unwrap();
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
        let handler = AllSongsPageHandler::new(main_window, sm);
        handler.initialize();

        let song_with_path = crate::SongModel {
            path: "/path/to/song.mp3".into(),
            ..Default::default()
        };
        let models = ModelRc::new(slint::VecModel::from(vec![song_with_path]));

        let items = main_window
            .global::<crate::ContextMenuCallbacks>()
            .invoke_get_all_songs_menu_items(models);

        let items_vec: Vec<crate::ContextMenuItem> = items.into_vec();
        assert!(items_vec.iter().any(|i| i.action_id == "play_now"));
        assert!(
            items_vec
                .iter()
                .any(|i| i.action_id == "open_in_file_manager")
        );
    });
}
