use i_slint_backend_testing::ElementHandle;
use slint::{ComponentHandle, Model};
use slint_app::{
    AccountsProps, AppCallbacks, AppProps, MainWindow, OAuthState, Pages, SearchPageProps,
    accounts::AccountsHandler,
    test_utils::integration::{ExtensionFixture, create_test_song, integration_test, wait_until},
};
use songs_proto::moosync::types::Playlist;
use state_manager::StateManager;

#[tracing::instrument(level = "debug", skip_all)]
async fn do_search_category_songs(
    main_window: &'static MainWindow,
    state_manager: &'static StateManager,
) {
    let database = state_manager.get_database();
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
    let database = state_manager.get_database();
    let song = create_test_song("s_album_disc", "Mix Track", "Searchable Disc", "Mix Artist");
    database.insert_songs(vec![song]).unwrap();

    main_window
        .global::<AppCallbacks>()
        .invoke_active_page_changed(Pages::Search);
    let nav =
        wait_until(|| main_window.global::<AppProps>().get_active_page() == Pages::Search).await;
    assert!(nav);

    main_window
        .global::<AppCallbacks>()
        .invoke_search_term_changed("Disc".into());

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

    let album_tabs: Vec<ElementHandle> =
        ElementHandle::find_by_accessible_label(main_window, "Search Albums").collect();
    assert_eq!(album_tabs.len(), 1);
    assert!(album_tabs[0].is_valid());
    album_tabs[0]
        .single_click(slint::platform::PointerEventButton::Left)
        .await;
    main_window.global::<SearchPageProps>().set_active_tab(1);

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
    let database = state_manager.get_database();
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
    artist_tabs[0]
        .single_click(slint::platform::PointerEventButton::Left)
        .await;

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
    let database = state_manager.get_database();
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
    playlist_tabs[0]
        .single_click(slint::platform::PointerEventButton::Left)
        .await;

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
    let _ext = ExtensionFixture::new(state_manager).await;

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

#[tracing::instrument(level = "debug", skip_all)]
async fn do_search_not_logged_in_message(
    main_window: &'static MainWindow,
    state_manager: &'static StateManager,
) {
    let _ext = ExtensionFixture::new_empty(state_manager).await;

    main_window
        .global::<AppCallbacks>()
        .invoke_active_page_changed(Pages::Search);
    let navigated =
        wait_until(|| main_window.global::<AppProps>().get_active_page() == Pages::Search).await;
    assert!(navigated);

    AccountsHandler::fetch_and_render_accounts(main_window.as_weak(), state_manager);
    let accounts_loaded = wait_until(|| {
        main_window
            .global::<AccountsProps>()
            .get_accounts()
            .iter()
            .any(|a| a.package_name == "rs.sample.empty" && !a.logged_in)
    })
    .await;
    assert!(accounts_loaded);

    main_window
        .global::<SearchPageProps>()
        .set_search_query("test".into());
    main_window
        .global::<AppCallbacks>()
        .invoke_search_term_changed("test".into());

    let provider_loaded = wait_until(|| {
        let results = main_window
            .global::<SearchPageProps>()
            .get_provider_results();
        results.iter().any(|r| r.extension == "rs.sample.empty")
    })
    .await;
    assert!(provider_loaded);

    let sample_idx = main_window
        .global::<SearchPageProps>()
        .get_provider_results()
        .iter()
        .position(|r| r.extension == "rs.sample.empty")
        .unwrap();
    main_window
        .global::<SearchPageProps>()
        .set_selected_provider(sample_idx as i32);

    let prompt_visible = wait_until(|| {
        ElementHandle::find_by_accessible_label(main_window, "Not logged in").count() > 0
    })
    .await;
    assert!(prompt_visible);

    let handles: Vec<ElementHandle> =
        ElementHandle::find_by_accessible_label(main_window, "Not logged in").collect();
    assert_eq!(handles.len(), 1);
    assert!(handles[0].is_valid());

    main_window
        .global::<AppCallbacks>()
        .invoke_account_login("rs.sample.empty".into(), "sample_spotify".into());

    let modal_shown =
        wait_until(|| main_window.global::<OAuthState>().get_show_oauth_modal()).await;
    assert!(modal_shown);

    main_window
        .global::<AppCallbacks>()
        .invoke_submit_oauth_code("sample_callback?code=sample_auth_code_123".into());

    let prompt_hidden = wait_until(|| {
        ElementHandle::find_by_accessible_label(main_window, "Not logged in").count() == 0
    })
    .await;
    assert!(prompt_hidden);
}

#[tracing::instrument(level = "debug", skip_all)]
async fn do_search_no_results_tooltip(
    main_window: &'static MainWindow,
    _state_manager: &'static StateManager,
) {
    main_window
        .global::<AppCallbacks>()
        .invoke_active_page_changed(Pages::Search);
    let navigated =
        wait_until(|| main_window.global::<AppProps>().get_active_page() == Pages::Search).await;
    assert!(navigated);

    main_window
        .global::<SearchPageProps>()
        .set_search_query("nonexistent_song_query_12345".into());
    main_window
        .global::<AppCallbacks>()
        .invoke_search_term_changed("nonexistent_song_query_12345".into());

    let results_loaded = wait_until(|| {
        main_window
            .global::<SearchPageProps>()
            .get_provider_results()
            .row_count()
            > 0
    })
    .await;
    assert!(results_loaded);

    main_window
        .global::<SearchPageProps>()
        .set_selected_provider(0);

    let tooltip_visible = wait_until(|| {
        ElementHandle::find_by_accessible_label(main_window, "No results found").count() > 0
    })
    .await;
    assert!(tooltip_visible);

    let handles: Vec<ElementHandle> =
        ElementHandle::find_by_accessible_label(main_window, "No results found").collect();
    assert_eq!(handles.len(), 1);
    assert!(handles[0].is_valid());
}

#[tracing::instrument(level = "debug", skip_all)]
async fn do_search_network_covers_ui_hang(
    main_window: &'static MainWindow,
    state_manager: &'static StateManager,
) {
    use std::time::Duration;

    use wiremock::{Mock, MockServer, ResponseTemplate, matchers::method};

    let server = MockServer::start().await;
    const PNG_BYTES: &[u8] = &[
        0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a, 0x00, 0x00, 0x00, 0x0d, 0x49, 0x48, 0x44,
        0x52, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x08, 0x06, 0x00, 0x00, 0x00, 0x1f,
        0x15, 0xc4, 0x89, 0x00, 0x00, 0x00, 0x0a, 0x49, 0x44, 0x41, 0x54, 0x78, 0x9c, 0x63, 0x00,
        0x01, 0x00, 0x00, 0x05, 0x00, 0x01, 0x0d, 0x0a, 0x2d, 0xb4, 0x00, 0x00, 0x00, 0x00, 0x49,
        0x45, 0x4e, 0x44, 0xae, 0x42, 0x60, 0x82,
    ];

    Mock::given(method("GET"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_delay(Duration::from_secs(5))
                .set_body_bytes(PNG_BYTES.to_vec()),
        )
        .mount(&server)
        .await;

    let database = state_manager.get_database();
    let mut songs = Vec::new();
    for i in 0..10 {
        let mut song = create_test_song(
            &format!("net_song_{}", i),
            &format!("Track {}", i),
            "Test Album",
            "Test Artist",
        );
        if let Some(inner) = song.song.as_mut() {
            inner.song_cover_path_low = Some(format!("{}/cover_{}.png", server.uri(), i));
        }
        songs.push(song);
    }
    database.insert_songs(songs).unwrap();

    main_window.global::<AppCallbacks>().invoke_resize();

    main_window
        .global::<AppCallbacks>()
        .invoke_active_page_changed(Pages::Search);
    let nav =
        wait_until(|| main_window.global::<AppProps>().get_active_page() == Pages::Search).await;
    assert!(nav);

    main_window
        .global::<AppCallbacks>()
        .invoke_search_term_changed("Track".into());

    let searched = wait_until(|| {
        let results = main_window
            .global::<SearchPageProps>()
            .get_provider_results();
        results.row_count() > 0
            && results.row_data(0).is_some_and(|r| {
                r.songs.row_count() == 10
                    && r.songs.row_data(0).is_some_and(|s| s.title == "Track 0")
            })
    })
    .await;
    assert!(searched);
    let handles_found =
        wait_until(|| ElementHandle::find_by_accessible_label(main_window, "Track 0").count() == 1)
            .await;
    assert!(handles_found);

    let results = main_window
        .global::<SearchPageProps>()
        .get_provider_results();
    let song_before = results.row_data(0).unwrap().songs.row_data(0).unwrap();
    assert_eq!(
        song_before.coverPathLow.size().width,
        0,
        "Expected placeholder before fetch"
    );

    let start_time = std::time::Instant::now();
    let album_tabs: Vec<ElementHandle> =
        ElementHandle::find_by_accessible_label(main_window, "Search Albums").collect();
    assert_eq!(album_tabs.len(), 1);
    album_tabs[0]
        .single_click(slint::platform::PointerEventButton::Left)
        .await;

    let tab_switched =
        wait_until(|| main_window.global::<SearchPageProps>().get_active_tab() == 1).await;
    let elapsed = start_time.elapsed();
    assert!(tab_switched);

    let song_tabs: Vec<ElementHandle> =
        ElementHandle::find_by_accessible_label(main_window, "Search Songs").collect();
    assert_eq!(song_tabs.len(), 1);
    let switch_back_start = std::time::Instant::now();
    song_tabs[0]
        .single_click(slint::platform::PointerEventButton::Left)
        .await;

    let switched_back =
        wait_until(|| main_window.global::<SearchPageProps>().get_active_tab() == 0).await;
    let switch_back_elapsed = switch_back_start.elapsed();
    assert!(switched_back);

    let received = server.received_requests().await.unwrap();
    assert!(
        !received.is_empty(),
        "Wiremock received requests during fetch"
    );

    assert!(
        elapsed < Duration::from_millis(500),
        "UI hung switching to Albums! Elapsed: {:?}",
        elapsed
    );
    assert!(
        switch_back_elapsed < Duration::from_millis(500),
        "UI hung switching back to Songs! Elapsed: {:?}",
        switch_back_elapsed
    );

    let songs_loaded = wait_until(|| {
        let results = main_window
            .global::<SearchPageProps>()
            .get_provider_results();
        results.row_data(0).is_some_and(|r| {
            r.songs
                .row_data(0)
                .is_some_and(|s| s.coverPathLow.size().width > 0)
        })
    })
    .await;
    assert!(
        songs_loaded,
        "Song cover failed to load after network fetch"
    );
}

#[tracing::instrument(level = "debug", skip_all)]
async fn do_search_extension_pills_appear_immediately(
    main_window: &'static MainWindow,
    state_manager: &'static StateManager,
) {
    let _ext = ExtensionFixture::new(state_manager).await;

    main_window
        .global::<AppCallbacks>()
        .invoke_active_page_changed(Pages::Search);
    let navigated =
        wait_until(|| main_window.global::<AppProps>().get_active_page() == Pages::Search).await;
    assert!(navigated);

    let pills_appear = wait_until(|| {
        let results = main_window
            .global::<SearchPageProps>()
            .get_provider_results();
        results.row_count() == 2
            && results.row_data(0).is_some_and(|r| r.extension.is_empty())
            && results.row_data(1).is_some_and(|r| {
                r.extension == "rs.sample" && r.extension_name == "Sample Extension" && !r.loading
            })
    })
    .await;
    assert!(pills_appear);

    let display_name_pill_found = wait_until(|| {
        ElementHandle::find_by_accessible_label(main_window, "Sample Extension").count() == 1
    })
    .await;
    assert!(display_name_pill_found);
    assert_eq!(
        ElementHandle::find_by_accessible_label(main_window, "rs.sample").count(),
        0
    );

    main_window
        .global::<AppCallbacks>()
        .invoke_search_term_changed("Test".into());

    let searched = wait_until(|| {
        let results = main_window
            .global::<SearchPageProps>()
            .get_provider_results();
        results.row_count() == 2
            && results
                .iter()
                .any(|r| r.extension == "rs.sample" && !r.loading && r.songs.row_count() == 2)
    })
    .await;
    assert!(searched);
}

integration_test!(
    test_search_songs => do_search_category_songs,
    test_search_albums => do_search_category_albums,
    test_search_artists => do_search_category_artists,
    test_search_playlists => do_search_category_playlists,
    test_search_extension_integration => do_search_extension_integration,
    test_search_not_logged_in_message => do_search_not_logged_in_message,
    test_search_no_results_tooltip => do_search_no_results_tooltip,
    test_search_network_covers_ui_hang => do_search_network_covers_ui_hang,
    test_search_extension_pills_appear_immediately => do_search_extension_pills_appear_immediately,
);
