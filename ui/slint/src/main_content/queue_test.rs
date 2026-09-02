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

use assertables::{assert_len_eq_x, assert_ok};
use slint::ComponentHandle;
use songs_proto::moosync::types::{GetEntityOptions, InnerSong, Playlist, Song, entity_result};
use tracing_test::traced_test;

use crate::{
    AppCallbacks, MainWindow,
    main_content::queue::QueuePageHandler,
    pages::PageHandler,
    test_utils::{TestSlintSmContext, run_slint_test, wait_until},
};

#[test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_queue_page_handler_initialize() { run_slint_test(do_queue_page_handler_initialize); }

#[tracing::instrument(level = "debug", skip_all)]
async fn do_queue_page_handler_initialize(
    main_window: &'static MainWindow,
    state_manager_fixture: TestSlintSmContext,
) {
    let TestSlintSmContext { sm, .. } = state_manager_fixture;
    let song1 = Song {
        song: Some(InnerSong {
            id: Some("q_song_1".into()),
            title: Some("Queue Song 1".into()),
            path: Some("/music/q1.mp3".into()),
            ..Default::default()
        }),
        ..Default::default()
    };
    let song2 = Song {
        song: Some(InnerSong {
            id: Some("q_song_2".into()),
            title: Some("Queue Song 2".into()),
            path: Some("/music/q2.mp3".into()),
            ..Default::default()
        }),
        ..Default::default()
    };
    {
        let mut ph = sm.get_player_handler_mut().await;
        ph.add_to_queue(vec![song1, song2]);
    }

    let handler = QueuePageHandler::new(main_window, &sm);
    handler.initialize();

    main_window
        .global::<AppCallbacks>()
        .invoke_play_queue_index(1);
    tokio::time::sleep(Duration::from_millis(50)).await;
    {
        let ph = sm.get_player_handler().await;
        assert_eq!(ph.get_current_idx(), 1);
    }

    main_window
        .global::<AppCallbacks>()
        .invoke_remove_from_queue(0);
    tokio::time::sleep(Duration::from_millis(50)).await;
    {
        let ph = sm.get_player_handler().await;
        assert_eq!(ph.get_queue().len(), 1);
    }

    main_window.global::<AppCallbacks>().invoke_clear_queue();
    tokio::time::sleep(Duration::from_millis(50)).await;
    {
        let ph = sm.get_player_handler().await;
        assert_eq!(ph.get_queue().len(), 0);
    }
}

#[test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_queue_page_handler_save_queue_as_playlist() {
    run_slint_test(do_queue_page_handler_save_queue_as_playlist);
}

#[tracing::instrument(level = "debug", skip_all)]
async fn do_queue_page_handler_save_queue_as_playlist(
    main_window: &'static MainWindow,
    state_manager_fixture: TestSlintSmContext,
) {
    let TestSlintSmContext { sm, .. } = state_manager_fixture;
    let handler = QueuePageHandler::new(main_window, &sm);
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

    let loaded = wait_until(|| {
        let sm = sm.clone();
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                let db = sm.get_database().await;
                let res = db.get_entity_by_options(GetEntityOptions {
                    playlist: Some(Playlist::default()),
                    ..Default::default()
                });
                if let Ok(res) = res {
                    if let Some(entity_result::Result::Playlists(list)) = res.result {
                        return list.playlists.len() == 1;
                    }
                }
                false
            })
        })
    })
    .await;

    assert!(loaded);
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
