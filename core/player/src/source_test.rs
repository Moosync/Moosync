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

use std::{borrow::Cow, env::temp_dir, fs, path::PathBuf};

use songs_proto::moosync::types::{InnerSong, Song};

use crate::{
    error::PlayerError,
    source::{SourceResolver, ValidSrc, get_valid_src},
};

#[test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_valid_src_inner_and_display() {
    let path_src = ValidSrc::Path(PathBuf::from("/music/song.mp3"));
    assert_eq!(path_src.inner(), "/music/song.mp3");
    assert_eq!(format!("{}", path_src), "/music/song.mp3");

    let url_src = ValidSrc::Url(Cow::Borrowed("https://stream.org/audio.mp3"));
    assert_eq!(url_src.inner(), "https://stream.org/audio.mp3");
    assert_eq!(format!("{}", url_src), "https://stream.org/audio.mp3");
}

#[test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_get_valid_src_file_exists() {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let test_file = temp_dir().join(format!("moosync_src_test_{}.mp3", now));
    fs::write(&test_file, b"audio").unwrap();

    let song = Song {
        song: Some(InnerSong {
            path: Some(test_file.to_string_lossy().to_string()),
            ..Default::default()
        }),
        ..Default::default()
    };

    let valid_src = get_valid_src(&song);
    assert!(valid_src.is_ok());
    match valid_src.unwrap() {
        ValidSrc::Path(p) => assert_eq!(p, test_file),
        _ => panic!("Expected ValidSrc::Path"),
    }

    let _ = fs::remove_file(test_file);
}

#[test]
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
    assert!(valid_src.is_ok());
    match valid_src.unwrap() {
        ValidSrc::Url(u) => assert_eq!(u, "https://example.com/audio"),
        _ => panic!("Expected ValidSrc::Url"),
    }
}

#[test]
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
    assert!(valid_src.is_err());
    match valid_src.unwrap_err() {
        PlayerError::NoSrcFound(_) => {}
        err => panic!("Expected PlayerError::NoSrcFound, got: {:?}", err),
    }
}

#[test]
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

    // Unset resolver fails
    let res = resolver.resolve_playback_url(&mut song);
    assert!(res.is_err());

    // Set resolver
    resolver.set_resolver(Box::new(|_s| Ok("https://resolved.stream/123".to_string())));
    let _ = resolver.resolve_playback_url(&mut song);
    assert_eq!(
        song.song.unwrap().playback_url,
        Some("https://resolved.stream/123".to_string())
    );
}
