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

use std::{
    sync::{Arc, Mutex},
    time::Duration,
};

use assertables::{assert_is_empty, assert_len_eq_x, assert_none, assert_some_eq_x};
use extensions_proto::moosync::types::PlayerState;
use player_proto::moosync::types::PlayerData;
use rstest::{fixture, rstest};
use songs_proto::moosync::types::{InnerSong, Song};
use tracing_test::traced_test;
use types::plugin::{Plugin, PluginContext};

use crate::{
    PlayerHandler, RepeatMode,
    context::{AudioPlayerContext, DummyAudioPlayerContext, DummyPersistContext, PersistContext},
    error::PlayerError,
    source::ValidSrc,
};

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
fn create_unplayable_mock_song(id: &str, title: &str) -> Song {
    Song {
        song: Some(InnerSong {
            id: Some(id.to_string()),
            title: Some(title.to_string()),
            path: Some("/nonexistent/file.mp3".to_string()),
            ..Default::default()
        }),
        ..Default::default()
    }
}

#[fixture]
#[tracing::instrument(level = "debug", skip_all)]
fn player_handler() -> PlayerHandler {
    let (ended_tx, _ended_rx) = tokio::sync::mpsc::unbounded_channel();
    PlayerHandler::new_with_contexts(
        ended_tx,
        vec![Box::new(DummyAudioPlayerContext::new())],
        Box::new(DummyPersistContext {}),
    )
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
async fn test_add_to_queue_sets_initial_current_song(player_handler: PlayerHandler) {
    let song1 = create_mock_song("1", "Song One");

    player_handler.add_to_queue(vec![song1]).await;
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
async fn test_add_to_queue_preserves_current_song(player_handler: PlayerHandler) {
    let song1 = create_mock_song("1", "Song One");
    let song2 = create_mock_song("2", "Song Two");
    player_handler.add_to_queue(vec![song1]).await;

    player_handler.add_to_queue(vec![song2]).await;
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
async fn test_play_now_overwrites_current_song(player_handler: PlayerHandler) {
    let song1 = create_mock_song("1", "Song One");
    let song2 = create_mock_song("2", "Song Two");
    player_handler.add_to_queue(vec![song1]).await;

    player_handler.play_now(vec![song2]).await;
    let current = player_handler.current_song();

    assert_some_eq_x!(
        current
            .as_ref()
            .and_then(|s| s.song.as_ref())
            .and_then(|s| s.id.as_deref()),
        "2"
    );
    assert_len_eq_x!(&player_handler.get_queue(), 2);
}

#[rstest]
#[tokio::test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_play_now_multiple_songs(player_handler: PlayerHandler) {
    let song1 = create_mock_song("1", "Song One");
    let song2 = create_mock_song("2", "Song Two");
    let song3 = create_mock_song("3", "Song Three");

    player_handler.play_now(vec![song1, song2, song3]).await;
    let current = player_handler.current_song();

    assert_len_eq_x!(&player_handler.get_queue(), 3);
    assert_eq!(player_handler.get_current_idx(), 0);
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
async fn test_play_now_empty(player_handler: PlayerHandler) {
    player_handler.play_now(vec![]).await;
    let current = player_handler.current_song();

    assert_none!(current);
    assert_is_empty!(&player_handler.get_queue());
}

#[rstest]
#[tokio::test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_play_now_empty_with_existing_queue(player_handler: PlayerHandler) {
    let song = create_mock_song("1", "Song One");
    player_handler.add_to_queue(vec![song]).await;

    player_handler.play_now(vec![]).await;
    let current = player_handler.current_song();

    assert_len_eq_x!(&player_handler.get_queue(), 1);
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
async fn test_add_to_queue_multiple_songs(player_handler: PlayerHandler) {
    let song1 = create_mock_song("1", "Song One");
    let song2 = create_mock_song("2", "Song Two");
    let song3 = create_mock_song("3", "Song Three");
    player_handler.add_to_queue(vec![song1]).await;

    player_handler.add_to_queue(vec![song2, song3]).await;

    assert_len_eq_x!(&player_handler.get_queue(), 3);
    assert_eq!(player_handler.get_current_idx(), 0);
}

#[rstest]
#[tokio::test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_add_to_queue_empty(player_handler: PlayerHandler) {
    player_handler.add_to_queue(vec![]).await;
    let current = player_handler.current_song();

    assert_none!(current);
    assert_is_empty!(&player_handler.get_queue());
}

#[rstest]
#[tokio::test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_add_to_queue_empty_with_existing_queue(player_handler: PlayerHandler) {
    let song = create_mock_song("1", "Song One");
    player_handler.add_to_queue(vec![song]).await;

    player_handler.add_to_queue(vec![]).await;
    let current = player_handler.current_song();

    assert_len_eq_x!(&player_handler.get_queue(), 1);
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
async fn test_player_next_advances_index(player_handler: PlayerHandler) {
    let song1 = create_mock_song("1", "Song One");
    let song2 = create_mock_song("2", "Song Two");
    let song3 = create_mock_song("3", "Song Three");
    player_handler.add_to_queue(vec![song1, song2, song3]).await;

    player_handler.next().await;

    assert_eq!(player_handler.get_current_idx(), 1);
}

#[rstest]
#[tokio::test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_player_next_wraps_around(player_handler: PlayerHandler) {
    let song1 = create_mock_song("1", "Song One");
    let song2 = create_mock_song("2", "Song Two");
    player_handler.add_to_queue(vec![song1, song2]).await;
    player_handler.player_data.write().unwrap().current_idx = 1;

    player_handler.next().await;

    assert_eq!(player_handler.get_current_idx(), 0);
}

#[rstest]
#[tokio::test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_player_prev_decrements_index(player_handler: PlayerHandler) {
    let song1 = create_mock_song("1", "Song One");
    let song2 = create_mock_song("2", "Song Two");
    player_handler.add_to_queue(vec![song1, song2]).await;
    player_handler.player_data.write().unwrap().current_idx = 1;

    player_handler.prev().await;

    assert_eq!(player_handler.get_current_idx(), 0);
}

#[rstest]
#[tokio::test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_player_prev_at_start_stays_zero(player_handler: PlayerHandler) {
    let song1 = create_mock_song("1", "Song One");
    player_handler.add_to_queue(vec![song1]).await;

    player_handler.prev().await;

    assert_eq!(player_handler.get_current_idx(), 0);
}

#[rstest]
#[tokio::test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_player_repeat_once_resets_on_ended(player_handler: PlayerHandler) {
    let song1 = create_mock_song("1", "Song One");
    let song2 = create_mock_song("2", "Song Two");
    player_handler.add_to_queue(vec![song1, song2]).await;
    player_handler.repeat(RepeatMode::RepeatOnce);

    player_handler.on_song_ended().await;

    assert_eq!(player_handler.get_current_idx(), 0);
    assert_eq!(player_handler.get_repeat_mode(), RepeatMode::RepeatNone);
}

#[rstest]
#[tokio::test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_player_repeat_infinite_stays_on_ended(player_handler: PlayerHandler) {
    let song1 = create_mock_song("1", "Song One");
    let song2 = create_mock_song("2", "Song Two");
    player_handler.add_to_queue(vec![song1, song2]).await;
    player_handler.repeat(RepeatMode::RepeatInfinite);

    player_handler.on_song_ended().await;

    assert_eq!(player_handler.get_current_idx(), 0);
    assert_eq!(player_handler.get_repeat_mode(), RepeatMode::RepeatInfinite);
}

#[rstest]
#[tokio::test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_player_repeat_event_fires(player_handler: PlayerHandler) {
    let repeat_changed_fired = Arc::new(Mutex::new(Option::<RepeatMode>::None));
    let rc_clone = repeat_changed_fired.clone();
    player_handler.on_repeat_changed.listen(move |mode| {
        let mut fired = rc_clone.lock().unwrap();
        *fired = Some(mode);
    });

    player_handler.repeat(RepeatMode::RepeatOnce);

    assert_some_eq_x!(
        &*repeat_changed_fired.lock().unwrap(),
        &RepeatMode::RepeatOnce
    );
}

#[rstest]
#[tokio::test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_player_shuffle_preserves_current_song(player_handler: PlayerHandler) {
    for i in 1..=8 {
        player_handler
            .add_to_queue(vec![create_mock_song(
                &i.to_string(),
                &format!("Song {}", i),
            )])
            .await;
    }
    player_handler.player_data.write().unwrap().current_idx = 3;
    let current_id = player_handler
        .current_song()
        .unwrap()
        .song
        .as_ref()
        .unwrap()
        .id
        .clone();

    player_handler.shuffle();

    assert_eq!(player_handler.get_current_idx(), 3);
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
async fn test_player_move_queue_item(player_handler: PlayerHandler) {
    for i in 1..=4 {
        player_handler
            .add_to_queue(vec![create_mock_song(
                &i.to_string(),
                &format!("Song {}", i),
            )])
            .await;
    }

    player_handler.move_queue_item(0, 3);

    assert_len_eq_x!(&player_handler.get_queue(), 4);
    assert_some_eq_x!(
        player_handler.get_queue()[3]
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
async fn test_player_remove_from_queue(player_handler: PlayerHandler) {
    let song1 = create_mock_song("1", "Song One");
    let song2 = create_mock_song("2", "Song Two");
    player_handler.add_to_queue(vec![song1, song2]).await;

    player_handler.remove_from_queue(0).await;

    assert_len_eq_x!(&player_handler.get_queue(), 1);
    assert_some_eq_x!(
        player_handler.get_queue()[0]
            .song
            .as_ref()
            .and_then(|s| s.id.as_deref()),
        "2"
    );
    assert_some_eq_x!(
        player_handler
            .current_song()
            .as_ref()
            .and_then(|s| s.song.as_ref())
            .and_then(|s| s.id.as_deref()),
        "2"
    );
}

#[rstest]
#[tokio::test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_player_remove_currently_playing_song_updates_playback(player_handler: PlayerHandler) {
    let song1 = create_mock_song("1", "Song One");
    let song2 = create_mock_song("2", "Song Two");
    let song3 = create_mock_song("3", "Song Three");
    player_handler.add_to_queue(vec![song1, song2, song3]).await;
    player_handler.play_index(1).await;

    player_handler.remove_from_queue(1).await;

    assert_len_eq_x!(&player_handler.get_queue(), 2);
    assert_eq!(player_handler.get_current_idx(), 1);
    assert_some_eq_x!(
        player_handler
            .current_song()
            .as_ref()
            .and_then(|s| s.song.as_ref())
            .and_then(|s| s.id.as_deref()),
        "3"
    );
    assert_eq!(
        player_handler.get_player_state(),
        PlayerState::Playing as i32
    );
}

#[rstest]
#[tokio::test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_player_remove_currently_playing_last_song(player_handler: PlayerHandler) {
    let song1 = create_mock_song("1", "Song One");
    let song2 = create_mock_song("2", "Song Two");
    player_handler.add_to_queue(vec![song1, song2]).await;
    player_handler.play_index(1).await;

    player_handler.remove_from_queue(1).await;

    assert_len_eq_x!(&player_handler.get_queue(), 1);
    assert_eq!(player_handler.get_current_idx(), 0);
    assert_some_eq_x!(
        player_handler
            .current_song()
            .as_ref()
            .and_then(|s| s.song.as_ref())
            .and_then(|s| s.id.as_deref()),
        "1"
    );
    assert_eq!(
        player_handler.get_player_state(),
        PlayerState::Playing as i32
    );
}

#[rstest]
#[tokio::test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_player_remove_only_song(player_handler: PlayerHandler) {
    let song1 = create_mock_song("1", "Song One");
    player_handler.add_to_queue(vec![song1]).await;
    player_handler.play_index(0).await;

    player_handler.remove_from_queue(0).await;

    assert_is_empty!(&player_handler.get_queue());
    assert_none!(player_handler.current_song());
    assert_eq!(
        player_handler.get_player_state(),
        PlayerState::Stopped as i32
    );
}

#[rstest]
#[tokio::test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_player_remove_non_current_song(player_handler: PlayerHandler) {
    let song1 = create_mock_song("1", "Song One");
    let song2 = create_mock_song("2", "Song Two");
    let song3 = create_mock_song("3", "Song Three");
    player_handler.add_to_queue(vec![song1, song2, song3]).await;
    player_handler.play_index(1).await;

    player_handler.remove_from_queue(0).await;

    assert_len_eq_x!(&player_handler.get_queue(), 2);
    assert_eq!(player_handler.get_current_idx(), 0);
    assert_some_eq_x!(
        player_handler
            .current_song()
            .as_ref()
            .and_then(|s| s.song.as_ref())
            .and_then(|s| s.id.as_deref()),
        "2"
    );
    assert_eq!(
        player_handler.get_player_state(),
        PlayerState::Playing as i32
    );
}

#[rstest]
#[tokio::test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_player_song_failure_updates_current_song(player_handler: PlayerHandler) {
    let song1 = create_mock_song("1", "Song One");
    let song2 = create_unplayable_mock_song("2", "Song Two");
    player_handler.add_to_queue(vec![song1, song2]).await;
    player_handler.play_index(0).await;

    let received_song = Arc::new(Mutex::new(None));
    let received_song_clone = received_song.clone();
    player_handler.on_song_changed.listen(move |song| {
        *received_song_clone.lock().unwrap() = song.clone();
    });

    player_handler.play_index(1).await;

    assert_eq!(player_handler.get_current_idx(), 1);
    assert_some_eq_x!(
        player_handler
            .current_song()
            .as_ref()
            .and_then(|s| s.song.as_ref())
            .and_then(|s| s.id.as_deref()),
        "2"
    );
    assert_eq!(
        received_song
            .lock()
            .unwrap()
            .as_ref()
            .and_then(|s| s.song.as_ref())
            .and_then(|s| s.id.as_deref()),
        Some("2")
    );
    assert_eq!(
        player_handler.get_player_state(),
        PlayerState::Stopped as i32
    );
}

#[rstest]
#[tokio::test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_player_clear_queue(player_handler: PlayerHandler) {
    let song1 = create_mock_song("1", "Song One");
    let song2 = create_mock_song("2", "Song Two");
    player_handler.add_to_queue(vec![song1, song2]).await;

    player_handler.clear_queue().await;

    assert_len_eq_x!(&player_handler.get_queue(), 1);
}

#[rstest]
#[tokio::test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_player_empty_queue_navigation(player_handler: PlayerHandler) {
    player_handler.next().await;
    player_handler.prev().await;
    player_handler.shuffle();

    assert_none!(player_handler.current_song());
    assert_is_empty!(&player_handler.get_queue());
}

#[rstest]
#[tokio::test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_player_single_song_queue_navigation(player_handler: PlayerHandler) {
    let single = create_mock_song("single", "Single Song");
    player_handler.add_to_queue(vec![single]).await;

    player_handler.next().await;
    player_handler.prev().await;
    player_handler.shuffle();

    assert_len_eq_x!(&player_handler.get_queue(), 1);
    assert_eq!(player_handler.get_current_idx(), 0);
}

#[rstest]
#[tokio::test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_play_next_empty_queue(player_handler: PlayerHandler) {
    let song = create_mock_song("1", "Song One");

    player_handler.play_next(vec![song]).await;
    let current = player_handler.current_song();

    assert_len_eq_x!(&player_handler.get_queue(), 1);
    assert_eq!(player_handler.get_current_idx(), 0);
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
async fn test_play_next_with_existing_queue(player_handler: PlayerHandler) {
    let song1 = create_mock_song("1", "Song One");
    let song2 = create_mock_song("2", "Song Two");
    let song_next = create_mock_song("next", "Next Song");
    player_handler.add_to_queue(vec![song1, song2]).await;

    player_handler.play_next(vec![song_next]).await;

    assert_len_eq_x!(&player_handler.get_queue(), 3);
    assert_some_eq_x!(
        player_handler.get_queue()[1]
            .song
            .as_ref()
            .and_then(|s| s.id.as_deref()),
        "next"
    );
    assert_some_eq_x!(
        player_handler.get_queue()[2]
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
async fn test_clear_and_play(player_handler: PlayerHandler) {
    let old1 = create_mock_song("old1", "Old 1");
    let old2 = create_mock_song("old2", "Old 2");
    player_handler.add_to_queue(vec![old1, old2]).await;
    player_handler.next().await;
    let new_song = create_mock_song("new1", "New 1");

    player_handler.clear_and_play(vec![new_song]).await;
    let current = player_handler.current_song();

    assert_len_eq_x!(&player_handler.get_queue(), 1);
    assert_eq!(player_handler.get_current_idx(), 0);
    assert_some_eq_x!(
        current
            .as_ref()
            .and_then(|s| s.song.as_ref())
            .and_then(|s| s.id.as_deref()),
        "new1"
    );
}

struct MockPersistContext {
    data: PlayerData,
}

impl PersistContext for MockPersistContext {
    #[tracing::instrument(level = "debug", skip_all)]
    fn persist(&self, _: &PlayerData) -> Result<(), PlayerError> { Ok(()) }

    #[tracing::instrument(level = "debug", skip_all)]
    fn load(&self) -> Result<PlayerData, PlayerError> { Ok(self.data.clone()) }
}

struct TrackingPersistContext {
    persisted: Arc<Mutex<Option<PlayerData>>>,
}

impl PersistContext for TrackingPersistContext {
    #[tracing::instrument(level = "debug", skip_all)]
    fn persist(&self, data: &PlayerData) -> Result<(), PlayerError> {
        *self.persisted.lock().unwrap() = Some(data.clone());
        Ok(())
    }

    #[tracing::instrument(level = "debug", skip_all)]
    fn load(&self) -> Result<PlayerData, PlayerError> { Ok(PlayerData::default()) }
}

#[tokio::test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_restore_player_data_sets_current_song() {
    let song1 = create_mock_song("1", "Song One");
    let song2 = create_mock_song("2", "Song Two");
    let player_data = PlayerData {
        song_queue: vec![song1, song2],
        current_idx: 1,
        repeat_mode: RepeatMode::RepeatInfinite.into(),
    };
    let (ended_tx, _ended_rx) = tokio::sync::mpsc::unbounded_channel();
    let player_handler = PlayerHandler::new_with_contexts(
        ended_tx,
        vec![Box::new(DummyAudioPlayerContext::new())],
        Box::new(MockPersistContext { data: player_data }),
    );

    let current = player_handler.current_song();

    assert_eq!(player_handler.get_current_idx(), 1);
    assert_len_eq_x!(&player_handler.get_queue(), 2);
    assert_eq!(player_handler.get_repeat_mode(), RepeatMode::RepeatInfinite);
    assert_some_eq_x!(
        current
            .as_ref()
            .and_then(|s| s.song.as_ref())
            .and_then(|s| s.id.as_deref()),
        "2"
    );
}

#[tokio::test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_restore_player_data_play_loads_and_plays() {
    let song1 = create_mock_song("1", "Song One");
    let player_data = PlayerData {
        song_queue: vec![song1],
        current_idx: 0,
        repeat_mode: RepeatMode::RepeatNone.into(),
    };
    let (ended_tx, _ended_rx) = tokio::sync::mpsc::unbounded_channel();
    let player_handler = PlayerHandler::new_with_contexts(
        ended_tx,
        vec![Box::new(DummyAudioPlayerContext::new())],
        Box::new(MockPersistContext { data: player_data }),
    );

    let play_result = player_handler.play().await;

    assert!(play_result.is_ok());
    assert_eq!(
        player_handler.get_player_state(),
        PlayerState::Playing as i32
    );
}

#[tokio::test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_restore_player_data_out_of_bounds_idx_resets_to_zero() {
    let song1 = create_mock_song("1", "Song One");
    let player_data = PlayerData {
        song_queue: vec![song1],
        current_idx: 5,
        repeat_mode: RepeatMode::RepeatNone.into(),
    };
    let (ended_tx, _ended_rx) = tokio::sync::mpsc::unbounded_channel();
    let player_handler = PlayerHandler::new_with_contexts(
        ended_tx,
        vec![Box::new(DummyAudioPlayerContext::new())],
        Box::new(MockPersistContext { data: player_data }),
    );

    let current = player_handler.current_song();

    assert_eq!(player_handler.get_current_idx(), 0);
    assert_some_eq_x!(
        current
            .as_ref()
            .and_then(|s| s.song.as_ref())
            .and_then(|s| s.id.as_deref()),
        "1"
    );
}

#[tokio::test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_repeat_persists_player_data() {
    let persisted = Arc::new(Mutex::new(None));
    let (ended_tx, _ended_rx) = tokio::sync::mpsc::unbounded_channel();
    let player_handler = PlayerHandler::new_with_contexts(
        ended_tx,
        vec![Box::new(DummyAudioPlayerContext::new())],
        Box::new(TrackingPersistContext {
            persisted: persisted.clone(),
        }),
    );

    player_handler.repeat(RepeatMode::RepeatOnce);

    let guard = persisted.lock().unwrap();
    assert!(guard.is_some());
    assert_eq!(
        guard.as_ref().unwrap().repeat_mode,
        RepeatMode::RepeatOnce as i32
    );
}

struct FailingPersistContext;

impl PersistContext for FailingPersistContext {
    #[tracing::instrument(level = "debug", skip_all)]
    fn persist(&self, _data: &PlayerData) -> Result<(), PlayerError> {
        Err(PlayerError::IoError(std::io::Error::other(
            "persist failed",
        )))
    }
    #[tracing::instrument(level = "debug", skip_all)]
    fn load(&self) -> Result<PlayerData, PlayerError> {
        Err(PlayerError::IoError(std::io::Error::other("load failed")))
    }
}

#[tokio::test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_player_handler_new_dummy_and_methods() {
    let (ended_tx, _ended_rx) = tokio::sync::mpsc::unbounded_channel();
    let handler = PlayerHandler::new_dummy(ended_tx);

    let vol = handler.get_volume();
    assert_eq!(vol, 100);

    let pos = handler.get_current_pos();
    assert!(pos.is_ok());

    let curr = handler.get_current_song();
    assert_none!(curr);

    let pause_res = handler.pause();
    assert!(pause_res.is_ok());

    handler.set_volume(75);
    handler.seek(std::time::Duration::from_secs(10));

    handler.set_resolver(Box::new(|_song| {
        Box::new(Box::pin(std::future::ready(Ok(
            extensions_proto::moosync::types::PlaybackDetailsRequestedResponse {
                url: "https://example.com/audio.mp3".to_string(),
                duration: None,
                decryption_key: None,
            },
        ))))
    }));
}

#[tokio::test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_player_handler_failing_persist() {
    let (ended_tx, _ended_rx) = tokio::sync::mpsc::unbounded_channel();
    let handler = PlayerHandler::new_with_contexts(
        ended_tx,
        vec![Box::new(DummyAudioPlayerContext::new())],
        Box::new(FailingPersistContext),
    );

    assert_eq!(handler.get_current_idx(), 0);
    assert_is_empty!(&handler.get_queue());

    handler.repeat(RepeatMode::RepeatInfinite);
    let songs = vec![create_mock_song("s1", "Song 1")];
    handler.add_to_queue(songs).await;
    assert_len_eq_x!(&handler.get_queue(), 1);
}

struct FailingSeekAndVolumePlayer;

impl AudioPlayerContext for FailingSeekAndVolumePlayer {
    #[tracing::instrument(level = "debug", skip_all)]
    fn can_play(&self, _src: &ValidSrc) -> bool { true }
    #[tracing::instrument(level = "debug", skip_all)]
    fn set_src(
        &self,
        _src: ValidSrc,
        _events_tx: tokio::sync::mpsc::UnboundedSender<
            extensions_proto::moosync::types::player_event::Event,
        >,
    ) -> Result<(), PlayerError> {
        Ok(())
    }
    #[tracing::instrument(level = "debug", skip_all)]
    fn play(&self) -> Result<(), PlayerError> { Ok(()) }
    #[tracing::instrument(level = "debug", skip_all)]
    fn pause(&self) -> Result<(), PlayerError> { Ok(()) }
    #[tracing::instrument(level = "debug", skip_all)]
    fn stop(&self) -> Result<(), PlayerError> { Ok(()) }
    #[tracing::instrument(level = "debug", skip_all)]
    fn set_volume(&self, _vol: u8) -> Result<(), PlayerError> {
        Err(PlayerError::PlaybackUrlResolutionFailed(
            "set volume error".to_string(),
        ))
    }
    #[tracing::instrument(level = "debug", skip_all)]
    fn seek(&self, _pos: Duration) -> Result<(), PlayerError> {
        Err(PlayerError::PlaybackUrlResolutionFailed(
            "seek error".to_string(),
        ))
    }
    #[tracing::instrument(level = "debug", skip_all)]
    fn get_volume(&self) -> Result<u8, PlayerError> { Ok(50) }
    #[tracing::instrument(level = "debug", skip_all)]
    fn get_current_pos(&self) -> Result<Duration, PlayerError> {
        Err(PlayerError::PlaybackUrlResolutionFailed(
            "get current pos error".to_string(),
        ))
    }
    #[tracing::instrument(level = "debug", skip_all)]
    fn get_player_state(&self) -> Result<PlayerState, PlayerError> { Ok(PlayerState::Stopped) }
}

#[tokio::test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_player_handler_seek_volume_errors() {
    let (ended_tx, _ended_rx) = tokio::sync::mpsc::unbounded_channel();
    let handler = PlayerHandler::new_with_contexts(
        ended_tx,
        vec![Box::new(FailingSeekAndVolumePlayer)],
        Box::new(DummyPersistContext {}),
    );

    handler.set_volume(40);
    handler.seek(Duration::from_secs(15));
    assert_eq!(handler.get_volume(), 50);
}

#[tokio::test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_player_handler_plugin_init() {
    let tmp = tempdir::TempDir::new("test_player_plugin_init").unwrap();
    let ctx = PluginContext {
        data_dir: tmp.path().to_path_buf(),
        cache_dir: tmp.path().join("cache"),
        tmp_dir: tmp.path().join("tmp"),
    };
    let ph = <PlayerHandler as Plugin>::init(&ctx);
    assert_eq!(ph.get_volume(), 100);
    assert_eq!(ph.get_current_idx(), 0);
    tokio::time::sleep(Duration::from_millis(50)).await;
}
