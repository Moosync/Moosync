use i_slint_backend_testing::ElementHandle;
use slint::{ComponentHandle, Model, ModelRc, VecModel};
use slint_app::{
    AlbumContentPageProps, AlbumsPageProps, AllSongsPageProps, AppCallbacks, AppProps,
    ArtistContentPageProps, ArtistsPageProps, ContextMenuCallbacks, MainWindow, Pages,
    SettingsState, WindowInfo,
    test_utils::integration::{
        click_element, create_test_song, integration_test, parameterized_test, wait_until,
    },
};
use state_manager::StateManager;

#[tracing::instrument(level = "debug", skip_all)]
async fn do_sidebar_navigate_target(
    main_window: &'static MainWindow,
    _state_manager: &'static StateManager,
    label: &'static str,
    target_page: Pages,
) {
    if main_window.global::<AppProps>().get_active_page() == target_page {
        main_window
            .global::<AppCallbacks>()
            .invoke_active_page_changed(Pages::Explore);
        let _ = wait_until(|| main_window.global::<AppProps>().get_active_page() == Pages::Explore)
            .await;
    }
    let handles: Vec<ElementHandle> =
        ElementHandle::find_by_accessible_label(main_window, label).collect();
    assert_eq!(handles.len(), 1);
    assert!(handles[0].is_valid());
    click_element(&handles[0]).await;
    let nav =
        wait_until(|| main_window.global::<AppProps>().get_active_page() == target_page).await;
    assert!(nav);
    let headers: Vec<ElementHandle> =
        ElementHandle::find_by_accessible_label(main_window, label).collect();
    assert_eq!(headers.len(), 1);
    assert!(headers[0].is_valid());
}

#[tracing::instrument(level = "debug", skip_all)]
async fn do_sidebar_toggle_collapse(
    main_window: &'static MainWindow,
    _state_manager: &'static StateManager,
) {
    let initial_width = main_window.global::<WindowInfo>().get_sidebar_width();
    let toggle_handles: Vec<ElementHandle> =
        ElementHandle::find_by_accessible_label(main_window, "Toggle Sidebar").collect();
    assert_eq!(toggle_handles.len(), 1);
    assert!(toggle_handles[0].is_valid());
    click_element(&toggle_handles[0]).await;
    let toggled = wait_until(|| {
        let current_width = main_window.global::<WindowInfo>().get_sidebar_width();
        (current_width - initial_width).abs() > 1.0
    })
    .await;
    assert!(toggled);
}

#[tracing::instrument(level = "debug", skip_all)]
async fn do_topbar_navigate_back(
    main_window: &'static MainWindow,
    _state_manager: &'static StateManager,
) {
    main_window
        .global::<AppCallbacks>()
        .invoke_active_page_changed(Pages::AllSongs);
    let _ =
        wait_until(|| main_window.global::<AppProps>().get_active_page() == Pages::AllSongs).await;
    main_window
        .global::<AppCallbacks>()
        .invoke_active_page_changed(Pages::Albums);
    let _ =
        wait_until(|| main_window.global::<AppProps>().get_active_page() == Pages::Albums).await;
    assert_eq!(
        main_window.global::<AppProps>().get_active_page(),
        Pages::Albums
    );
    let back_handles: Vec<ElementHandle> =
        ElementHandle::find_by_accessible_label(main_window, "Back").collect();
    assert_eq!(back_handles.len(), 1);
    assert!(back_handles[0].is_valid());
    click_element(&back_handles[0]).await;
    let back_nav =
        wait_until(|| main_window.global::<AppProps>().get_active_page() == Pages::AllSongs).await;
    assert!(back_nav);
    let song_headers: Vec<ElementHandle> =
        ElementHandle::find_by_accessible_label(main_window, "Sidebar Songs").collect();
    assert_eq!(song_headers.len(), 1);
    assert!(song_headers[0].is_valid());
}

#[tracing::instrument(level = "debug", skip_all)]
async fn do_topbar_navigate_forward(
    main_window: &'static MainWindow,
    _state_manager: &'static StateManager,
) {
    main_window
        .global::<AppCallbacks>()
        .invoke_active_page_changed(Pages::AllSongs);
    let _ =
        wait_until(|| main_window.global::<AppProps>().get_active_page() == Pages::AllSongs).await;
    main_window
        .global::<AppCallbacks>()
        .invoke_active_page_changed(Pages::Albums);
    let _ =
        wait_until(|| main_window.global::<AppProps>().get_active_page() == Pages::Albums).await;
    main_window.global::<AppCallbacks>().invoke_navigate_back();
    let _ =
        wait_until(|| main_window.global::<AppProps>().get_active_page() == Pages::AllSongs).await;
    let forward_handles: Vec<ElementHandle> =
        ElementHandle::find_by_accessible_label(main_window, "Forward").collect();
    assert_eq!(forward_handles.len(), 1);
    assert!(forward_handles[0].is_valid());
    click_element(&forward_handles[0]).await;
    let forward_nav =
        wait_until(|| main_window.global::<AppProps>().get_active_page() == Pages::Albums).await;
    assert!(forward_nav);
    let album_headers: Vec<ElementHandle> =
        ElementHandle::find_by_accessible_label(main_window, "Sidebar Albums").collect();
    assert_eq!(album_headers.len(), 1);
    assert!(album_headers[0].is_valid());
}

#[tracing::instrument(level = "debug", skip_all)]
async fn do_topbar_open_settings_modal(
    main_window: &'static MainWindow,
    _state_manager: &'static StateManager,
) {
    let settings_handles: Vec<ElementHandle> =
        ElementHandle::find_by_accessible_label(main_window, "Settings").collect();
    assert_eq!(settings_handles.len(), 1);
    assert!(settings_handles[0].is_valid());
    click_element(&settings_handles[0]).await;
    let modal_open = wait_until(|| main_window.global::<SettingsState>().get_show_settings()).await;
    assert!(modal_open);
    let path_headers: Vec<ElementHandle> =
        ElementHandle::find_by_accessible_label(main_window, "Settings Paths").collect();
    assert_eq!(path_headers.len(), 1);
    assert!(path_headers[0].is_valid());
}

#[tracing::instrument(level = "debug", skip_all)]
async fn do_topbar_close_settings_modal(
    main_window: &'static MainWindow,
    _state_manager: &'static StateManager,
) {
    main_window
        .global::<SettingsState>()
        .set_show_settings(true);
    main_window
        .global::<AppCallbacks>()
        .invoke_settings_toggled(true);
    let modal_open = wait_until(|| main_window.global::<SettingsState>().get_show_settings()).await;
    assert!(modal_open);
    let path_headers: Vec<ElementHandle> =
        ElementHandle::find_by_accessible_label(main_window, "Settings Paths").collect();
    assert_eq!(path_headers.len(), 1);
    assert!(path_headers[0].is_valid());
    main_window
        .global::<SettingsState>()
        .set_show_settings(false);
    main_window
        .global::<AppCallbacks>()
        .invoke_settings_toggled(false);
    let modal_closed =
        wait_until(|| !main_window.global::<SettingsState>().get_show_settings()).await;
    assert!(modal_closed);
}

#[tracing::instrument(level = "debug", skip_all)]
async fn do_context_menu_goto_album(
    main_window: &'static MainWindow,
    state_manager: &'static StateManager,
) {
    let database = state_manager.get_database().await;
    let song = create_test_song("cm_al", "Song For Album", "Target Album", "Target Artist");
    database.insert_songs(vec![song]).unwrap();

    main_window
        .global::<AppCallbacks>()
        .invoke_active_page_changed(Pages::Explore);
    main_window
        .global::<AppCallbacks>()
        .invoke_active_page_changed(Pages::AllSongs);
    let _ = wait_until(|| {
        main_window
            .global::<AllSongsPageProps>()
            .get_songs()
            .row_count()
            == 1
    })
    .await;

    let song_model = main_window
        .global::<AllSongsPageProps>()
        .get_songs()
        .row_data(0)
        .unwrap();
    let target_album_id = song_model.album_id.clone();

    main_window
        .global::<ContextMenuCallbacks>()
        .invoke_song_action(
            ModelRc::new(VecModel::from(vec![song_model])),
            format!("goto_album:{}", target_album_id).into(),
        );

    let navigated = wait_until(|| {
        main_window.global::<AppProps>().get_active_page() == Pages::AlbumContent
            && main_window
                .global::<AlbumContentPageProps>()
                .get_songs()
                .row_count()
                == 1
    })
    .await;

    assert!(navigated);
    assert_eq!(
        main_window.global::<AppProps>().get_active_page(),
        Pages::AlbumContent
    );
    assert_eq!(
        main_window
            .global::<AlbumsPageProps>()
            .get_selected_album()
            .id,
        target_album_id
    );

    let handles: Vec<ElementHandle> =
        ElementHandle::find_by_accessible_label(main_window, "Song For Album").collect();
    assert_eq!(handles.len(), 1);
    assert!(handles[0].is_valid());
}

#[tracing::instrument(level = "debug", skip_all)]
async fn do_context_menu_goto_artist(
    main_window: &'static MainWindow,
    state_manager: &'static StateManager,
) {
    let database = state_manager.get_database().await;
    let song = create_test_song("cm_ar", "Song For Artist", "Target Album", "Target Artist");
    database.insert_songs(vec![song]).unwrap();

    main_window
        .global::<AppCallbacks>()
        .invoke_active_page_changed(Pages::Explore);
    main_window
        .global::<AppCallbacks>()
        .invoke_active_page_changed(Pages::AllSongs);
    let _ = wait_until(|| {
        main_window
            .global::<AllSongsPageProps>()
            .get_songs()
            .row_count()
            == 1
    })
    .await;

    let song_model = main_window
        .global::<AllSongsPageProps>()
        .get_songs()
        .row_data(0)
        .unwrap();
    let target_artist_id = song_model.artists.row_data(0).unwrap().id.clone();

    main_window
        .global::<ContextMenuCallbacks>()
        .invoke_song_action(
            ModelRc::new(VecModel::from(vec![song_model])),
            format!("goto_artist:{}", target_artist_id).into(),
        );

    let navigated = wait_until(|| {
        main_window.global::<AppProps>().get_active_page() == Pages::ArtistContent
            && main_window
                .global::<ArtistContentPageProps>()
                .get_songs()
                .row_count()
                == 1
    })
    .await;

    assert!(navigated);
    assert_eq!(
        main_window.global::<AppProps>().get_active_page(),
        Pages::ArtistContent
    );
    assert_eq!(
        main_window
            .global::<ArtistsPageProps>()
            .get_selected_artist()
            .id,
        target_artist_id
    );

    let handles: Vec<ElementHandle> =
        ElementHandle::find_by_accessible_label(main_window, "Song For Artist").collect();
    assert_eq!(handles.len(), 1);
    assert!(handles[0].is_valid());
}

parameterized_test!(
    do_sidebar_navigate_target,
    (
        test_sidebar_navigate_songs,
        "Sidebar Songs",
        Pages::AllSongs
    ),
    (
        test_sidebar_navigate_playlists,
        "Sidebar Playlists",
        Pages::Playlists
    ),
    (
        test_sidebar_navigate_artists,
        "Sidebar Artists",
        Pages::Artists
    ),
    (
        test_sidebar_navigate_albums,
        "Sidebar Albums",
        Pages::Albums
    ),
    (
        test_sidebar_navigate_genres,
        "Sidebar Genres",
        Pages::Genres
    ),
    (
        test_sidebar_navigate_explore,
        "Sidebar Explore",
        Pages::Explore
    ),
);

integration_test!(
    test_sidebar_toggle_collapse => do_sidebar_toggle_collapse,
    test_topbar_navigate_back => do_topbar_navigate_back,
    test_topbar_navigate_forward => do_topbar_navigate_forward,
    test_topbar_open_settings_modal => do_topbar_open_settings_modal,
    test_topbar_close_settings_modal => do_topbar_close_settings_modal,
    test_context_menu_goto_album => do_context_menu_goto_album,
    test_context_menu_goto_artist => do_context_menu_goto_artist,
);
