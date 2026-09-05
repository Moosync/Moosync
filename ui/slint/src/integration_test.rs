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

use std::{env, fs, path::PathBuf, time::Duration};

use i_slint_backend_testing::ElementHandle;
use slint::{ComponentHandle, Model, ModelRc, VecModel};
use songs_proto::moosync::types::{Album, Artist, Genre, InnerSong, Playlist, Song, SongType};
use state_manager::StateManager;
use tracing_test::traced_test;
use types::prelude::SongsExt;

use crate::{
    AlbumContentPageProps, AlbumModel, AlbumsPageProps, AllSongsPageProps, AppCallbacks,
    ArtistContentPageProps, ArtistModel, ArtistsPageProps, BottomBarCallbacks,
    ContextMenuCallbacks, ExtensionProviderItem, MainWindow, Pages, PlaylistContentPageProps,
    PlaylistModel, PlaylistsPageProps, SearchPageProps, SongModel, UtilCallbacks, setup_ui,
    test_utils::{TestSlintSmContext, run_slint_test, wait_until},
    utils::IntoVec,
};

macro_rules! integration_test {
    ($($test_name:ident => $async_fn:ident),* $(,)?) => {
        $(
            #[test]
            #[traced_test]
            #[tracing::instrument(level = "debug", skip_all)]
            fn $test_name() {
                run_slint_test($async_fn);
            }
        )*
    };
}

#[tracing::instrument(level = "debug", skip_all)]
async fn setup_test_context(state_manager: &StateManager) {
    state_manager.setup().await;
    let mut ph = state_manager.get_player_handler_mut().await;
    ph.set_context(Box::new(player::DummyAudioPlayerContext::new()));
}

#[tracing::instrument(level = "debug", skip_all)]
fn create_test_song(id: &str, title: &str, album: &str, artist: &str) -> Song {
    Song {
        song: Some(InnerSong {
            id: Some(id.to_string()),
            title: Some(title.to_string()),
            playback_url: Some(format!("https://example.com/{}", id)),
            path: Some(format!("/music/{}.mp3", id)),
            r#type: SongType::Local.into(),
            duration: Some(songs_proto::duration_proto::google::protobuf::Duration {
                seconds: 180,
                nanos: 0,
            }),
            ..Default::default()
        }),
        album: Some(Album {
            album_id: Some(format!("album_{}", id)),
            album_name: Some(album.to_string()),
            ..Default::default()
        }),
        artists: vec![Artist {
            artist_id: Some(format!("artist_{}", id)),
            artist_name: Some(artist.to_string()),
            ..Default::default()
        }],
        genre: vec![Genre {
            genre_id: Some(format!("genre_{}", id)),
            genre_name: Some("Rock".to_string()),
            ..Default::default()
        }],
    }
}

#[tracing::instrument(level = "debug", skip_all)]
async fn do_view_all_songs(
    main_window: &'static MainWindow,
    state_manager_fixture: TestSlintSmContext,
) {
    let TestSlintSmContext { sm, .. } = state_manager_fixture;
    let state_manager: &'static StateManager = Box::leak(Box::new(sm));
    setup_test_context(state_manager).await;
    let database = state_manager.get_database().await;
    let song_alpha = create_test_song("1", "Song Alpha", "Album Alpha", "Artist Alpha");
    let song_beta = create_test_song("2", "Song Beta", "Album Beta", "Artist Beta");
    database.insert_songs(vec![song_alpha, song_beta]).unwrap();
    setup_ui(main_window, state_manager);

    main_window
        .global::<AppCallbacks>()
        .invoke_active_page_changed(Pages::AllSongs);
    let loaded = wait_until(|| {
        main_window
            .global::<AllSongsPageProps>()
            .get_songs()
            .row_count()
            == 2
    })
    .await;

    assert!(loaded);
    assert_eq!(
        main_window
            .global::<AllSongsPageProps>()
            .get_songs()
            .row_count(),
        2
    );
    assert_eq!(
        main_window
            .global::<AllSongsPageProps>()
            .get_songs()
            .row_data(0)
            .unwrap()
            .title,
        "Song Alpha"
    );
    assert_eq!(
        main_window
            .global::<AllSongsPageProps>()
            .get_songs()
            .row_data(1)
            .unwrap()
            .title,
        "Song Beta"
    );
    let handles: Vec<ElementHandle> =
        ElementHandle::find_by_accessible_label(main_window, "Songs").collect();
    assert_eq!(handles.len(), 1);
    assert!(handles[0].is_valid());
}

#[tracing::instrument(level = "debug", skip_all)]
async fn do_view_playlists(
    main_window: &'static MainWindow,
    state_manager_fixture: TestSlintSmContext,
) {
    let TestSlintSmContext { sm, .. } = state_manager_fixture;
    let state_manager: &'static StateManager = Box::leak(Box::new(sm));
    setup_test_context(state_manager).await;
    let database = state_manager.get_database().await;
    let song = create_test_song("p1", "Playlist Song", "Playlist Album", "Playlist Artist");
    database.insert_songs(vec![song.clone()]).unwrap();
    let playlist = Playlist {
        playlist_id: Some("pl_1".into()),
        playlist_name: "Favorites Playlist".into(),
        ..Default::default()
    };
    database
        .create_playlist_with_songs(playlist, &[song])
        .unwrap();
    setup_ui(main_window, state_manager);

    main_window
        .global::<AppCallbacks>()
        .invoke_active_page_changed(Pages::Playlists);
    let loaded = wait_until(|| {
        let playlists = main_window.global::<PlaylistsPageProps>().get_playlists();
        playlists.row_count() == 1
            && playlists
                .row_data(0)
                .is_some_and(|p| p.title == "Favorites Playlist")
    })
    .await;

    assert!(loaded);
    assert_eq!(
        main_window
            .global::<PlaylistsPageProps>()
            .get_playlists()
            .row_count(),
        1
    );
    assert_eq!(
        main_window
            .global::<PlaylistsPageProps>()
            .get_playlists()
            .row_data(0)
            .unwrap()
            .title,
        "Favorites Playlist"
    );
    let handles: Vec<ElementHandle> =
        ElementHandle::find_by_accessible_label(main_window, "Playlists").collect();
    assert_eq!(handles.len(), 1);
    assert!(handles[0].is_valid());
}

#[tracing::instrument(level = "debug", skip_all)]
async fn do_view_playlist_content(
    main_window: &'static MainWindow,
    state_manager_fixture: TestSlintSmContext,
) {
    let TestSlintSmContext { sm, .. } = state_manager_fixture;
    let state_manager: &'static StateManager = Box::leak(Box::new(sm));
    setup_test_context(state_manager).await;
    let database = state_manager.get_database().await;
    let song = create_test_song("p2", "Content Song", "Content Album", "Content Artist");
    database.insert_songs(vec![song.clone()]).unwrap();
    let playlist = Playlist {
        playlist_id: Some("pl_2".into()),
        playlist_name: "Content Playlist".into(),
        ..Default::default()
    };
    database
        .create_playlist_with_songs(playlist, &[song])
        .unwrap();
    setup_ui(main_window, state_manager);
    main_window
        .global::<AppCallbacks>()
        .invoke_active_page_changed(Pages::Playlists);
    let _ = wait_until(|| {
        let playlists = main_window.global::<PlaylistsPageProps>().get_playlists();
        playlists.row_count() == 1
            && playlists
                .row_data(0)
                .is_some_and(|p| p.title == "Content Playlist")
    })
    .await;
    let playlist_model = main_window
        .global::<PlaylistsPageProps>()
        .get_playlists()
        .row_data(0)
        .unwrap();

    main_window
        .global::<PlaylistsPageProps>()
        .set_selected_playlist(playlist_model);
    main_window
        .global::<AppCallbacks>()
        .invoke_active_page_changed(Pages::PlaylistContent);
    let loaded = wait_until(|| {
        main_window
            .global::<PlaylistContentPageProps>()
            .get_songs()
            .row_count()
            == 1
    })
    .await;

    assert!(loaded);
    assert_eq!(
        main_window
            .global::<PlaylistContentPageProps>()
            .get_songs()
            .row_count(),
        1
    );
    assert_eq!(
        main_window
            .global::<PlaylistContentPageProps>()
            .get_songs()
            .row_data(0)
            .unwrap()
            .title,
        "Content Song"
    );
    let handles: Vec<ElementHandle> =
        ElementHandle::find_by_accessible_label(main_window, "Playlists").collect();
    assert_eq!(handles.len(), 1);
    assert!(handles[0].is_valid());
}

#[tracing::instrument(level = "debug", skip_all)]
async fn do_view_albums(
    main_window: &'static MainWindow,
    state_manager_fixture: TestSlintSmContext,
) {
    let TestSlintSmContext { sm, .. } = state_manager_fixture;
    let state_manager: &'static StateManager = Box::leak(Box::new(sm));
    setup_test_context(state_manager).await;
    let database = state_manager.get_database().await;
    let song = create_test_song("a1", "Album Song", "Classic Album", "Album Artist");
    database.insert_songs(vec![song]).unwrap();
    setup_ui(main_window, state_manager);

    main_window
        .global::<AppCallbacks>()
        .invoke_active_page_changed(Pages::Albums);
    let loaded = wait_until(|| {
        let albums = main_window.global::<AlbumsPageProps>().get_albums();
        albums.row_count() == 1
            && albums
                .row_data(0)
                .is_some_and(|a| a.title == "Classic Album")
    })
    .await;

    assert!(loaded);
    assert_eq!(
        main_window
            .global::<AlbumsPageProps>()
            .get_albums()
            .row_count(),
        1
    );
    assert_eq!(
        main_window
            .global::<AlbumsPageProps>()
            .get_albums()
            .row_data(0)
            .unwrap()
            .title,
        "Classic Album"
    );
    let handles: Vec<ElementHandle> =
        ElementHandle::find_by_accessible_label(main_window, "Albums").collect();
    assert_eq!(handles.len(), 1);
    assert!(handles[0].is_valid());
}

#[tracing::instrument(level = "debug", skip_all)]
async fn do_view_album_content(
    main_window: &'static MainWindow,
    state_manager_fixture: TestSlintSmContext,
) {
    let TestSlintSmContext { sm, .. } = state_manager_fixture;
    let state_manager: &'static StateManager = Box::leak(Box::new(sm));
    setup_test_context(state_manager).await;
    let database = state_manager.get_database().await;
    let song = create_test_song("a2", "Track in Album", "Target Album", "Target Artist");
    database.insert_songs(vec![song]).unwrap();
    setup_ui(main_window, state_manager);
    main_window
        .global::<AppCallbacks>()
        .invoke_active_page_changed(Pages::Albums);
    let _ = wait_until(|| {
        let albums = main_window.global::<AlbumsPageProps>().get_albums();
        albums.row_count() == 1
            && albums
                .row_data(0)
                .is_some_and(|a| a.title == "Target Album")
    })
    .await;
    let album_model = main_window
        .global::<AlbumsPageProps>()
        .get_albums()
        .row_data(0)
        .unwrap();

    main_window
        .global::<AlbumsPageProps>()
        .set_selected_album(album_model);
    main_window
        .global::<AppCallbacks>()
        .invoke_active_page_changed(Pages::AlbumContent);
    let loaded = wait_until(|| {
        main_window
            .global::<AlbumContentPageProps>()
            .get_songs()
            .row_count()
            == 1
    })
    .await;

    assert!(loaded);
    assert_eq!(
        main_window
            .global::<AlbumContentPageProps>()
            .get_songs()
            .row_count(),
        1
    );
    assert_eq!(
        main_window
            .global::<AlbumContentPageProps>()
            .get_songs()
            .row_data(0)
            .unwrap()
            .title,
        "Track in Album"
    );
    let handles: Vec<ElementHandle> =
        ElementHandle::find_by_accessible_label(main_window, "Albums").collect();
    assert_eq!(handles.len(), 1);
    assert!(handles[0].is_valid());
}

#[tracing::instrument(level = "debug", skip_all)]
async fn do_view_artists(
    main_window: &'static MainWindow,
    state_manager_fixture: TestSlintSmContext,
) {
    let TestSlintSmContext { sm, .. } = state_manager_fixture;
    let state_manager: &'static StateManager = Box::leak(Box::new(sm));
    setup_test_context(state_manager).await;
    let database = state_manager.get_database().await;
    let song = create_test_song("ar1", "Artist Song", "Artist Album", "Lead Artist");
    database.insert_songs(vec![song]).unwrap();
    setup_ui(main_window, state_manager);

    main_window
        .global::<AppCallbacks>()
        .invoke_active_page_changed(Pages::Artists);
    let loaded = wait_until(|| {
        let artists = main_window.global::<ArtistsPageProps>().get_artists();
        artists.row_count() == 1
            && artists
                .row_data(0)
                .is_some_and(|a| a.title == "Lead Artist")
    })
    .await;

    assert!(loaded);
    assert_eq!(
        main_window
            .global::<ArtistsPageProps>()
            .get_artists()
            .row_count(),
        1
    );
    assert_eq!(
        main_window
            .global::<ArtistsPageProps>()
            .get_artists()
            .row_data(0)
            .unwrap()
            .title,
        "Lead Artist"
    );
    let handles: Vec<ElementHandle> =
        ElementHandle::find_by_accessible_label(main_window, "Artists").collect();
    assert_eq!(handles.len(), 1);
    assert!(handles[0].is_valid());
}

#[tracing::instrument(level = "debug", skip_all)]
async fn do_view_artist_content(
    main_window: &'static MainWindow,
    state_manager_fixture: TestSlintSmContext,
) {
    let TestSlintSmContext { sm, .. } = state_manager_fixture;
    let state_manager: &'static StateManager = Box::leak(Box::new(sm));
    setup_test_context(state_manager).await;
    let database = state_manager.get_database().await;
    let song = create_test_song("ar2", "Track by Artist", "Artist Album", "Special Artist");
    database.insert_songs(vec![song]).unwrap();
    setup_ui(main_window, state_manager);
    main_window
        .global::<AppCallbacks>()
        .invoke_active_page_changed(Pages::Artists);
    let _ = wait_until(|| {
        let artists = main_window.global::<ArtistsPageProps>().get_artists();
        artists.row_count() == 1
            && artists
                .row_data(0)
                .is_some_and(|a| a.title == "Special Artist")
    })
    .await;
    let artist_model = main_window
        .global::<ArtistsPageProps>()
        .get_artists()
        .row_data(0)
        .unwrap();

    main_window
        .global::<ArtistsPageProps>()
        .set_selected_artist(artist_model);
    main_window
        .global::<AppCallbacks>()
        .invoke_active_page_changed(Pages::ArtistContent);
    let loaded = wait_until(|| {
        main_window
            .global::<ArtistContentPageProps>()
            .get_songs()
            .row_count()
            == 1
    })
    .await;

    assert!(loaded);
    assert_eq!(
        main_window
            .global::<ArtistContentPageProps>()
            .get_songs()
            .row_count(),
        1
    );
    assert_eq!(
        main_window
            .global::<ArtistContentPageProps>()
            .get_songs()
            .row_data(0)
            .unwrap()
            .title,
        "Track by Artist"
    );
    let handles: Vec<ElementHandle> =
        ElementHandle::find_by_accessible_label(main_window, "Artists").collect();
    assert_eq!(handles.len(), 1);
    assert!(handles[0].is_valid());
}

#[tracing::instrument(level = "debug", skip_all)]
async fn do_search_songs(
    main_window: &'static MainWindow,
    state_manager_fixture: TestSlintSmContext,
) {
    let TestSlintSmContext { sm, .. } = state_manager_fixture;
    let state_manager: &'static StateManager = Box::leak(Box::new(sm));
    setup_test_context(state_manager).await;
    let database = state_manager.get_database().await;
    let song = create_test_song("ss1", "Searchable Melody", "Other Album", "Other Artist");
    database.insert_songs(vec![song]).unwrap();
    setup_ui(main_window, state_manager);
    main_window
        .global::<AppCallbacks>()
        .invoke_active_page_changed(Pages::Search);

    main_window
        .global::<AppCallbacks>()
        .invoke_search_term_changed("Melody".into());
    let loaded = wait_until(|| {
        let results = main_window
            .global::<SearchPageProps>()
            .get_provider_results();
        results.row_count() > 0
            && results
                .row_data(0)
                .is_some_and(|r| r.songs.row_count() == 1)
    })
    .await;

    assert!(loaded);
    let results = main_window
        .global::<SearchPageProps>()
        .get_provider_results();
    let first_result = results.row_data(0).unwrap();
    assert_eq!(first_result.songs.row_count(), 1);
    assert_eq!(
        first_result.songs.row_data(0).unwrap().title,
        "Searchable Melody"
    );
}

#[tracing::instrument(level = "debug", skip_all)]
async fn do_search_albums(
    main_window: &'static MainWindow,
    state_manager_fixture: TestSlintSmContext,
) {
    let TestSlintSmContext { sm, .. } = state_manager_fixture;
    let state_manager: &'static StateManager = Box::leak(Box::new(sm));
    setup_test_context(state_manager).await;
    let database = state_manager.get_database().await;
    let song = create_test_song("ss2", "Song Title", "Searchable Disc", "Disc Artist");
    database.insert_songs(vec![song]).unwrap();
    setup_ui(main_window, state_manager);
    main_window
        .global::<AppCallbacks>()
        .invoke_active_page_changed(Pages::Search);

    main_window
        .global::<AppCallbacks>()
        .invoke_search_term_changed("Disc".into());
    let loaded = wait_until(|| {
        let results = main_window
            .global::<SearchPageProps>()
            .get_provider_results();
        results.row_count() > 0
            && results
                .row_data(0)
                .is_some_and(|r| r.albums.row_count() == 1)
    })
    .await;

    assert!(loaded);
    let results = main_window
        .global::<SearchPageProps>()
        .get_provider_results();
    let first_result = results.row_data(0).unwrap();
    assert_eq!(first_result.albums.row_count(), 1);
    assert_eq!(
        first_result.albums.row_data(0).unwrap().title,
        "Searchable Disc"
    );
}

#[tracing::instrument(level = "debug", skip_all)]
async fn do_search_artists(
    main_window: &'static MainWindow,
    state_manager_fixture: TestSlintSmContext,
) {
    let TestSlintSmContext { sm, .. } = state_manager_fixture;
    let state_manager: &'static StateManager = Box::leak(Box::new(sm));
    setup_test_context(state_manager).await;
    let database = state_manager.get_database().await;
    let song = create_test_song("ss3", "Track Title", "Album Title", "Searchable Singer");
    database.insert_songs(vec![song]).unwrap();
    setup_ui(main_window, state_manager);
    main_window
        .global::<AppCallbacks>()
        .invoke_active_page_changed(Pages::Search);

    main_window
        .global::<AppCallbacks>()
        .invoke_search_term_changed("Singer".into());
    let loaded = wait_until(|| {
        let results = main_window
            .global::<SearchPageProps>()
            .get_provider_results();
        results.row_count() > 0
            && results
                .row_data(0)
                .is_some_and(|r| r.artists.row_count() == 1)
    })
    .await;

    assert!(loaded);
    let results = main_window
        .global::<SearchPageProps>()
        .get_provider_results();
    let first_result = results.row_data(0).unwrap();
    assert_eq!(first_result.artists.row_count(), 1);
    assert_eq!(
        first_result.artists.row_data(0).unwrap().title,
        "Searchable Singer"
    );
}

#[tracing::instrument(level = "debug", skip_all)]
async fn do_search_playlists(
    main_window: &'static MainWindow,
    state_manager_fixture: TestSlintSmContext,
) {
    let TestSlintSmContext { sm, .. } = state_manager_fixture;
    let state_manager: &'static StateManager = Box::leak(Box::new(sm));
    setup_test_context(state_manager).await;
    let database = state_manager.get_database().await;
    let song = create_test_song("ss4", "Song Four", "Album Four", "Artist Four");
    database.insert_songs(vec![song.clone()]).unwrap();
    let playlist = Playlist {
        playlist_id: Some("pl_search".into()),
        playlist_name: "Searchable Mix".into(),
        ..Default::default()
    };
    database
        .create_playlist_with_songs(playlist, &[song])
        .unwrap();
    setup_ui(main_window, state_manager);
    main_window
        .global::<AppCallbacks>()
        .invoke_active_page_changed(Pages::Search);

    main_window
        .global::<AppCallbacks>()
        .invoke_search_term_changed("Mix".into());
    let loaded = wait_until(|| {
        let results = main_window
            .global::<SearchPageProps>()
            .get_provider_results();
        results.row_count() > 0
            && results
                .row_data(0)
                .is_some_and(|r| r.playlists.row_count() == 1)
    })
    .await;

    assert!(loaded);
    let results = main_window
        .global::<SearchPageProps>()
        .get_provider_results();
    let first_result = results.row_data(0).unwrap();
    assert_eq!(first_result.playlists.row_count(), 1);
    assert_eq!(
        first_result.playlists.row_data(0).unwrap().title,
        "Searchable Mix"
    );
}

#[tracing::instrument(level = "debug", skip_all)]
async fn do_select_single_song(
    main_window: &'static MainWindow,
    state_manager_fixture: TestSlintSmContext,
) {
    let TestSlintSmContext { sm, .. } = state_manager_fixture;
    let state_manager: &'static StateManager = Box::leak(Box::new(sm));
    setup_test_context(state_manager).await;
    setup_ui(main_window, state_manager);

    let selected = main_window
        .global::<UtilCallbacks>()
        .invoke_update_selection(ModelRc::default(), 2, 2, false, false, false, 5);

    let selected_vec: Vec<i32> = selected.into_vec();
    assert_eq!(selected_vec, vec![2]);
}

#[tracing::instrument(level = "debug", skip_all)]
async fn do_select_multiple_songs_ctrl(
    main_window: &'static MainWindow,
    state_manager_fixture: TestSlintSmContext,
) {
    let TestSlintSmContext { sm, .. } = state_manager_fixture;
    let state_manager: &'static StateManager = Box::leak(Box::new(sm));
    setup_test_context(state_manager).await;
    setup_ui(main_window, state_manager);
    let initial_selected = main_window
        .global::<UtilCallbacks>()
        .invoke_update_selection(ModelRc::default(), 1, 1, false, false, false, 5);

    let selected = main_window
        .global::<UtilCallbacks>()
        .invoke_update_selection(initial_selected, 3, 1, true, false, false, 5);

    let selected_vec: Vec<i32> = selected.clone().into_vec();
    assert_eq!(selected_vec, vec![1, 3]);
    assert!(
        main_window
            .global::<UtilCallbacks>()
            .invoke_is_index_selected(selected.clone(), 1)
    );
    assert!(
        main_window
            .global::<UtilCallbacks>()
            .invoke_is_index_selected(selected.clone(), 3)
    );
    assert!(
        !main_window
            .global::<UtilCallbacks>()
            .invoke_is_index_selected(selected, 2)
    );
}

#[tracing::instrument(level = "debug", skip_all)]
async fn do_select_range_songs_shift(
    main_window: &'static MainWindow,
    state_manager_fixture: TestSlintSmContext,
) {
    let TestSlintSmContext { sm, .. } = state_manager_fixture;
    let state_manager: &'static StateManager = Box::leak(Box::new(sm));
    setup_test_context(state_manager).await;
    setup_ui(main_window, state_manager);

    let selected = main_window
        .global::<UtilCallbacks>()
        .invoke_update_selection(ModelRc::default(), 3, 1, false, true, false, 5);

    let selected_vec: Vec<i32> = selected.into_vec();
    assert_eq!(selected_vec, vec![1, 2, 3]);
}

#[tracing::instrument(level = "debug", skip_all)]
async fn do_get_selected_songs(
    main_window: &'static MainWindow,
    state_manager_fixture: TestSlintSmContext,
) {
    let TestSlintSmContext { sm, .. } = state_manager_fixture;
    let state_manager: &'static StateManager = Box::leak(Box::new(sm));
    setup_test_context(state_manager).await;
    setup_ui(main_window, state_manager);
    let songs = vec![
        SongModel {
            title: "Track 0".into(),
            ..Default::default()
        },
        SongModel {
            title: "Track 1".into(),
            ..Default::default()
        },
        SongModel {
            title: "Track 2".into(),
            ..Default::default()
        },
    ];
    let display_songs = ModelRc::new(VecModel::from(songs));
    let selected_indices = ModelRc::new(VecModel::from(vec![0, 2]));

    let selected_songs = main_window
        .global::<UtilCallbacks>()
        .invoke_get_selected_songs(display_songs, selected_indices);

    assert_eq!(selected_songs.row_count(), 2);
    assert_eq!(selected_songs.row_data(0).unwrap().title, "Track 0");
    assert_eq!(selected_songs.row_data(1).unwrap().title, "Track 2");
}

#[tracing::instrument(level = "debug", skip_all)]
async fn do_has_active_extension_providers(
    main_window: &'static MainWindow,
    state_manager_fixture: TestSlintSmContext,
) {
    let TestSlintSmContext { sm, .. } = state_manager_fixture;
    let state_manager: &'static StateManager = Box::leak(Box::new(sm));
    setup_test_context(state_manager).await;
    setup_ui(main_window, state_manager);

    let empty_providers = ModelRc::new(VecModel::from(vec![]));
    let disabled_providers = ModelRc::new(VecModel::from(vec![
        ExtensionProviderItem {
            id: "ext1".into(),
            name: "Ext 1".into(),
            enabled: false,
        },
        ExtensionProviderItem {
            id: "ext2".into(),
            name: "Ext 2".into(),
            enabled: false,
        },
    ]));
    let active_providers = ModelRc::new(VecModel::from(vec![
        ExtensionProviderItem {
            id: "ext1".into(),
            name: "Ext 1".into(),
            enabled: false,
        },
        ExtensionProviderItem {
            id: "ext2".into(),
            name: "Ext 2".into(),
            enabled: true,
        },
    ]));

    let empty_active = main_window
        .global::<UtilCallbacks>()
        .invoke_has_active_extension_providers(empty_providers);
    let disabled_active = main_window
        .global::<UtilCallbacks>()
        .invoke_has_active_extension_providers(disabled_providers);
    let enabled_active = main_window
        .global::<UtilCallbacks>()
        .invoke_has_active_extension_providers(active_providers);

    assert!(!empty_active);
    assert!(!disabled_active);
    assert!(enabled_active);
}

#[tracing::instrument(level = "debug", skip_all)]
async fn do_play_song(main_window: &'static MainWindow, state_manager_fixture: TestSlintSmContext) {
    let TestSlintSmContext { sm, .. } = state_manager_fixture;
    let state_manager: &'static StateManager = Box::leak(Box::new(sm));
    setup_test_context(state_manager).await;
    setup_ui(main_window, state_manager);
    let song = SongModel {
        id: "play_1".into(),
        title: "Playing Song".into(),
        playback_url: "https://example.com/play_1".into(),
        ..Default::default()
    };

    main_window.global::<AppCallbacks>().invoke_play_song(song);
    let updated =
        wait_until(|| main_window.get_playing() && main_window.get_current_song().id == "play_1")
            .await;

    assert!(updated);
    assert!(main_window.get_playing());
    assert_eq!(main_window.get_current_song().id, "play_1");
    assert_eq!(main_window.get_current_song().title, "Playing Song");
    let ph = state_manager.get_player_handler().await;
    assert_eq!(ph.current_song().unwrap().get_id().unwrap(), "play_1");
}

#[tracing::instrument(level = "debug", skip_all)]
async fn do_pause_and_resume_song(
    main_window: &'static MainWindow,
    state_manager_fixture: TestSlintSmContext,
) {
    let TestSlintSmContext { sm, .. } = state_manager_fixture;
    let state_manager: &'static StateManager = Box::leak(Box::new(sm));
    setup_test_context(state_manager).await;
    setup_ui(main_window, state_manager);
    let song = SongModel {
        id: "pause_1".into(),
        title: "Pause Song".into(),
        playback_url: "https://example.com/pause_1".into(),
        ..Default::default()
    };
    main_window.global::<AppCallbacks>().invoke_play_song(song);
    let started = wait_until(|| main_window.get_playing()).await;
    assert!(started);

    main_window
        .global::<BottomBarCallbacks>()
        .invoke_play_pause_clicked();
    let paused = wait_until(|| !main_window.get_playing()).await;
    assert!(paused);
    assert!(!main_window.get_playing());

    main_window
        .global::<BottomBarCallbacks>()
        .invoke_play_pause_clicked();
    let resumed = wait_until(|| main_window.get_playing()).await;
    assert!(resumed);
    assert!(main_window.get_playing());
}

#[tracing::instrument(level = "debug", skip_all)]
async fn do_skip_song(main_window: &'static MainWindow, state_manager_fixture: TestSlintSmContext) {
    let TestSlintSmContext { sm, .. } = state_manager_fixture;
    let state_manager: &'static StateManager = Box::leak(Box::new(sm));
    setup_test_context(state_manager).await;
    setup_ui(main_window, state_manager);
    let song_first = SongModel {
        id: "skip_1".into(),
        title: "Track One".into(),
        playback_url: "https://example.com/skip_1".into(),
        ..Default::default()
    };
    let song_second = SongModel {
        id: "skip_2".into(),
        title: "Track Two".into(),
        playback_url: "https://example.com/skip_2".into(),
        ..Default::default()
    };
    main_window
        .global::<AppCallbacks>()
        .invoke_play_song(song_first);
    let _ = wait_until(|| main_window.get_current_song().id == "skip_1").await;
    main_window
        .global::<AppCallbacks>()
        .invoke_add_song_to_queue(song_second);

    main_window
        .global::<BottomBarCallbacks>()
        .invoke_next_song();
    let skipped = wait_until(|| main_window.get_current_song().id == "skip_2").await;

    assert!(skipped);
    assert_eq!(main_window.get_current_song().id, "skip_2");
    assert_eq!(main_window.get_current_song().title, "Track Two");
    let ph = state_manager.get_player_handler().await;
    assert_eq!(ph.current_song().unwrap().get_id().unwrap(), "skip_2");
}

#[tracing::instrument(level = "debug", skip_all)]
async fn do_change_volume(
    main_window: &'static MainWindow,
    state_manager_fixture: TestSlintSmContext,
) {
    let TestSlintSmContext { sm, .. } = state_manager_fixture;
    let state_manager: &'static StateManager = Box::leak(Box::new(sm));
    setup_test_context(state_manager).await;
    setup_ui(main_window, state_manager);

    main_window
        .global::<BottomBarCallbacks>()
        .invoke_set_volume(72);
    tokio::time::sleep(Duration::from_millis(50)).await;

    let ph = state_manager.get_player_handler().await;
    assert_eq!(ph.get_volume(), 72);
}

#[tracing::instrument(level = "debug", skip_all)]
fn get_sample_wasm_path() -> PathBuf {
    let Ok(runfiles_dir) = env::var("TEST_SRCDIR") else {
        panic!("TEST_SRCDIR not set or sample_extension.wasm not found in runfiles");
    };
    let candidates = [
        PathBuf::from(&runfiles_dir)
            .join("moosync_ext+/sample_extensions/rs/sample_extension.wasm"),
        PathBuf::from(&runfiles_dir).join("moosync_ext/sample_extensions/rs/sample_extension.wasm"),
        PathBuf::from(&runfiles_dir).join("_main/sample_extensions/rs/sample_extension.wasm"),
    ];
    candidates
        .iter()
        .find(|p| p.exists())
        .cloned()
        .unwrap_or_else(|| candidates[0].clone())
}

#[tracing::instrument(level = "debug", skip_all)]
async fn load_custom_extension(
    state_manager: &StateManager,
    dir_name: &str,
    pkg_name: &str,
    display_name: &str,
) {
    let ext_handler = state_manager.get_extension_handler().await;
    let extensions_dir = ext_handler.extensions_dir.clone();
    let ext_dir = extensions_dir.join(dir_name);
    fs::create_dir_all(&ext_dir).unwrap();
    let manifest = format!(
        r#"{{
        "name": "{pkg_name}",
        "displayName": "{display_name}",
        "version": "1.0.0",
        "extensionEntry": "main.wasm",
        "moosyncExtension": true,
        "description": "Sample Rust Extension",
        "icon": "",
        "author": "Moosync"
    }}"#
    );
    fs::write(ext_dir.join("package.json"), manifest).unwrap();
    fs::copy(get_sample_wasm_path(), ext_dir.join("main.wasm")).unwrap();

    ext_handler.find_new_extensions().unwrap();
}

#[tracing::instrument(level = "debug", skip_all)]
async fn load_sample_extension(state_manager: &StateManager) {
    load_custom_extension(state_manager, "sample_rs", "sample.rs", "Sample Extension").await;
}

#[tracing::instrument(level = "debug", skip_all)]
async fn do_search_extension_integration(
    main_window: &'static MainWindow,
    state_manager_fixture: TestSlintSmContext,
) {
    let TestSlintSmContext { sm, .. } = state_manager_fixture;
    let state_manager: &'static StateManager = Box::leak(Box::new(sm));
    setup_test_context(state_manager).await;
    load_sample_extension(state_manager).await;
    setup_ui(main_window, state_manager);

    main_window
        .global::<AppCallbacks>()
        .invoke_active_page_changed(Pages::Search);

    main_window
        .global::<AppCallbacks>()
        .invoke_search_term_changed("test".into());

    let loaded = wait_until(|| {
        let results = main_window
            .global::<SearchPageProps>()
            .get_provider_results();
        results.row_count() >= 2
            && results
                .row_data(1)
                .is_some_and(|r| r.songs.row_count() == 2 && r.extension == "sample.rs")
    })
    .await;

    assert!(loaded);
    let results = main_window
        .global::<SearchPageProps>()
        .get_provider_results();
    let ext_result = results.row_data(1).unwrap();
    assert_eq!(ext_result.extension, "sample.rs");
    assert_eq!(ext_result.songs.row_count(), 2);
    assert_eq!(ext_result.albums.row_count(), 2);
    assert_eq!(ext_result.artists.row_count(), 2);
    assert_eq!(ext_result.playlists.row_count(), 2);
    assert_eq!(ext_result.songs.row_data(0).unwrap().title, "Search Song 1");
    assert_eq!(ext_result.songs.row_data(0).unwrap().extension, "sample.rs");
}

#[tracing::instrument(level = "debug", skip_all)]
async fn do_playlists_extension_integration(
    main_window: &'static MainWindow,
    state_manager_fixture: TestSlintSmContext,
) {
    let TestSlintSmContext { sm, .. } = state_manager_fixture;
    let state_manager: &'static StateManager = Box::leak(Box::new(sm));
    setup_test_context(state_manager).await;
    load_sample_extension(state_manager).await;
    setup_ui(main_window, state_manager);

    main_window
        .global::<AppCallbacks>()
        .invoke_active_page_changed(Pages::Playlists);

    let loaded = wait_until(|| {
        let playlists = main_window.global::<PlaylistsPageProps>().get_playlists();
        playlists.row_count() == 2
            && playlists
                .row_data(0)
                .is_some_and(|p| p.extension == "sample.rs")
    })
    .await;

    assert!(
        loaded,
        "Playlists not loaded, count: {}",
        main_window
            .global::<PlaylistsPageProps>()
            .get_playlists()
            .row_count()
    );
    let playlists = main_window.global::<PlaylistsPageProps>().get_playlists();
    let first_pl = playlists.row_data(0).unwrap();
    assert_eq!(first_pl.title, "Extension Playlist 1");
    assert_eq!(first_pl.extension, "sample.rs");

    main_window
        .global::<PlaylistsPageProps>()
        .set_selected_playlist(first_pl);
    main_window
        .global::<AppCallbacks>()
        .invoke_active_page_changed(Pages::PlaylistContent);

    let content_loaded = wait_until(|| {
        main_window
            .global::<PlaylistContentPageProps>()
            .get_songs()
            .row_count()
            == 25
    })
    .await;

    assert!(
        content_loaded,
        "Content not loaded in PlaylistContentPageProps, count: {}, providers count: {}",
        main_window
            .global::<PlaylistContentPageProps>()
            .get_songs()
            .row_count(),
        main_window
            .global::<PlaylistContentPageProps>()
            .get_extension_providers()
            .row_count()
    );
    let songs = main_window.global::<PlaylistContentPageProps>().get_songs();
    assert_eq!(songs.row_data(0).unwrap().title, "Playlist Song 1");
    assert_eq!(songs.row_data(0).unwrap().extension, "sample.rs");

    let providers = main_window
        .global::<PlaylistContentPageProps>()
        .get_extension_providers();
    assert_eq!(providers.row_count(), 1);
    assert!(providers.row_data(0).unwrap().enabled);
}

#[tracing::instrument(level = "debug", skip_all)]
async fn do_playlist_content_extension_toggle(
    main_window: &'static MainWindow,
    state_manager_fixture: TestSlintSmContext,
) {
    let TestSlintSmContext { sm, .. } = state_manager_fixture;
    let state_manager: &'static StateManager = Box::leak(Box::new(sm));
    setup_test_context(state_manager).await;
    let database = state_manager.get_database().await;
    let song = create_test_song("pl_local", "Local Song", "Local Album", "Local Artist");
    database.insert_songs(vec![song.clone()]).unwrap();
    let playlist = Playlist {
        playlist_id: Some("pl_local_1".into()),
        playlist_name: "Local Playlist".into(),
        ..Default::default()
    };
    database
        .create_playlist_with_songs(playlist, &[song])
        .unwrap();

    load_sample_extension(state_manager).await;
    setup_ui(main_window, state_manager);

    main_window
        .global::<AppCallbacks>()
        .invoke_active_page_changed(Pages::Playlists);
    let _ = wait_until(|| {
        main_window
            .global::<PlaylistsPageProps>()
            .get_playlists()
            .row_count()
            >= 1
    })
    .await;

    let playlist_model = PlaylistModel {
        id: "pl_local_1".into(),
        title: "Local Playlist".into(),
        ..Default::default()
    };
    main_window
        .global::<PlaylistsPageProps>()
        .set_selected_playlist(playlist_model);
    main_window
        .global::<AppCallbacks>()
        .invoke_active_page_changed(Pages::PlaylistContent);

    let loaded = wait_until(|| {
        main_window
            .global::<PlaylistContentPageProps>()
            .get_songs()
            .row_count()
            == 1
            && main_window
                .global::<PlaylistContentPageProps>()
                .get_extension_providers()
                .row_count()
                == 1
    })
    .await;
    assert!(loaded);

    let providers = main_window
        .global::<PlaylistContentPageProps>()
        .get_extension_providers();
    assert_eq!(providers.row_count(), 1);
    assert!(!providers.row_data(0).unwrap().enabled);

    main_window
        .global::<AppCallbacks>()
        .invoke_toggle_playlist_content_extension("sample.rs".into(), true);

    let toggled_on = wait_until(|| {
        main_window
            .global::<PlaylistContentPageProps>()
            .get_songs()
            .row_count()
            == 26
    })
    .await;
    assert!(toggled_on);

    main_window
        .global::<AppCallbacks>()
        .invoke_toggle_playlist_content_extension("sample.rs".into(), false);

    let toggled_off = wait_until(|| {
        main_window
            .global::<PlaylistContentPageProps>()
            .get_songs()
            .row_count()
            == 1
    })
    .await;
    assert!(toggled_off);
}

#[tracing::instrument(level = "debug", skip_all)]
async fn do_artist_content_extension_toggle_and_open(
    main_window: &'static MainWindow,
    state_manager_fixture: TestSlintSmContext,
) {
    let TestSlintSmContext { sm, .. } = state_manager_fixture;
    let state_manager: &'static StateManager = Box::leak(Box::new(sm));
    setup_test_context(state_manager).await;
    let database = state_manager.get_database().await;
    let song = create_test_song("art_local", "Artist Track", "Artist Album", "Main Artist");
    database.insert_songs(vec![song]).unwrap();

    load_sample_extension(state_manager).await;
    setup_ui(main_window, state_manager);

    let artist_model = ArtistModel {
        id: "artist_art_local".into(),
        title: "Main Artist".into(),
        ..Default::default()
    };
    main_window
        .global::<ArtistsPageProps>()
        .set_selected_artist(artist_model);
    main_window
        .global::<AppCallbacks>()
        .invoke_active_page_changed(Pages::ArtistContent);

    let loaded = wait_until(|| {
        main_window
            .global::<ArtistContentPageProps>()
            .get_songs()
            .row_count()
            == 1
            && main_window
                .global::<ArtistContentPageProps>()
                .get_extension_providers()
                .row_count()
                == 1
    })
    .await;
    assert!(loaded);
    assert!(
        !main_window
            .global::<ArtistContentPageProps>()
            .get_extension_providers()
            .row_data(0)
            .unwrap()
            .enabled
    );

    main_window
        .global::<AppCallbacks>()
        .invoke_toggle_artist_content_extension("sample.rs".into(), true);

    let toggled_on = wait_until(|| {
        main_window
            .global::<ArtistContentPageProps>()
            .get_songs()
            .row_count()
            == 26
    })
    .await;
    assert!(toggled_on);

    main_window
        .global::<AppCallbacks>()
        .invoke_toggle_artist_content_extension("sample.rs".into(), false);

    let toggled_off = wait_until(|| {
        main_window
            .global::<ArtistContentPageProps>()
            .get_songs()
            .row_count()
            == 1
    })
    .await;
    assert!(toggled_off);

    let ext_artist = ArtistModel {
        id: "ext_art".into(),
        title: "Ext Artist".into(),
        extension: "sample.rs".into(),
        ..Default::default()
    };
    main_window
        .global::<AppCallbacks>()
        .invoke_active_page_changed(Pages::Artists);
    main_window
        .global::<ArtistsPageProps>()
        .set_selected_artist(ext_artist);
    main_window
        .global::<AppCallbacks>()
        .invoke_active_page_changed(Pages::ArtistContent);

    let ext_loaded = wait_until(|| {
        main_window
            .global::<ArtistContentPageProps>()
            .get_songs()
            .row_count()
            == 25
            && main_window
                .global::<ArtistContentPageProps>()
                .get_extension_providers()
                .row_data(0)
                .is_some_and(|p| p.enabled)
    })
    .await;
    assert!(ext_loaded);
    let songs = main_window.global::<ArtistContentPageProps>().get_songs();
    assert_eq!(songs.row_count(), 25);
    assert_eq!(songs.row_data(0).unwrap().title, "Artist Song 1");
    assert_eq!(songs.row_data(0).unwrap().extension, "sample.rs");

    let providers = main_window
        .global::<ArtistContentPageProps>()
        .get_extension_providers();
    assert_eq!(providers.row_count(), 1);
    assert!(providers.row_data(0).unwrap().enabled);
}

#[tracing::instrument(level = "debug", skip_all)]
async fn do_album_content_extension_toggle_and_open(
    main_window: &'static MainWindow,
    state_manager_fixture: TestSlintSmContext,
) {
    let TestSlintSmContext { sm, .. } = state_manager_fixture;
    let state_manager: &'static StateManager = Box::leak(Box::new(sm));
    setup_test_context(state_manager).await;
    let database = state_manager.get_database().await;
    let song = create_test_song("alb_local", "Album Track", "Main Album", "Main Artist");
    database.insert_songs(vec![song]).unwrap();

    load_sample_extension(state_manager).await;
    setup_ui(main_window, state_manager);

    let album_model = AlbumModel {
        id: "album_alb_local".into(),
        title: "Main Album".into(),
        ..Default::default()
    };
    main_window
        .global::<AlbumsPageProps>()
        .set_selected_album(album_model);
    main_window
        .global::<AppCallbacks>()
        .invoke_active_page_changed(Pages::AlbumContent);

    let loaded = wait_until(|| {
        main_window
            .global::<AlbumContentPageProps>()
            .get_songs()
            .row_count()
            == 1
            && main_window
                .global::<AlbumContentPageProps>()
                .get_extension_providers()
                .row_count()
                == 1
    })
    .await;
    assert!(loaded);
    assert!(
        !main_window
            .global::<AlbumContentPageProps>()
            .get_extension_providers()
            .row_data(0)
            .unwrap()
            .enabled
    );

    main_window
        .global::<AppCallbacks>()
        .invoke_toggle_album_content_extension("sample.rs".into(), true);

    let toggled_on = wait_until(|| {
        main_window
            .global::<AlbumContentPageProps>()
            .get_songs()
            .row_count()
            == 26
    })
    .await;
    assert!(toggled_on);

    main_window
        .global::<AppCallbacks>()
        .invoke_toggle_album_content_extension("sample.rs".into(), false);

    let toggled_off = wait_until(|| {
        main_window
            .global::<AlbumContentPageProps>()
            .get_songs()
            .row_count()
            == 1
    })
    .await;
    assert!(toggled_off);

    let ext_album = AlbumModel {
        id: "ext_alb".into(),
        title: "Ext Album".into(),
        extension: "sample.rs".into(),
        ..Default::default()
    };
    main_window
        .global::<AppCallbacks>()
        .invoke_active_page_changed(Pages::Albums);
    main_window
        .global::<AlbumsPageProps>()
        .set_selected_album(ext_album);
    main_window
        .global::<AppCallbacks>()
        .invoke_active_page_changed(Pages::AlbumContent);

    let ext_loaded = wait_until(|| {
        main_window
            .global::<AlbumContentPageProps>()
            .get_songs()
            .row_count()
            == 25
            && main_window
                .global::<AlbumContentPageProps>()
                .get_extension_providers()
                .row_data(0)
                .is_some_and(|p| p.enabled)
    })
    .await;
    assert!(ext_loaded);
    let songs = main_window.global::<AlbumContentPageProps>().get_songs();
    assert_eq!(songs.row_count(), 25);
    assert_eq!(songs.row_data(0).unwrap().title, "Album Song 1");
    assert_eq!(songs.row_data(0).unwrap().extension, "sample.rs");

    let providers = main_window
        .global::<AlbumContentPageProps>()
        .get_extension_providers();
    assert_eq!(providers.row_count(), 1);
    assert!(providers.row_data(0).unwrap().enabled);
}

#[tracing::instrument(level = "debug", skip_all)]
async fn do_context_menu_goto_album(
    main_window: &'static MainWindow,
    state_manager_fixture: TestSlintSmContext,
) {
    let TestSlintSmContext { sm, .. } = state_manager_fixture;
    let state_manager: &'static StateManager = Box::leak(Box::new(sm));
    setup_test_context(state_manager).await;
    let database = state_manager.get_database().await;
    let song = create_test_song("nav_alb", "Song For Album", "Target Album", "Artist");
    database.insert_songs(vec![song.clone()]).unwrap();
    setup_ui(main_window, state_manager);

    main_window
        .global::<AppCallbacks>()
        .invoke_active_page_changed(Pages::AllSongs);
    let loaded = wait_until(|| {
        main_window
            .global::<AllSongsPageProps>()
            .get_songs()
            .row_count()
            == 1
    })
    .await;
    assert!(loaded);

    let song_model = main_window
        .global::<AllSongsPageProps>()
        .get_songs()
        .row_data(0)
        .unwrap();

    let song_models = ModelRc::new(VecModel::from(vec![song_model.clone()]));
    let action_id = format!("goto_album:{}", song_model.album_id);

    main_window
        .global::<ContextMenuCallbacks>()
        .invoke_song_action(song_models, action_id.into());

    let nav_loaded = wait_until(|| {
        main_window.get_active_page() == Pages::AlbumContent
            && main_window
                .global::<AlbumContentPageProps>()
                .get_songs()
                .row_count()
                == 1
    })
    .await;
    assert!(nav_loaded);
    assert_eq!(main_window.get_active_page(), Pages::AlbumContent);
    assert_eq!(
        main_window
            .global::<AlbumsPageProps>()
            .get_selected_album()
            .title,
        "Target Album"
    );
}

#[tracing::instrument(level = "debug", skip_all)]
async fn do_context_menu_goto_artist(
    main_window: &'static MainWindow,
    state_manager_fixture: TestSlintSmContext,
) {
    let TestSlintSmContext { sm, .. } = state_manager_fixture;
    let state_manager: &'static StateManager = Box::leak(Box::new(sm));
    setup_test_context(state_manager).await;
    let database = state_manager.get_database().await;
    let song = create_test_song("nav_art", "Song For Artist", "Album", "Target Artist");
    database.insert_songs(vec![song.clone()]).unwrap();
    setup_ui(main_window, state_manager);

    main_window
        .global::<AppCallbacks>()
        .invoke_active_page_changed(Pages::AllSongs);
    let loaded = wait_until(|| {
        main_window
            .global::<AllSongsPageProps>()
            .get_songs()
            .row_count()
            == 1
    })
    .await;
    assert!(loaded);

    let song_model = main_window
        .global::<AllSongsPageProps>()
        .get_songs()
        .row_data(0)
        .unwrap();

    let first_artist = song_model.artists.row_data(0).unwrap();
    let song_models = ModelRc::new(VecModel::from(vec![song_model]));
    let action_id = format!("goto_artist:{}", first_artist.id);

    main_window
        .global::<ContextMenuCallbacks>()
        .invoke_song_action(song_models, action_id.into());

    let nav_loaded = wait_until(|| {
        main_window.get_active_page() == Pages::ArtistContent
            && main_window
                .global::<ArtistContentPageProps>()
                .get_songs()
                .row_count()
                == 1
    })
    .await;
    assert!(nav_loaded);
    assert_eq!(main_window.get_active_page(), Pages::ArtistContent);
    assert_eq!(
        main_window
            .global::<ArtistsPageProps>()
            .get_selected_artist()
            .title,
        "Target Artist"
    );
}

#[tracing::instrument(level = "debug", skip_all)]
async fn do_playlist_content_pagination_integration(
    main_window: &'static MainWindow,
    state_manager_fixture: TestSlintSmContext,
) {
    let TestSlintSmContext { sm, .. } = state_manager_fixture;
    let state_manager: &'static StateManager = Box::leak(Box::new(sm));
    setup_test_context(state_manager).await;
    load_sample_extension(state_manager).await;
    setup_ui(main_window, state_manager);

    let ext_playlist = PlaylistModel {
        id: "ext-playlist-1".into(),
        title: "Extension Playlist 1".into(),
        extension: "sample.rs".into(),
        ..Default::default()
    };
    main_window
        .global::<PlaylistsPageProps>()
        .set_selected_playlist(ext_playlist);
    main_window
        .global::<AppCallbacks>()
        .invoke_active_page_changed(Pages::PlaylistContent);

    let page1_loaded = wait_until(|| {
        main_window
            .global::<PlaylistContentPageProps>()
            .get_songs()
            .row_count()
            == 25
    })
    .await;
    assert!(page1_loaded);

    main_window
        .global::<AppCallbacks>()
        .invoke_load_more_playlist_content();

    let page2_loaded = wait_until(|| {
        main_window
            .global::<PlaylistContentPageProps>()
            .get_songs()
            .row_count()
            == 50
    })
    .await;
    assert!(page2_loaded);

    main_window
        .global::<AppCallbacks>()
        .invoke_load_more_playlist_content();

    let page3_loaded = wait_until(|| {
        main_window
            .global::<PlaylistContentPageProps>()
            .get_songs()
            .row_count()
            == 60
    })
    .await;
    assert!(page3_loaded);

    main_window
        .global::<AppCallbacks>()
        .invoke_load_more_playlist_content();

    tokio::time::sleep(Duration::from_millis(50)).await;
    assert_eq!(
        main_window
            .global::<PlaylistContentPageProps>()
            .get_songs()
            .row_count(),
        60
    );
}

#[tracing::instrument(level = "debug", skip_all)]
async fn do_multiple_extensions_pagination_integration(
    main_window: &'static MainWindow,
    state_manager_fixture: TestSlintSmContext,
) {
    let TestSlintSmContext { sm, .. } = state_manager_fixture;
    let state_manager: &'static StateManager = Box::leak(Box::new(sm));
    setup_test_context(state_manager).await;
    load_custom_extension(state_manager, "sample_rs_1", "sample.rs.1", "Sample 1").await;
    load_custom_extension(state_manager, "sample_rs_2", "sample.rs.2", "Sample 2").await;

    let database = state_manager.get_database().await;
    let song = create_test_song("pl_multi", "Local Song", "Local Album", "Local Artist");
    database.insert_songs(vec![song.clone()]).unwrap();
    let playlist = Playlist {
        playlist_id: Some("pl_multi_1".into()),
        playlist_name: "Multi Playlist".into(),
        ..Default::default()
    };
    database
        .create_playlist_with_songs(playlist, &[song])
        .unwrap();

    setup_ui(main_window, state_manager);

    let playlist_model = PlaylistModel {
        id: "pl_multi_1".into(),
        title: "Multi Playlist".into(),
        ..Default::default()
    };
    main_window
        .global::<PlaylistsPageProps>()
        .set_selected_playlist(playlist_model);
    main_window
        .global::<AppCallbacks>()
        .invoke_active_page_changed(Pages::PlaylistContent);

    let loaded = wait_until(|| {
        main_window
            .global::<PlaylistContentPageProps>()
            .get_songs()
            .row_count()
            == 1
            && main_window
                .global::<PlaylistContentPageProps>()
                .get_extension_providers()
                .row_count()
                == 2
    })
    .await;
    assert!(loaded);

    main_window
        .global::<AppCallbacks>()
        .invoke_toggle_playlist_content_extension("sample.rs.1".into(), true);
    main_window
        .global::<AppCallbacks>()
        .invoke_toggle_playlist_content_extension("sample.rs.2".into(), true);

    let initial_ext_loaded = wait_until(|| {
        main_window
            .global::<PlaylistContentPageProps>()
            .get_songs()
            .row_count()
            == 51
    })
    .await;
    assert!(initial_ext_loaded);

    main_window
        .global::<AppCallbacks>()
        .invoke_load_more_playlist_content();

    let page2_multi_loaded = wait_until(|| {
        main_window
            .global::<PlaylistContentPageProps>()
            .get_songs()
            .row_count()
            == 101
    })
    .await;
    assert!(page2_multi_loaded);
}

integration_test!(
    test_view_all_songs => do_view_all_songs,
    test_view_playlists => do_view_playlists,
    test_view_playlist_content => do_view_playlist_content,
    test_view_albums => do_view_albums,
    test_view_album_content => do_view_album_content,
    test_view_artists => do_view_artists,
    test_view_artist_content => do_view_artist_content,
    test_search_songs => do_search_songs,
    test_search_albums => do_search_albums,
    test_search_artists => do_search_artists,
    test_search_playlists => do_search_playlists,
    test_select_single_song => do_select_single_song,
    test_select_multiple_songs_ctrl => do_select_multiple_songs_ctrl,
    test_select_range_songs_shift => do_select_range_songs_shift,
    test_get_selected_songs => do_get_selected_songs,
    test_has_active_extension_providers => do_has_active_extension_providers,
    test_play_song => do_play_song,
    test_pause_and_resume_song => do_pause_and_resume_song,
    test_skip_song => do_skip_song,
    test_change_volume => do_change_volume,
    test_search_extension_integration => do_search_extension_integration,
    test_playlists_extension_integration => do_playlists_extension_integration,
    test_playlist_content_extension_toggle => do_playlist_content_extension_toggle,
    test_artist_content_extension_toggle_and_open => do_artist_content_extension_toggle_and_open,
    test_album_content_extension_toggle_and_open => do_album_content_extension_toggle_and_open,
    test_context_menu_goto_album => do_context_menu_goto_album,
    test_context_menu_goto_artist => do_context_menu_goto_artist,
    test_playlist_content_pagination_integration => do_playlist_content_pagination_integration,
    test_multiple_extensions_pagination_integration => do_multiple_extensions_pagination_integration,
);
