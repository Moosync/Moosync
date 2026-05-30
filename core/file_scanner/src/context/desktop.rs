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

use fast_image_resize::{self as fr, FilterType, ResizeAlg::Convolution, ResizeOptions};
use image::ColorType;
use lazy_static::lazy_static;
use lofty::{
    file::TaggedFile, picture::Picture, prelude::Accessor, prelude::AudioFile,
    prelude::TaggedFileExt, probe::Probe, read_from_path, tag::Tag,
};
use regex::Regex;
use std::{
    fs::{self, File},
    io::{self, BufRead},
    num::NonZeroU32,
    path::{Path, PathBuf},
    str::FromStr,
};
use substring::Substring;
use uuid::Uuid;

use crate::{FileList, OnPlaylistScanned, OnProgressUpdated, OnSongScanned, ScanProgress};
use songs_proto::moosync::types::{Album, Artist, Genre, InnerSong, Playlist, Song, SongType};
use types::errors::{MoosyncError, Result, error_helpers};

// ==========================================
// Directory Utilities
// ==========================================

#[tracing::instrument(level = "debug", skip(dir))]
pub fn check_directory(dir: PathBuf) -> Result<()> {
    if !dir.is_dir() {
        fs::create_dir_all(dir).map_err(error_helpers::to_file_system_error)?;
    }
    Ok(())
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
    files: &mut Vec<(PathBuf, f64)>,
    playlists: &mut Vec<PathBuf>,
) -> Result<()> {
    let dir_entries = fs::read_dir(path).map_err(error_helpers::to_file_system_error)?;
    for entry in dir_entries {
        if let Ok(entry) = entry {
            let res = get_files_recursively(entry.path())?;
            files.extend(res.file_list);
            playlists.extend(res.playlist_list);
        }
    }
    Ok(())
}

#[tracing::instrument(level = "debug", skip(dir))]
pub fn get_files_recursively(dir: PathBuf) -> Result<FileList> {
    let mut file_list = vec![];
    let mut playlist_list = vec![];
    if !dir.exists() {
        return Ok(FileList {
            file_list,
            playlist_list,
        });
    }
    if dir.is_file() {
        process_single_file(dir, &mut file_list, &mut playlist_list)?;
    } else {
        process_directory(dir, &mut file_list, &mut playlist_list)?;
    }
    Ok(FileList {
        file_list,
        playlist_list,
    })
}

// ==========================================
// Image Processing (Sync helpers)
// ==========================================

fn to_src_image(img: &image::DynamicImage) -> Result<fr::images::Image<'static>> {
    let width =
        NonZeroU32::new(img.width()).ok_or_else(|| MoosyncError::String("Zero width".into()))?;
    let height =
        NonZeroU32::new(img.height()).ok_or_else(|| MoosyncError::String("Zero height".into()))?;
    fr::images::Image::from_vec_u8(
        width.into(),
        height.into(),
        img.to_rgba8().into_vec(),
        fr::PixelType::U8x4,
    )
    .map_err(error_helpers::to_media_error)
}

fn resize_image(
    src_image: &fr::images::Image,
    dimensions: u32,
) -> Result<fr::images::Image<'static>> {
    let dst_width =
        NonZeroU32::new(dimensions).ok_or_else(|| MoosyncError::String("Zero width".into()))?;
    let dst_height =
        NonZeroU32::new(dimensions).ok_or_else(|| MoosyncError::String("Zero height".into()))?;
    let mut dst_image =
        fr::images::Image::new(dst_width.into(), dst_height.into(), src_image.pixel_type());
    let mut resizer = fr::Resizer::new();
    resizer
        .resize(
            src_image,
            &mut dst_image,
            Some(&ResizeOptions {
                algorithm: Convolution(FilterType::Hamming),
                mul_div_alpha: false,
                ..Default::default()
            }),
        )
        .map_err(error_helpers::to_media_error)?;
    Ok(dst_image)
}

fn save_image_buffer(path: &Path, dst_image: &fr::images::Image, dimensions: u32) -> Result<()> {
    let dst_width = NonZeroU32::new(dimensions)
        .ok_or_else(|| MoosyncError::String("Zero dimensions".into()))?;
    let dst_height = NonZeroU32::new(dimensions)
        .ok_or_else(|| MoosyncError::String("Zero dimensions".into()))?;
    image::save_buffer(
        path,
        dst_image.buffer(),
        dst_width.get(),
        dst_height.get(),
        ColorType::Rgba8,
    )
    .map_err(error_helpers::to_media_error)
}

fn generate_image(data: &[u8], path: &Path, dimensions: u32) -> Result<()> {
    let img = image::load_from_memory(data).map_err(error_helpers::to_media_error)?;
    let src_image = to_src_image(&img)?;
    let dst_image = resize_image(&src_image, dimensions)?;
    save_image_buffer(path, &dst_image, dimensions)
}

#[tracing::instrument(level = "debug", skip(thumbnail_dir, picture))]
async fn store_picture(thumbnail_dir: &Path, picture: &Picture) -> Result<(PathBuf, PathBuf)> {
    let data = picture.data().to_vec();
    let hash = blake3::hash(&data).to_hex();
    let hash_str = hash.as_str();
    let low_path = thumbnail_dir.join(format!("{}-low.png", hash_str));
    let high_path = thumbnail_dir.join(format!("{}.png", hash_str));

    if !high_path.exists() {
        let d = data.clone();
        let hp = high_path.clone();
        tokio::task::spawn_blocking(move || generate_image(&d, &hp, 400))
            .await
            .map_err(|e| MoosyncError::String(e.to_string()))??;
    }
    if !low_path.exists() {
        let lp = low_path.clone();
        tokio::task::spawn_blocking(move || generate_image(&data, &lp, 80))
            .await
            .map_err(|e| MoosyncError::String(e.to_string()))??;
    }
    Ok((
        dunce::canonicalize(high_path).map_err(error_helpers::to_file_system_error)?,
        dunce::canonicalize(low_path).map_err(error_helpers::to_file_system_error)?,
    ))
}

// ==========================================
// Song Scanning & File Metadata Helpers
// ==========================================

fn read_tagged_file(path: &PathBuf, guess: bool) -> Result<TaggedFile> {
    if guess {
        read_from_path(path.clone()).map_err(error_helpers::to_media_error)
    } else {
        Probe::open(path.clone())
            .map_err(error_helpers::to_media_error)?
            .guess_file_type()
            .map_err(error_helpers::to_media_error)?
            .read()
            .map_err(error_helpers::to_media_error)
    }
}

fn extract_audio_properties(file: &TaggedFile, inner_song: &mut InnerSong) {
    let properties = file.properties();
    inner_song.bitrate = Some((properties.audio_bitrate().unwrap_or_default() * 1000) as f64);
    inner_song.sample_rate = properties.sample_rate().map(|v| v as f64);
    inner_song.duration = Some(properties.duration().as_secs() as f64);
}

fn scan_directory_for_cover(path: &Path) -> Option<String> {
    let mut base_path = path.to_path_buf();
    base_path.pop();
    if let Ok(files) = base_path.read_dir() {
        for e in files {
            if let Ok(dir_entry) = e {
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
        }
    }
    None
}

async fn extract_cover_art(
    metadata: &Tag,
    path: &Path,
    thumbnail_dir: &Path,
    inner_song: &mut InnerSong,
) {
    if let Some(picture) = metadata.pictures().first() {
        match store_picture(thumbnail_dir, picture).await {
            Ok((high_path, low_path)) => {
                inner_song.song_cover_path_high = Some(high_path.to_string_lossy().to_string());
                inner_song.song_cover_path_low = Some(low_path.to_string_lossy().to_string());
            }
            Err(e) => {
                tracing::error!("Error storing picture {:?}", e);
            }
        }
    } else {
        inner_song.song_cover_path_high = scan_directory_for_cover(path);
    }
}

#[tracing::instrument(level = "debug", skip(path))]
fn scan_lrc(mut path: PathBuf) -> Option<String> {
    path.set_extension("lrc");
    if path.exists() {
        lazy_static! {
            static ref LRC_REGEX: Regex = Regex::new(r"\[\d{2}:\d{2}.\d{2}\]").unwrap();
        }
        let data = fs::read(path).ok()?;
        let mut parsed_lyrics = "".to_string();
        let parsed = String::from_utf8_lossy(&data).to_string();
        for line in parsed.split('\n') {
            if LRC_REGEX.is_match(line) {
                parsed_lyrics.push_str(&LRC_REGEX.replace_all(line, ""));
                parsed_lyrics.push('\n');
            }
        }
        return Some(parsed_lyrics);
    }
    None
}

fn extract_lyrics(metadata: &Tag, path: &PathBuf) -> Option<String> {
    metadata
        .get_string(&lofty::prelude::ItemKey::Lyrics)
        .map(|s| s.to_string())
        .or_else(|| scan_lrc(path.clone()))
}

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

fn extract_metadata(
    metadata: &Tag,
    path: &PathBuf,
    artist_split: &str,
    song: &mut Song,
    inner_song: &mut InnerSong,
) {
    inner_song.lyrics = extract_lyrics(metadata, path);
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

#[tracing::instrument(level = "debug", skip(path, thumbnail_dir, size, guess, artist_split))]
pub async fn scan_file(
    path: &PathBuf,
    thumbnail_dir: &Path,
    size: f64,
    guess: bool,
    artist_split: &str,
) -> Result<Song> {
    let mut inner_song = InnerSong {
        id: Some(Uuid::new_v4().to_string()),
        title: Some(path.file_name().unwrap().to_string_lossy().to_string()),
        path: Some(
            dunce::canonicalize(path)
                .map_err(error_helpers::to_file_system_error)?
                .to_string_lossy()
                .to_string(),
        ),
        size: Some(size),
        duration: Some(0f64),
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
        Err(_) => return Ok(song),
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

// ==========================================
// Struct Definitions and Scanner Implementations
// ==========================================

pub struct SongScanner {
    #[allow(dead_code)]
    dir: PathBuf,
    thumbnail_dir: PathBuf,
    artist_split: String,
}

impl SongScanner {
    #[tracing::instrument(level = "debug", skip(dir, thumbnail_dir, artist_split))]
    pub fn new(dir: PathBuf, thumbnail_dir: PathBuf, artist_split: String) -> Self {
        Self {
            dir,
            thumbnail_dir,
            artist_split,
        }
    }

    #[tracing::instrument(level = "debug", skip(self))]
    fn check_dirs(&self) -> Result<()> {
        check_directory(self.thumbnail_dir.clone())?;
        Ok(())
    }

    #[tracing::instrument(level = "debug", skip(self, size, path))]
    pub async fn scan_song(&self, size: f64, path: PathBuf) -> Result<Song> {
        self.check_dirs()?;
        let thumbnail_dir = self.thumbnail_dir.clone();
        let artist_split = self.artist_split.clone();
        let mut metadata = scan_file(&path, &thumbnail_dir, size, false, &artist_split).await;
        if metadata.is_err() {
            metadata = scan_file(&path, &thumbnail_dir, size, true, &artist_split).await;
        }
        metadata
    }
}

// ==========================================
// Playlist Scanner & Parsing Helpers
// ==========================================

struct PlaylistParserState {
    song_type: Option<String>,
    duration: Option<f64>,
    title: Option<String>,
    artists: Option<String>,
    playlist_title: String,
    playlist_id: String,
    songs: Vec<Song>,
}

impl PlaylistParserState {
    fn new() -> Self {
        Self {
            song_type: None,
            duration: None,
            title: None,
            artists: None,
            playlist_title: "".to_string(),
            playlist_id: Uuid::new_v4().to_string(),
            songs: vec![],
        }
    }

    fn clear_metadata(&mut self) {
        self.song_type = None;
        self.duration = None;
        self.title = None;
        self.artists = None;
    }

    fn into_playlist_and_songs(self, path: &Path) -> (Playlist, Vec<Song>) {
        (
            Playlist {
                playlist_id: Some(self.playlist_id),
                playlist_name: self.playlist_title,
                playlist_path: Some(path.to_string_lossy().to_string()),
                ..Default::default()
            },
            self.songs,
        )
    }
}

#[allow(dead_code)]
pub struct PlaylistScanner<'a> {
    dir: PathBuf,
    song_scanner: &'a SongScanner,
    thumbnail_dir: PathBuf,
}

impl<'a> PlaylistScanner<'a> {
    #[tracing::instrument(level = "debug", skip(dir, thumbnail_dir, song_scanner))]
    pub fn new(dir: PathBuf, thumbnail_dir: PathBuf, song_scanner: &'a SongScanner) -> Self {
        Self {
            dir,
            thumbnail_dir,
            song_scanner,
        }
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
        let mut state = PlaylistParserState::new();

        for line_res in lines {
            let line = line_res.map_err(error_helpers::to_file_system_error)?;
            self.parse_playlist_line(path, &line, &mut state)?;
        }
        Ok(state.into_playlist_and_songs(path))
    }

    fn parse_playlist_line(
        &self,
        path: &Path,
        line: &str,
        state: &mut PlaylistParserState,
    ) -> Result<()> {
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

    fn parse_extinf(&self, line: &str, state: &mut PlaylistParserState) -> Result<()> {
        let metadata = line.substring(8, line.len());
        let split_index = metadata.find(',').unwrap_or_default();
        state.duration = Some(
            metadata
                .substring(0, split_index)
                .parse::<f64>()
                .map_err(error_helpers::to_parse_error)?,
        );
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

    fn parse_song_entry(
        &self,
        path: &Path,
        line: &str,
        state: &mut PlaylistParserState,
    ) -> Result<()> {
        let (parsed_line, is_url) = parse_line_url_or_file(line);
        if !is_url {
            let local_path = resolve_playlist_song_path(path, &parsed_line)?;
            if !local_path.exists() {
                state.clear_metadata();
                return Ok(());
            }
            self.push_local_song(&local_path, state)?;
        } else {
            self.push_url_song(&parsed_line, state);
        }
        state.clear_metadata();
        Ok(())
    }

    fn push_local_song(&self, path: &Path, state: &mut PlaylistParserState) -> Result<()> {
        let metadata = fs::metadata(path).map_err(error_helpers::to_file_system_error)?;
        let mut song = InnerSong::default();
        song.id = Some(Uuid::new_v4().to_string());
        song.r#type = SongType::Local.into();
        song.size = Some(metadata.len() as f64);
        song.path = Some(path.to_string_lossy().to_string());
        song.duration = state.duration;
        song.title = state.title.clone();
        state.songs.push(Song {
            song: Some(song),
            album: None,
            artists: self.parse_artists(state.artists.clone()),
            genre: vec![],
        });
        Ok(())
    }

    fn push_url_song(&self, url: &str, state: &mut PlaylistParserState) {
        let mut song = InnerSong::default();
        song.id = Some(Uuid::new_v4().to_string());
        song.r#type = SongType::Url.into();
        song.playback_url = Some(url.to_string());
        song.duration = state.duration;
        song.title = state.title.clone();
        state.songs.push(Song {
            song: Some(song),
            album: None,
            artists: self.parse_artists(state.artists.clone()),
            genre: vec![],
        });
    }
}

fn parse_line_url_or_file(line: &str) -> (String, bool) {
    if line.starts_with("file://") {
        (line[8..].to_string(), false)
    } else if line.starts_with("http") {
        (line.replace("http://", "").replace("https://", ""), true)
    } else {
        (line.to_string(), false)
    }
}

fn resolve_playlist_song_path(playlist_path: &Path, song_path: &str) -> Result<PathBuf> {
    let mut parsed_path =
        PathBuf::from_str(song_path).map_err(error_helpers::to_file_system_error)?;
    if parsed_path.is_relative() {
        parsed_path = playlist_path
            .parent()
            .unwrap_or(playlist_path)
            .join(parsed_path)
            .canonicalize()
            .map_err(error_helpers::to_file_system_error)?;
    }
    Ok(parsed_path)
}

// ==========================================
// Desktop Context & Trait Implementation
// ==========================================

pub struct DesktopScannerContext {
    scan_dir: PathBuf,
    thumbnail_dir: PathBuf,
    artist_split: String,
}

impl DesktopScannerContext {
    pub fn new(scan_dir: PathBuf, thumbnail_dir: PathBuf, artist_split: String) -> Self {
        Self {
            scan_dir,
            thumbnail_dir,
            artist_split,
        }
    }

    fn scan_playlists(
        &self,
        playlist_scanner: &PlaylistScanner,
        playlists: &[PathBuf],
    ) -> Vec<Result<(Playlist, Vec<Song>)>> {
        let mut parsed_playlists = Vec::new();
        for playlist_path in playlists {
            match playlist_scanner.scan_playlist(playlist_path) {
                Ok(res) => parsed_playlists.push(Ok(res)),
                Err(e) => {
                    let err_msg = format!("Failed to scan {}: {:?}", playlist_path.display(), e);
                    parsed_playlists.push(Err(MoosyncError::String(err_msg)));
                }
            }
        }
        parsed_playlists
    }

    async fn scan_library_songs(
        &self,
        song_scanner: &SongScanner,
        files: Vec<(PathBuf, f64)>,
        total_songs: usize,
        scanned_count: &mut usize,
        on_song: &OnSongScanned,
        on_progress: &OnProgressUpdated,
    ) {
        let batch_size = 100;
        let mut scan_futures = Vec::new();
        for (file_path, size) in files {
            scan_futures.push(song_scanner.scan_song(size, file_path));
            if scan_futures.len() >= batch_size {
                let results = futures::future::join_all(scan_futures).await;
                scan_futures = Vec::new();
                self.process_scan_results(
                    None,
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
            self.process_scan_results(
                None,
                results,
                total_songs,
                scanned_count,
                on_song,
                on_progress,
            )
            .await;
        }
    }

    async fn scan_playlist_songs(
        &self,
        song_scanner: &SongScanner,
        playlists: Vec<Result<(Playlist, Vec<Song>)>>,
        total_songs: usize,
        scanned_count: &mut usize,
        on_song: &OnSongScanned,
        on_playlist: &OnPlaylistScanned,
        on_progress: &OnProgressUpdated,
    ) {
        for playlist_res in playlists {
            if let Ok((playlist, playlist_songs)) = playlist_res {
                on_playlist(vec![playlist.clone()]).await;
                self.scan_single_playlist_songs(
                    song_scanner,
                    playlist,
                    playlist_songs,
                    total_songs,
                    scanned_count,
                    on_song,
                    on_progress,
                )
                .await;
            } else if let Err(e) = playlist_res {
                tracing::error!("Playlist error: {:?}", e);
            }
        }
    }

    async fn scan_single_playlist_songs(
        &self,
        song_scanner: &SongScanner,
        playlist: Playlist,
        playlist_songs: Vec<Song>,
        total_songs: usize,
        scanned_count: &mut usize,
        on_song: &OnSongScanned,
        on_progress: &OnProgressUpdated,
    ) {
        let batch_size = 100;
        let mut pl_scan_futures = Vec::new();
        for song in playlist_songs {
            pl_scan_futures.push(create_playlist_song_fut(song_scanner, song));
            if pl_scan_futures.len() >= batch_size {
                let pl_results = futures::future::join_all(pl_scan_futures).await;
                pl_scan_futures = Vec::new();
                let playlist_id = playlist.playlist_id.clone();
                self.process_scan_results(
                    playlist_id,
                    pl_results,
                    total_songs,
                    scanned_count,
                    on_song,
                    on_progress,
                )
                .await;
            }
        }
        if !pl_scan_futures.is_empty() {
            let pl_results = futures::future::join_all(pl_scan_futures).await;
            let playlist_id = playlist.playlist_id.clone();
            self.process_scan_results(
                playlist_id,
                pl_results,
                total_songs,
                scanned_count,
                on_song,
                on_progress,
            )
            .await;
        }
    }

    async fn process_scan_results(
        &self,
        playlist_id: Option<String>,
        results: Vec<Result<Song>>,
        total_songs: usize,
        scanned_count: &mut usize,
        on_song: &OnSongScanned,
        on_progress: &OnProgressUpdated,
    ) {
        for res in results {
            if let Ok(song) = res {
                tracing::info!("Scanned song {:?}", song);
                on_song(playlist_id.clone(), vec![song]).await;
            } else if let Err(e) = res {
                tracing::error!("Scan error: {:?}", e);
            }
            *scanned_count += 1;
            update_scan_progress(total_songs, *scanned_count, on_progress);
        }
    }
}

fn count_playlist_songs(playlists: &[Result<(Playlist, Vec<Song>)>]) -> usize {
    let mut count = 0;
    for p in playlists {
        if let Ok((_, songs)) = p {
            count += songs.len();
        }
    }
    count
}

fn update_scan_progress(total: usize, current: usize, on_progress: &OnProgressUpdated) {
    if total > 0 {
        let progress = ((current * 100) / total) as u8;
        on_progress(ScanProgress::PROGRESS(progress));
    }
}

async fn create_playlist_song_fut(song_scanner: &SongScanner, song: Song) -> Result<Song> {
    if let Some(song_inner) = song.song.as_ref()
        && SongType::try_from(song_inner.r#type).unwrap_or(SongType::Local) == SongType::Local
        && let Some(path) = song_inner.path.as_ref()
    {
        song_scanner
            .scan_song(
                song_inner.size.unwrap_or_default(),
                PathBuf::from_str(path.as_str()).unwrap_or_default(),
            )
            .await
    } else {
        Ok(song)
    }
}

impl super::ScannerContext for DesktopScannerContext {
    async fn start_scan(
        &self,
        on_song: &OnSongScanned,
        on_playlist: &OnPlaylistScanned,
        on_progress: &OnProgressUpdated,
    ) -> Result<()> {
        let song_scanner = SongScanner::new(
            self.scan_dir.clone(),
            self.thumbnail_dir.clone(),
            self.artist_split.clone(),
        );
        let playlist_scanner = PlaylistScanner::new(
            self.scan_dir.clone(),
            self.thumbnail_dir.clone(),
            &song_scanner,
        );
        let file_list = get_files_recursively(self.scan_dir.clone())?;
        let parsed_playlists = self.scan_playlists(&playlist_scanner, &file_list.playlist_list);
        let total_songs = file_list.file_list.len() + count_playlist_songs(&parsed_playlists);

        on_progress(ScanProgress::PROGRESS(0));
        let mut scanned_count = 0;
        self.scan_library_songs(
            &song_scanner,
            file_list.file_list,
            total_songs,
            &mut scanned_count,
            on_song,
            on_progress,
        )
        .await;
        self.scan_playlist_songs(
            &song_scanner,
            parsed_playlists,
            total_songs,
            &mut scanned_count,
            on_song,
            on_playlist,
            on_progress,
        )
        .await;
        on_progress(ScanProgress::STOPPED);

        Ok(())
    }
}
