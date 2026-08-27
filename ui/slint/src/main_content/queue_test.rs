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
use songs_proto::moosync::types::{GetEntityOptions, InnerSong, Playlist, Song, entity_result};
use state_manager::StateManager;
use tempdir::TempDir;
use types::plugin::PluginContext;

use crate::{
    AppCallbacks, MainWindow, main_content::queue::QueuePageHandler, pages::PageHandler,
    test_utils::run_async_test,
};

#[test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_queue_page_handler_initialize() {
    run_async_test(|| async move {
        let tmp = TempDir::new("moosync_ui_queue_init").unwrap();
        let test_dir = tmp.path().to_path_buf();
        let context = PluginContext {
            data_dir: test_dir.clone(),
            cache_dir: test_dir.clone(),
            tmp_dir: test_dir.clone(),
            #[cfg(target_os = "android")]
            android_context: types::android::AndroidJNIContext::default(),
        };
        let state_manager: &'static StateManager =
            Box::leak(Box::new(StateManager::new_with_context(context).unwrap()));
        let main_window = Box::leak(Box::new(MainWindow::new().unwrap()));
        let handler = QueuePageHandler::new(main_window, state_manager);

        handler.initialize();
        main_window
            .global::<AppCallbacks>()
            .invoke_play_queue_index(0);
        main_window
            .global::<AppCallbacks>()
            .invoke_remove_from_queue(0);
        main_window.global::<AppCallbacks>().invoke_clear_queue();

        assert!(!main_window.get_playing());
    });
}

#[test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_queue_page_handler_save_queue_as_playlist() {
    run_async_test(|| async move {
        let tmp = TempDir::new("moosync_ui_queue_save").unwrap();
        let test_dir = tmp.path().to_path_buf();
        let context = PluginContext {
            data_dir: test_dir.clone(),
            cache_dir: test_dir.clone(),
            tmp_dir: test_dir.clone(),
            #[cfg(target_os = "android")]
            android_context: types::android::AndroidJNIContext::default(),
        };
        let state_manager: &'static StateManager =
            Box::leak(Box::new(StateManager::new_with_context(context).unwrap()));
        let main_window = Box::leak(Box::new(MainWindow::new().unwrap()));
        let handler = QueuePageHandler::new(main_window, state_manager);
        handler.initialize();

        let song = Song {
            song: Some(InnerSong {
                id: Some("song_queue_1".into()),
                title: Some("Queue Song 1".into()),
                path: Some("/music/test1.mp3".into()),
                ..Default::default()
            }),
            ..Default::default()
        };
        {
            let mut ph = state_manager.get_player_handler_mut().await;
            ph.add_to_queue(vec![song]);
        }

        main_window
            .global::<AppCallbacks>()
            .invoke_save_queue_as_playlist("My Saved Queue".into(), "My Description".into());

        tokio::time::sleep(std::time::Duration::from_millis(100)).await;

        let db = state_manager.get_database().await;
        let playlists_res = db.get_entity_by_options(GetEntityOptions {
            playlist: Some(Playlist::default()),
            ..Default::default()
        });

        assert!(playlists_res.is_ok());
        let res = playlists_res.unwrap().result;
        match res {
            Some(entity_result::Result::Playlists(list)) => {
                assert_eq!(list.playlists.len(), 1);
                assert_eq!(list.playlists[0].playlist_name, "My Saved Queue");
                assert_eq!(
                    list.playlists[0].playlist_desc,
                    Some("My Description".to_string())
                );
            }
            _ => panic!("Expected playlists in entity result"),
        }
    });
}
