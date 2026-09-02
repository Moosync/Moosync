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

use assertables::{assert_err, assert_matches, assert_not_empty, assert_ok};
use rstest::{fixture, rstest};
use songs_proto::moosync::types::Playlist;
use tempdir::TempDir;
use tracing_test::traced_test;

use crate::{PlaylistSongId, ScanProgress, ScannerHolder, error::ScannerError};

#[fixture]
#[tracing::instrument(level = "debug", skip_all)]
fn temp_dir_fixture() -> TempDir {
    TempDir::new("moosync_holder_test").expect("failed to create temp dir")
}

#[fixture]
#[tracing::instrument(level = "debug", skip_all)]
fn default_scanner() -> ScannerHolder { ScannerHolder::new() }

#[rstest]
#[tokio::test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_scanner_holder_scan_dirs_not_configured(default_scanner: ScannerHolder) {
    let res = default_scanner.start_scan().await;

    assert_err!(res.as_ref());
    assert_matches!(res.unwrap_err(), ScannerError::ScanDirsNotConfigured);
}

#[rstest]
#[tokio::test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_scanner_holder_thumbnail_dir_not_configured(
    mut default_scanner: ScannerHolder,
    temp_dir_fixture: TempDir,
) {
    default_scanner.set_scan_dirs(vec![temp_dir_fixture.path().to_path_buf()]);

    let res = default_scanner.start_scan().await;

    assert_err!(res.as_ref());
    assert_matches!(res.unwrap_err(), ScannerError::ThumbnailDirNotConfigured);
}

#[rstest]
#[tokio::test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_scanner_holder_song_callback_not_configured(
    mut default_scanner: ScannerHolder,
    temp_dir_fixture: TempDir,
) {
    default_scanner.set_scan_dirs(vec![temp_dir_fixture.path().to_path_buf()]);
    default_scanner.set_thumbnail_dir(temp_dir_fixture.path().to_path_buf());

    let res = default_scanner.start_scan().await;

    assert_err!(res.as_ref());
    assert_matches!(res.unwrap_err(), ScannerError::SongCallbackNotConfigured);
}

#[rstest]
#[tokio::test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_scanner_holder_playlist_callback_not_configured(
    mut default_scanner: ScannerHolder,
    temp_dir_fixture: TempDir,
) {
    default_scanner.set_scan_dirs(vec![temp_dir_fixture.path().to_path_buf()]);
    default_scanner.set_thumbnail_dir(temp_dir_fixture.path().to_path_buf());
    default_scanner.set_on_song(|_pl, _songs| async {});

    let res = default_scanner.start_scan().await;

    assert_err!(res.as_ref());
    assert_matches!(
        res.unwrap_err(),
        ScannerError::PlaylistCallbackNotConfigured
    );
}

#[rstest]
#[tokio::test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_scanner_holder_scan_lifecycle_and_subscribers(temp_dir_fixture: TempDir) {
    let playlist_contents = r#"
#EXTM3U
#EXTINF:0,stream
https://cast.animu.com.br:9079/stream
#EXTINF:0,320
https://radio.stereoanime.net/listen/stereoanime/320
"#;

    let in_dir = temp_dir_fixture.path().join("in");
    let out_dir = temp_dir_fixture.path().join("out");
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
    let mut progress_events = Vec::new();
    while let Ok(evt) = progress_rx.try_recv() {
        progress_events.push(evt);
    }

    assert_ok!(scan_res);
    assert_not_empty!(&progress_events);
    assert_eq!(*progress_events.last().unwrap(), ScanProgress::STOPPED);
    assert_eq!(*playlist_count.lock().unwrap(), 1);
}
