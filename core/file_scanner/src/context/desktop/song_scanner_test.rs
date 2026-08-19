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

use std::{env::temp_dir, fs, path::PathBuf, sync::Mutex};

use uuid::Uuid;

use crate::{
    FileList, ScanProgress,
    context::desktop::song_scanner::{SongScanner, check_directory},
};

#[tracing::instrument(level = "debug", skip_all)]
fn get_test_dir() -> PathBuf {
    let dir = temp_dir().join(format!("moosync_song_scan_test_{}", Uuid::new_v4()));
    fs::create_dir_all(&dir).unwrap();
    dir
}

#[test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_check_directory_creates_dir_if_missing() {
    let dir = temp_dir().join(format!("moosync_chk_dir_{}", Uuid::new_v4()));
    assert!(!dir.exists());

    let res = check_directory(dir.clone());
    assert!(res.is_ok());
    assert!(dir.exists());

    let _ = fs::remove_dir_all(dir);
}

#[tokio::test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_song_scanner_scan_song_non_audio() {
    let dir = get_test_dir();
    let thumb_dir = dir.join("thumbs");
    let song_path = dir.join("not_audio.mp3");
    fs::write(&song_path, b"fake data").unwrap();

    let file_list = FileList {
        file_list: vec![(song_path.clone(), 1234.0)],
        playlist_list: vec![],
    };

    let scanner = SongScanner::new(&file_list, thumb_dir.clone(), ";".to_string(), Some(1));
    let result = scanner.scan_song(1234.0, song_path.clone()).await;

    assert!(result.is_ok());
    assert!(result.unwrap().song.is_none());

    let _ = fs::remove_dir_all(dir);
}

#[tokio::test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_song_scanner_lifecycle() {
    let dir = get_test_dir();
    let thumb_dir = dir.join("thumbs");

    let file_list = FileList {
        file_list: vec![],
        playlist_list: vec![],
    };

    let scanner = SongScanner::new(&file_list, thumb_dir.clone(), ";".to_string(), Some(2));
    let mut scanned = 0;
    let progress_events = std::sync::Arc::new(Mutex::new(Vec::new()));
    let p_clone = progress_events.clone();

    let on_song: crate::OnSongScanned = Box::new(|_pl_id, _songs| Box::pin(async {}));
    let on_progress: crate::OnProgressUpdated = Box::new(move |p: ScanProgress| {
        p_clone.lock().unwrap().push(p);
    });

    let res = scanner.scan(&mut scanned, 0, &on_song, &on_progress).await;
    assert!(res.is_ok());
    assert_eq!(scanned, 0);

    let _ = fs::remove_dir_all(dir);
}
