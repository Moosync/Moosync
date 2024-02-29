use std::{
    fs::{self, File},
    io::{self, BufRead},
    path::PathBuf,
    str::FromStr,
    sync::mpsc::Sender,
};

use types::{
    entities::{QueryableArtist, QueryablePlaylist},
    songs::{QueryableSong, Song, SongType},
};

use substring::Substring;
use types::errors::errors::{MoosyncError, Result};

use uuid::Uuid;

use crate::{
    song_scanner::SongScanner,
    utils::{check_directory, get_files_recursively},
};

pub struct PlaylistScanner<'a> {
    dir: PathBuf,
    song_scanner: SongScanner<'a>,
    thumbnail_dir: PathBuf,
}

impl<'a> PlaylistScanner<'a> {
    pub fn new(dir: PathBuf, thumbnail_dir: PathBuf, song_scanner: SongScanner<'a>) -> Self {
        Self {
            dir,
            thumbnail_dir,
            song_scanner,
        }
    }

    fn check_dirs(&self) -> Result<()> {
        check_directory(self.thumbnail_dir.clone())
    }

    fn parse_artists(&self, artists: Option<String>) -> Vec<QueryableArtist> {
        let mut ret: Vec<QueryableArtist> = vec![];
        if artists.is_some() {
            for artist in artists.unwrap().split(';') {
                ret.push(QueryableArtist {
                    artist_id: Some(Uuid::new_v4().to_string()),
                    artist_name: Some(artist.to_string()),
                    ..Default::default()
                })
            }
        }
        ret
    }

    fn scan_playlist(&self, path: &PathBuf) -> Result<(QueryablePlaylist, Vec<Song>)> {
        let file = File::open(path)?;
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

                duration = Some(metadata.substring(0, split_index).parse::<f64>()?);

                let non_duration = metadata.substring(split_index + 1, metadata.len());

                let mut artists_str = "";
                let title_str;

                let separator_with_space = non_duration.find(" - ");
                if separator_with_space.is_some() {
                    (artists_str, title_str) =
                        non_duration.split_at(separator_with_space.unwrap() + 1);
                } else {
                    let separator_without_space = non_duration.find('-');
                    if separator_without_space.is_some() {
                        (artists_str, title_str) =
                            non_duration.split_at(separator_without_space.unwrap());
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
                } else {
                    continue;
                }

                let mut song = QueryableSong::empty();

                let s_type = song_type.clone();

                song.type_ = SongType::from_str(s_type.unwrap_or("LOCAL".to_string()).as_str())?;
                song._id = Some(Uuid::new_v4().to_string());

                if song.type_ == SongType::LOCAL {
                    let song_path = PathBuf::from_str(line.as_str());
                    if let Ok(mut path_parsed) = song_path {
                        if path_parsed.is_relative() {
                            path_parsed =
                                path.parent().unwrap().join(path_parsed).canonicalize()?;
                        }

                        if !path_parsed.exists() {
                            artists = None;
                            duration = None;
                            title = None;
                            song_type = None;
                            continue;
                        }

                        let metadata = fs::metadata(&path_parsed)?;
                        song.size = Some(metadata.len() as f64);
                        song.path = Some(path_parsed.to_string_lossy().to_string());
                    }

                    if song.path.is_none() {
                        song.path = Some(line);
                    }

                    song.playback_url = None;
                } else {
                    song._id = Some(format!("{}:{}", song.type_, line));
                    song.playback_url = Some(line);
                }

                // song.artists = ;
                song.duration = duration;
                song.title = title;
                // song.playlist_id = Some(playlist_id.clone());
                songs.push(Song {
                    song,
                    album: None,
                    artists: Some(self.parse_artists(artists)),
                    genre: Some(vec![]),
                });

                artists = None;
                duration = None;
                title = None;
                song_type = None;
            }
        }

        Ok((
            QueryablePlaylist {
                playlist_id: Some(playlist_id),
                playlist_name: playlist_title,
                playlist_path: Some(path.to_string_lossy().to_string()),
                ..Default::default()
            },
            songs,
        ))
    }

    fn scan_song_in_pool(
        &self,
        tx_song: Sender<(Option<String>, Result<Song>)>,
        s: Song,
        playlist_id: Option<String>,
    ) {
        if s.song.type_ == SongType::LOCAL && s.song.path.is_some() {
            self.song_scanner.scan_in_pool(
                tx_song,
                s.song.size.unwrap_or_default(),
                PathBuf::from_str(s.song.path.unwrap().as_str()).unwrap(),
                playlist_id,
            )
        } else {
            tx_song
                .send((playlist_id, Ok(s)))
                .expect("channel will be there waiting for the pool");
        }
    }

    pub fn start(
        &self,
        tx_song: Sender<(Option<String>, Result<Song>)>,
        tx_playlist: Sender<Result<QueryablePlaylist>>,
    ) -> Result<usize> {
        self.check_dirs()?;

        let file_list = get_files_recursively(self.dir.clone())?;

        let mut len = 0;

        for playlist in file_list.playlist_list {
            let playlist_scan_res = self.scan_playlist(&playlist);
            if playlist_scan_res.is_err() {
                tx_playlist
                    .send(Err(MoosyncError::String(format!(
                        "Failed to scan {}: {:?}",
                        playlist.display(),
                        playlist_scan_res.unwrap_err()
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
