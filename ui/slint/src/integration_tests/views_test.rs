use i_slint_backend_testing::ElementHandle;
use slint::{ComponentHandle, Model, ModelRc, VecModel};
use slint_app::{
    AlbumContentPageProps, AlbumModel, AlbumsPageProps, AllSongsPageProps, AppCallbacks, AppProps,
    ArtistContentPageProps, ArtistModel, ArtistsPageProps, ExtensionProviderItem, MainWindow,
    Pages, PlaylistContentPageProps, PlaylistModel, PlaylistsPageProps, SongModel, UtilCallbacks,
    test_utils::integration::{create_test_song, integration_test, wait_until},
};
use songs_proto::moosync::types::Playlist;
use state_manager::StateManager;

#[tracing::instrument(level = "debug", skip_all)]
async fn do_view_all_songs(main_window: &'static MainWindow, state_manager: &'static StateManager) {
    let database = state_manager.get_database().await;
    let song = create_test_song("s1", "Song Alpha", "Album Alpha", "Artist Alpha");
    database.insert_songs(vec![song]).unwrap();

    if main_window.global::<AppProps>().get_active_page() == Pages::AllSongs {
        main_window
            .global::<AppCallbacks>()
            .invoke_active_page_changed(Pages::Explore);
        let _ = wait_until(|| main_window.global::<AppProps>().get_active_page() == Pages::Explore)
            .await;
    }

    main_window
        .global::<AppCallbacks>()
        .invoke_active_page_changed(Pages::AllSongs);

    let updated = wait_until(|| {
        let songs = main_window.global::<AllSongsPageProps>().get_songs();
        songs.row_count() > 0
            && songs
                .row_data(0)
                .is_some_and(|s| s.title == "Song Alpha" && s.id == "s1")
    })
    .await;
    assert!(updated);

    let found = wait_until(|| {
        ElementHandle::find_by_accessible_label(main_window, "Song Alpha").count() > 0
    })
    .await;
    assert!(found);

    let song_handles: Vec<ElementHandle> =
        ElementHandle::find_by_accessible_label(main_window, "Song Alpha").collect();
    assert_eq!(song_handles.len(), 1);
    assert!(song_handles[0].is_valid());
}

#[tracing::instrument(level = "debug", skip_all)]
async fn do_view_playlists(main_window: &'static MainWindow, state_manager: &'static StateManager) {
    let database = state_manager.get_database().await;
    let playlist = Playlist {
        playlist_id: Some("pl1".into()),
        playlist_name: "Favorites Playlist".into(),
        ..Default::default()
    };
    database.create_playlist(playlist).unwrap();

    main_window
        .global::<AppCallbacks>()
        .invoke_active_page_changed(Pages::Playlists);

    let updated = wait_until(|| {
        let playlists = main_window.global::<PlaylistsPageProps>().get_playlists();
        playlists.row_count() > 0
            && playlists
                .row_data(0)
                .is_some_and(|p| p.title == "Favorites Playlist" && p.id == "pl1")
    })
    .await;
    assert!(updated);

    let found = wait_until(|| {
        ElementHandle::find_by_accessible_label(main_window, "Favorites Playlist").count() > 0
    })
    .await;
    assert!(found);

    let handles: Vec<ElementHandle> =
        ElementHandle::find_by_accessible_label(main_window, "Favorites Playlist").collect();
    assert_eq!(handles.len(), 1);
    assert!(handles[0].is_valid());
}

#[tracing::instrument(level = "debug", skip_all)]
async fn do_view_playlist_content(
    main_window: &'static MainWindow,
    state_manager: &'static StateManager,
) {
    let database = state_manager.get_database().await;
    let song = create_test_song("s1", "Content Song", "Content Album", "Content Artist");
    database.insert_songs(vec![song.clone()]).unwrap();
    let playlist = Playlist {
        playlist_id: Some("pl2".into()),
        playlist_name: "Greatest Hits".into(),
        ..Default::default()
    };
    database
        .create_playlist_with_songs(playlist, &[song])
        .unwrap();

    main_window
        .global::<PlaylistsPageProps>()
        .set_selected_playlist(PlaylistModel {
            id: "pl2".into(),
            title: "Greatest Hits".into(),
            ..Default::default()
        });

    main_window
        .global::<AppCallbacks>()
        .invoke_active_page_changed(Pages::PlaylistContent);

    let updated = wait_until(|| {
        let songs = main_window.global::<PlaylistContentPageProps>().get_songs();
        songs.row_count() > 0
            && songs
                .row_data(0)
                .is_some_and(|s| s.title == "Content Song" && s.id == "s1")
    })
    .await;
    assert!(updated);

    let found_song = wait_until(|| {
        ElementHandle::find_by_accessible_label(main_window, "Content Song").count() > 0
    })
    .await;
    assert!(found_song);

    let handles: Vec<ElementHandle> =
        ElementHandle::find_by_accessible_label(main_window, "Content Song").collect();
    assert_eq!(handles.len(), 1);
    assert!(handles[0].is_valid());

    let found_header = wait_until(|| {
        ElementHandle::find_by_accessible_label(main_window, "Greatest Hits").count() > 0
    })
    .await;
    assert!(found_header);

    let header_handles: Vec<ElementHandle> =
        ElementHandle::find_by_accessible_label(main_window, "Greatest Hits").collect();
    assert_eq!(header_handles.len(), 1);
    assert!(header_handles[0].is_valid());
}

#[tracing::instrument(level = "debug", skip_all)]
async fn do_view_albums(main_window: &'static MainWindow, state_manager: &'static StateManager) {
    let database = state_manager.get_database().await;
    let song = create_test_song("s2", "Rock Song", "Rock Album", "Rock Artist");
    database.insert_songs(vec![song]).unwrap();

    main_window
        .global::<AppCallbacks>()
        .invoke_active_page_changed(Pages::Albums);

    let updated = wait_until(|| {
        let albums = main_window.global::<AlbumsPageProps>().get_albums();
        albums.row_count() > 0 && albums.row_data(0).is_some_and(|a| a.title == "Rock Album")
    })
    .await;
    assert!(updated);

    let found = wait_until(|| {
        ElementHandle::find_by_accessible_label(main_window, "Rock Album").count() > 0
    })
    .await;
    assert!(found);

    let handles: Vec<ElementHandle> =
        ElementHandle::find_by_accessible_label(main_window, "Rock Album").collect();
    assert_eq!(handles.len(), 1);
    assert!(handles[0].is_valid());
}

#[tracing::instrument(level = "debug", skip_all)]
async fn do_view_album_content(
    main_window: &'static MainWindow,
    state_manager: &'static StateManager,
) {
    let database = state_manager.get_database().await;
    let song = create_test_song("s2", "Album Song", "Rock Album", "Rock Artist");
    let inserted = database.insert_songs(vec![song]).unwrap();
    let album_id = inserted[0]
        .album
        .as_ref()
        .unwrap()
        .album_id
        .clone()
        .unwrap_or_default();

    main_window
        .global::<AlbumsPageProps>()
        .set_selected_album(AlbumModel {
            id: album_id.into(),
            title: "Rock Album".into(),
            ..Default::default()
        });

    main_window
        .global::<AppCallbacks>()
        .invoke_active_page_changed(Pages::AlbumContent);

    let updated = wait_until(|| {
        let songs = main_window.global::<AlbumContentPageProps>().get_songs();
        songs.row_count() > 0
            && songs
                .row_data(0)
                .is_some_and(|s| s.title == "Album Song" && s.id == "s2")
    })
    .await;
    assert!(updated);

    let found_song = wait_until(|| {
        ElementHandle::find_by_accessible_label(main_window, "Album Song").count() > 0
    })
    .await;
    assert!(found_song);

    let handles: Vec<ElementHandle> =
        ElementHandle::find_by_accessible_label(main_window, "Album Song").collect();
    assert_eq!(handles.len(), 1);
    assert!(handles[0].is_valid());

    let found_header = wait_until(|| {
        ElementHandle::find_by_accessible_label(main_window, "Rock Album").count() > 0
    })
    .await;
    assert!(found_header);

    let header_handles: Vec<ElementHandle> =
        ElementHandle::find_by_accessible_label(main_window, "Rock Album").collect();
    assert_eq!(header_handles.len(), 1);
    assert!(header_handles[0].is_valid());
}

#[tracing::instrument(level = "debug", skip_all)]
async fn do_view_artists(main_window: &'static MainWindow, state_manager: &'static StateManager) {
    let database = state_manager.get_database().await;
    let song = create_test_song("s3", "Pop Song", "Pop Album", "Rock Star");
    database.insert_songs(vec![song]).unwrap();

    main_window
        .global::<AppCallbacks>()
        .invoke_active_page_changed(Pages::Artists);

    let updated = wait_until(|| {
        let artists = main_window.global::<ArtistsPageProps>().get_artists();
        artists.row_count() > 0 && artists.row_data(0).is_some_and(|a| a.title == "Rock Star")
    })
    .await;
    assert!(updated);

    let found = wait_until(|| {
        ElementHandle::find_by_accessible_label(main_window, "Rock Star").count() > 0
    })
    .await;
    assert!(found);

    let handles: Vec<ElementHandle> =
        ElementHandle::find_by_accessible_label(main_window, "Rock Star").collect();
    assert_eq!(handles.len(), 1);
    assert!(handles[0].is_valid());
}

#[tracing::instrument(level = "debug", skip_all)]
async fn do_view_artist_content(
    main_window: &'static MainWindow,
    state_manager: &'static StateManager,
) {
    let database = state_manager.get_database().await;
    let song = create_test_song("s3", "Artist Song", "Pop Album", "Rock Star");
    let inserted = database.insert_songs(vec![song]).unwrap();
    let artist_id = inserted[0].artists[0].artist_id.clone().unwrap_or_default();

    main_window
        .global::<ArtistsPageProps>()
        .set_selected_artist(ArtistModel {
            id: artist_id.into(),
            title: "Rock Star".into(),
            ..Default::default()
        });

    main_window
        .global::<AppCallbacks>()
        .invoke_active_page_changed(Pages::ArtistContent);

    let updated = wait_until(|| {
        let songs = main_window.global::<ArtistContentPageProps>().get_songs();
        songs.row_count() > 0
            && songs
                .row_data(0)
                .is_some_and(|s| s.title == "Artist Song" && s.id == "s3")
    })
    .await;
    assert!(updated);

    let found_song = wait_until(|| {
        ElementHandle::find_by_accessible_label(main_window, "Artist Song").count() > 0
    })
    .await;
    assert!(found_song);

    let handles: Vec<ElementHandle> =
        ElementHandle::find_by_accessible_label(main_window, "Artist Song").collect();
    assert_eq!(handles.len(), 1);
    assert!(handles[0].is_valid());

    let found_header = wait_until(|| {
        ElementHandle::find_by_accessible_label(main_window, "Rock Star").count() > 0
    })
    .await;
    assert!(found_header);

    let header_handles: Vec<ElementHandle> =
        ElementHandle::find_by_accessible_label(main_window, "Rock Star").collect();
    assert_eq!(header_handles.len(), 1);
    assert!(header_handles[0].is_valid());
}

#[tracing::instrument(level = "debug", skip_all)]
async fn do_select_single_song(
    main_window: &'static MainWindow,
    _state_manager: &'static StateManager,
) {
    let current_selected = ModelRc::new(VecModel::default());
    let new_selected = main_window
        .global::<UtilCallbacks>()
        .invoke_update_selection(current_selected, 2, 0, false, false, false, 5);

    assert_eq!(new_selected.row_count(), 1);
    assert_eq!(new_selected.row_data(0), Some(2));
    assert!(
        main_window
            .global::<UtilCallbacks>()
            .invoke_is_index_selected(new_selected, 2)
    );
}

#[tracing::instrument(level = "debug", skip_all)]
async fn do_select_multiple_songs_ctrl(
    main_window: &'static MainWindow,
    _state_manager: &'static StateManager,
) {
    let current_selected = ModelRc::new(VecModel::from(vec![1, 3]));
    let new_selected = main_window
        .global::<UtilCallbacks>()
        .invoke_update_selection(current_selected, 2, 0, true, false, false, 5);

    assert_eq!(new_selected.row_count(), 3);
    assert!(
        main_window
            .global::<UtilCallbacks>()
            .invoke_is_index_selected(new_selected.clone(), 1)
    );
    assert!(
        main_window
            .global::<UtilCallbacks>()
            .invoke_is_index_selected(new_selected.clone(), 2)
    );
    assert!(
        main_window
            .global::<UtilCallbacks>()
            .invoke_is_index_selected(new_selected, 3)
    );
}

#[tracing::instrument(level = "debug", skip_all)]
async fn do_select_range_songs_shift(
    main_window: &'static MainWindow,
    _state_manager: &'static StateManager,
) {
    let current_selected = ModelRc::new(VecModel::from(vec![1]));
    let new_selected = main_window
        .global::<UtilCallbacks>()
        .invoke_update_selection(current_selected, 3, 1, false, true, false, 5);

    assert_eq!(new_selected.row_count(), 3);
    assert!(
        main_window
            .global::<UtilCallbacks>()
            .invoke_is_index_selected(new_selected.clone(), 1)
    );
    assert!(
        main_window
            .global::<UtilCallbacks>()
            .invoke_is_index_selected(new_selected.clone(), 2)
    );
    assert!(
        main_window
            .global::<UtilCallbacks>()
            .invoke_is_index_selected(new_selected, 3)
    );
}

#[tracing::instrument(level = "debug", skip_all)]
async fn do_get_selected_songs(
    main_window: &'static MainWindow,
    _state_manager: &'static StateManager,
) {
    let s0 = create_test_song("s0", "Song 0", "Album 0", "Artist 0");
    let s1 = create_test_song("s1", "Song 1", "Album 1", "Artist 1");
    let s2 = create_test_song("s2", "Song 2", "Album 2", "Artist 2");

    let songs_model = ModelRc::new(VecModel::from(vec![
        SongModel::from(s0),
        SongModel::from(s1.clone()),
        SongModel::from(s2.clone()),
    ]));
    let selected_indices = ModelRc::new(VecModel::from(vec![1, 2]));

    let selected_songs = main_window
        .global::<UtilCallbacks>()
        .invoke_get_selected_songs(songs_model, selected_indices);

    assert_eq!(selected_songs.row_count(), 2);
    assert_eq!(selected_songs.row_data(0).map(|s| s.id), Some("s1".into()));
    assert_eq!(selected_songs.row_data(1).map(|s| s.id), Some("s2".into()));
}

#[tracing::instrument(level = "debug", skip_all)]
async fn do_has_active_extension_providers(
    main_window: &'static MainWindow,
    _state_manager: &'static StateManager,
) {
    let empty_providers = ModelRc::new(VecModel::default());
    assert!(
        !main_window
            .global::<UtilCallbacks>()
            .invoke_has_active_extension_providers(empty_providers)
    );

    let active_providers = ModelRc::new(VecModel::from(vec![ExtensionProviderItem {
        id: "p1".into(),
        name: "Provider 1".into(),
        enabled: true,
    }]));
    assert!(
        main_window
            .global::<UtilCallbacks>()
            .invoke_has_active_extension_providers(active_providers)
    );
}

integration_test!(
    test_view_all_songs => do_view_all_songs,
    test_view_playlists => do_view_playlists,
    test_view_playlist_content => do_view_playlist_content,
    test_view_albums => do_view_albums,
    test_view_album_content => do_view_album_content,
    test_view_artists => do_view_artists,
    test_view_artist_content => do_view_artist_content,
    test_select_single_song => do_select_single_song,
    test_select_multiple_songs_ctrl => do_select_multiple_songs_ctrl,
    test_select_range_songs_shift => do_select_range_songs_shift,
    test_get_selected_songs => do_get_selected_songs,
    test_has_active_extension_providers => do_has_active_extension_providers,
);
