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

use crate::{
    song_scanner::SongScanner,
    utils::{check_directory, get_files_recursively},
};
use songs_proto::moosync::types::{Artist, InnerSong, Playlist, Song, SongType};
use std::{
    fs::{self, File},
    io::{self, BufRead},
    path::PathBuf,
    str::FromStr,
    sync::mpsc::Sender,
};
use substring::Substring;
use types::errors::error_helpers;
use types::errors::{MoosyncError, Result};
use uuid::Uuid;

pub struct PlaylistScanner<'a> {
    dir: PathBuf,
    song_scanner: SongScanner<'a>,
    thumbnail_dir: PathBuf,
}

impl<'a> PlaylistScanner<'a> {
    #[tracing::instrument(level = "debug", skip(dir, thumbnail_dir, song_scanner))]
    pub fn new(dir: PathBuf, thumbnail_dir: PathBuf, song_scanner: SongScanner<'a>) -> Self {
        Self {
            dir,
            thumbnail_dir,
            song_scanner,
        }
    }

    #[tracing::instrument(level = "debug", skip(self))]
    fn check_dirs(&self) -> Result<()> {
        check_directory(self.thumbnail_dir.clone())
    }

    #[tracing::instrument(level = "debug", skip(self, artists))]
    fn parse_artists(&self, artists: Option<String>) -> Vec<Artist> {
        let mut ret: Vec<Artist> = vec![];
        if let Some(artists) = artists {
            for artist in artists.split(';') {
                ret.push(Artist {
                    artist_id: Some(Uuid::new_v4().to_string()),
                    artist_name: Some(artist.to_string()),
                    ..Default::default()
                })
            }
        }
        ret
    }

    #[tracing::instrument(level = "debug", skip(self, path))]
    fn scan_playlist(&self, path: &PathBuf) -> Result<(Playlist, Vec<Song>)> {
        let file = File::open(path).map_err(error_helpers::to_file_system_error)?;
        let lines = io::BufReader::new(file).lines();

        let mut songs: Vec<Song> = vec![];

        let mut song_type: Option<String> = None;
        let mut duration: Option<f64> = None;
        let mut title: Option<String> = None;
        let mut artists: Option<String> = None;
        let mut playlist_title: String = "".to_string();

        let playlist_id = Uuid::new_v4().to_string();
        for line_res in lines {
            let mut line = line_res.unwrap();
            if line.starts_with("#EXTINF:") {
                let metadata = line.substring(8, line.len());
                let split_index = metadata.find(',').unwrap_or_default();

                duration = Some(
                    metadata
                        .substring(0, split_index)
                        .parse::<f64>()
                        .map_err(error_helpers::to_parse_error)?,
                );

                let non_duration = metadata.substring(split_index + 1, metadata.len());

                let mut artists_str = "";
                let title_str;

                let separator_with_space = non_duration.find(" - ");
                if let Some(separator_with_space) = separator_with_space {
                    (artists_str, title_str) = non_duration.split_at(separator_with_space + 1);
                } else {
                    let separator_without_space = non_duration.find('-');
                    if let Some(separator_without_space) = separator_without_space {
                        (artists_str, title_str) = non_duration.split_at(separator_without_space);
                    } else {
                        title_str = non_duration;
                    }
                }

                artists = Some(artists_str.trim().to_string());
                title = Some(title_str.replacen('-', "", 1).trim().to_string());

                continue;
            }

            if line.starts_with("#MOOSINF:") {
                song_type = Some(line.substring(9, line.len()).to_string());
                continue;
            }

            if line.starts_with("#PLAYLIST:") {
                playlist_title = line.substring(10, line.len()).to_string();
                continue;
            }

            if !line.starts_with('#') {
                if line.starts_with("file://") {
                    line = line[8..].to_string();
                } else if line.starts_with("http") {
                    line = line.replace("http://", "").replace("https://", "");
                    song_type = Some("URL".to_string());
                } else if !line.is_empty() {
                    // pass
                } else {
                    continue;
                }

                let mut song = InnerSong::default();

                let s_type = song_type.clone();

                song.r#type = SongType::from_str_name(s_type.unwrap_or_default().as_str())
                    .unwrap_or(SongType::Local)
                    .into();
                song.id = Some(Uuid::new_v4().to_string());

                if SongType::try_from(song.r#type).unwrap() == SongType::Local {
                    let song_path = PathBuf::from_str(line.as_str());
                    let Ok(mut path_parsed) = song_path;
                    if path_parsed.is_relative() {
                        path_parsed = path
                            .parent()
                            .unwrap()
                            .join(path_parsed)
                            .canonicalize()
                            .map_err(error_helpers::to_file_system_error)?;
                    }

                    if !path_parsed.exists() {
                        artists = None;
                        duration = None;
                        title = None;
                        song_type = None;
                        continue;
                    }

                    let metadata =
                        fs::metadata(&path_parsed).map_err(error_helpers::to_file_system_error)?;
                    song.size = Some(metadata.len() as f64);
                    song.path = Some(path_parsed.to_string_lossy().to_string());

                    if song.path.is_none() {
                        song.path = Some(line);
                    }

                    song.playback_url = None;
                } else {
                    song.playback_url = Some(line);
                }

                // song.artists = ;
                song.duration = duration;
                song.title = title;
                // song.playlist_id = Some(playlist_id.clone());
                songs.push(Song {
                    song: Some(song),
                    album: None,
                    artists: self.parse_artists(artists),
                    genre: vec![],
                });

                artists = None;
                duration = None;
                title = None;
                song_type = None;
            }
        }

        Ok((
            Playlist {
                playlist_id: Some(playlist_id),
                playlist_name: playlist_title,
                playlist_path: Some(path.to_string_lossy().to_string()),
                ..Default::default()
            },
            songs,
        ))
    }

    #[tracing::instrument(level = "debug", skip(self, tx_song, s, playlist_id))]
    fn scan_song_in_pool(
        &self,
        tx_song: Sender<(Option<String>, Result<Song>)>,
        s: Song,
        playlist_id: Option<String>,
    ) {
        if let Some(song) = s.song.as_ref()
            && SongType::try_from(song.r#type).unwrap() == SongType::Local
            && let Some(path) = song.path.as_ref()
        {
            self.song_scanner.scan_in_pool(
                tx_song,
                song.size.unwrap_or_default(),
                PathBuf::from_str(path.as_str()).unwrap(),
                playlist_id,
            )
        } else {
            tx_song
                .send((playlist_id, Ok(s)))
                .expect("channel will be there waiting for the pool");
        }
    }

    #[tracing::instrument(level = "debug", skip(self, tx_song, tx_playlist))]
    pub fn start(
        &self,
        tx_song: Sender<(Option<String>, Result<Song>)>,
        tx_playlist: Sender<Result<Playlist>>,
    ) -> Result<usize> {
        self.check_dirs()?;

        let file_list = get_files_recursively(self.dir.clone())?;

        let mut len = 0;

        for playlist in file_list.playlist_list {
            let playlist_scan_res = self.scan_playlist(&playlist);
            if let Err(e) = playlist_scan_res {
                tx_playlist
                    .send(Err(MoosyncError::String(format!(
                        "Failed to scan {}: {:?}",
                        playlist.display(),
                        e
                    ))))
                    .expect("channel will be there waiting for the pool");
                continue;
            }

            let (playlist_dets, songs) = playlist_scan_res.unwrap();
            tx_playlist
                .send(Ok(playlist_dets.clone()))
                .expect("channel will be there waiting for the pool");

            len += songs.len();

            for s in songs {
                self.scan_song_in_pool(tx_song.clone(), s, playlist_dets.playlist_id.clone());
            }
            continue;
        }

        drop(tx_song);
        drop(tx_playlist);

        Ok(len)
    }
}
