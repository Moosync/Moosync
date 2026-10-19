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
use rstest::rstest;
use slint::{ComponentHandle, Model};
use songs_proto::moosync::types::{
    GetEntityOptions, InnerSong, LyricLine, Lyrics, Playlist, Song, entity_result,
};
use tracing_test::traced_test;

use crate::{
    AppCallbacks, ContextMenuCallbacks, LyricLineModel, MainWindow, QueuePageProps, SongModel,
    main_content::queue::QueuePageHandler,
    pages::PageHandler,
    test_utils::{TestSlintSmContext, main_window, state_manager_fixture},
    utils::save_queue,
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
        let ph = sm.get_player_handler();
        ph.add_to_queue(vec![song1, song2]).await;
    }

    let main_window = Box::leak(Box::new(main_window));
    let state_manager = Box::leak(Box::new(sm.clone()));
    let handler = QueuePageHandler::new(main_window, state_manager);
    handler.initialize();

    main_window
        .global::<AppCallbacks>()
        .invoke_play_queue_index(1);
    tokio::time::sleep(Duration::from_millis(50)).await;
    {
        let ph = sm.get_player_handler();
        assert_eq!(ph.get_current_idx(), 1);
    }

    main_window
        .global::<AppCallbacks>()
        .invoke_remove_from_queue(0);
    tokio::time::sleep(Duration::from_millis(50)).await;
    {
        let ph = sm.get_player_handler();
        assert_eq!(ph.get_queue().len(), 1);
    }

    main_window.global::<AppCallbacks>().invoke_clear_queue();
    tokio::time::sleep(Duration::from_millis(50)).await;
    {
        let ph = sm.get_player_handler();
        assert_eq!(ph.get_queue().len(), 0);
    }
}

#[rstest]
#[tokio::test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_queue_save_queue_as_playlist(state_manager_fixture: TestSlintSmContext) {
    let TestSlintSmContext { sm, .. } = state_manager_fixture;
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
        let ph = sm.get_player_handler();
        ph.add_to_queue(vec![song]).await;
    }

    save_queue(&sm, "My Saved Queue".into(), "My Description".into()).await;

    let db = sm.get_database();
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

#[rstest]
#[tokio::test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_queue_remove_currently_playing_song(
    main_window: MainWindow,
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
    let player_handler = sm.get_player_handler();
    player_handler.add_to_queue(vec![song1, song2]).await;

    let main_window = Box::leak(Box::new(main_window));
    let state_manager = Box::leak(Box::new(sm.clone()));
    let handler = QueuePageHandler::new(main_window, state_manager);
    handler.initialize();

    player_handler.play_index(0).await;
    tokio::time::sleep(Duration::from_millis(50)).await;
    assert_eq!(
        player_handler
            .current_song()
            .and_then(|s| s.song)
            .and_then(|s| s.id),
        Some("q_song_1".to_string())
    );

    main_window
        .global::<AppCallbacks>()
        .invoke_remove_from_queue(0);
    tokio::time::sleep(Duration::from_millis(100)).await;

    assert_eq!(player_handler.get_queue().len(), 1);
    assert_eq!(
        player_handler
            .current_song()
            .and_then(|s| s.song)
            .and_then(|s| s.id),
        Some("q_song_2".to_string())
    );
}

#[rstest]
#[tokio::test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_queue_page_handler_move_item_and_save_queue(
    main_window: MainWindow,
    state_manager_fixture: TestSlintSmContext,
) {
    let TestSlintSmContext { sm, .. } = state_manager_fixture;
    let song1 = Song {
        song: Some(InnerSong {
            id: Some("q_move_1".into()),
            title: Some("Move 1".into()),
            path: Some("/music/m1.mp3".into()),
            ..Default::default()
        }),
        ..Default::default()
    };
    let song2 = Song {
        song: Some(InnerSong {
            id: Some("q_move_2".into()),
            title: Some("Move 2".into()),
            path: Some("/music/m2.mp3".into()),
            ..Default::default()
        }),
        ..Default::default()
    };
    let player_handler = sm.get_player_handler();
    player_handler.add_to_queue(vec![song1, song2]).await;

    let main_window = Box::leak(Box::new(main_window));
    let state_manager = Box::leak(Box::new(sm.clone()));
    let handler = QueuePageHandler::new(main_window, state_manager);
    handler.initialize();

    main_window
        .global::<AppCallbacks>()
        .invoke_move_queue_item("0".into(), 1);
    tokio::time::sleep(Duration::from_millis(50)).await;

    let queue = player_handler.get_queue();
    assert_eq!(queue.len(), 2);
    assert_eq!(
        queue[0].song.as_ref().and_then(|s| s.id.as_deref()),
        Some("q_move_2")
    );

    main_window
        .global::<AppCallbacks>()
        .invoke_save_queue_as_playlist("Saved Q".into(), "Q Desc".into());
    tokio::time::sleep(Duration::from_millis(50)).await;
}

#[rstest]
#[tokio::test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_queue_page_handler_lyric_index_and_data_transfer(
    main_window: MainWindow,
    state_manager_fixture: TestSlintSmContext,
) {
    let TestSlintSmContext { sm, .. } = state_manager_fixture;
    let main_window = Box::leak(Box::new(main_window));
    let state_manager = Box::leak(Box::new(sm));
    let handler = QueuePageHandler::new(main_window, state_manager);
    handler.initialize();

    let callbacks = main_window.global::<AppCallbacks>();
    let transfer = callbacks.invoke_string_to_transfer("dragged_song".into());
    let extracted = callbacks.invoke_transfer_to_string(transfer);
    assert_eq!(extracted.as_str(), "dragged_song");

    let line1 = LyricLineModel {
        text: "Line 1".into(),
        time_ms: 1000,
    };
    let line2 = LyricLineModel {
        text: "Line 2".into(),
        time_ms: 3000,
    };
    let lines = slint::ModelRc::new(slint::VecModel::from(vec![line1, line2]));

    let active_idx_before = callbacks.invoke_get_active_lyric_index(lines.clone(), 500);
    let active_idx_first = callbacks.invoke_get_active_lyric_index(lines.clone(), 1500);
    let active_idx_second = callbacks.invoke_get_active_lyric_index(lines, 3500);

    assert_eq!(active_idx_before, -1);
    assert_eq!(active_idx_first, 0);
    assert_eq!(active_idx_second, 1);
}

#[rstest]
#[tokio::test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_queue_page_handler_context_menu(
    main_window: MainWindow,
    state_manager_fixture: TestSlintSmContext,
) {
    let TestSlintSmContext { sm, .. } = state_manager_fixture;
    let song = Song {
        song: Some(InnerSong {
            id: Some("ctx_q_1".into()),
            title: Some("Ctx Queue 1".into()),
            ..Default::default()
        }),
        ..Default::default()
    };
    let player_handler = sm.get_player_handler();
    player_handler.add_to_queue(vec![song]).await;

    let main_window = Box::leak(Box::new(main_window));
    let state_manager = Box::leak(Box::new(sm));
    let handler = QueuePageHandler::new(main_window, state_manager);
    handler.initialize();

    let song_model = SongModel {
        id: "ctx_q_1".into(),
        title: "Ctx Queue 1".into(),
        ..Default::default()
    };
    let song_models = slint::ModelRc::new(slint::VecModel::from(vec![song_model]));

    let menu_callbacks = main_window.global::<ContextMenuCallbacks>();
    let items = menu_callbacks.invoke_get_queue_menu_items(song_models.clone(), 0);
    assert!(items.row_count() > 0);

    menu_callbacks.invoke_queue_action(song_models.clone(), 0, "play_now".into());
    tokio::time::sleep(Duration::from_millis(50)).await;
    assert_eq!(player_handler.get_current_idx(), 0);

    menu_callbacks.invoke_queue_action(song_models.clone(), 0, "remove_from_queue".into());
    tokio::time::sleep(Duration::from_millis(50)).await;
    assert_eq!(player_handler.get_queue().len(), 0);
}

#[rstest]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_queue_page_handler_update_ui_lyrics_synced_and_empty(main_window: MainWindow) {
    let lyrics = Lyrics {
        is_synced: true,
        lines: vec![
            LyricLine {
                text: "Hello".to_string(),
                time_ms: 1000,
            },
            LyricLine {
                text: "World".to_string(),
                time_ms: 2000,
            },
        ],
    };

    QueuePageHandler::update_ui_lyrics(&main_window, Some(lyrics));

    let props = main_window.global::<QueuePageProps>();
    assert!(props.get_has_lyrics());
    assert!(props.get_is_synced());
    assert_eq!(props.get_lyrics_lines().row_count(), 2);
    assert!(!props.get_lyrics_loading());

    QueuePageHandler::update_ui_lyrics(&main_window, None);
    assert!(!props.get_has_lyrics());
    assert_eq!(props.get_lyrics_lines().row_count(), 0);
}

#[rstest]
#[tokio::test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_queue_page_handler_update_ui_blurred_cover_and_queue(
    main_window: MainWindow,
    state_manager_fixture: TestSlintSmContext,
) {
    let TestSlintSmContext { sm, .. } = state_manager_fixture;
    let song = Song {
        song: Some(InnerSong {
            id: Some("q_blur_1".into()),
            title: Some("Blur 1".into()),
            ..Default::default()
        }),
        ..Default::default()
    };

    QueuePageHandler::update_ui_queue(&main_window, &sm, vec![song]);
    QueuePageHandler::update_ui_blurred_cover(&main_window, &None);

    let props = main_window.global::<QueuePageProps>();
    assert_eq!(props.get_queue().row_count(), 1);
}

#[rstest]
#[tokio::test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_queue_page_handler_on_show_and_hide(
    main_window: MainWindow,
    state_manager_fixture: TestSlintSmContext,
) {
    let TestSlintSmContext { sm, .. } = state_manager_fixture;
    let main_window = Box::leak(Box::new(main_window));
    let state_manager = Box::leak(Box::new(sm));
    let handler = QueuePageHandler::new(main_window, state_manager);

    handler.on_show();
    assert!(handler.is_visible());

    handler.on_hide();
    assert!(!handler.is_visible());

    handler.clear_queue_ui();

    let props = main_window.global::<QueuePageProps>();
    assert_eq!(props.get_queue().row_count(), 0);
}
