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

use std::{env::temp_dir, fs, path::PathBuf};

use uuid::Uuid;

use crate::context::desktop::{DesktopScannerContext, get_files_recursively};

#[tracing::instrument(level = "debug", skip_all)]
fn get_test_dir() -> PathBuf {
    let dir = temp_dir().join(format!("moosync_desk_test_{}", Uuid::new_v4()));
    fs::create_dir_all(&dir).unwrap();
    dir
}

#[test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_get_files_recursively_filters_and_excludes() {
    let dir = get_test_dir();
    let sub1 = dir.join("included");
    let sub2 = dir.join("excluded");
    fs::create_dir_all(&sub1).unwrap();
    fs::create_dir_all(&sub2).unwrap();

    let song1 = sub1.join("song1.mp3");
    let playlist1 = sub1.join("pl.m3u");
    let txt1 = sub1.join("notes.txt");
    let song2 = sub2.join("song2.flac");

    fs::write(&song1, b"mp3").unwrap();
    fs::write(&playlist1, b"#EXTM3U").unwrap();
    fs::write(&txt1, b"hello").unwrap();
    fs::write(&song2, b"flac").unwrap();

    let res = get_files_recursively(dir.clone(), std::slice::from_ref(&sub2)).unwrap();
    assert_eq!(res.file_list.len(), 1);
    assert_eq!(res.file_list[0].0, song1);
    assert_eq!(res.playlist_list.len(), 1);
    assert_eq!(res.playlist_list[0], playlist1);

    let _ = fs::remove_dir_all(dir);
}

#[tokio::test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_desktop_scanner_context_start_scan() {
    let dir = get_test_dir();
    let thumb_dir = dir.join("thumbs");
    let scan_dir = dir.join("music");
    fs::create_dir_all(&scan_dir).unwrap();

    let ctx = DesktopScannerContext::new(
        vec![scan_dir.clone()],
        thumb_dir,
        ";".to_string(),
        vec![],
        Some(2),
    );

    use crate::context::ScannerContext;

    let on_song: crate::OnSongScanned = Box::new(|_pl_id, _songs| Box::pin(async {}));
    let on_playlist: crate::OnPlaylistScanned = Box::new(|_pls| Box::pin(async {}));
    let on_progress: crate::OnProgressUpdated = Box::new(|_p| {});

    let res = ctx.start_scan(&on_song, &on_playlist, &on_progress).await;
    assert!(res.is_ok());

    let _ = fs::remove_dir_all(dir);
}
