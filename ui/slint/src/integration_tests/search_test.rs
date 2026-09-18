use i_slint_backend_testing::ElementHandle;
use slint::{ComponentHandle, Model};
use slint_app::{
    AppCallbacks, AppProps, MainWindow, Pages, SearchPageProps,
    test_utils::integration::{
        click_element, create_test_song, integration_test, load_sample_extension, set_test_step,
        wait_until,
    },
};
use songs_proto::moosync::types::Playlist;
use state_manager::StateManager;

#[tracing::instrument(level = "debug", skip_all)]
async fn do_search_category_songs(
    main_window: &'static MainWindow,
    state_manager: &'static StateManager,
) {
    let database = state_manager.get_database().await;
    let song = create_test_song("s4", "Searchable Melody", "Mix Album", "Mix Artist");
    database.insert_songs(vec![song]).unwrap();

    main_window
        .global::<AppCallbacks>()
        .invoke_active_page_changed(Pages::Search);
    let nav =
        wait_until(|| main_window.global::<AppProps>().get_active_page() == Pages::Search).await;
    assert!(nav);

    main_window
        .global::<AppCallbacks>()
        .invoke_search_term_changed("Melody".into());

    let searched = wait_until(|| {
        let results = main_window
            .global::<SearchPageProps>()
            .get_provider_results();
        results.row_count() > 0
            && results.row_data(0).is_some_and(|r| {
                r.songs.row_count() == 1
                    && r.songs
                        .row_data(0)
                        .is_some_and(|s| s.title == "Searchable Melody")
            })
    })
    .await;
    assert!(searched);
    assert_eq!(
        main_window.global::<AppProps>().get_active_page(),
        Pages::Search
    );

    let handles_found = wait_until(|| {
        ElementHandle::find_by_accessible_label(main_window, "Searchable Melody").count() == 1
    })
    .await;
    assert!(handles_found);

    let handles: Vec<ElementHandle> =
        ElementHandle::find_by_accessible_label(main_window, "Searchable Melody").collect();
    assert_eq!(handles.len(), 1);
    assert!(handles[0].is_valid());
}

#[tracing::instrument(level = "debug", skip_all)]
async fn do_search_category_albums(
    main_window: &'static MainWindow,
    state_manager: &'static StateManager,
) {
    set_test_step("search_albums_insert_song");
    let database = state_manager.get_database().await;
    let song = create_test_song("s_album_disc", "Mix Track", "Searchable Disc", "Mix Artist");
    database.insert_songs(vec![song]).unwrap();

    set_test_step("search_albums_goto_search_page");
    main_window
        .global::<AppCallbacks>()
        .invoke_active_page_changed(Pages::Search);
    let nav =
        wait_until(|| main_window.global::<AppProps>().get_active_page() == Pages::Search).await;
    assert!(nav);

    set_test_step("search_albums_invoke_search");
    main_window
        .global::<AppCallbacks>()
        .invoke_search_term_changed("Disc".into());

    set_test_step("search_albums_wait_results");
    let searched = wait_until(|| {
        let results = main_window
            .global::<SearchPageProps>()
            .get_provider_results();
        results.row_count() > 0
            && results
                .iter()
                .any(|r| r.albums.iter().any(|a| a.title == "Searchable Disc"))
    })
    .await;
    assert!(searched);

    set_test_step("search_albums_click_tab");
    let album_tabs: Vec<ElementHandle> =
        ElementHandle::find_by_accessible_label(main_window, "Search Albums").collect();
    assert_eq!(album_tabs.len(), 1);
    assert!(album_tabs[0].is_valid());
    click_element(&album_tabs[0]).await;
    main_window.global::<SearchPageProps>().set_active_tab(1);

    set_test_step("search_albums_wait_handle");
    let handles_found = wait_until(|| {
        ElementHandle::find_by_accessible_label(main_window, "Searchable Disc").count() > 0
    })
    .await;
    assert!(handles_found);

    let handles: Vec<ElementHandle> =
        ElementHandle::find_by_accessible_label(main_window, "Searchable Disc").collect();
    assert!(!handles.is_empty());
    assert!(handles[0].is_valid());
}

#[tracing::instrument(level = "debug", skip_all)]
async fn do_search_category_artists(
    main_window: &'static MainWindow,
    state_manager: &'static StateManager,
) {
    let database = state_manager.get_database().await;
    let song = create_test_song("s4", "Mix Track", "Mix Album", "Searchable Singer");
    database.insert_songs(vec![song]).unwrap();

    main_window
        .global::<AppCallbacks>()
        .invoke_active_page_changed(Pages::Search);
    let nav =
        wait_until(|| main_window.global::<AppProps>().get_active_page() == Pages::Search).await;
    assert!(nav);

    main_window
        .global::<AppCallbacks>()
        .invoke_search_term_changed("Singer".into());

    let searched = wait_until(|| {
        let results = main_window
            .global::<SearchPageProps>()
            .get_provider_results();
        results.row_count() > 0
            && results
                .iter()
                .any(|r| r.artists.iter().any(|a| a.title == "Searchable Singer"))
    })
    .await;
    assert!(searched);

    let artist_tabs: Vec<ElementHandle> =
        ElementHandle::find_by_accessible_label(main_window, "Search Artists").collect();
    assert_eq!(artist_tabs.len(), 1);
    assert!(artist_tabs[0].is_valid());
    click_element(&artist_tabs[0]).await;

    let handles_found = wait_until(|| {
        ElementHandle::find_by_accessible_label(main_window, "Searchable Singer").count() > 0
    })
    .await;
    assert!(handles_found);

    let handles: Vec<ElementHandle> =
        ElementHandle::find_by_accessible_label(main_window, "Searchable Singer").collect();
    assert!(!handles.is_empty());
    assert!(handles[0].is_valid());
}

#[tracing::instrument(level = "debug", skip_all)]
async fn do_search_category_playlists(
    main_window: &'static MainWindow,
    state_manager: &'static StateManager,
) {
    let database = state_manager.get_database().await;
    let song = create_test_song("s4", "Mix Track", "Mix Album", "Mix Artist");
    database.insert_songs(vec![song.clone()]).unwrap();
    let playlist = Playlist {
        playlist_id: Some("pl_search".into()),
        playlist_name: "Searchable Mix".into(),
        ..Default::default()
    };
    database
        .create_playlist_with_songs(playlist, &[song])
        .unwrap();

    main_window
        .global::<AppCallbacks>()
        .invoke_active_page_changed(Pages::Search);
    let nav =
        wait_until(|| main_window.global::<AppProps>().get_active_page() == Pages::Search).await;
    assert!(nav);

    main_window
        .global::<AppCallbacks>()
        .invoke_search_term_changed("Mix".into());

    let searched = wait_until(|| {
        let results = main_window
            .global::<SearchPageProps>()
            .get_provider_results();
        results.row_count() > 0
            && results.row_data(0).is_some_and(|r| {
                r.playlists.row_count() == 1
                    && r.playlists
                        .row_data(0)
                        .is_some_and(|p| p.title == "Searchable Mix")
            })
    })
    .await;
    assert!(searched);

    let playlist_tabs: Vec<ElementHandle> =
        ElementHandle::find_by_accessible_label(main_window, "Search Playlists").collect();
    assert_eq!(playlist_tabs.len(), 1);
    assert!(playlist_tabs[0].is_valid());
    click_element(&playlist_tabs[0]).await;

    let handles_found = wait_until(|| {
        let handles: Vec<ElementHandle> =
            ElementHandle::find_by_accessible_label(main_window, "Searchable Mix").collect();
        handles.len() == 1
    })
    .await;
    assert!(handles_found);

    let handles: Vec<ElementHandle> =
        ElementHandle::find_by_accessible_label(main_window, "Searchable Mix").collect();
    assert_eq!(handles.len(), 1);
    assert!(handles[0].is_valid());
}

#[tracing::instrument(level = "debug", skip_all)]
async fn do_search_extension_integration(
    main_window: &'static MainWindow,
    state_manager: &'static StateManager,
) {
    load_sample_extension(state_manager).await;

    main_window
        .global::<AppCallbacks>()
        .invoke_active_page_changed(Pages::Search);
    let navigated =
        wait_until(|| main_window.global::<AppProps>().get_active_page() == Pages::Search).await;
    assert!(navigated);

    main_window
        .global::<AppCallbacks>()
        .invoke_search_term_changed("Test".into());

    let searched = wait_until(|| {
        let results = main_window
            .global::<SearchPageProps>()
            .get_provider_results();
        results.row_count() > 0
            && results
                .iter()
                .any(|r| r.extension == "rs.sample" && r.songs.row_count() == 2)
    })
    .await;

    assert!(searched);
    assert_eq!(
        main_window.global::<AppProps>().get_active_page(),
        Pages::Search
    );
}

integration_test!(
    test_search_songs => do_search_category_songs,
    test_search_albums => do_search_category_albums,
    test_search_artists => do_search_category_artists,
    test_search_playlists => do_search_category_playlists,
    test_search_extension_integration => do_search_extension_integration,
);
