use std::time::Duration;

use i_slint_backend_testing::ElementHandle;
use slint::{ComponentHandle, Model, ModelRc, VecModel};
use slint_app::{
    AccountsProps, AllSongsPageProps, AppCallbacks, AppProps, ExtensionDetailsState, MainWindow,
    OAuthState, Pages, PlayerProps, QueuePageProps, SavePlaylistState, SaveThemeState,
    SettingsPages, SettingsState, SongModel, SongSelectionState,
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
    tokio::time::sleep(std::time::Duration::from_millis(300)).await;

    let card_handles: Vec<ElementHandle> =
        ElementHandle::find_by_accessible_label(main_window, "Accounts Card").collect();
    assert_eq!(card_handles.len(), 1);
    let opacity_val: f32 = card_handles[0]
        .accessible_value()
        .expect("Accounts Card should have accessible value for opacity")
        .parse()
        .expect("opacity should be a valid float");
    assert!(
        opacity_val > 0.0,
        "Accounts card opacity must be greater than 0"
    );

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
async fn do_mobile_accounts_popup_settings_click(
    main_window: &'static MainWindow,
    _state_manager: &'static StateManager,
) {
    main_window
        .global::<AccountsProps>()
        .set_show_accounts(true);
    let opened = wait_until(|| main_window.global::<AccountsProps>().get_show_accounts()).await;
    assert!(opened);

    let settings_handles: Vec<ElementHandle> =
        ElementHandle::find_by_accessible_label(main_window, "Settings").collect();
    assert_eq!(settings_handles.len(), 1);
    assert!(settings_handles[0].is_valid());

    let start_time = std::time::Instant::now();
    click_element(&settings_handles[0]).await;

    assert!(main_window.global::<SettingsState>().get_show_settings());
    assert!(main_window.global::<AccountsProps>().get_show_accounts());

    let settings_opened = wait_until(|| {
        main_window.global::<SettingsState>().get_show_settings()
            && !main_window.global::<AccountsProps>().get_show_accounts()
            && ElementHandle::find_by_accessible_label(main_window, "Paths Settings").count() > 0
    })
    .await;
    assert!(settings_opened);
    assert!(
        start_time.elapsed() >= std::time::Duration::from_millis(200),
        "accounts popup must close after settings animation completes (took {:?})",
        start_time.elapsed()
    );
    assert!(main_window.global::<SettingsState>().get_show_settings());
    assert!(!main_window.global::<AccountsProps>().get_show_accounts());

    main_window
        .global::<SettingsState>()
        .set_show_settings(false);
    let closed = wait_until(|| !main_window.global::<SettingsState>().get_show_settings()).await;
    assert!(closed);
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

    if main_window.global::<AppProps>().get_active_page() == Pages::AllSongs {
        main_window
            .global::<AppCallbacks>()
            .invoke_active_page_changed(Pages::Search);
        let _ = wait_until(|| main_window.global::<AppProps>().get_active_page() == Pages::Search)
            .await;
    }

    main_window
        .global::<AppCallbacks>()
        .invoke_active_page_changed(Pages::AllSongs);

    let songs_loaded = wait_until(|| {
        main_window
            .global::<AllSongsPageProps>()
            .get_songs()
            .row_count()
            > 0
    })
    .await;
    assert!(songs_loaded);
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
async fn do_mobile_topbar_new_playlist(
    main_window: &'static MainWindow,
    _state_manager: &'static StateManager,
) {
    assert_eq!(
        ElementHandle::find_by_accessible_label(main_window, "New Playlist").count(),
        0
    );

    main_window
        .global::<AppCallbacks>()
        .invoke_active_page_changed(Pages::Playlists);
    let shown = wait_until(|| {
        ElementHandle::find_by_accessible_label(main_window, "New Playlist").count() > 0
    })
    .await;
    assert!(shown);

    let handles: Vec<ElementHandle> =
        ElementHandle::find_by_accessible_label(main_window, "New Playlist").collect();
    assert_eq!(handles.len(), 1);
    click_element(&handles[0]).await;

    let opened = wait_until(|| main_window.global::<SavePlaylistState>().get_show_modal()).await;
    assert!(opened);
    assert_eq!(
        main_window
            .global::<SavePlaylistState>()
            .get_songs()
            .row_count(),
        0
    );
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

    if main_window.global::<AppProps>().get_active_page() == Pages::AllSongs {
        main_window
            .global::<AppCallbacks>()
            .invoke_active_page_changed(Pages::Search);
        let _ = wait_until(|| main_window.global::<AppProps>().get_active_page() == Pages::Search)
            .await;
    }

    main_window
        .global::<AppCallbacks>()
        .invoke_active_page_changed(Pages::AllSongs);

    let songs_loaded = wait_until(|| {
        main_window
            .global::<AllSongsPageProps>()
            .get_songs()
            .row_count()
            > 0
    })
    .await;
    assert!(songs_loaded);

    // 1. Verify "Select songs" button no longer exists beside sort button
    assert_eq!(
        ElementHandle::find_by_accessible_label(main_window, "Select songs").count(),
        0
    );

    // Initial selection mode is false
    assert!(
        !main_window
            .global::<SongSelectionState>()
            .get_selection_mode()
    );

    // 2. Long press / right-click on song item toggles selection mode
    let song_loaded = wait_until(|| {
        ElementHandle::find_by_accessible_label(main_window, "Selection Track").count() > 0
    })
    .await;
    assert!(song_loaded);

    let song_handles: Vec<ElementHandle> =
        ElementHandle::find_by_accessible_label(main_window, "Selection Track").collect();
    assert_eq!(song_handles.len(), 1);

    song_handles[0]
        .single_click(slint::platform::PointerEventButton::Right)
        .await;

    let selection_active = wait_until(|| {
        main_window
            .global::<SongSelectionState>()
            .get_selection_mode()
    })
    .await;
    assert!(selection_active);
    assert_eq!(
        main_window
            .global::<SongSelectionState>()
            .get_selected_count(),
        1
    );

    // 4. Top bar replaced by selection count text and select all icon
    let count_text_loaded = wait_until(|| {
        ElementHandle::find_by_accessible_label(main_window, "Selection Count").count() > 0
    })
    .await;
    assert!(count_text_loaded);

    let select_all_handles: Vec<ElementHandle> =
        ElementHandle::find_by_accessible_label(main_window, "Select all").collect();
    assert_eq!(select_all_handles.len(), 1);

    // Single click while in selection mode deselects song
    song_handles[0]
        .single_click(slint::platform::PointerEventButton::Left)
        .await;

    // 3. Selection mode cancelled by deselecting all songs
    let selection_cancelled = wait_until(|| {
        !main_window
            .global::<SongSelectionState>()
            .get_selection_mode()
    })
    .await;
    assert!(selection_cancelled);

    // Test "Select all" in top bar:
    // Re-enable selection mode
    song_handles[0]
        .single_click(slint::platform::PointerEventButton::Right)
        .await;
    let selection_active_again = wait_until(|| {
        main_window
            .global::<SongSelectionState>()
            .get_selection_mode()
    })
    .await;
    assert!(selection_active_again);

    let select_all_btn: Vec<ElementHandle> =
        ElementHandle::find_by_accessible_label(main_window, "Select all").collect();
    assert_eq!(select_all_btn.len(), 1);

    // Clicking "Select all" when all songs are selected toggles / deselects all
    select_all_btn[0]
        .single_click(slint::platform::PointerEventButton::Left)
        .await;
    let selection_cleared_by_all = wait_until(|| {
        !main_window
            .global::<SongSelectionState>()
            .get_selection_mode()
    })
    .await;
    assert!(selection_cleared_by_all);

    // 3. Test back cancels selection mode
    song_handles[0]
        .single_click(slint::platform::PointerEventButton::Right)
        .await;
    let selection_for_back = wait_until(|| {
        main_window
            .global::<SongSelectionState>()
            .get_selection_mode()
    })
    .await;
    assert!(selection_for_back);

    let closed = main_window.invoke_close_all_modals();
    assert!(closed);
    assert!(
        !main_window
            .global::<SongSelectionState>()
            .get_selection_mode()
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
    main_window
        .global::<SongSelectionState>()
        .set_selection_mode(true);

    let all_open = wait_until(|| {
        main_window.global::<AccountsProps>().get_show_accounts()
            && main_window.global::<SettingsState>().get_show_settings()
            && main_window.global::<QueuePageProps>().get_show_queue()
            && main_window
                .global::<SongSelectionState>()
                .get_selection_mode()
    })
    .await;
    assert!(all_open);

    let closed_any = main_window.invoke_close_all_modals();
    assert!(closed_any);

    let all_closed = wait_until(|| {
        !main_window.global::<AccountsProps>().get_show_accounts()
            && !main_window.global::<SettingsState>().get_show_settings()
            && !main_window.global::<QueuePageProps>().get_show_queue()
            && !main_window
                .global::<SongSelectionState>()
                .get_selection_mode()
    })
    .await;
    assert!(all_closed);
    assert!(!main_window.global::<AccountsProps>().get_show_accounts());
    assert!(!main_window.global::<SettingsState>().get_show_settings());
    assert!(!main_window.global::<QueuePageProps>().get_show_queue());
    assert!(
        !main_window
            .global::<SongSelectionState>()
            .get_selection_mode()
    );
}

#[tracing::instrument(level = "debug", skip_all)]
async fn do_mobile_context_menu_drag_handle_click(
    main_window: &'static MainWindow,
    state_manager: &'static StateManager,
) {
    let song = create_test_song("mob_ctx", "Context Track", "Ctx Album", "Ctx Artist");
    let database = state_manager.get_database();
    database.insert_songs(vec![song]).unwrap();

    if main_window.global::<AppProps>().get_active_page() == Pages::AllSongs {
        main_window
            .global::<AppCallbacks>()
            .invoke_active_page_changed(Pages::Search);
        let _ = wait_until(|| main_window.global::<AppProps>().get_active_page() == Pages::Search)
            .await;
    }

    main_window
        .global::<AppCallbacks>()
        .invoke_active_page_changed(Pages::AllSongs);

    let songs_loaded = wait_until(|| {
        main_window
            .global::<AllSongsPageProps>()
            .get_songs()
            .row_count()
            > 0
    })
    .await;
    assert!(songs_loaded);

    let song_handles: Vec<ElementHandle> =
        ElementHandle::find_by_accessible_label(main_window, "Context Track").collect();
    assert_eq!(song_handles.len(), 1);
    song_handles[0]
        .single_click(slint::platform::PointerEventButton::Right)
        .await;

    let loaded = wait_until(|| {
        ElementHandle::find_by_accessible_label(main_window, "More options").count() > 0
    })
    .await;
    assert!(loaded);

    let more_handles: Vec<ElementHandle> =
        ElementHandle::find_by_accessible_label(main_window, "More options").collect();
    assert!(!more_handles.is_empty());

    click_element(&more_handles[0]).await;

    let menu_opened = wait_until(|| {
        ElementHandle::find_by_accessible_label(main_window, "Context Menu Drag Handle").count() > 0
    })
    .await;
    assert!(menu_opened);
    tokio::time::sleep(std::time::Duration::from_millis(400)).await;

    let handle: Vec<ElementHandle> =
        ElementHandle::find_by_accessible_label(main_window, "Context Menu Drag Handle").collect();
    assert_eq!(handle.len(), 1);

    click_element(&handle[0]).await;
    tokio::time::sleep(std::time::Duration::from_millis(300)).await;

    assert_eq!(
        ElementHandle::find_by_accessible_label(main_window, "Context Menu Drag Handle").count(),
        1
    );
    main_window
        .global::<SongSelectionState>()
        .invoke_trigger_clear_selection();
}

#[tracing::instrument(level = "debug", skip_all)]
async fn do_mobile_context_menu_drag_fills_bottom(
    main_window: &'static MainWindow,
    state_manager: &'static StateManager,
) {
    let song = create_test_song("mob_ctx2", "Context Track 2", "Ctx Album 2", "Ctx Artist 2");
    let database = state_manager.get_database();
    database.insert_songs(vec![song]).unwrap();

    if main_window.global::<AppProps>().get_active_page() == Pages::AllSongs {
        main_window
            .global::<AppCallbacks>()
            .invoke_active_page_changed(Pages::Search);
        let _ = wait_until(|| main_window.global::<AppProps>().get_active_page() == Pages::Search)
            .await;
    }

    main_window
        .global::<AppCallbacks>()
        .invoke_active_page_changed(Pages::AllSongs);

    let songs_loaded = wait_until(|| {
        main_window
            .global::<AllSongsPageProps>()
            .get_songs()
            .row_count()
            > 0
    })
    .await;
    assert!(songs_loaded);

    let song_handles: Vec<ElementHandle> =
        ElementHandle::find_by_accessible_label(main_window, "Context Track 2").collect();
    assert_eq!(song_handles.len(), 1);
    song_handles[0]
        .single_click(slint::platform::PointerEventButton::Right)
        .await;

    let more_loaded = wait_until(|| {
        ElementHandle::find_by_accessible_label(main_window, "More options").count() > 0
    })
    .await;
    assert!(more_loaded);

    let more_handles: Vec<ElementHandle> =
        ElementHandle::find_by_accessible_label(main_window, "More options").collect();
    assert!(!more_handles.is_empty());

    click_element(&more_handles[0]).await;

    let menu_opened = wait_until(|| {
        ElementHandle::find_by_accessible_label(main_window, "Context Menu Drag Handle").count() > 0
    })
    .await;
    assert!(menu_opened);
    tokio::time::sleep(std::time::Duration::from_millis(400)).await;

    let handle: Vec<ElementHandle> =
        ElementHandle::find_by_accessible_label(main_window, "Context Menu Drag Handle").collect();
    assert_eq!(handle.len(), 1);

    let initial_y = handle[0].absolute_position().y;
    assert!(initial_y > 0.0);

    let initial_val: f32 = handle[0]
        .accessible_value()
        .expect("Drag handle should have accessible value for current_y")
        .parse()
        .expect("current_y should be a valid float");
    assert!(initial_val > 0.0);

    let handle_pos = handle[0].absolute_position();
    let handle_size = handle[0].size();
    let center_x = handle_pos.x + handle_size.width / 2.0;
    let center_y = handle_pos.y + handle_size.height / 2.0;

    let window = main_window.window();
    window.dispatch_event(slint::platform::WindowEvent::PointerPressed {
        position: slint::LogicalPosition::new(center_x, center_y),
        button: slint::platform::PointerEventButton::Left,
    });
    window.dispatch_event(slint::platform::WindowEvent::PointerMoved {
        position: slint::LogicalPosition::new(center_x, center_y - 150.0),
    });
    window.dispatch_event(slint::platform::WindowEvent::PointerReleased {
        position: slint::LogicalPosition::new(center_x, center_y - 150.0),
        button: slint::platform::PointerEventButton::Left,
    });

    let expanded = wait_until(|| {
        let h: Vec<ElementHandle> =
            ElementHandle::find_by_accessible_label(main_window, "Context Menu Drag Handle")
                .collect();
        if h.is_empty() {
            return false;
        }
        h[0].absolute_position().y < initial_y - 50.0
    })
    .await;
    assert!(expanded);

    tokio::time::sleep(std::time::Duration::from_millis(300)).await;

    let expanded_handles: Vec<ElementHandle> =
        ElementHandle::find_by_accessible_label(main_window, "Context Menu Drag Handle").collect();
    assert_eq!(expanded_handles.len(), 1);

    let exp_pos = expanded_handles[0].absolute_position();
    let exp_size = expanded_handles[0].size();
    let exp_center_x = exp_pos.x + exp_size.width / 2.0;
    let exp_center_y = exp_pos.y + exp_size.height / 2.0;

    window.dispatch_event(slint::platform::WindowEvent::PointerPressed {
        position: slint::LogicalPosition::new(exp_center_x, exp_center_y),
        button: slint::platform::PointerEventButton::Left,
    });
    window.dispatch_event(slint::platform::WindowEvent::PointerMoved {
        position: slint::LogicalPosition::new(exp_center_x, 0.0),
    });

    let dragged_val: f32 = expanded_handles[0]
        .accessible_value()
        .expect("Drag handle should have accessible value for current_y")
        .parse()
        .expect("current_y should be a valid float");
    assert_eq!(
        dragged_val, 0.0,
        "Context menu should be draggable to fill entire screen (current_y = 0)"
    );

    window.dispatch_event(slint::platform::WindowEvent::PointerReleased {
        position: slint::LogicalPosition::new(exp_center_x, 0.0),
        button: slint::platform::PointerEventButton::Left,
    });

    tokio::time::sleep(std::time::Duration::from_millis(300)).await;

    main_window
        .global::<SongSelectionState>()
        .invoke_trigger_clear_selection();
}

#[tracing::instrument(level = "debug", skip_all)]
async fn do_mobile_context_menu_submenu_navigation(
    main_window: &'static MainWindow,
    state_manager: &'static StateManager,
) {
    let song = create_test_song("mob_ctx_sub", "Submenu Track", "Sub Album", "Sub Artist");
    let database = state_manager.get_database();
    database.insert_songs(vec![song]).unwrap();

    if main_window.global::<AppProps>().get_active_page() == Pages::AllSongs {
        main_window
            .global::<AppCallbacks>()
            .invoke_active_page_changed(Pages::Search);
        let _ = wait_until(|| main_window.global::<AppProps>().get_active_page() == Pages::Search)
            .await;
    }

    main_window
        .global::<AppCallbacks>()
        .invoke_active_page_changed(Pages::AllSongs);

    let songs_loaded = wait_until(|| {
        main_window
            .global::<AllSongsPageProps>()
            .get_songs()
            .row_count()
            > 0
    })
    .await;
    assert!(songs_loaded);

    let song_handles: Vec<ElementHandle> =
        ElementHandle::find_by_accessible_label(main_window, "Submenu Track").collect();
    assert_eq!(song_handles.len(), 1);
    song_handles[0]
        .single_click(slint::platform::PointerEventButton::Right)
        .await;

    let more_loaded = wait_until(|| {
        ElementHandle::find_by_accessible_label(main_window, "More options").count() > 0
    })
    .await;
    assert!(more_loaded);

    let more_handles: Vec<ElementHandle> =
        ElementHandle::find_by_accessible_label(main_window, "More options").collect();
    assert!(!more_handles.is_empty());

    click_element(&more_handles[0]).await;

    let submenu_item_ready = wait_until(|| {
        ElementHandle::find_by_accessible_label(main_window, "Go to artists").count() > 0
    })
    .await;
    assert!(submenu_item_ready);
    tokio::time::sleep(std::time::Duration::from_millis(400)).await;

    let goto_artists_handles: Vec<ElementHandle> =
        ElementHandle::find_by_accessible_label(main_window, "Go to artists").collect();
    click_element(&goto_artists_handles[0]).await;

    let back_button_ready =
        wait_until(|| ElementHandle::find_by_accessible_label(main_window, "Back").count() > 0)
            .await;
    assert!(back_button_ready);
    tokio::time::sleep(std::time::Duration::from_millis(250)).await;
    assert!(ElementHandle::find_by_accessible_label(main_window, "Sub Artist").count() > 0);

    let back_handles: Vec<ElementHandle> =
        ElementHandle::find_by_accessible_label(main_window, "Back").collect();
    click_element(&back_handles[0]).await;

    let returned_to_main = wait_until(|| {
        ElementHandle::find_by_accessible_label(main_window, "Add to Queue").count() > 0
    })
    .await;
    assert!(returned_to_main);
    main_window
        .global::<SongSelectionState>()
        .invoke_trigger_clear_selection();
}

#[tracing::instrument(level = "debug", skip_all)]
async fn do_mobile_progress_bar_tap_bounds(
    main_window: &'static MainWindow,
    state_manager: &'static StateManager,
) {
    let s1 = create_test_song("mob_seek1", "Seek Track", "Seek Album", "Seek Artist");
    let database = state_manager.get_database();
    database.insert_songs(vec![s1.clone()]).unwrap();

    main_window
        .global::<AppCallbacks>()
        .invoke_play_song(SongModel::from(s1));
    let playing = wait_until(|| main_window.global::<PlayerProps>().get_playing()).await;
    assert!(playing);

    main_window.global::<QueuePageProps>().set_show_queue(true);
    main_window.global::<QueuePageProps>().set_active_tab(0);
    main_window
        .global::<AppCallbacks>()
        .invoke_queue_toggled(true);

    tokio::time::sleep(std::time::Duration::from_millis(500)).await;

    let found_slider = wait_until(|| {
        ElementHandle::find_by_accessible_label(main_window, "Playback Progress").count() > 0
    })
    .await;
    assert!(found_slider);

    let slider_handles: Vec<ElementHandle> =
        ElementHandle::find_by_accessible_label(main_window, "Playback Progress").collect();
    assert_eq!(slider_handles.len(), 1);

    slider_handles[0]
        .single_click(slint::platform::PointerEventButton::Left)
        .await;
    tokio::time::sleep(std::time::Duration::from_millis(200)).await;

    let pos = state_manager
        .get_player_handler()
        .get_current_pos()
        .unwrap()
        .as_secs();
    assert!(pos > 0);
}

#[tracing::instrument(level = "debug", skip_all)]
async fn do_mobile_multi_song_selection_and_context_menu(
    main_window: &'static MainWindow,
    state_manager: &'static StateManager,
) {
    let s1 = create_test_song("mob_msel1", "Multi Track 1", "Sel Alb", "Sel Art");
    let s2 = create_test_song("mob_msel2", "Multi Track 2", "Sel Alb", "Sel Art");
    let database = state_manager.get_database();
    database.insert_songs(vec![s1, s2]).unwrap();

    if main_window.global::<AppProps>().get_active_page() == Pages::AllSongs {
        main_window
            .global::<AppCallbacks>()
            .invoke_active_page_changed(Pages::Search);
        let _ = wait_until(|| main_window.global::<AppProps>().get_active_page() == Pages::Search)
            .await;
    }

    main_window
        .global::<AppCallbacks>()
        .invoke_active_page_changed(Pages::AllSongs);

    let songs_loaded = wait_until(|| {
        main_window
            .global::<AllSongsPageProps>()
            .get_songs()
            .row_count()
            >= 2
    })
    .await;
    assert!(songs_loaded);

    let s1_handles: Vec<ElementHandle> =
        ElementHandle::find_by_accessible_label(main_window, "Multi Track 1").collect();
    assert_eq!(s1_handles.len(), 1);

    let s2_handles: Vec<ElementHandle> =
        ElementHandle::find_by_accessible_label(main_window, "Multi Track 2").collect();
    assert_eq!(s2_handles.len(), 1);

    s1_handles[0]
        .single_click(slint::platform::PointerEventButton::Right)
        .await;

    let selection_active = wait_until(|| {
        main_window
            .global::<SongSelectionState>()
            .get_selection_mode()
    })
    .await;
    assert!(selection_active);
    assert_eq!(
        main_window
            .global::<SongSelectionState>()
            .get_selected_count(),
        1
    );

    s2_handles[0]
        .single_click(slint::platform::PointerEventButton::Left)
        .await;

    let two_selected = wait_until(|| {
        main_window
            .global::<SongSelectionState>()
            .get_selected_count()
            == 2
    })
    .await;
    assert!(two_selected);

    let more_handles: Vec<ElementHandle> =
        ElementHandle::find_by_accessible_label(main_window, "More options").collect();
    assert_eq!(more_handles.len(), 1);

    s2_handles[0]
        .single_click(slint::platform::PointerEventButton::Left)
        .await;

    let one_selected = wait_until(|| {
        main_window
            .global::<SongSelectionState>()
            .get_selected_count()
            == 1
    })
    .await;
    assert!(one_selected);
    assert!(
        main_window
            .global::<SongSelectionState>()
            .get_selection_mode()
    );

    let select_all_handles: Vec<ElementHandle> =
        ElementHandle::find_by_accessible_label(main_window, "Select all").collect();
    assert_eq!(select_all_handles.len(), 1);

    click_element(&select_all_handles[0]).await;

    let all_selected = wait_until(|| {
        main_window
            .global::<SongSelectionState>()
            .get_selected_count()
            == 2
    })
    .await;
    assert!(all_selected);

    s1_handles[0]
        .single_click(slint::platform::PointerEventButton::Left)
        .await;

    let deselected_one = wait_until(|| {
        main_window
            .global::<SongSelectionState>()
            .get_selected_count()
            == 1
    })
    .await;
    assert!(deselected_one);
    assert!(
        main_window
            .global::<SongSelectionState>()
            .get_selection_mode()
    );

    let select_all_again: Vec<ElementHandle> =
        ElementHandle::find_by_accessible_label(main_window, "Select all").collect();
    assert_eq!(select_all_again.len(), 1);
    click_element(&select_all_again[0]).await;

    let all_selected_again = wait_until(|| {
        main_window
            .global::<SongSelectionState>()
            .get_selected_count()
            == 2
    })
    .await;
    assert!(all_selected_again);

    let deselect_all_btn: Vec<ElementHandle> =
        ElementHandle::find_by_accessible_label(main_window, "Select all").collect();
    assert_eq!(deselect_all_btn.len(), 1);
    click_element(&deselect_all_btn[0]).await;

    let all_deselected = wait_until(|| {
        !main_window
            .global::<SongSelectionState>()
            .get_selection_mode()
    })
    .await;
    assert!(all_deselected);
}

#[tracing::instrument(level = "debug", skip_all)]
async fn do_mobile_context_menu_close_animation_and_on_hide(
    main_window: &'static MainWindow,
    state_manager: &'static StateManager,
) {
    let song = create_test_song("mob_ctx_anim", "Anim Track", "Anim Album", "Anim Artist");
    let database = state_manager.get_database();
    database.insert_songs(vec![song]).unwrap();

    if main_window.global::<AppProps>().get_active_page() == Pages::AllSongs {
        main_window
            .global::<AppCallbacks>()
            .invoke_active_page_changed(Pages::Search);
        let _ = wait_until(|| main_window.global::<AppProps>().get_active_page() == Pages::Search)
            .await;
    }

    main_window
        .global::<AppCallbacks>()
        .invoke_active_page_changed(Pages::AllSongs);

    let songs_loaded = wait_until(|| {
        main_window
            .global::<AllSongsPageProps>()
            .get_songs()
            .row_count()
            > 0
    })
    .await;
    assert!(songs_loaded);

    let song_handles: Vec<ElementHandle> =
        ElementHandle::find_by_accessible_label(main_window, "Anim Track").collect();
    assert_eq!(song_handles.len(), 1);
    song_handles[0]
        .single_click(slint::platform::PointerEventButton::Right)
        .await;

    let more_loaded = wait_until(|| {
        ElementHandle::find_by_accessible_label(main_window, "More options").count() > 0
    })
    .await;
    assert!(more_loaded);

    let more_handles: Vec<ElementHandle> =
        ElementHandle::find_by_accessible_label(main_window, "More options").collect();
    assert!(!more_handles.is_empty());

    click_element(&more_handles[0]).await;

    let menu_opened = wait_until(|| {
        ElementHandle::find_by_accessible_label(main_window, "Context Menu Drag Handle").count() > 0
    })
    .await;
    assert!(menu_opened);
    tokio::time::sleep(std::time::Duration::from_millis(300)).await;

    let add_queue_handles: Vec<ElementHandle> =
        ElementHandle::find_by_accessible_label(main_window, "Add to Queue").collect();
    assert!(!add_queue_handles.is_empty());

    let start_time = std::time::Instant::now();
    click_element(&add_queue_handles[0]).await;

    assert!(
        main_window
            .global::<SongSelectionState>()
            .get_selection_mode(),
        "selection_mode must remain true while close animation is running"
    );

    let closed = wait_until(|| {
        !main_window
            .global::<SongSelectionState>()
            .get_selection_mode()
    })
    .await;
    assert!(closed);

    let elapsed = start_time.elapsed();
    assert!(
        elapsed >= std::time::Duration::from_millis(130),
        "on_hide must not be called before animation completes (took {:?})",
        elapsed
    );

    let handle_count =
        ElementHandle::find_by_accessible_label(main_window, "Context Menu Drag Handle").count();
    assert_eq!(handle_count, 0);

    main_window
        .global::<SongSelectionState>()
        .invoke_trigger_clear_selection();
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
    test_mobile_accounts_popup_settings_click => do_mobile_accounts_popup_settings_click,
    test_mobile_bottombar_play_pause => do_mobile_bottombar_play_pause,
    test_mobile_now_playing_controls => do_mobile_now_playing_controls,
    test_mobile_queue_sheet_toggle => do_mobile_queue_sheet_toggle,
    test_mobile_settings_modal_navigation_and_close => do_mobile_settings_modal_navigation_and_close,
    test_mobile_save_playlist_modal_close => do_mobile_save_playlist_modal_close,
    test_mobile_topbar_new_playlist => do_mobile_topbar_new_playlist,
    test_mobile_save_theme_modal_close => do_mobile_save_theme_modal_close,
    test_mobile_oauth_modal_close => do_mobile_oauth_modal_close,
    test_mobile_extension_details_modal_close => do_mobile_extension_details_modal_close,
    test_mobile_song_selection_toggle => do_mobile_song_selection_toggle,
    test_mobile_multi_song_selection_and_context_menu => do_mobile_multi_song_selection_and_context_menu,
    test_mobile_close_all_modals => do_mobile_close_all_modals,
    test_mobile_context_menu_drag_handle_click => do_mobile_context_menu_drag_handle_click,
    test_mobile_context_menu_drag_fills_bottom => do_mobile_context_menu_drag_fills_bottom,
    test_mobile_context_menu_submenu_navigation => do_mobile_context_menu_submenu_navigation,
    test_mobile_context_menu_close_animation_and_on_hide => do_mobile_context_menu_close_animation_and_on_hide,
    test_mobile_progress_bar_tap_bounds => do_mobile_progress_bar_tap_bounds,
);
