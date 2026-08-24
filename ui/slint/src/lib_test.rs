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

use slint::{ComponentHandle, ModelRc};
use state_manager::StateManager;
use tempdir::TempDir;
use types::plugin::PluginContext;

use crate::{
    AppCallbacks, BottomBarCallbacks, CoverHelper, MainWindow, PageLifecycleManager, Pages,
    SettingsPages, get_all_pages, pages::AppPage, setup_ui, test_utils::run_async_test, utils,
};

#[test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_ui_app_page_from_mappings() {
    assert_eq!(AppPage::from(Pages::AllSongs), AppPage::AllSongs);
    assert_eq!(AppPage::from(Pages::Albums), AppPage::Albums);
    assert_eq!(AppPage::from(Pages::Artists), AppPage::Artists);
    assert_eq!(AppPage::from(Pages::Playlists), AppPage::Playlists);
    assert_eq!(AppPage::from(Pages::Genres), AppPage::Genres);
    assert_eq!(AppPage::from(Pages::Explore), AppPage::Explore);
    assert_eq!(AppPage::from(Pages::Search), AppPage::Search);
    assert_eq!(
        AppPage::from(Pages::PlaylistContent),
        AppPage::PlaylistContent
    );
    assert_eq!(AppPage::from(Pages::AlbumContent), AppPage::AlbumContent);
    assert_eq!(AppPage::from(Pages::ArtistContent), AppPage::ArtistContent);
    assert_eq!(AppPage::from(Pages::GenreContent), AppPage::GenreContent);

    assert_eq!(AppPage::from(SettingsPages::Paths), AppPage::Paths);
    assert_eq!(AppPage::from(SettingsPages::System), AppPage::System);
    assert_eq!(
        AppPage::from(SettingsPages::Extensions),
        AppPage::Extensions
    );
    assert_eq!(AppPage::from(SettingsPages::Themes), AppPage::Themes);
}

#[test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_page_lifecycle_manager_queue_open_does_not_hide_active_page() {
    let all_pages = vec![
        AppPage::AllSongs,
        AppPage::Albums,
        AppPage::Queue,
        AppPage::Paths,
        AppPage::Extensions,
    ];
    let mut manager = PageLifecycleManager::new(&all_pages, AppPage::AllSongs);
    manager.compute_visibility_changes(&all_pages);

    manager.queue_open = true;
    let queue_open_actions = manager.compute_visibility_changes(&all_pages);

    assert_eq!(queue_open_actions, vec![(AppPage::Queue, true)]);
}

#[test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_page_lifecycle_manager_queue_close_does_not_show_active_page() {
    let all_pages = vec![
        AppPage::AllSongs,
        AppPage::Albums,
        AppPage::Queue,
        AppPage::Paths,
        AppPage::Extensions,
    ];
    let mut manager = PageLifecycleManager::new(&all_pages, AppPage::AllSongs);
    manager.compute_visibility_changes(&all_pages);
    manager.queue_open = true;
    manager.compute_visibility_changes(&all_pages);

    manager.queue_open = false;
    let queue_close_actions = manager.compute_visibility_changes(&all_pages);

    assert_eq!(queue_close_actions, vec![(AppPage::Queue, false)]);
}

#[test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_page_lifecycle_manager_queue_toggle_with_settings_open() {
    let all_pages = vec![AppPage::AllSongs, AppPage::Queue, AppPage::Extensions];
    let mut manager = PageLifecycleManager::new(&all_pages, AppPage::AllSongs);
    manager.settings_open = true;
    manager.compute_visibility_changes(&all_pages);

    manager.queue_open = true;
    let actions_open = manager.compute_visibility_changes(&all_pages);

    manager.queue_open = false;
    let actions_close = manager.compute_visibility_changes(&all_pages);

    assert_eq!(actions_open, vec![(AppPage::Queue, true)]);
    assert_eq!(actions_close, vec![(AppPage::Queue, false)]);
}

#[test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_ui_get_all_pages_and_setup() {
    run_async_test(|| async move {
        let tmp = TempDir::new("moosync_ui_test").unwrap();
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

        let pages = get_all_pages(main_window, sm);
        for (_page_type, page) in pages {
            page.initialize();
            page.on_show();
            page.on_hide();
        }

        setup_ui(main_window, sm);

        // Invoke AppCallbacks
        main_window
            .global::<AppCallbacks>()
            .invoke_active_page_changed(Pages::Albums);
        main_window
            .global::<AppCallbacks>()
            .invoke_active_page_changed(Pages::Artists);
        main_window
            .global::<AppCallbacks>()
            .invoke_active_page_changed(Pages::Playlists);
        main_window
            .global::<AppCallbacks>()
            .invoke_active_page_changed(Pages::Genres);
        main_window
            .global::<AppCallbacks>()
            .invoke_active_page_changed(Pages::Explore);
        main_window
            .global::<AppCallbacks>()
            .invoke_active_page_changed(Pages::Search);
        main_window
            .global::<AppCallbacks>()
            .invoke_active_page_changed(Pages::PlaylistContent);
        main_window
            .global::<AppCallbacks>()
            .invoke_active_page_changed(Pages::AlbumContent);
        main_window
            .global::<AppCallbacks>()
            .invoke_active_page_changed(Pages::ArtistContent);
        main_window
            .global::<AppCallbacks>()
            .invoke_active_page_changed(Pages::GenreContent);

        // Invoke Settings callbacks
        main_window
            .global::<AppCallbacks>()
            .invoke_settings_toggled(true);
        main_window
            .global::<AppCallbacks>()
            .invoke_settings_active_page_changed(SettingsPages::Paths);
        main_window
            .global::<AppCallbacks>()
            .invoke_settings_active_page_changed(SettingsPages::System);
        main_window
            .global::<AppCallbacks>()
            .invoke_settings_active_page_changed(SettingsPages::Extensions);
        main_window
            .global::<AppCallbacks>()
            .invoke_settings_active_page_changed(SettingsPages::Themes);
        main_window
            .global::<AppCallbacks>()
            .invoke_settings_toggled(false);

        // Invoke Queue toggle
        main_window
            .global::<AppCallbacks>()
            .invoke_queue_toggled(true);
        main_window
            .global::<AppCallbacks>()
            .invoke_queue_toggled(false);

        // Invoke song and bottom bar callbacks
        let song_model = utils::to_song_model(&songs_proto::moosync::types::Song::default(), None);
        main_window
            .global::<AppCallbacks>()
            .invoke_play_song(song_model.clone());
        main_window
            .global::<AppCallbacks>()
            .invoke_add_song_to_queue(song_model.clone());
        main_window
            .global::<AppCallbacks>()
            .invoke_song_detail_action(
                crate::SongDetailAction::Play,
                ModelRc::new(slint::VecModel::from(vec![song_model.clone()])),
            );
        main_window
            .global::<AppCallbacks>()
            .invoke_song_detail_action(
                crate::SongDetailAction::AddToQueue,
                ModelRc::new(slint::VecModel::from(vec![song_model.clone()])),
            );

        main_window
            .global::<BottomBarCallbacks>()
            .invoke_play_pause_clicked();
        main_window
            .global::<BottomBarCallbacks>()
            .invoke_toggle_repeat();
        main_window
            .global::<BottomBarCallbacks>()
            .invoke_next_song();
        main_window
            .global::<BottomBarCallbacks>()
            .invoke_prev_song();
        main_window
            .global::<BottomBarCallbacks>()
            .invoke_set_volume(75);
        main_window.global::<BottomBarCallbacks>().invoke_shuffle();
        main_window.global::<BottomBarCallbacks>().invoke_seek(30);

        // Invoke cover helper
        let _ = main_window
            .global::<CoverHelper>()
            .invoke_fetch_cover_high(song_model.clone());
        let _ = main_window
            .global::<CoverHelper>()
            .invoke_fetch_cover_low(song_model.clone());

        assert!(!main_window.get_playing());
    });
}
