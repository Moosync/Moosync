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
    fs,
    path::{Path, PathBuf},
};

use lofty::{
    file::TaggedFile,
    prelude::{Accessor, AudioFile, TaggedFileExt},
    probe::Probe,
    read_from_path,
    tag::Tag,
};
use songs_proto::{
    duration_proto::google::protobuf::Duration,
    moosync::types::{Album, Artist, Genre, InnerSong, Song, SongType},
};
use types::prelude::core_to_proto_duration;
use uuid::Uuid;

use crate::{
    FileList, OnProgressUpdated, OnSongScanned, ScanProgress,
    context::desktop::{image_processor::ImageProcessor, lyrics_scanner::LyricsScanner},
    error::ScannerError,
};

#[tracing::instrument(level = "debug", skip_all)]
pub fn check_directory(dir: PathBuf) -> Result<(), ScannerError> {
    if !dir.is_dir() {
        fs::create_dir_all(dir).map_err(ScannerError::Io)?;
    }
    Ok(())
}

#[tracing::instrument(level = "debug", skip_all)]
fn read_tagged_file(path: &PathBuf, guess: bool) -> Result<TaggedFile, ScannerError> {
    if guess {
        Ok(read_from_path(path.clone())?)
    } else {
        Ok(Probe::open(path.clone())
            .map_err(ScannerError::AudioMeta)?
            .guess_file_type()
            .map_err(ScannerError::Io)?
            .read()?)
    }
}

#[tracing::instrument(level = "debug", skip_all)]
fn extract_audio_properties(file: &TaggedFile, inner_song: &mut InnerSong) {
    let properties = file.properties();
    inner_song.bitrate = Some((properties.audio_bitrate().unwrap_or_default() * 1000) as f64);
    inner_song.sample_rate = properties.sample_rate().map(|v| v as f64);
    inner_song.duration = Some(core_to_proto_duration(properties.duration()));
}

#[tracing::instrument(level = "debug", skip_all)]
fn scan_directory_for_cover(path: &Path) -> Option<String> {
    let mut base_path = path.to_path_buf();
    base_path.pop();
    let Ok(files) = base_path.read_dir() else {
        return None;
    };
    for dir_entry in files.flatten() {
        let file_name = dir_entry
            .path()
            .file_stem()
            .unwrap_or_default()
            .to_string_lossy()
            .to_lowercase();
        if file_name.starts_with("cover") {
            return Some(dir_entry.path().to_string_lossy().to_string());
        }
    }
    None
}

#[tracing::instrument(level = "debug", skip_all)]
async fn store_picture(
    thumbnail_dir: &Path,
    picture: &lofty::picture::Picture,
) -> Result<(PathBuf, PathBuf), ScannerError> {
    let data = picture.data().to_vec();
    let hash = blake3::hash(&data).to_hex();
    let hash_str = hash.as_str();
    let low_path = thumbnail_dir.join(format!("{}-low.png", hash_str));
    let high_path = thumbnail_dir.join(format!("{}.png", hash_str));

    if !high_path.exists() {
        let d = data.clone();
        let hp = high_path.clone();
        tokio::task::spawn_blocking(move || {
            ImageProcessor::new(&d).resize(400).compress().save(&hp)
        })
        .await
        .map_err(|e| ScannerError::Join(e))??;
    }
    if !low_path.exists() {
        let lp = low_path.clone();
        tokio::task::spawn_blocking(move || {
            ImageProcessor::new(&data).resize(80).compress().save(&lp)
        })
        .await
        .map_err(|e| ScannerError::Join(e))??;
    }
    Ok((
        dunce::canonicalize(high_path).map_err(ScannerError::Io)?,
        dunce::canonicalize(low_path).map_err(ScannerError::Io)?,
    ))
}

#[tracing::instrument(level = "debug", skip_all)]
async fn extract_cover_art(
    metadata: &Tag,
    path: &Path,
    thumbnail_dir: &Path,
    inner_song: &mut InnerSong,
) {
    let Some(picture) = metadata.pictures().first() else {
        inner_song.song_cover_path_high = scan_directory_for_cover(path);
        return;
    };

    match store_picture(thumbnail_dir, picture).await {
        Ok((high_path, low_path)) => {
            inner_song.song_cover_path_high = Some(high_path.to_string_lossy().to_string());
            inner_song.song_cover_path_low = Some(low_path.to_string_lossy().to_string());
        }
        Err(e) => {
            tracing::error!("Error storing picture {:?}", e);
        }
    }
}

#[tracing::instrument(level = "debug", skip_all)]
fn parse_artists_string(artist_str: &str, artist_split: &str) -> Vec<Artist> {
    artist_str
        .split(artist_split)
        .map(|s| Artist {
            artist_id: Some(Uuid::new_v4().to_string()),
            artist_name: Some(s.trim().to_string()),
            ..Default::default()
        })
        .collect()
}

#[tracing::instrument(level = "debug", skip_all)]
fn extract_album(metadata: &Tag, inner_song: &InnerSong) -> Option<Album> {
    let album = metadata.album()?;
    Some(Album {
        album_id: Some(Uuid::new_v4().to_string()),
        album_name: Some(album.to_string()),
        album_coverpath_high: inner_song.song_cover_path_high.clone(),
        album_coverpath_low: inner_song.song_cover_path_low.clone(),
        album_artist: metadata
            .get_string(&lofty::prelude::ItemKey::AlbumArtist)
            .map(|s| s.to_owned()),
        ..Default::default()
    })
}

#[tracing::instrument(level = "debug", skip_all)]
fn extract_metadata(
    metadata: &Tag,
    path: &PathBuf,
    artist_split: &str,
    song: &mut Song,
    inner_song: &mut InnerSong,
) {
    inner_song.lyrics = LyricsScanner::extract_lyrics(metadata, path);
    inner_song.title = metadata
        .title()
        .map(|s| s.to_string())
        .or_else(|| path.file_name().map(|s| s.to_string_lossy().to_string()));
    song.artists = metadata
        .artist()
        .map(|s| parse_artists_string(&s, artist_split))
        .unwrap_or_default();
    if metadata.album().is_some() {
        inner_song.track_no = metadata
            .get_string(&lofty::prelude::ItemKey::TrackNumber)
            .and_then(|s| s.parse().ok());
        song.album = extract_album(metadata, inner_song);
    }
    inner_song.year = metadata.year().map(|s| s.to_string());
    song.genre = metadata
        .genre()
        .map(|s| {
            vec![Genre {
                genre_name: Some(s.to_string()),
                ..Default::default()
            }]
        })
        .unwrap_or_default();
}

#[tracing::instrument(level = "debug", skip_all)]
pub async fn scan_file(
    path: &PathBuf,
    thumbnail_dir: &Path,
    size: f64,
    guess: bool,
    artist_split: &str,
) -> Result<Song, ScannerError> {
    let mut inner_song = InnerSong {
        id: Some(Uuid::new_v4().to_string()),
        title: Some(path.file_name().unwrap().to_string_lossy().to_string()),
        path: Some(
            dunce::canonicalize(path)
                .map_err(ScannerError::Io)?
                .to_string_lossy()
                .to_string(),
        ),
        size: Some(size),
        duration: Some(Duration::default()),
        r#type: SongType::Local.into(),
        ..Default::default()
    };
    let mut song = Song {
        song: None,
        album: None,
        artists: vec![],
        genre: vec![],
    };
    let file = match read_tagged_file(path, guess) {
        Ok(f) => f,
        Err(_) => {
            return Ok(song);
        }
    };
    extract_audio_properties(&file, &mut inner_song);
    let mut tags = file.primary_tag();
    if tags.is_none() {
        tags = file.first_tag();
    }
    if let Some(metadata) = tags {
        extract_cover_art(metadata, path, thumbnail_dir, &mut inner_song).await;
        extract_metadata(metadata, path, artist_split, &mut song, &mut inner_song);
    }
    song.song = Some(inner_song);
    Ok(song)
}

pub struct SongScanner<'a> {
    file_list: &'a FileList,
    thumbnail_dir: PathBuf,
    artist_split: String,
    scan_threads: Option<i32>,
}

impl<'a> SongScanner<'a> {
    #[tracing::instrument(level = "debug", skip_all)]
    pub fn new(
        file_list: &'a FileList,
        thumbnail_dir: PathBuf,
        artist_split: String,
        scan_threads: Option<i32>,
    ) -> Self {
        Self {
            file_list,
            thumbnail_dir,
            artist_split,
            scan_threads,
        }
    }

    #[tracing::instrument(level = "debug", skip_all)]
    fn check_dirs(&self) -> Result<(), ScannerError> {
        check_directory(self.thumbnail_dir.clone())?;
        Ok(())
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub async fn scan_song(&self, size: f64, path: PathBuf) -> Result<Song, ScannerError> {
        self.check_dirs()?;
        let thumbnail_dir = self.thumbnail_dir.clone();
        let artist_split = self.artist_split.clone();
        let mut metadata = scan_file(&path, &thumbnail_dir, size, false, &artist_split).await;
        if metadata.is_err() {
            metadata = scan_file(&path, &thumbnail_dir, size, true, &artist_split).await;
        }
        metadata
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub async fn scan(
        &self,
        scanned_count: &mut usize,
        total_songs: usize,
        on_song: &OnSongScanned,
        on_progress: &OnProgressUpdated,
    ) -> Result<(), ScannerError> {
        let batch_size = self.scan_threads.unwrap_or(4) as usize;
        let mut scan_futures = Vec::new();
        for (file_path, size) in &self.file_list.file_list {
            scan_futures.push(self.scan_song(*size, file_path.clone()));
            if scan_futures.len() >= batch_size {
                let results = futures::future::join_all(scan_futures).await;
                scan_futures = Vec::new();
                self.process_scan_results(
                    results,
                    total_songs,
                    scanned_count,
                    on_song,
                    on_progress,
                )
                .await;
            }
        }
        if !scan_futures.is_empty() {
            let results = futures::future::join_all(scan_futures).await;
            self.process_scan_results(results, total_songs, scanned_count, on_song, on_progress)
                .await;
        }
        Ok(())
    }

    #[tracing::instrument(level = "debug", skip_all)]
    async fn process_scan_results(
        &self,
        results: Vec<Result<Song, ScannerError>>,
        total_songs: usize,
        scanned_count: &mut usize,
        on_song: &OnSongScanned,
        on_progress: &OnProgressUpdated,
    ) {
        for res in results {
            if let Ok(song) = res {
                tracing::info!("Scanned song {:?}", song);
                on_song(None, vec![song]).await;
            } else if let Err(e) = res {
                tracing::error!("Scan error: {:?}", e);
            }
            *scanned_count += 1;
            update_scan_progress(total_songs, *scanned_count, on_progress);
        }
    }
}

#[tracing::instrument(level = "debug", skip_all)]
fn update_scan_progress(total: usize, current: usize, on_progress: &OnProgressUpdated) {
    if total > 0 {
        let progress = ((current * 100) / total) as u8;
        on_progress(ScanProgress::PROGRESS(progress));
    }
}
