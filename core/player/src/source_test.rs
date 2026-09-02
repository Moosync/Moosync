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

use std::{borrow::Cow, fs, path::PathBuf};

use assertables::{assert_err, assert_matches, assert_ok, assert_some_eq_x};
use rstest::{fixture, rstest};
use songs_proto::moosync::types::{InnerSong, Song};
use tempdir::TempDir;
use tracing_test::traced_test;

use crate::{
    error::PlayerError,
    source::{SourceResolver, ValidSrc, get_valid_src},
};

#[fixture]
#[tracing::instrument(level = "debug", skip_all)]
fn temp_dir_fixture() -> TempDir {
    TempDir::new("moosync_src_test").expect("failed to create temp dir")
}

#[test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_valid_src_inner_and_display() {
    let path_src = ValidSrc::Path(PathBuf::from("/music/song.mp3"));
    let url_src = ValidSrc::Url(Cow::Borrowed("https://stream.org/audio.mp3"));

    assert_eq!(path_src.inner(), "/music/song.mp3");
    assert_eq!(format!("{}", path_src), "/music/song.mp3");
    assert_eq!(url_src.inner(), "https://stream.org/audio.mp3");
    assert_eq!(format!("{}", url_src), "https://stream.org/audio.mp3");
}

#[rstest]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_get_valid_src_file_exists(temp_dir_fixture: TempDir) {
    let test_file = temp_dir_fixture.path().join("song.mp3");
    fs::write(&test_file, b"audio").unwrap();
    let song = Song {
        song: Some(InnerSong {
            path: Some(test_file.to_string_lossy().to_string()),
            ..Default::default()
        }),
        ..Default::default()
    };

    let valid_src = get_valid_src(&song);

    assert_ok!(valid_src.as_ref());
    assert_matches!(valid_src.unwrap(), ValidSrc::Path(p) if p == test_file);
}

#[test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_get_valid_src_playback_url_when_file_absent() {
    let song = Song {
        song: Some(InnerSong {
            path: Some("/non_existent_file.mp3".to_string()),
            playback_url: Some("https://example.com/audio".to_string()),
            ..Default::default()
        }),
        ..Default::default()
    };

    let valid_src = get_valid_src(&song);

    assert_ok!(valid_src.as_ref());
    assert_matches!(valid_src.unwrap(), ValidSrc::Url(u) if u == "https://example.com/audio");
}

#[test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_get_valid_src_error_when_none_present() {
    let song = Song {
        song: Some(InnerSong {
            path: None,
            playback_url: None,
            ..Default::default()
        }),
        ..Default::default()
    };

    let valid_src = get_valid_src(&song);

    assert_err!(valid_src.as_ref());
    assert_matches!(valid_src.unwrap_err(), PlayerError::NoSrcFound(_));
}

#[test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_source_resolver_resolve_playback_url() {
    let resolver = SourceResolver::new();
    let mut song = Song {
        song: Some(InnerSong {
            id: Some("123".to_string()),
            ..Default::default()
        }),
        ..Default::default()
    };

    assert_err!(resolver.resolve_playback_url(&mut song).as_ref());

    resolver.set_resolver(Box::new(|_s| Ok("https://resolved.stream/123".to_string())));
    assert_ok!(resolver.resolve_playback_url(&mut song));
    assert_some_eq_x!(
        song.song.as_ref().unwrap().playback_url.as_deref(),
        "https://resolved.stream/123"
    );
}
