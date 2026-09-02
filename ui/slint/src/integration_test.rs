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

use std::time::Duration;

use i_slint_backend_testing::ElementHandle;
use slint::{ComponentHandle, Model, ModelRc, VecModel};
use songs_proto::moosync::types::{Album, Artist, Genre, InnerSong, Playlist, Song, SongType};
use state_manager::StateManager;
use tracing_test::traced_test;
use types::prelude::SongsExt;

use crate::{
    AlbumContentPageProps, AlbumsPageProps, AllSongsPageProps, AppCallbacks,
    ArtistContentPageProps, ArtistsPageProps, BottomBarCallbacks, MainWindow, Pages,
    PlaylistContentPageProps, PlaylistsPageProps, SearchPageProps, SongModel, UtilCallbacks,
    setup_ui,
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
        main_window
            .global::<PlaylistsPageProps>()
            .get_playlists()
            .row_count()
            == 1
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
        main_window
            .global::<PlaylistsPageProps>()
            .get_playlists()
            .row_count()
            == 1
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
        main_window
            .global::<AlbumsPageProps>()
            .get_albums()
            .row_count()
            == 1
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
        main_window
            .global::<AlbumsPageProps>()
            .get_albums()
            .row_count()
            == 1
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
        main_window
            .global::<ArtistsPageProps>()
            .get_artists()
            .row_count()
            == 1
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
        main_window
            .global::<ArtistsPageProps>()
            .get_artists()
            .row_count()
            == 1
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
    test_play_song => do_play_song,
    test_pause_and_resume_song => do_pause_and_resume_song,
    test_skip_song => do_skip_song,
    test_change_volume => do_change_volume,
);
