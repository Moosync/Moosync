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

use std::{borrow::Cow, time::Duration};

use assertables::{assert_len_eq_x, assert_none, assert_not_empty, assert_some, assert_some_eq_x};
use extensions_proto::moosync::types::RequestedSearchResultResponse;
use rstest::rstest;
use songs_proto::moosync::types::{
    Album, Artist, EntityResult, Genre, InnerSong, Lyrics, Playlist, Song, SongType, entity_result,
};
use themes_proto::moosync::types::ThemeDetails;
use tracing_test::traced_test;

use crate::{
    ScanProgress,
    prelude::{
        EntityResultExt, InnerSongExt, SearchResultExt, SongsExt, ThemeExt, core_to_proto_duration,
        format_duration,
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

#[test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_entity_result_ext_artists() {
    let entity = EntityResult {
        result: Some(entity_result::Result::Artists(
            songs_proto::moosync::types::ArtistList {
                artists: vec![Artist {
                    artist_name: Some("Art".to_string()),
                    ..Default::default()
                }],
            },
        )),
    };

    let artists = entity.get_artists().unwrap();

    assert_len_eq_x!(artists, 1);
}

#[test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_entity_result_ext_genres() {
    let entity = EntityResult {
        result: Some(entity_result::Result::Genres(
            songs_proto::moosync::types::GenreList {
                genres: vec![Genre {
                    genre_name: Some("Rock".to_string()),
                    ..Default::default()
                }],
            },
        )),
    };

    let genres = entity.get_genres().unwrap();

    assert_len_eq_x!(genres, 1);
}

#[test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_entity_result_ext_playlists() {
    let entity = EntityResult {
        result: Some(entity_result::Result::Playlists(
            songs_proto::moosync::types::PlaylistList {
                playlists: vec![Playlist {
                    playlist_name: "Play".to_string(),
                    ..Default::default()
                }],
            },
        )),
    };

    let playlists = entity.get_playlists().unwrap();

    assert_len_eq_x!(playlists, 1);
}

#[test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_entity_result_ext_none_variants() {
    let entity_none = EntityResult { result: None };

    assert_none!(entity_none.clone().get_albums());
    assert_none!(entity_none.clone().get_artists());
    assert_none!(entity_none.clone().get_genres());
    assert_none!(entity_none.get_playlists());
}

#[test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_song_cover_getters() {
    let both = Song {
        song: Some(InnerSong {
            song_cover_path_high: Some("high.jpg".to_string()),
            song_cover_path_low: Some("low.jpg".to_string()),
            ..Default::default()
        }),
        ..Default::default()
    };
    let only_high = Song {
        song: Some(InnerSong {
            song_cover_path_high: Some("high.jpg".to_string()),
            song_cover_path_low: None,
            ..Default::default()
        }),
        ..Default::default()
    };
    let neither = Song {
        song: Some(InnerSong::default()),
        ..Default::default()
    };

    assert_some_eq_x!(both.get_cover_high(), "high.jpg");
    assert_some_eq_x!(both.get_cover_low(), "low.jpg");
    assert_some_eq_x!(only_high.get_cover_low(), "high.jpg");
    assert_none!(neither.get_cover_high());
    assert_none!(neither.get_cover_low());
}

#[test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_song_playback_url_and_path_and_date() {
    let song = Song {
        song: Some(InnerSong {
            playback_url: Some("http://stream.url".to_string()),
            path: Some("/music/song.mp3".to_string()),
            date: Some("2024-01-01".to_string()),
            ..Default::default()
        }),
        ..Default::default()
    };
    let empty_song = Song::default();

    assert_some_eq_x!(song.get_playback_url(), "http://stream.url");
    assert_some_eq_x!(song.get_path(), "/music/song.mp3");
    assert_some_eq_x!(song.get_date(), "2024-01-01");
    assert_none!(empty_song.get_playback_url());
    assert_none!(empty_song.get_path());
    assert_none!(empty_song.get_date());
}

#[test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_song_lyrics_getter() {
    let lyrics = Lyrics::default();
    let song_with_lyrics = Song {
        song: Some(InnerSong {
            lyrics: Some(lyrics),
            ..Default::default()
        }),
        ..Default::default()
    };
    let song_without = Song::default();

    assert_some!(song_with_lyrics.get_lyrics());
    assert_none!(song_without.get_lyrics());
}

#[test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_song_artists_string_formatting() {
    let empty_artists_song = Song {
        artists: vec![],
        ..Default::default()
    };
    let multi_artists_song = Song {
        artists: vec![
            Artist {
                artist_name: Some("Artist A".to_string()),
                ..Default::default()
            },
            Artist {
                artist_name: Some("Artist B".to_string()),
                ..Default::default()
            },
        ],
        ..Default::default()
    };

    assert_none!(empty_artists_song.get_artist_string());
    assert_some_eq_x!(
        multi_artists_song.get_artist_string().as_deref(),
        "Artist A,Artist B"
    );
}

#[test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_song_album_string_none() {
    let no_album_song = Song::default();

    assert_none!(no_album_song.get_album_string());
}

#[test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_song_invalid_type_fallback() {
    let invalid_song = Song {
        song: Some(InnerSong {
            r#type: 9999,
            ..Default::default()
        }),
        ..Default::default()
    };
    let inner_invalid = InnerSong {
        r#type: 9999,
        ..Default::default()
    };

    assert_eq!(invalid_song.get_type_or_default(), SongType::Local);
    assert_eq!(inner_invalid.get_type_or_default(), SongType::Local);
}

#[test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_song_negative_proto_duration_defaults_to_zero() {
    let negative_duration_song = Song {
        song: Some(InnerSong {
            duration: Some(songs_proto::duration_proto::google::protobuf::Duration {
                seconds: -10,
                nanos: 0,
            }),
            ..Default::default()
        }),
        ..Default::default()
    };

    assert_eq!(
        negative_duration_song.get_duration_or_default(),
        Duration::ZERO
    );
}

#[test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_format_duration_live_trait() {
    struct LiveSong;
    impl SongsExt for LiveSong {
        fn get_id(&self) -> Option<Cow<'_, str>> { None }
        fn get_title(&self) -> Option<Cow<'_, str>> { None }
        fn get_duration_or_default(&self) -> Duration { Duration::from_secs(u64::MAX) }
        fn get_cover_high(&self) -> Option<Cow<'_, str>> { None }
        fn get_cover_low(&self) -> Option<Cow<'_, str>> { None }
        fn get_playback_url(&self) -> Option<Cow<'_, str>> { None }
        fn get_type_or_default(&self) -> SongType { SongType::Local }
        fn get_path(&self) -> Option<Cow<'_, str>> { None }
        fn get_lyrics(&self) -> Option<&Lyrics> { None }
        fn get_date(&self) -> Option<Cow<'_, str>> { None }
        fn get_artist_string(&self) -> Option<String> { None }
        fn get_album_string(&self) -> Option<Cow<'_, str>> { None }
    }

    let live = LiveSong;

    assert_eq!(live.format_duration(), "Live");
}

#[test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_search_result_ext_to_songs_proto() {
    let requested = RequestedSearchResultResponse {
        songs: vec![Song::default()],
        albums: vec![Album::default()],
        artists: vec![Artist::default()],
        playlists: vec![Playlist::default()],
    };

    let converted = requested.to_songs_proto();

    assert_len_eq_x!(converted.songs, 1);
    assert_len_eq_x!(converted.albums, 1);
    assert_len_eq_x!(converted.artists, 1);
    assert_len_eq_x!(converted.playlists, 1);
    assert!(converted.genres.is_empty());
}
