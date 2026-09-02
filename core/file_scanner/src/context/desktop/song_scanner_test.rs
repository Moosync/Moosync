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

use std::fs;

use assertables::{assert_none, assert_ok};
use rstest::{fixture, rstest};
use tempdir::TempDir;
use tracing_test::traced_test;

use crate::{
    FileList, ScanProgress,
    context::desktop::song_scanner::{SongScanner, check_directory},
};

#[fixture]
#[tracing::instrument(level = "debug", skip_all)]
fn temp_dir_fixture() -> TempDir {
    TempDir::new("moosync_song_scan_test").expect("failed to create temp dir")
}

#[rstest]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_check_directory_creates_dir_if_missing(temp_dir_fixture: TempDir) {
    let dir = temp_dir_fixture.path().join("missing_sub_dir");
    assert!(!dir.exists());

    let res = check_directory(dir.clone());

    assert_ok!(res);
    assert!(dir.exists());
}

#[rstest]
#[tokio::test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_song_scanner_scan_song_non_audio(temp_dir_fixture: TempDir) {
    let thumb_dir = temp_dir_fixture.path().join("thumbs");
    let song_path = temp_dir_fixture.path().join("not_audio.mp3");
    fs::write(&song_path, b"fake data").unwrap();
    let file_list = FileList {
        file_list: vec![(song_path.clone(), 1234.0)],
        playlist_list: vec![],
    };
    let scanner = SongScanner::new(&file_list, thumb_dir, ";".to_string(), Some(1));

    let result = scanner.scan_song(1234.0, song_path).await;

    assert_ok!(result.as_ref());
    assert_none!(result.unwrap().song);
}

#[rstest]
#[tokio::test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_song_scanner_lifecycle(temp_dir_fixture: TempDir) {
    let thumb_dir = temp_dir_fixture.path().join("thumbs");
    let file_list = FileList {
        file_list: vec![],
        playlist_list: vec![],
    };
    let scanner = SongScanner::new(&file_list, thumb_dir, ";".to_string(), Some(2));
    let mut scanned = 0;
    let on_song: crate::OnSongScanned = Box::new(|_pl_id, _songs| Box::pin(async {}));
    let on_progress: crate::OnProgressUpdated = Box::new(|_p: ScanProgress| {});

    assert_ok!(scanner.scan(&mut scanned, 0, &on_song, &on_progress).await);
    assert_eq!(scanned, 0);
}
