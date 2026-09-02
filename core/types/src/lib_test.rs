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

use assertables::{assert_len_eq_x, assert_not_empty, assert_some_eq_x};
use rstest::rstest;
use songs_proto::moosync::types::{
    Album, Artist, EntityResult, InnerSong, Song, SongType, entity_result,
};
use themes_proto::moosync::types::ThemeDetails;
use tracing_test::traced_test;

use crate::{
    ScanProgress,
    prelude::{
        EntityResultExt, InnerSongExt, SongsExt, ThemeExt, core_to_proto_duration, format_duration,
    },
};

#[rstest]
#[case(0, "00:00")]
#[case(59, "00:59")]
#[case(125, "02:05")]
#[case(3599, "59:59")]
#[case(3600, "60:00")]
#[case(3661, "61:01")]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_format_duration(#[case] seconds: i64, #[case] expected: &str) {
    let formatted = format_duration(seconds);

    assert_eq!(formatted, expected);
}

#[rstest]
#[case(0, 0, 0, 0)]
#[case(120, 500, 120, 500)]
#[case(120, 999_999_999, 120, 999_999_999)]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_core_to_proto_duration(
    #[case] secs: u64,
    #[case] nanos: u32,
    #[case] expected_secs: i64,
    #[case] expected_nanos: i32,
) {
    let duration = Duration::new(secs, nanos);

    let proto = core_to_proto_duration(duration);

    assert_eq!(proto.seconds, expected_secs);
    assert_eq!(proto.nanos, expected_nanos);
}

#[test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_scan_progress_states() {
    let stopped = ScanProgress::STOPPED;
    let in_progress = ScanProgress::PROGRESS(75);

    assert_ne!(stopped, in_progress);
}

#[test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_inner_song_ext_and_song_ext() {
    let inner = InnerSong {
        id: Some("s1".to_string()),
        title: Some("Title 1".to_string()),
        duration: Some(songs_proto::duration_proto::google::protobuf::Duration {
            seconds: 180,
            nanos: 0,
        }),
        r#type: SongType::Local as i32,
        ..Default::default()
    };
    let song = Song {
        song: Some(inner),
        artists: vec![Artist {
            artist_name: Some("Artist 1".to_string()),
            ..Default::default()
        }],
        album: Some(Album {
            album_name: Some("Album 1".to_string()),
            ..Default::default()
        }),
        ..Default::default()
    };

    let inner_ref = song.song.as_ref().unwrap();
    let song_type = inner_ref.get_type_or_default();
    let formatted = song.format_duration();

    assert_eq!(song_type, SongType::Local);
    assert_some_eq_x!(song.get_id(), "s1");
    assert_some_eq_x!(song.get_title(), "Title 1");
    assert_eq!(song.get_duration_or_default(), Duration::from_secs(180));
    assert_some_eq_x!(song.get_artist_string(), "Artist 1");
    assert_some_eq_x!(song.get_album_string(), "Album 1");
    assert_eq!(formatted, "03:00");
}

#[test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_song_format_duration_without_duration() {
    let live_song = Song {
        song: Some(InnerSong {
            duration: None,
            ..Default::default()
        }),
        ..Default::default()
    };

    let formatted = live_song.format_duration();

    assert_eq!(formatted, "00:00");
}

#[test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_theme_ext_default() {
    let theme_details = ThemeDetails::default();

    let theme_item = theme_details.get_theme_item_or_default();

    assert_not_empty!(theme_item.primary);
}

#[test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_entity_result_ext_albums() {
    let entity = EntityResult {
        result: Some(entity_result::Result::Albums(
            songs_proto::moosync::types::AlbumList {
                albums: vec![Album {
                    album_name: Some("A".to_string()),
                    ..Default::default()
                }],
            },
        )),
    };

    let albums = entity.get_albums().unwrap();

    assert_len_eq_x!(albums, 1);
}
