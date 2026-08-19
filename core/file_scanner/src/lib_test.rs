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
    fs::{self, File},
    io::Write,
    sync::{Arc, Mutex},
};

use songs_proto::moosync::types::Playlist;
use tempdir::TempDir;

use crate::{PlaylistSongId, ScanProgress, ScannerHolder, error::ScannerError};

#[tokio::test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_scanner_holder_scan_dirs_not_configured() {
    let scanner = ScannerHolder::new();
    let res = scanner.start_scan().await;
    assert!(matches!(res, Err(ScannerError::ScanDirsNotConfigured)));
}

#[tokio::test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_scanner_holder_thumbnail_dir_not_configured() {
    let mut scanner = ScannerHolder::new();
    let tmp = TempDir::new("moosync_holder_val").unwrap();
    scanner.set_scan_dirs(vec![tmp.path().to_path_buf()]);

    let res = scanner.start_scan().await;
    assert!(matches!(res, Err(ScannerError::ThumbnailDirNotConfigured)));
}

#[tokio::test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_scanner_holder_song_callback_not_configured() {
    let mut scanner = ScannerHolder::new();
    let tmp = TempDir::new("moosync_holder_val").unwrap();
    scanner.set_scan_dirs(vec![tmp.path().to_path_buf()]);
    scanner.set_thumbnail_dir(tmp.path().to_path_buf());

    let res = scanner.start_scan().await;
    assert!(matches!(res, Err(ScannerError::SongCallbackNotConfigured)));
}

#[tokio::test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_scanner_holder_playlist_callback_not_configured() {
    let mut scanner = ScannerHolder::new();
    let tmp = TempDir::new("moosync_holder_val").unwrap();
    scanner.set_scan_dirs(vec![tmp.path().to_path_buf()]);
    scanner.set_thumbnail_dir(tmp.path().to_path_buf());
    scanner.set_on_song(|_pl, _songs| async {});

    let res = scanner.start_scan().await;
    assert!(matches!(
        res,
        Err(ScannerError::PlaylistCallbackNotConfigured)
    ));
}

#[tokio::test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_scanner_holder_scan_lifecycle_and_subscribers() {
    let playlist_contents = r#"
#EXTM3U
#EXTINF:0,stream
https://cast.animu.com.br:9079/stream
#EXTINF:0,320
https://radio.stereoanime.net/listen/stereoanime/320
"#;

    let tmp = TempDir::new("moosync_holder_test").unwrap();
    let in_dir = tmp.path().join("in");
    let out_dir = tmp.path().join("out");

    fs::create_dir_all(&in_dir).unwrap();
    fs::create_dir_all(&out_dir).unwrap();

    let mut input = File::create(in_dir.join("playlist.m3u")).unwrap();
    input.write_all(playlist_contents.as_bytes()).unwrap();

    let playlist_count = Arc::new(Mutex::new(0));
    let playlist_count_clone = playlist_count.clone();

    let mut scanner = ScannerHolder::new();
    scanner.set_scan_dirs(vec![in_dir.clone()]);
    scanner.set_exclude_dirs(vec![]);
    scanner.set_scan_threads(2);
    scanner.set_thumbnail_dir(out_dir.clone());
    scanner.set_artist_split(";".to_string());

    scanner.set_on_song(move |_pl_id, _songs| async move {});

    scanner.set_on_playlist(move |playlists: Vec<(Playlist, Vec<PlaylistSongId>)>| {
        let count_clone = playlist_count_clone.clone();
        async move {
            let mut count = count_clone.lock().unwrap();
            *count += playlists.len();
        }
    });

    let mut progress_rx = scanner.add_subscriber();

    let scan_res = scanner.start_scan().await;
    assert!(scan_res.is_ok());

    let mut progress_events = Vec::new();
    while let Ok(evt) = progress_rx.try_recv() {
        progress_events.push(evt);
    }
    assert!(!progress_events.is_empty());
    assert_eq!(*progress_events.last().unwrap(), ScanProgress::STOPPED);

    assert_eq!(*playlist_count.lock().unwrap(), 1);
}
