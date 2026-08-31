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

use songs_proto::moosync::types::{InnerSong, Song};

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

#[tracing::instrument(level = "debug", skip_all)]
fn create_player_handler() -> PlayerHandler {
    let (ended_tx, _ended_rx) = tokio::sync::mpsc::unbounded_channel();
    PlayerHandler::new(ended_tx)
}

#[tokio::test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_add_and_current_song() {
    let mut ph = create_player_handler();
    assert!(ph.current_song().is_none());

    let song1 = create_mock_song("1", "Song One");
    ph.add_to_queue(vec![song1.clone()]);

    assert_eq!(
        ph.current_song()
            .unwrap()
            .song
            .as_ref()
            .unwrap()
            .id
            .as_ref()
            .unwrap(),
        "1"
    );

    let song2 = create_mock_song("2", "Song Two");
    ph.add_to_queue(vec![song2]);

    assert_eq!(
        ph.current_song()
            .unwrap()
            .song
            .as_ref()
            .unwrap()
            .id
            .as_ref()
            .unwrap(),
        "1"
    );
}

#[tokio::test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_play_now_and_queue_ops() {
    let mut ph = create_player_handler();
    let song1 = create_mock_song("1", "Song One");
    let song2 = create_mock_song("2", "Song Two");

    ph.add_to_queue(vec![song1]);
    ph.play_now(vec![song2]);

    assert_eq!(
        ph.current_song()
            .unwrap()
            .song
            .as_ref()
            .unwrap()
            .id
            .as_ref()
            .unwrap(),
        "2"
    );

    assert_eq!(ph.song_queue.len(), 2);
}

#[tokio::test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_play_now_multiple_songs() {
    let mut ph = create_player_handler();
    let song1 = create_mock_song("1", "Song One");
    let song2 = create_mock_song("2", "Song Two");
    let song3 = create_mock_song("3", "Song Three");

    ph.play_now(vec![song1, song2, song3]);

    assert_eq!(ph.song_queue.len(), 3);
    assert_eq!(ph.current_idx, 0);
    assert_eq!(
        ph.current_song()
            .unwrap()
            .song
            .as_ref()
            .unwrap()
            .id
            .as_deref(),
        Some("1")
    );
}

#[tokio::test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_play_now_empty() {
    let mut ph = create_player_handler();

    ph.play_now(vec![]);

    assert!(ph.current_song().is_none());
    assert!(ph.song_queue.is_empty());
}

#[tokio::test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_play_now_empty_with_existing_queue() {
    let mut ph = create_player_handler();
    let song = create_mock_song("1", "Song One");
    ph.add_to_queue(vec![song]);

    ph.play_now(vec![]);

    assert_eq!(ph.song_queue.len(), 1);
    assert_eq!(
        ph.current_song()
            .unwrap()
            .song
            .as_ref()
            .unwrap()
            .id
            .as_deref(),
        Some("1")
    );
}

#[tokio::test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_add_to_queue_multiple_songs() {
    let mut ph = create_player_handler();
    let song1 = create_mock_song("1", "Song One");
    let song2 = create_mock_song("2", "Song Two");
    let song3 = create_mock_song("3", "Song Three");
    ph.add_to_queue(vec![song1]);

    ph.add_to_queue(vec![song2, song3]);

    assert_eq!(ph.song_queue.len(), 3);
    assert_eq!(ph.current_idx, 0);
}

#[tokio::test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_add_to_queue_empty() {
    let mut ph = create_player_handler();

    ph.add_to_queue(vec![]);

    assert!(ph.current_song().is_none());
    assert!(ph.song_queue.is_empty());
}

#[tokio::test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_add_to_queue_empty_with_existing_queue() {
    let mut ph = create_player_handler();
    let song = create_mock_song("1", "Song One");
    ph.add_to_queue(vec![song]);

    ph.add_to_queue(vec![]);

    assert_eq!(ph.song_queue.len(), 1);
    assert_eq!(
        ph.current_song()
            .unwrap()
            .song
            .as_ref()
            .unwrap()
            .id
            .as_deref(),
        Some("1")
    );
}

#[tokio::test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_next_prev_and_repeat_cycle() {
    let mut ph = create_player_handler();
    let song1 = create_mock_song("1", "Song One");
    let song2 = create_mock_song("2", "Song Two");
    let song3 = create_mock_song("3", "Song Three");

    ph.add_to_queue(vec![song1]);
    ph.add_to_queue(vec![song2]);
    ph.add_to_queue(vec![song3]);

    assert_eq!(ph.current_idx, 0);

    ph.next();
    assert_eq!(ph.current_idx, 1);

    ph.next();
    assert_eq!(ph.current_idx, 2);

    ph.next();
    assert_eq!(ph.current_idx, 0);

    ph.current_idx = 2;
    ph.prev();
    assert_eq!(ph.current_idx, 1);
    ph.prev();
    assert_eq!(ph.current_idx, 0);
    ph.prev();
    assert_eq!(ph.current_idx, 0);
}

#[tokio::test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_repeat_modes_and_events() {
    let mut ph = create_player_handler();
    let song1 = create_mock_song("1", "Song One");
    let song2 = create_mock_song("2", "Song Two");

    ph.add_to_queue(vec![song1]);
    ph.add_to_queue(vec![song2]);

    let repeat_changed_fired = Arc::new(Mutex::new(Option::<RepeatMode>::None));
    let rc_clone = repeat_changed_fired.clone();
    ph.on_repeat_changed(move |mode| {
        let mut fired = rc_clone.lock().unwrap();
        *fired = Some(mode);
    });

    ph.repeat(RepeatMode::Once);
    assert_eq!(
        *repeat_changed_fired.lock().unwrap(),
        Some(RepeatMode::Once)
    );

    ph.on_song_ended();
    assert_eq!(ph.current_idx, 0);
    assert_eq!(ph.repeat_mode, RepeatMode::None);

    ph.repeat(RepeatMode::Infinite);
    ph.on_song_ended();
    assert_eq!(ph.current_idx, 0);
    assert_eq!(ph.repeat_mode, RepeatMode::Infinite);
}

#[tokio::test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_shuffle_and_reorder() {
    let mut ph = create_player_handler();
    for i in 1..=8 {
        ph.add_to_queue(vec![create_mock_song(
            &i.to_string(),
            &format!("Song {}", i),
        )]);
    }

    ph.current_idx = 3;
    let current_id = ph.current_song().unwrap().song.as_ref().unwrap().id.clone();

    ph.shuffle();
    assert_eq!(ph.current_idx, 3);
    assert_eq!(
        ph.current_song().unwrap().song.as_ref().unwrap().id,
        current_id
    );

    // move_queue_item
    ph.move_queue_item(0, 5);
    assert_eq!(ph.song_queue.len(), 8);

    // remove_from_queue
    ph.remove_from_queue(0);
    assert_eq!(ph.song_queue.len(), 7);

    // clear_queue (first click keeps current song, second click clears all)
    ph.clear_queue();
    assert_eq!(ph.song_queue.len(), 1);
    ph.clear_queue();
    assert!(ph.song_queue.is_empty());
}

#[tokio::test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_empty_and_single_queue_boundaries() {
    let mut ph = create_player_handler();

    // Operations on empty queue
    assert!(ph.current_song().is_none());
    ph.next();
    assert_eq!(ph.current_idx, 0);
    ph.prev();
    assert_eq!(ph.current_idx, 0);
    ph.shuffle();
    assert!(ph.song_queue.is_empty());

    // Single item queue
    let single = create_mock_song("single", "Single Song");
    ph.add_to_queue(vec![single]);
    assert_eq!(ph.song_queue.len(), 1);
    ph.next();
    assert_eq!(ph.current_idx, 0);
    ph.prev();
    assert_eq!(ph.current_idx, 0);
    ph.shuffle();
    assert_eq!(ph.song_queue.len(), 1);
}

#[tokio::test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_play_next_empty_queue() {
    let mut ph = create_player_handler();
    let song = create_mock_song("1", "Song One");

    ph.play_next(vec![song]);

    assert_eq!(ph.song_queue.len(), 1);
    assert_eq!(ph.current_idx, 0);
    assert_eq!(
        ph.current_song()
            .unwrap()
            .song
            .as_ref()
            .unwrap()
            .id
            .as_deref(),
        Some("1")
    );
}

#[tokio::test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_play_next_with_existing_queue() {
    let mut ph = create_player_handler();
    let song1 = create_mock_song("1", "Song One");
    let song2 = create_mock_song("2", "Song Two");
    let song_next = create_mock_song("next", "Next Song");
    ph.add_to_queue(vec![song1, song2]);

    ph.play_next(vec![song_next]);

    assert_eq!(ph.song_queue.len(), 3);
    assert_eq!(
        ph.song_queue[1].song.as_ref().unwrap().id.as_deref(),
        Some("next")
    );
    assert_eq!(
        ph.song_queue[2].song.as_ref().unwrap().id.as_deref(),
        Some("2")
    );
}

#[tokio::test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_clear_and_play() {
    let mut ph = create_player_handler();
    let old1 = create_mock_song("old1", "Old 1");
    let old2 = create_mock_song("old2", "Old 2");
    ph.add_to_queue(vec![old1, old2]);
    ph.next();
    let new_song = create_mock_song("new1", "New 1");

    ph.clear_and_play(vec![new_song]);

    assert_eq!(ph.song_queue.len(), 1);
    assert_eq!(ph.current_idx, 0);
    assert_eq!(
        ph.current_song()
            .unwrap()
            .song
            .as_ref()
            .unwrap()
            .id
            .as_deref(),
        Some("new1")
    );
}
