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

use player_proto::moosync::types::{PlayerData, RepeatMode};
use rstest::{fixture, rstest};
use songs_proto::moosync::types::{InnerSong, Song};
use tracing_test::traced_test;

use crate::player_data_ext::PlayerDataExt;

#[fixture]
#[tracing::instrument(level = "debug", skip_all)]
fn make_song(#[default("test_id")] id: &str) -> Song {
    Song {
        song: Some(InnerSong {
            id: Some(id.to_string()),
            title: Some(id.to_string()),
            ..Default::default()
        }),
        ..Default::default()
    }
}

#[rstest]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_append_songs_empty_queue() {
    let mut data = PlayerData::default();
    let s1 = make_song("s1");
    let s2 = make_song("s2");

    data.append_songs(vec![s1.clone(), s2.clone()]);

    assert_eq!(data.song_queue.len(), 2);
    assert_eq!(
        data.song_queue[0].song.as_ref().unwrap().id.as_deref(),
        Some("s1")
    );
    assert_eq!(
        data.song_queue[1].song.as_ref().unwrap().id.as_deref(),
        Some("s2")
    );
}

#[rstest]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_insert_songs_at_middle() {
    let mut data = PlayerData {
        song_queue: vec![make_song("s1"), make_song("s3")],
        ..Default::default()
    };
    let s2 = make_song("s2");

    data.insert_songs_at(1, vec![s2]);

    assert_eq!(data.song_queue.len(), 3);
    assert_eq!(
        data.song_queue[0].song.as_ref().unwrap().id.as_deref(),
        Some("s1")
    );
    assert_eq!(
        data.song_queue[1].song.as_ref().unwrap().id.as_deref(),
        Some("s2")
    );
    assert_eq!(
        data.song_queue[2].song.as_ref().unwrap().id.as_deref(),
        Some("s3")
    );
}

#[rstest]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_insert_songs_at_out_of_bounds() {
    let mut data = PlayerData {
        song_queue: vec![make_song("s1")],
        ..Default::default()
    };
    let s2 = make_song("s2");

    data.insert_songs_at(100, vec![s2]);

    assert_eq!(data.song_queue.len(), 2);
    assert_eq!(
        data.song_queue[1].song.as_ref().unwrap().id.as_deref(),
        Some("s2")
    );
}

#[rstest]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_insert_songs_next() {
    let mut data = PlayerData {
        song_queue: vec![make_song("s1"), make_song("s3")],
        current_idx: 0,
        ..Default::default()
    };
    let s2 = make_song("s2");

    data.insert_songs_next(vec![s2]);

    assert_eq!(data.song_queue.len(), 3);
    assert_eq!(
        data.song_queue[1].song.as_ref().unwrap().id.as_deref(),
        Some("s2")
    );
}

#[rstest]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_clear_and_set() {
    let mut data = PlayerData {
        song_queue: vec![make_song("old1"), make_song("old2")],
        current_idx: 1,
        ..Default::default()
    };
    let new_songs = vec![make_song("new1"), make_song("new2")];

    data.clear_and_set(new_songs);

    assert_eq!(data.song_queue.len(), 2);
    assert_eq!(
        data.song_queue[0].song.as_ref().unwrap().id.as_deref(),
        Some("new1")
    );
    assert_eq!(data.current_idx, 0);
}

#[rstest]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_shuffle_queue_single_or_empty() {
    let mut empty_data = PlayerData::default();
    let mut single_data = PlayerData {
        song_queue: vec![make_song("s1")],
        current_idx: 0,
        ..Default::default()
    };

    empty_data.shuffle_queue();
    single_data.shuffle_queue();

    assert!(empty_data.song_queue.is_empty());
    assert_eq!(single_data.song_queue.len(), 1);
    assert_eq!(
        single_data.song_queue[0]
            .song
            .as_ref()
            .unwrap()
            .id
            .as_deref(),
        Some("s1")
    );
}

#[rstest]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_shuffle_queue_preserves_current_song() {
    let mut data = PlayerData {
        song_queue: vec![
            make_song("s1"),
            make_song("s2"),
            make_song("s3"),
            make_song("s4"),
        ],
        current_idx: 1,
        ..Default::default()
    };

    data.shuffle_queue();

    assert_eq!(data.song_queue.len(), 4);
    assert_eq!(
        data.song_queue[1].song.as_ref().unwrap().id.as_deref(),
        Some("s2")
    );
    assert_eq!(data.current_idx, 1);
}

#[rstest]
#[case(RepeatMode::RepeatNone)]
#[case(RepeatMode::RepeatOnce)]
#[case(RepeatMode::RepeatInfinite)]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_set_repeat_mode(#[case] mode: RepeatMode) {
    let mut data = PlayerData::default();

    data.set_repeat_mode(mode);

    assert_eq!(data.repeat_mode, mode as i32);
}

#[rstest]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_advance_index_middle() {
    let mut data = PlayerData {
        song_queue: vec![make_song("s1"), make_song("s2"), make_song("s3")],
        current_idx: 0,
        ..Default::default()
    };

    data.advance_index();

    assert_eq!(data.current_idx, 1);
}

#[rstest]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_advance_index_wraparound() {
    let mut data = PlayerData {
        song_queue: vec![make_song("s1"), make_song("s2")],
        current_idx: 1,
        ..Default::default()
    };

    data.advance_index();

    assert_eq!(data.current_idx, 0);
}

#[rstest]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_advance_index_empty() {
    let mut data = PlayerData::default();

    data.advance_index();

    assert_eq!(data.current_idx, 0);
}

#[rstest]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_rewind_index_middle() {
    let mut data = PlayerData {
        song_queue: vec![make_song("s1"), make_song("s2")],
        current_idx: 1,
        ..Default::default()
    };

    data.rewind_index();

    assert_eq!(data.current_idx, 0);
}

#[rstest]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_rewind_index_already_zero() {
    let mut data = PlayerData {
        song_queue: vec![make_song("s1"), make_song("s2")],
        current_idx: 0,
        ..Default::default()
    };

    data.rewind_index();

    assert_eq!(data.current_idx, 0);
}

#[rstest]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_rewind_index_empty() {
    let mut data = PlayerData::default();

    data.rewind_index();

    assert_eq!(data.current_idx, 0);
}

#[rstest]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_set_index_valid() {
    let mut data = PlayerData {
        song_queue: vec![make_song("s1"), make_song("s2"), make_song("s3")],
        current_idx: 0,
        ..Default::default()
    };

    data.set_index(2);

    assert_eq!(data.current_idx, 2);
}

#[rstest]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_set_index_out_of_bounds() {
    let mut data = PlayerData {
        song_queue: vec![make_song("s1"), make_song("s2")],
        current_idx: 1,
        ..Default::default()
    };

    data.set_index(10);

    assert_eq!(data.current_idx, 1);
}

#[rstest]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_remove_at_index_valid() {
    let mut data = PlayerData {
        song_queue: vec![make_song("s1"), make_song("s2"), make_song("s3")],
        current_idx: 0,
        ..Default::default()
    };

    data.remove_at_index(1);

    assert_eq!(data.song_queue.len(), 2);
    assert_eq!(
        data.song_queue[1].song.as_ref().unwrap().id.as_deref(),
        Some("s3")
    );
    assert_eq!(data.current_idx, 0);
}

#[rstest]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_remove_at_index_clamps_current_idx() {
    let mut data = PlayerData {
        song_queue: vec![make_song("s1"), make_song("s2")],
        current_idx: 1,
        ..Default::default()
    };

    data.remove_at_index(1);

    assert_eq!(data.song_queue.len(), 1);
    assert_eq!(data.current_idx, 0);
}

#[rstest]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_remove_at_index_out_of_bounds() {
    let mut data = PlayerData {
        song_queue: vec![make_song("s1")],
        current_idx: 0,
        ..Default::default()
    };

    data.remove_at_index(5);

    assert_eq!(data.song_queue.len(), 1);
    assert_eq!(data.current_idx, 0);
}

#[rstest]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_remove_at_index_before_current_idx() {
    let mut data = PlayerData {
        song_queue: vec![make_song("s1"), make_song("s2"), make_song("s3")],
        current_idx: 2,
        ..Default::default()
    };

    data.remove_at_index(0);

    assert_eq!(data.song_queue.len(), 2);
    assert_eq!(
        data.song_queue[1].song.as_ref().unwrap().id.as_deref(),
        Some("s3")
    );
    assert_eq!(data.current_idx, 1);
}

#[rstest]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_remove_at_index_single_item() {
    let mut data = PlayerData {
        song_queue: vec![make_song("s1")],
        current_idx: 0,
        ..Default::default()
    };

    data.remove_at_index(0);

    assert!(data.song_queue.is_empty());
    assert_eq!(data.current_idx, 0);
}

#[rstest]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_clear_except_current_multi_songs() {
    let mut data = PlayerData {
        song_queue: vec![make_song("s1"), make_song("s2"), make_song("s3")],
        current_idx: 1,
        ..Default::default()
    };

    data.clear_except_current();

    assert_eq!(data.song_queue.len(), 1);
    assert_eq!(
        data.song_queue[0].song.as_ref().unwrap().id.as_deref(),
        Some("s2")
    );
    assert_eq!(data.current_idx, 0);
}

#[rstest]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_clear_except_current_single_song() {
    let mut data = PlayerData {
        song_queue: vec![make_song("s1")],
        current_idx: 0,
        ..Default::default()
    };

    data.clear_except_current();

    assert!(data.song_queue.is_empty());
    assert_eq!(data.current_idx, 0);
}

#[rstest]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_clear_except_current_empty() {
    let mut data = PlayerData::default();

    data.clear_except_current();

    assert!(data.song_queue.is_empty());
    assert_eq!(data.current_idx, 0);
}

#[rstest]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_reorder_current_song_forward() {
    let mut data = PlayerData {
        song_queue: vec![make_song("s1"), make_song("s2"), make_song("s3")],
        current_idx: 0,
        ..Default::default()
    };

    data.reorder(0, 2);

    assert_eq!(
        data.song_queue[2].song.as_ref().unwrap().id.as_deref(),
        Some("s1")
    );
    assert_eq!(data.current_idx, 2);
}

#[rstest]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_reorder_from_before_current_to_after() {
    let mut data = PlayerData {
        song_queue: vec![
            make_song("s1"),
            make_song("s2"),
            make_song("s3"),
            make_song("s4"),
        ],
        current_idx: 2,
        ..Default::default()
    };

    data.reorder(0, 3);

    assert_eq!(
        data.song_queue[3].song.as_ref().unwrap().id.as_deref(),
        Some("s1")
    );
    assert_eq!(data.current_idx, 1);
}

#[rstest]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_reorder_from_after_current_to_before() {
    let mut data = PlayerData {
        song_queue: vec![
            make_song("s1"),
            make_song("s2"),
            make_song("s3"),
            make_song("s4"),
        ],
        current_idx: 1,
        ..Default::default()
    };

    data.reorder(3, 0);

    assert_eq!(
        data.song_queue[0].song.as_ref().unwrap().id.as_deref(),
        Some("s4")
    );
    assert_eq!(data.current_idx, 2);
}

#[rstest]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_reorder_out_of_bounds() {
    let mut data = PlayerData {
        song_queue: vec![make_song("s1"), make_song("s2")],
        current_idx: 0,
        ..Default::default()
    };

    data.reorder(0, 10);

    assert_eq!(
        data.song_queue[0].song.as_ref().unwrap().id.as_deref(),
        Some("s1")
    );
    assert_eq!(
        data.song_queue[1].song.as_ref().unwrap().id.as_deref(),
        Some("s2")
    );
    assert_eq!(data.current_idx, 0);
}

#[rstest]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_current_song_present() {
    let data = PlayerData {
        song_queue: vec![make_song("s1"), make_song("s2")],
        current_idx: 1,
        ..Default::default()
    };

    let song = data.current_song();

    assert!(song.is_some());
    assert_eq!(
        song.unwrap().song.as_ref().unwrap().id.as_deref(),
        Some("s2")
    );
}

#[rstest]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_current_song_empty() {
    let data = PlayerData::default();

    let song = data.current_song();

    assert!(song.is_none());
}
