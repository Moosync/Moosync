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
    fs::File,
    io::{self, BufRead},
    path::{Path, PathBuf},
    str::FromStr,
};

use songs_proto::moosync::types::Playlist;
use substring::Substring;
use uuid::Uuid;

use crate::{FileList, PlaylistSongId, error::ScannerError};

struct PlaylistParserState {
    song_type: Option<String>,
    duration: Option<std::time::Duration>,
    title: Option<String>,
    artists: Option<String>,
    playlist_title: String,
    playlist_id: String,
    song_identifiers: Vec<PlaylistSongId>,
}

impl PlaylistParserState {
    #[tracing::instrument(level = "debug", skip_all)]
    fn new() -> Self {
        Self {
            song_type: None,
            duration: None,
            title: None,
            artists: None,
            playlist_title: "".to_string(),
            playlist_id: Uuid::new_v4().to_string(),
            song_identifiers: vec![],
        }
    }

    #[tracing::instrument(level = "debug", skip_all)]
    fn clear_metadata(&mut self) {
        self.song_type = None;
        self.duration = None;
        self.title = None;
        self.artists = None;
    }

    #[tracing::instrument(level = "debug", skip_all)]
    fn into_playlist_and_songs(self, path: &Path) -> (Playlist, Vec<PlaylistSongId>) {
        (
            Playlist {
                playlist_id: Some(self.playlist_id),
                playlist_name: self.playlist_title,
                playlist_path: Some(path.to_string_lossy().to_string()),
                ..Default::default()
            },
            self.song_identifiers,
        )
    }
}

pub struct PlaylistScanner<'a> {
    file_list: &'a FileList,
}

impl<'a> PlaylistScanner<'a> {
    #[tracing::instrument(level = "debug", skip_all)]
    pub fn new(file_list: &'a FileList) -> Self { Self { file_list } }

    #[tracing::instrument(level = "debug", skip_all)]
    pub fn scan(&self) -> Result<Vec<(Playlist, Vec<PlaylistSongId>)>, ScannerError> {
        let mut parsed_playlists = Vec::new();
        for playlist_path in &self.file_list.playlist_list {
            match self.scan_playlist(playlist_path) {
                Ok(res) => parsed_playlists.push(res),
                Err(e) => {
                    tracing::error!("Failed to scan {}: {:?}", playlist_path.display(), e);
                }
            }
        }
        Ok(parsed_playlists)
    }

    #[tracing::instrument(level = "debug", skip_all)]
    fn scan_playlist(
        &self,
        path: &PathBuf,
    ) -> Result<(Playlist, Vec<PlaylistSongId>), ScannerError> {
        let file = File::open(path).map_err(ScannerError::Io)?;
        let lines = io::BufReader::new(file).lines();
        let mut state = PlaylistParserState::new();

        for line_res in lines {
            let line = line_res.map_err(ScannerError::Io)?;
            self.parse_playlist_line(path, &line, &mut state)?;
        }
        Ok(state.into_playlist_and_songs(path))
    }

    #[tracing::instrument(level = "debug", skip_all)]
    fn parse_playlist_line(
        &self,
        path: &Path,
        line: &str,
        state: &mut PlaylistParserState,
    ) -> Result<(), ScannerError> {
        if line.starts_with("#EXTINF:") {
            self.parse_extinf(line, state)?;
        } else if line.starts_with("#MOOSINF:") {
            state.song_type = Some(line.substring(9, line.len()).to_string());
        } else if line.starts_with("#PLAYLIST:") {
            state.playlist_title = line.substring(10, line.len()).to_string();
        } else if !line.starts_with('#') && !line.is_empty() {
            self.parse_song_entry(path, line, state)?;
        }
        Ok(())
    }

    #[tracing::instrument(level = "debug", skip_all)]
    fn parse_extinf(
        &self,
        line: &str,
        state: &mut PlaylistParserState,
    ) -> Result<(), ScannerError> {
        let metadata = line.substring(8, line.len());
        let split_index = metadata.find(',').unwrap_or_default();
        let secs = metadata
            .substring(0, split_index)
            .parse::<f64>()
            .map_err(ScannerError::ParseFloat)?;
        state.duration = Some(std::time::Duration::from_secs_f64(secs));
        let non_duration = metadata.substring(split_index + 1, metadata.len());
        let mut artists_str = "";
        let title_str;
        if let Some(separator_with_space) = non_duration.find(" - ") {
            (artists_str, title_str) = non_duration.split_at(separator_with_space + 1);
        } else if let Some(separator_without_space) = non_duration.find('-') {
            (artists_str, title_str) = non_duration.split_at(separator_without_space);
        } else {
            title_str = non_duration;
        }
        state.artists = Some(artists_str.trim().to_string());
        state.title = Some(title_str.replacen('-', "", 1).trim().to_string());
        Ok(())
    }

    #[tracing::instrument(level = "debug", skip_all)]
    fn parse_song_entry(
        &self,
        path: &Path,
        line: &str,
        state: &mut PlaylistParserState,
    ) -> Result<(), ScannerError> {
        let (parsed_line, is_url) = parse_line_url_or_file(line);
        if !is_url {
            let local_path = resolve_playlist_song_path(path, &parsed_line)?;
            state
                .song_identifiers
                .push(PlaylistSongId::Path(local_path));
        } else {
            state
                .song_identifiers
                .push(PlaylistSongId::Url(parsed_line));
        }
        state.clear_metadata();
        Ok(())
    }
}

#[tracing::instrument(level = "debug", skip_all)]
fn parse_line_url_or_file(line: &str) -> (String, bool) {
    if line.starts_with("file://") {
        (line[8..].to_string(), false)
    } else if line.starts_with("http") {
        (line.replace("http://", "").replace("https://", ""), true)
    } else {
        (line.to_string(), false)
    }
}

#[tracing::instrument(level = "debug", skip_all)]
fn resolve_playlist_song_path(
    playlist_path: &Path,
    song_path: &str,
) -> Result<PathBuf, ScannerError> {
    let mut parsed_path = PathBuf::from_str(song_path).unwrap();
    if parsed_path.is_relative() {
        parsed_path = playlist_path
            .parent()
            .unwrap_or(playlist_path)
            .join(parsed_path)
            .canonicalize()
            .map_err(ScannerError::Io)?;
    }
    Ok(parsed_path)
}
