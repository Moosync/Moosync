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

use assertables::{assert_len_eq_x, assert_ok};
use rstest::rstest;
use slint::ComponentHandle;
use songs_proto::moosync::types::{GetEntityOptions, InnerSong, Playlist, Song, entity_result};
use tracing_test::traced_test;

use crate::{
    AppCallbacks, MainWindow,
    main_content::queue::QueuePageHandler,
    pages::PageHandler,
    test_utils::{TestSlintSmContext, main_window, state_manager_fixture},
};

#[rstest]
#[tokio::test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_queue_page_handler_initialize(
    main_window: MainWindow,
    state_manager_fixture: TestSlintSmContext,
) {
    let TestSlintSmContext { sm, .. } = state_manager_fixture;
    let handler = QueuePageHandler::new(&main_window, &sm);

    handler.initialize();
    main_window
        .global::<AppCallbacks>()
        .invoke_play_queue_index(0);
    main_window
        .global::<AppCallbacks>()
        .invoke_remove_from_queue(0);
    main_window.global::<AppCallbacks>().invoke_clear_queue();

    assert!(!main_window.get_playing());
}

#[rstest]
#[tokio::test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_queue_page_handler_save_queue_as_playlist(
    main_window: MainWindow,
    state_manager_fixture: TestSlintSmContext,
) {
    let TestSlintSmContext { sm, .. } = state_manager_fixture;
    let handler = QueuePageHandler::new(&main_window, &sm);
    handler.initialize();

    let song = Song {
        song: Some(InnerSong {
            id: Some("song_queue_1".into()),
            title: Some("Queue Song 1".into()),
            path: Some("/music/test1.mp3".into()),
            ..Default::default()
        }),
        ..Default::default()
    };
    {
        let mut ph = sm.get_player_handler_mut().await;
        ph.add_to_queue(vec![song]);
    }

    main_window
        .global::<AppCallbacks>()
        .invoke_save_queue_as_playlist("My Saved Queue".into(), "My Description".into());

    tokio::time::sleep(std::time::Duration::from_millis(100)).await;

    let db = sm.get_database().await;
    let playlists_res = db.get_entity_by_options(GetEntityOptions {
        playlist: Some(Playlist::default()),
        ..Default::default()
    });

    assert_ok!(playlists_res.as_ref());
    let res = playlists_res.unwrap().result;
    let Some(entity_result::Result::Playlists(list)) = res else {
        panic!("Expected playlists in entity result");
    };
    assert_len_eq_x!(&list.playlists, 1);
    assert_eq!(list.playlists[0].playlist_name, "My Saved Queue");
    assert_eq!(
        list.playlists[0].playlist_desc,
        Some("My Description".to_string())
    );
}
