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

use std::{fs, path::PathBuf};

use assertables::{assert_err, assert_matches, assert_ok};
use extensions_proto::moosync::types::PlaybackDetailsRequestedResponse;
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
    let url_src = ValidSrc::new_url("https://stream.org/audio.mp3");
    let key_src = ValidSrc::new_url_with_key("https://stream.org/audio.mp3", Some("mykey"));

    assert_eq!(path_src.inner(), "/music/song.mp3");
    assert_eq!(format!("{}", path_src), "/music/song.mp3");
    assert_eq!(url_src.inner(), "https://stream.org/audio.mp3");
    assert_eq!(format!("{}", url_src), "https://stream.org/audio.mp3");
    assert_eq!(url_src.decryption_key(), None);
    assert_eq!(key_src.decryption_key().as_deref(), Some("mykey"));
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
    assert_matches!(valid_src.unwrap(), ValidSrc::Url { url, .. } if url == "https://example.com/audio");
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

#[tokio::test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_source_resolver_resolve() {
    let resolver = SourceResolver::new();
    let song = Song {
        song: Some(InnerSong {
            id: Some("123".to_string()),
            ..Default::default()
        }),
        ..Default::default()
    };

    assert_err!(resolver.resolve(&song).await.as_ref());

    resolver.set_resolver(Box::new(|_s| {
        Box::new(Box::pin(std::future::ready(Ok(
            PlaybackDetailsRequestedResponse {
                url: "https://resolved.stream/123".to_string(),
                duration: None,
                decryption_key: Some("0123456789abcdef0123456789abcdef".to_string()),
            },
        ))))
    }));
    let res = resolver.resolve(&song).await;
    assert_ok!(res.as_ref());
    let resp = res.unwrap();
    assert_eq!(resp.url, "https://resolved.stream/123");
    assert_eq!(
        resp.decryption_key.as_deref(),
        Some("0123456789abcdef0123456789abcdef")
    );
}
