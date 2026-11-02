use std::time::Duration;

use i_slint_backend_testing::ElementHandle;
use slint::{ComponentHandle, Model, ModelRc, VecModel};
use slint_app::{
    AccountsProps, AllSongsPageProps, AppCallbacks, AppProps, ExtensionDetailsState, MainWindow,
    OAuthState, Pages, PlayerProps, QueuePageProps, SavePlaylistState, SaveThemeState,
    SettingsPages, SettingsState, SongModel,
    test_utils::integration::{
        click_element, create_test_song, integration_test, parameterized_test, wait_until,
        wait_until_timeout,
    },
};
use state_manager::StateManager;

#[tracing::instrument(level = "debug", skip_all)]
async fn do_mobile_bottom_nav_target(
    main_window: &'static MainWindow,
    _state_manager: &'static StateManager,
    label: &'static str,
    target_page: Pages,
) {
    if main_window.global::<AppProps>().get_active_page() == target_page {
        main_window
            .global::<AppCallbacks>()
            .invoke_active_page_changed(Pages::Search);
        let _ = wait_until(|| main_window.global::<AppProps>().get_active_page() == Pages::Search)
            .await;
    }

    let handles: Vec<ElementHandle> =
        ElementHandle::find_by_accessible_label(main_window, label).collect();
    assert_eq!(handles.len(), 1);
    assert!(handles[0].is_valid());

    click_element(&handles[0]).await;

    let navigated =
        wait_until(|| main_window.global::<AppProps>().get_active_page() == target_page).await;
    assert!(navigated);
    assert_eq!(
        main_window.global::<AppProps>().get_active_page(),
        target_page
    );
}

#[tracing::instrument(level = "debug", skip_all)]
async fn do_mobile_topbar_search_toggle(
    main_window: &'static MainWindow,
    _state_manager: &'static StateManager,
) {
    let search_handles: Vec<ElementHandle> =
        ElementHandle::find_by_accessible_label(main_window, "Search").collect();
    assert_eq!(search_handles.len(), 1);
    assert!(search_handles[0].is_valid());

    click_element(&search_handles[0]).await;

    let opened = wait_until(|| {
        ElementHandle::find_by_accessible_label(main_window, "Close Search").count() > 0
    })
    .await;
    assert!(opened);
    tokio::time::sleep(std::time::Duration::from_millis(400)).await;

    let close_handles: Vec<ElementHandle> =
        ElementHandle::find_by_accessible_label(main_window, "Close Search").collect();
    assert_eq!(close_handles.len(), 1);
    assert!(close_handles[0].is_valid());

    click_element(&close_handles[0]).await;

    let closed = wait_until_timeout(Duration::from_secs(5), || {
        ElementHandle::find_by_accessible_label(main_window, "Close Search").count() == 0
    })
    .await;
    assert!(closed);
    assert_eq!(
        ElementHandle::find_by_accessible_label(main_window, "Search").count(),
        1
    );
}

#[tracing::instrument(level = "debug", skip_all)]
async fn do_mobile_topbar_accounts_toggle(
    main_window: &'static MainWindow,
    _state_manager: &'static StateManager,
) {
    let accounts_handles: Vec<ElementHandle> =
        ElementHandle::find_by_accessible_label(main_window, "Accounts").collect();
    assert_eq!(accounts_handles.len(), 1);
    assert!(accounts_handles[0].is_valid());

    click_element(&accounts_handles[0]).await;

    let opened = wait_until(|| main_window.global::<AccountsProps>().get_show_accounts()).await;
    assert!(opened);

    let close_handles: Vec<ElementHandle> =
        ElementHandle::find_by_accessible_label(main_window, "Close").collect();
    assert_eq!(close_handles.len(), 1);
    assert!(close_handles[0].is_valid());

    click_element(&close_handles[0]).await;

    let closed = wait_until(|| !main_window.global::<AccountsProps>().get_show_accounts()).await;
    assert!(closed);
    assert!(!main_window.global::<AccountsProps>().get_show_accounts());
}

#[tracing::instrument(level = "debug", skip_all)]
async fn do_mobile_bottombar_play_pause(
    main_window: &'static MainWindow,
    state_manager: &'static StateManager,
) {
    let database = state_manager.get_database();
    let song = create_test_song(
        "mob_pp",
        "Mobile Play Song",
        "Mobile Album",
        "Mobile Artist",
    );
    database.insert_songs(vec![song.clone()]).unwrap();

    main_window
        .global::<AppCallbacks>()
        .invoke_active_page_changed(Pages::AllSongs);
    let _ = wait_until(|| {
        main_window
            .global::<AllSongsPageProps>()
            .get_songs()
            .row_count()
            > 0
    })
    .await;

    main_window
        .global::<AppCallbacks>()
        .invoke_play_song(SongModel::from(song));
    let playing = wait_until(|| main_window.global::<PlayerProps>().get_playing()).await;
    assert!(playing);

    let pause_handles: Vec<ElementHandle> =
        ElementHandle::find_by_accessible_label(main_window, "Pause").collect();
    assert_eq!(pause_handles.len(), 1);
    assert!(pause_handles[0].is_valid());

    click_element(&pause_handles[0]).await;

    let paused = wait_until(|| !main_window.global::<PlayerProps>().get_playing()).await;
    assert!(paused);

    let play_handles: Vec<ElementHandle> =
        ElementHandle::find_by_accessible_label(main_window, "Play").collect();
    assert_eq!(play_handles.len(), 1);
    assert!(play_handles[0].is_valid());
}

#[tracing::instrument(level = "debug", skip_all)]
async fn do_mobile_now_playing_controls(
    main_window: &'static MainWindow,
    state_manager: &'static StateManager,
) {
    let s1 = create_test_song("mob_c1", "Track One", "Alb", "Art");
    let s2 = create_test_song("mob_c2", "Track Two", "Alb", "Art");
    let database = state_manager.get_database();
    database.insert_songs(vec![s1.clone(), s2.clone()]).unwrap();

    let ph = state_manager.get_player_handler();
    ph.add_to_queue(vec![s1.clone(), s2.clone()]).await;

    main_window.global::<QueuePageProps>().set_show_queue(true);
    main_window.global::<QueuePageProps>().set_active_tab(0);
    main_window
        .global::<AppCallbacks>()
        .invoke_queue_toggled(true);
    tokio::time::sleep(std::time::Duration::from_millis(300)).await;

    let found_shuffle =
        wait_until(|| ElementHandle::find_by_accessible_label(main_window, "Shuffle").count() > 0)
            .await;
    assert!(found_shuffle);

    let shuffle_handles: Vec<ElementHandle> =
        ElementHandle::find_by_accessible_label(main_window, "Shuffle").collect();
    assert_eq!(shuffle_handles.len(), 1);
    assert!(shuffle_handles[0].is_valid());

    let repeat_handles: Vec<ElementHandle> =
        ElementHandle::find_by_accessible_label(main_window, "Repeat").collect();
    assert_eq!(repeat_handles.len(), 1);
    assert!(repeat_handles[0].is_valid());

    let next_handles: Vec<ElementHandle> =
        ElementHandle::find_by_accessible_label(main_window, "Next").collect();
    assert_eq!(next_handles.len(), 1);
    assert!(next_handles[0].is_valid());

    let prev_handles: Vec<ElementHandle> =
        ElementHandle::find_by_accessible_label(main_window, "Previous").collect();
    assert_eq!(prev_handles.len(), 1);
    assert!(prev_handles[0].is_valid());

    click_element(&repeat_handles[0]).await;

    let repeated = wait_until(|| main_window.global::<PlayerProps>().get_repeat_mode() != 0).await;
    assert!(repeated);
}

#[tracing::instrument(level = "debug", skip_all)]
async fn do_mobile_queue_sheet_toggle(
    main_window: &'static MainWindow,
    state_manager: &'static StateManager,
) {
    let s1 = create_test_song("mob_q1", "Queue Song 1", "Alb", "Art");
    let s2 = create_test_song("mob_q2", "Queue Song 2", "Alb", "Art");
    let ph = state_manager.get_player_handler();
    ph.add_to_queue(vec![s1.clone(), s2.clone()]).await;

    main_window
        .global::<QueuePageProps>()
        .set_queue(ModelRc::new(VecModel::from(vec![
            SongModel::from(s1),
            SongModel::from(s2),
        ])));
    main_window.global::<QueuePageProps>().set_show_queue(true);
    main_window.global::<QueuePageProps>().set_active_tab(1);
    main_window
        .global::<AppCallbacks>()
        .invoke_queue_toggled(true);

    let loaded = wait_until(|| {
        ElementHandle::find_by_accessible_label(main_window, "Remove from queue").count() > 0
    })
    .await;
    assert!(loaded);

    let remove_handles: Vec<ElementHandle> =
        ElementHandle::find_by_accessible_label(main_window, "Remove from queue").collect();
    assert_eq!(remove_handles.len(), 2);
    assert!(remove_handles[0].is_valid());

    main_window.global::<QueuePageProps>().set_show_queue(false);
    main_window
        .global::<AppCallbacks>()
        .invoke_queue_toggled(false);

    let closed = wait_until(|| !main_window.global::<QueuePageProps>().get_show_queue()).await;
    assert!(closed);
    assert!(!main_window.global::<QueuePageProps>().get_show_queue());
}

#[tracing::instrument(level = "debug", skip_all)]
async fn do_mobile_settings_modal_navigation_and_close(
    main_window: &'static MainWindow,
    _state_manager: &'static StateManager,
) {
    main_window
        .global::<SettingsState>()
        .set_show_settings(true);
    main_window
        .global::<AppCallbacks>()
        .invoke_settings_toggled(true);

    let opened = wait_until(|| {
        ElementHandle::find_by_accessible_label(main_window, "Paths Settings").count() > 0
    })
    .await;
    assert!(opened);

    let paths_handles: Vec<ElementHandle> =
        ElementHandle::find_by_accessible_label(main_window, "Paths Settings").collect();
    assert_eq!(paths_handles.len(), 1);
    assert!(paths_handles[0].is_valid());

    let system_handles: Vec<ElementHandle> =
        ElementHandle::find_by_accessible_label(main_window, "System Settings").collect();
    assert_eq!(system_handles.len(), 1);
    assert!(system_handles[0].is_valid());

    let extensions_handles: Vec<ElementHandle> =
        ElementHandle::find_by_accessible_label(main_window, "Extensions Settings").collect();
    assert_eq!(extensions_handles.len(), 1);
    assert!(extensions_handles[0].is_valid());

    let themes_handles: Vec<ElementHandle> =
        ElementHandle::find_by_accessible_label(main_window, "Themes Settings").collect();
    assert_eq!(themes_handles.len(), 1);
    assert!(themes_handles[0].is_valid());

    click_element(&system_handles[0]).await;
    let system_selected = wait_until(|| {
        main_window.global::<SettingsState>().get_active_page() == SettingsPages::System
    })
    .await;
    assert!(system_selected);

    click_element(&extensions_handles[0]).await;
    let extensions_selected = wait_until(|| {
        main_window.global::<SettingsState>().get_active_page() == SettingsPages::Extensions
    })
    .await;
    assert!(extensions_selected);

    let back_handles: Vec<ElementHandle> =
        ElementHandle::find_by_accessible_label(main_window, "Back").collect();
    assert_eq!(back_handles.len(), 1);
    assert!(back_handles[0].is_valid());

    click_element(&back_handles[0]).await;

    let closed = wait_until(|| !main_window.global::<SettingsState>().get_show_settings()).await;
    assert!(closed);
    assert!(!main_window.global::<SettingsState>().get_show_settings());
}

#[tracing::instrument(level = "debug", skip_all)]
async fn do_mobile_save_playlist_modal_close(
    main_window: &'static MainWindow,
    _state_manager: &'static StateManager,
) {
    main_window
        .global::<SavePlaylistState>()
        .set_show_modal(true);

    let opened =
        wait_until(|| ElementHandle::find_by_accessible_label(main_window, "Back").count() > 0)
            .await;
    assert!(opened);

    let back_handles: Vec<ElementHandle> =
        ElementHandle::find_by_accessible_label(main_window, "Back").collect();
    assert_eq!(back_handles.len(), 1);
    assert!(back_handles[0].is_valid());

    click_element(&back_handles[0]).await;

    let closed = wait_until(|| !main_window.global::<SavePlaylistState>().get_show_modal()).await;
    assert!(closed);
    assert!(!main_window.global::<SavePlaylistState>().get_show_modal());
}

#[tracing::instrument(level = "debug", skip_all)]
async fn do_mobile_save_theme_modal_close(
    main_window: &'static MainWindow,
    _state_manager: &'static StateManager,
) {
    main_window.global::<SaveThemeState>().set_show_modal(true);

    let opened =
        wait_until(|| ElementHandle::find_by_accessible_label(main_window, "Back").count() > 0)
            .await;
    assert!(opened);

    let back_handles: Vec<ElementHandle> =
        ElementHandle::find_by_accessible_label(main_window, "Back").collect();
    assert_eq!(back_handles.len(), 1);
    assert!(back_handles[0].is_valid());

    click_element(&back_handles[0]).await;

    let closed = wait_until(|| !main_window.global::<SaveThemeState>().get_show_modal()).await;
    assert!(closed);
    assert!(!main_window.global::<SaveThemeState>().get_show_modal());
}

#[tracing::instrument(level = "debug", skip_all)]
async fn do_mobile_oauth_modal_close(
    main_window: &'static MainWindow,
    _state_manager: &'static StateManager,
) {
    main_window
        .global::<OAuthState>()
        .set_show_oauth_modal(true);

    let opened =
        wait_until(|| ElementHandle::find_by_accessible_label(main_window, "Back").count() > 0)
            .await;
    assert!(opened);

    let back_handles: Vec<ElementHandle> =
        ElementHandle::find_by_accessible_label(main_window, "Back").collect();
    assert_eq!(back_handles.len(), 1);
    assert!(back_handles[0].is_valid());

    click_element(&back_handles[0]).await;

    let closed = wait_until(|| !main_window.global::<OAuthState>().get_show_oauth_modal()).await;
    assert!(closed);
    assert!(!main_window.global::<OAuthState>().get_show_oauth_modal());
}

#[tracing::instrument(level = "debug", skip_all)]
async fn do_mobile_extension_details_modal_close(
    main_window: &'static MainWindow,
    _state_manager: &'static StateManager,
) {
    main_window
        .global::<ExtensionDetailsState>()
        .set_show_modal(true);

    let opened =
        wait_until(|| ElementHandle::find_by_accessible_label(main_window, "Back").count() > 0)
            .await;
    assert!(opened);

    let back_handles: Vec<ElementHandle> =
        ElementHandle::find_by_accessible_label(main_window, "Back").collect();
    assert_eq!(back_handles.len(), 1);
    assert!(back_handles[0].is_valid());

    click_element(&back_handles[0]).await;

    let closed = wait_until(|| {
        !main_window
            .global::<ExtensionDetailsState>()
            .get_show_modal()
    })
    .await;
    assert!(closed);
    assert!(
        !main_window
            .global::<ExtensionDetailsState>()
            .get_show_modal()
    );
}

#[tracing::instrument(level = "debug", skip_all)]
async fn do_mobile_song_selection_toggle(
    main_window: &'static MainWindow,
    state_manager: &'static StateManager,
) {
    let song = create_test_song("mob_sel", "Selection Track", "Sel Album", "Sel Artist");
    let database = state_manager.get_database();
    database.insert_songs(vec![song]).unwrap();

    main_window
        .global::<AppCallbacks>()
        .invoke_active_page_changed(Pages::AllSongs);

    let loaded = wait_until(|| {
        ElementHandle::find_by_accessible_label(main_window, "Select songs").count() > 0
    })
    .await;
    assert!(loaded);

    let select_handles: Vec<ElementHandle> =
        ElementHandle::find_by_accessible_label(main_window, "Select songs").collect();
    assert_eq!(select_handles.len(), 1);
    assert!(select_handles[0].is_valid());

    click_element(&select_handles[0]).await;

    let selection_active = wait_until(|| {
        ElementHandle::find_by_accessible_label(main_window, "Cancel selection").count() > 0
    })
    .await;
    assert!(selection_active);

    let cancel_handles: Vec<ElementHandle> =
        ElementHandle::find_by_accessible_label(main_window, "Cancel selection").collect();
    assert_eq!(cancel_handles.len(), 1);
    assert!(cancel_handles[0].is_valid());

    click_element(&cancel_handles[0]).await;

    let selection_cancelled = wait_until(|| {
        ElementHandle::find_by_accessible_label(main_window, "Select songs").count() > 0
    })
    .await;
    assert!(selection_cancelled);
    assert_eq!(
        ElementHandle::find_by_accessible_label(main_window, "Cancel selection").count(),
        0
    );
}

#[tracing::instrument(level = "debug", skip_all)]
async fn do_mobile_close_all_modals(
    main_window: &'static MainWindow,
    _state_manager: &'static StateManager,
) {
    main_window
        .global::<AccountsProps>()
        .set_show_accounts(true);
    main_window
        .global::<SettingsState>()
        .set_show_settings(true);
    main_window.global::<QueuePageProps>().set_show_queue(true);

    let all_open = wait_until(|| {
        main_window.global::<AccountsProps>().get_show_accounts()
            && main_window.global::<SettingsState>().get_show_settings()
            && main_window.global::<QueuePageProps>().get_show_queue()
    })
    .await;
    assert!(all_open);

    let closed_any = main_window.invoke_close_all_modals();
    assert!(closed_any);

    let all_closed = wait_until(|| {
        !main_window.global::<AccountsProps>().get_show_accounts()
            && !main_window.global::<SettingsState>().get_show_settings()
            && !main_window.global::<QueuePageProps>().get_show_queue()
    })
    .await;
    assert!(all_closed);
    assert!(!main_window.global::<AccountsProps>().get_show_accounts());
    assert!(!main_window.global::<SettingsState>().get_show_settings());
    assert!(!main_window.global::<QueuePageProps>().get_show_queue());
}

parameterized_test!(
    do_mobile_bottom_nav_target,
    (
        test_mobile_bottom_nav_songs,
        "Bottom Bar Songs",
        Pages::AllSongs
    ),
    (
        test_mobile_bottom_nav_playlists,
        "Bottom Bar Playlists",
        Pages::Playlists
    ),
    (
        test_mobile_bottom_nav_artists,
        "Bottom Bar Artists",
        Pages::Artists
    ),
    (
        test_mobile_bottom_nav_albums,
        "Bottom Bar Albums",
        Pages::Albums
    ),
    (
        test_mobile_bottom_nav_explore,
        "Bottom Bar Explore",
        Pages::Explore
    ),
);

integration_test!(
    test_mobile_topbar_search_toggle => do_mobile_topbar_search_toggle,
    test_mobile_topbar_accounts_toggle => do_mobile_topbar_accounts_toggle,
    test_mobile_bottombar_play_pause => do_mobile_bottombar_play_pause,
    test_mobile_now_playing_controls => do_mobile_now_playing_controls,
    test_mobile_queue_sheet_toggle => do_mobile_queue_sheet_toggle,
    test_mobile_settings_modal_navigation_and_close => do_mobile_settings_modal_navigation_and_close,
    test_mobile_save_playlist_modal_close => do_mobile_save_playlist_modal_close,
    test_mobile_save_theme_modal_close => do_mobile_save_theme_modal_close,
    test_mobile_oauth_modal_close => do_mobile_oauth_modal_close,
    test_mobile_extension_details_modal_close => do_mobile_extension_details_modal_close,
    test_mobile_song_selection_toggle => do_mobile_song_selection_toggle,
    test_mobile_close_all_modals => do_mobile_close_all_modals,
);
