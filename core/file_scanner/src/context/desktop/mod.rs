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

use lazy_static::lazy_static;
use regex::Regex;
use types::errors::{Result, error_helpers};

use crate::{FileList, OnPlaylistScanned, OnProgressUpdated, OnSongScanned, ScanProgress};

pub mod image_processor;
pub mod lyrics_scanner;
pub mod playlist_scanner;
pub mod song_scanner;

use self::{playlist_scanner::PlaylistScanner, song_scanner::SongScanner};

#[tracing::instrument(level = "debug", skip(dir, exclude_dirs))]
pub fn get_files_recursively(dir: PathBuf, exclude_dirs: &[PathBuf]) -> Result<FileList> {
    tracing::trace!("Scanning dir {:?}", dir);
    let mut file_list = vec![];
    let mut playlist_list = vec![];
    if exclude_dirs.iter().any(|ex| dir.starts_with(ex)) {
        return Ok(FileList {
            file_list,
            playlist_list,
        });
    }
    if !dir.exists() {
        return Ok(FileList {
            file_list,
            playlist_list,
        });
    }
    if dir.is_file() {
        process_single_file(dir, &mut file_list, &mut playlist_list)?;
    } else if dir.is_dir() {
        process_directory(dir, exclude_dirs, &mut file_list, &mut playlist_list)?;
    }
    Ok(FileList {
        file_list,
        playlist_list,
    })
}

fn process_single_file(
    path: PathBuf,
    files: &mut Vec<(PathBuf, f64)>,
    playlists: &mut Vec<PathBuf>,
) -> Result<()> {
    lazy_static! {
        static ref SONG_RE: Regex = Regex::new("flac|mp3|ogg|m4a|webm|wav|wv|aac|opus").unwrap();
        static ref PLAYLIST_RE: Regex = Regex::new("m3u|m3u8").unwrap();
    }
    if let Ok(metadata) = fs::metadata(&path) {
        let extension = path
            .extension()
            .unwrap_or_default()
            .to_str()
            .unwrap_or_default();
        if !extension.is_empty() {
            if SONG_RE.is_match(extension) {
                files.push((path.clone(), metadata.len() as f64));
            }
            if PLAYLIST_RE.is_match(extension) {
                playlists.push(path);
            }
        }
    }
    Ok(())
}

fn process_directory(
    path: PathBuf,
    exclude_dirs: &[PathBuf],
    files: &mut Vec<(PathBuf, f64)>,
    playlists: &mut Vec<PathBuf>,
) -> Result<()> {
    let dir_entries = fs::read_dir(path).map_err(error_helpers::to_file_system_error)?;
    for entry in dir_entries {
        if let Ok(entry) = entry {
            let res = get_files_recursively(entry.path(), exclude_dirs)?;
            files.extend(res.file_list);
            playlists.extend(res.playlist_list);
        }
    }
    Ok(())
}

pub struct DesktopScannerContext {
    scan_dirs: Vec<PathBuf>,
    thumbnail_dir: PathBuf,
    artist_split: String,
    exclude_dirs: Vec<PathBuf>,
    scan_threads: Option<i32>,
}

impl DesktopScannerContext {
    pub fn new(
        scan_dirs: Vec<PathBuf>,
        thumbnail_dir: PathBuf,
        artist_split: String,
        exclude_dirs: Vec<PathBuf>,
        scan_threads: Option<i32>,
    ) -> Self {
        Self {
            scan_dirs,
            thumbnail_dir,
            artist_split,
            exclude_dirs,
            scan_threads,
        }
    }
}

impl super::ScannerContext for DesktopScannerContext {
    async fn start_scan(
        &self,
        on_song: &OnSongScanned,
        on_playlist: &OnPlaylistScanned,
        on_progress: &OnProgressUpdated,
    ) -> Result<()> {
        let mut file_list = FileList {
            file_list: Vec::new(),
            playlist_list: Vec::new(),
        };
        for dir in &self.scan_dirs {
            let res = get_files_recursively(dir.clone(), &self.exclude_dirs)?;
            file_list.file_list.extend(res.file_list);
            file_list.playlist_list.extend(res.playlist_list);
        }

        let song_scanner = SongScanner::new(
            &file_list,
            self.thumbnail_dir.clone(),
            self.artist_split.clone(),
            self.scan_threads,
        );
        let playlist_scanner = PlaylistScanner::new(&file_list);

        let total_songs = file_list.file_list.len();
        on_progress(ScanProgress::PROGRESS(0));
        let mut scanned_count = 0;

        song_scanner
            .scan(&mut scanned_count, total_songs, on_song, on_progress)
            .await?;

        let parsed_playlists = playlist_scanner.scan()?;
        on_playlist(parsed_playlists).await;

        on_progress(ScanProgress::STOPPED);
        Ok(())
    }
}
