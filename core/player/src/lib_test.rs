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

use std::sync::{Arc, Mutex};

use assertables::{assert_is_empty, assert_len_eq_x, assert_none, assert_some_eq_x};
use rstest::{fixture, rstest};
use songs_proto::moosync::types::{InnerSong, Song};
use tracing_test::traced_test;

use crate::{PlayerHandler, RepeatMode};

#[tracing::instrument(level = "debug", skip_all)]
fn create_mock_song(id: &str, title: &str) -> Song {
    Song {
        song: Some(InnerSong {
            id: Some(id.to_string()),
            title: Some(title.to_string()),
            playback_url: Some(format!("https://example.com/{}", id)),
            ..Default::default()
        }),
        ..Default::default()
    }
}

#[fixture]
#[tracing::instrument(level = "debug", skip_all)]
fn player_handler() -> PlayerHandler {
    let (ended_tx, _ended_rx) = tokio::sync::mpsc::unbounded_channel();
    PlayerHandler::new(ended_tx)
}

#[rstest]
#[tokio::test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_current_song_when_empty(player_handler: PlayerHandler) {
    let current = player_handler.current_song();

    assert_none!(current);
}

#[rstest]
#[tokio::test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_add_to_queue_sets_initial_current_song(mut player_handler: PlayerHandler) {
    let song1 = create_mock_song("1", "Song One");

    player_handler.add_to_queue(vec![song1]);
    let current = player_handler.current_song();

    assert_some_eq_x!(
        current
            .as_ref()
            .and_then(|s| s.song.as_ref())
            .and_then(|s| s.id.as_deref()),
        "1"
    );
}

#[rstest]
#[tokio::test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_add_to_queue_preserves_current_song(mut player_handler: PlayerHandler) {
    let song1 = create_mock_song("1", "Song One");
    let song2 = create_mock_song("2", "Song Two");
    player_handler.add_to_queue(vec![song1]);

    player_handler.add_to_queue(vec![song2]);
    let current = player_handler.current_song();

    assert_some_eq_x!(
        current
            .as_ref()
            .and_then(|s| s.song.as_ref())
            .and_then(|s| s.id.as_deref()),
        "1"
    );
}

#[rstest]
#[tokio::test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_play_now_overwrites_current_song(mut player_handler: PlayerHandler) {
    let song1 = create_mock_song("1", "Song One");
    let song2 = create_mock_song("2", "Song Two");
    player_handler.add_to_queue(vec![song1]);

    player_handler.play_now(vec![song2]);
    let current = player_handler.current_song();

    assert_some_eq_x!(
        current
            .as_ref()
            .and_then(|s| s.song.as_ref())
            .and_then(|s| s.id.as_deref()),
        "2"
    );
    assert_len_eq_x!(&player_handler.song_queue, 2);
}

#[rstest]
#[tokio::test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_play_now_multiple_songs(mut player_handler: PlayerHandler) {
    let song1 = create_mock_song("1", "Song One");
    let song2 = create_mock_song("2", "Song Two");
    let song3 = create_mock_song("3", "Song Three");

    player_handler.play_now(vec![song1, song2, song3]);
    let current = player_handler.current_song();

    assert_len_eq_x!(&player_handler.song_queue, 3);
    assert_eq!(player_handler.current_idx, 0);
    assert_some_eq_x!(
        current
            .as_ref()
            .and_then(|s| s.song.as_ref())
            .and_then(|s| s.id.as_deref()),
        "1"
    );
}

#[rstest]
#[tokio::test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_play_now_empty(mut player_handler: PlayerHandler) {
    player_handler.play_now(vec![]);
    let current = player_handler.current_song();

    assert_none!(current);
    assert_is_empty!(&player_handler.song_queue);
}

#[rstest]
#[tokio::test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_play_now_empty_with_existing_queue(mut player_handler: PlayerHandler) {
    let song = create_mock_song("1", "Song One");
    player_handler.add_to_queue(vec![song]);

    player_handler.play_now(vec![]);
    let current = player_handler.current_song();

    assert_len_eq_x!(&player_handler.song_queue, 1);
    assert_some_eq_x!(
        current
            .as_ref()
            .and_then(|s| s.song.as_ref())
            .and_then(|s| s.id.as_deref()),
        "1"
    );
}

#[rstest]
#[tokio::test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_add_to_queue_multiple_songs(mut player_handler: PlayerHandler) {
    let song1 = create_mock_song("1", "Song One");
    let song2 = create_mock_song("2", "Song Two");
    let song3 = create_mock_song("3", "Song Three");
    player_handler.add_to_queue(vec![song1]);

    player_handler.add_to_queue(vec![song2, song3]);

    assert_len_eq_x!(&player_handler.song_queue, 3);
    assert_eq!(player_handler.current_idx, 0);
}

#[rstest]
#[tokio::test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_add_to_queue_empty(mut player_handler: PlayerHandler) {
    player_handler.add_to_queue(vec![]);
    let current = player_handler.current_song();

    assert_none!(current);
    assert_is_empty!(&player_handler.song_queue);
}

#[rstest]
#[tokio::test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_add_to_queue_empty_with_existing_queue(mut player_handler: PlayerHandler) {
    let song = create_mock_song("1", "Song One");
    player_handler.add_to_queue(vec![song]);

    player_handler.add_to_queue(vec![]);
    let current = player_handler.current_song();

    assert_len_eq_x!(&player_handler.song_queue, 1);
    assert_some_eq_x!(
        current
            .as_ref()
            .and_then(|s| s.song.as_ref())
            .and_then(|s| s.id.as_deref()),
        "1"
    );
}

#[rstest]
#[tokio::test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_player_next_advances_index(mut player_handler: PlayerHandler) {
    let song1 = create_mock_song("1", "Song One");
    let song2 = create_mock_song("2", "Song Two");
    let song3 = create_mock_song("3", "Song Three");
    player_handler.add_to_queue(vec![song1, song2, song3]);

    player_handler.next();

    assert_eq!(player_handler.current_idx, 1);
}

#[rstest]
#[tokio::test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_player_next_wraps_around(mut player_handler: PlayerHandler) {
    let song1 = create_mock_song("1", "Song One");
    let song2 = create_mock_song("2", "Song Two");
    player_handler.add_to_queue(vec![song1, song2]);
    player_handler.current_idx = 1;

    player_handler.next();

    assert_eq!(player_handler.current_idx, 0);
}

#[rstest]
#[tokio::test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_player_prev_decrements_index(mut player_handler: PlayerHandler) {
    let song1 = create_mock_song("1", "Song One");
    let song2 = create_mock_song("2", "Song Two");
    player_handler.add_to_queue(vec![song1, song2]);
    player_handler.current_idx = 1;

    player_handler.prev();

    assert_eq!(player_handler.current_idx, 0);
}

#[rstest]
#[tokio::test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_player_prev_at_start_stays_zero(mut player_handler: PlayerHandler) {
    let song1 = create_mock_song("1", "Song One");
    player_handler.add_to_queue(vec![song1]);

    player_handler.prev();

    assert_eq!(player_handler.current_idx, 0);
}

#[rstest]
#[tokio::test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_player_repeat_once_resets_on_ended(mut player_handler: PlayerHandler) {
    let song1 = create_mock_song("1", "Song One");
    let song2 = create_mock_song("2", "Song Two");
    player_handler.add_to_queue(vec![song1, song2]);
    player_handler.repeat(RepeatMode::Once);

    player_handler.on_song_ended();

    assert_eq!(player_handler.current_idx, 0);
    assert_eq!(player_handler.repeat_mode, RepeatMode::None);
}

#[rstest]
#[tokio::test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_player_repeat_infinite_stays_on_ended(mut player_handler: PlayerHandler) {
    let song1 = create_mock_song("1", "Song One");
    let song2 = create_mock_song("2", "Song Two");
    player_handler.add_to_queue(vec![song1, song2]);
    player_handler.repeat(RepeatMode::Infinite);

    player_handler.on_song_ended();

    assert_eq!(player_handler.current_idx, 0);
    assert_eq!(player_handler.repeat_mode, RepeatMode::Infinite);
}

#[rstest]
#[tokio::test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_player_repeat_event_fires(mut player_handler: PlayerHandler) {
    let repeat_changed_fired = Arc::new(Mutex::new(Option::<RepeatMode>::None));
    let rc_clone = repeat_changed_fired.clone();
    player_handler.on_repeat_changed(move |mode| {
        let mut fired = rc_clone.lock().unwrap();
        *fired = Some(mode);
    });

    player_handler.repeat(RepeatMode::Once);

    assert_some_eq_x!(&*repeat_changed_fired.lock().unwrap(), &RepeatMode::Once);
}

#[rstest]
#[tokio::test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_player_shuffle_preserves_current_song(mut player_handler: PlayerHandler) {
    for i in 1..=8 {
        player_handler.add_to_queue(vec![create_mock_song(
            &i.to_string(),
            &format!("Song {}", i),
        )]);
    }
    player_handler.current_idx = 3;
    let current_id = player_handler
        .current_song()
        .unwrap()
        .song
        .as_ref()
        .unwrap()
        .id
        .clone();

    player_handler.shuffle();

    assert_eq!(player_handler.current_idx, 3);
    assert_eq!(
        player_handler
            .current_song()
            .unwrap()
            .song
            .as_ref()
            .unwrap()
            .id,
        current_id
    );
}

#[rstest]
#[tokio::test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_player_move_queue_item(mut player_handler: PlayerHandler) {
    for i in 1..=4 {
        player_handler.add_to_queue(vec![create_mock_song(
            &i.to_string(),
            &format!("Song {}", i),
        )]);
    }

    player_handler.move_queue_item(0, 3);

    assert_len_eq_x!(&player_handler.song_queue, 4);
    assert_some_eq_x!(
        player_handler.song_queue[3]
            .song
            .as_ref()
            .and_then(|s| s.id.as_deref()),
        "1"
    );
}

#[rstest]
#[tokio::test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_player_remove_from_queue(mut player_handler: PlayerHandler) {
    let song1 = create_mock_song("1", "Song One");
    let song2 = create_mock_song("2", "Song Two");
    player_handler.add_to_queue(vec![song1, song2]);

    player_handler.remove_from_queue(0);

    assert_len_eq_x!(&player_handler.song_queue, 1);
    assert_some_eq_x!(
        player_handler.song_queue[0]
            .song
            .as_ref()
            .and_then(|s| s.id.as_deref()),
        "2"
    );
}

#[rstest]
#[tokio::test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_player_clear_queue(mut player_handler: PlayerHandler) {
    let song1 = create_mock_song("1", "Song One");
    let song2 = create_mock_song("2", "Song Two");
    player_handler.add_to_queue(vec![song1, song2]);

    player_handler.clear_queue();

    assert_len_eq_x!(&player_handler.song_queue, 1);
}

#[rstest]
#[tokio::test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_player_empty_queue_navigation(mut player_handler: PlayerHandler) {
    player_handler.next();
    player_handler.prev();
    player_handler.shuffle();

    assert_none!(player_handler.current_song());
    assert_is_empty!(&player_handler.song_queue);
}

#[rstest]
#[tokio::test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_player_single_song_queue_navigation(mut player_handler: PlayerHandler) {
    let single = create_mock_song("single", "Single Song");
    player_handler.add_to_queue(vec![single]);

    player_handler.next();
    player_handler.prev();
    player_handler.shuffle();

    assert_len_eq_x!(&player_handler.song_queue, 1);
    assert_eq!(player_handler.current_idx, 0);
}

#[rstest]
#[tokio::test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_play_next_empty_queue(mut player_handler: PlayerHandler) {
    let song = create_mock_song("1", "Song One");

    player_handler.play_next(vec![song]);
    let current = player_handler.current_song();

    assert_len_eq_x!(&player_handler.song_queue, 1);
    assert_eq!(player_handler.current_idx, 0);
    assert_some_eq_x!(
        current
            .as_ref()
            .and_then(|s| s.song.as_ref())
            .and_then(|s| s.id.as_deref()),
        "1"
    );
}

#[rstest]
#[tokio::test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_play_next_with_existing_queue(mut player_handler: PlayerHandler) {
    let song1 = create_mock_song("1", "Song One");
    let song2 = create_mock_song("2", "Song Two");
    let song_next = create_mock_song("next", "Next Song");
    player_handler.add_to_queue(vec![song1, song2]);

    player_handler.play_next(vec![song_next]);

    assert_len_eq_x!(&player_handler.song_queue, 3);
    assert_some_eq_x!(
        player_handler.song_queue[1]
            .song
            .as_ref()
            .and_then(|s| s.id.as_deref()),
        "next"
    );
    assert_some_eq_x!(
        player_handler.song_queue[2]
            .song
            .as_ref()
            .and_then(|s| s.id.as_deref()),
        "2"
    );
}

#[rstest]
#[tokio::test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_clear_and_play(mut player_handler: PlayerHandler) {
    let old1 = create_mock_song("old1", "Old 1");
    let old2 = create_mock_song("old2", "Old 2");
    player_handler.add_to_queue(vec![old1, old2]);
    player_handler.next();
    let new_song = create_mock_song("new1", "New 1");

    player_handler.clear_and_play(vec![new_song]);
    let current = player_handler.current_song();

    assert_len_eq_x!(&player_handler.song_queue, 1);
    assert_eq!(player_handler.current_idx, 0);
    assert_some_eq_x!(
        current
            .as_ref()
            .and_then(|s| s.song.as_ref())
            .and_then(|s| s.id.as_deref()),
        "new1"
    );
}
