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

use songs_proto::moosync::types::{
    Album, Artist, EntityResult, InnerSong, Song, SongType, entity_result,
};
use themes_proto::moosync::types::ThemeDetails;

use crate::{
    ScanProgress,
    prelude::{
        EntityResultExt, InnerSongExt, SongsExt, ThemeExt, core_to_proto_duration, format_duration,
    },
};

#[test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_scan_progress_and_format_duration() {
    let stopped = ScanProgress::STOPPED;
    let in_progress = ScanProgress::PROGRESS(75);
    assert_ne!(stopped, in_progress);

    assert_eq!(format_duration(0), "00:00");
    assert_eq!(format_duration(59), "00:59");
    assert_eq!(format_duration(125), "02:05");
    assert_eq!(format_duration(3599), "59:59");
    assert_eq!(format_duration(3600), "60:00");
    assert_eq!(format_duration(3661), "61:01");

    let zero_proto = core_to_proto_duration(Duration::ZERO);
    assert_eq!(zero_proto.seconds, 0);
    assert_eq!(zero_proto.nanos, 0);

    let precise_proto = core_to_proto_duration(Duration::new(120, 999_999_999));
    assert_eq!(precise_proto.seconds, 120);
    assert_eq!(precise_proto.nanos, 999_999_999);
}

#[test]
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

    assert_eq!(inner.get_type_or_default(), SongType::Local);

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

    assert_eq!(song.get_id().unwrap(), "s1");
    assert_eq!(song.get_title().unwrap(), "Title 1");
    assert_eq!(song.get_duration_or_default(), Duration::from_secs(180));
    assert_eq!(song.get_artist_string().unwrap(), "Artist 1");
    assert_eq!(song.get_album_string().unwrap(), "Album 1");
    assert_eq!(song.format_duration(), "03:00");

    let live_song = Song {
        song: Some(InnerSong {
            duration: None,
            ..Default::default()
        }),
        ..Default::default()
    };
    assert_eq!(live_song.format_duration(), "00:00");
}

#[test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_theme_ext_and_entity_result_ext() {
    let theme_details = ThemeDetails::default();
    let theme_item = theme_details.get_theme_item_or_default();
    assert!(!theme_item.primary.is_empty());

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
    let albums = entity.get_albums();
    assert_eq!(albums.unwrap().len(), 1);
}

#[test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_core_to_proto_duration() {
    let dur = Duration::new(120, 500);
    let proto = core_to_proto_duration(dur);
    assert_eq!(proto.seconds, 120);
    assert_eq!(proto.nanos, 500);
}
