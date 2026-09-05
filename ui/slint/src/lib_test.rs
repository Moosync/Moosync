use rstest::rstest;
use slint::{ComponentHandle, ModelRc, VecModel};
use songs_proto::moosync::types::Song;
use state_manager::StateManager;
use tracing_test::traced_test;

use crate::{
    AppCallbacks, BottomBarCallbacks, CoverHelper, MainWindow, Pages, SettingsPages,
    SongDetailAction, SongModel, SongSortCriterion,
    pages::PageLifecycleManager,
    setup_ui,
    test_utils::{TestSlintSmContext, main_window, state_manager_fixture},
};

#[rstest]
#[tokio::test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_ui_get_all_pages_and_setup(
    main_window: MainWindow,
    state_manager_fixture: TestSlintSmContext,
) {
    let TestSlintSmContext { sm, .. } = state_manager_fixture;
    let main_window: &'static MainWindow = Box::leak(Box::new(main_window));
    let state_manager: &'static StateManager = Box::leak(Box::new(sm));

    let main_pages = PageLifecycleManager::get_main_pages(main_window, state_manager);
    for (_, page) in main_pages.iter() {
        page.initialize();
        page.on_show();
        page.on_hide();
    }
    let settings_pages = PageLifecycleManager::get_settings_pages(main_window, state_manager);
    for (_, page) in settings_pages.iter() {
        page.initialize();
        page.on_show();
        page.on_hide();
    }
    let queue_page = PageLifecycleManager::get_queue_page(main_window, state_manager);
    queue_page.initialize();
    queue_page.on_show();
    queue_page.on_hide();

    setup_ui(main_window, state_manager);

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

    assert!(main_window.get_can_go_back());
    assert!(!main_window.get_can_go_forward());

    main_window.global::<AppCallbacks>().invoke_navigate_back();

    assert_eq!(main_window.get_active_page(), Pages::ArtistContent);
    assert!(main_window.get_can_go_back());
    assert!(main_window.get_can_go_forward());

    main_window
        .global::<AppCallbacks>()
        .invoke_navigate_forward();

    assert_eq!(main_window.get_active_page(), Pages::GenreContent);
    assert!(main_window.get_can_go_back());
    assert!(!main_window.get_can_go_forward());

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

    main_window
        .global::<AppCallbacks>()
        .invoke_queue_toggled(true);
    main_window
        .global::<AppCallbacks>()
        .invoke_queue_toggled(false);

    let song_model = SongModel::from(Song::default());
    main_window
        .global::<AppCallbacks>()
        .invoke_play_song(song_model.clone());
    main_window
        .global::<AppCallbacks>()
        .invoke_add_song_to_queue(song_model.clone());
    main_window
        .global::<AppCallbacks>()
        .invoke_song_detail_action(
            SongDetailAction::Play,
            ModelRc::new(VecModel::from(vec![song_model.clone()])),
        );
    main_window
        .global::<AppCallbacks>()
        .invoke_song_detail_action(
            SongDetailAction::AddToQueue,
            ModelRc::new(VecModel::from(vec![song_model.clone()])),
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

    let _ = main_window
        .global::<CoverHelper>()
        .invoke_fetch_cover_high(song_model.clone());
    let _ = main_window
        .global::<CoverHelper>()
        .invoke_fetch_cover_low(song_model.clone());

    let _ = main_window
        .global::<AppCallbacks>()
        .invoke_filter_and_sort_songs(
            ModelRc::new(VecModel::from(vec![song_model.clone()])),
            "test".into(),
            SongSortCriterion::Title,
            true,
        );

    assert_eq!(main_pages.len(), 11);
    assert_eq!(settings_pages.len(), 4);
}
