use lazy_static::lazy_static;
use lofty::{
    picture::Picture, prelude::Accessor, prelude::AudioFile, prelude::TaggedFileExt, probe::Probe,
    read_from_path,
};
use regex::Regex;
use std::{
    f64, fs,
    num::NonZeroU32,
    path::{Path, PathBuf},
};
use types::{
    entities::{QueryableAlbum, QueryableArtist, QueryableGenre},
    songs::{QueryableSong, Song, SongType},
};
use uuid::Uuid;

use image::ColorType;
use types::errors::errors::Result;

use fast_image_resize::{self as fr, IntoImageView, IntoImageViewMut, ResizeOptions};

use crate::types::FileList;

pub fn check_directory(dir: PathBuf) -> Result<()> {
    if !dir.is_dir() {
        fs::create_dir_all(dir)?
    }

    Ok(())
}

pub fn get_files_recursively(dir: PathBuf) -> Result<FileList> {
    let mut file_list: Vec<(PathBuf, f64)> = vec![];
    let mut playlist_list: Vec<PathBuf> = vec![];

    lazy_static! {
        static ref SONG_RE: Regex = Regex::new("flac|mp3|ogg|m4a|webm|wav|wv|aac|opus").unwrap();
        static ref PLAYLIST_RE: Regex = Regex::new("m3u|m3u8").unwrap();
    }

    if !dir.exists() {
        return Ok(FileList {
            file_list,
            playlist_list,
        });
    }

    if dir.is_file() {
        if let Ok(metadata) = fs::metadata(&dir) {
            let extension = dir
                .extension()
                .unwrap_or_default()
                .to_str()
                .unwrap_or_default();
            if !extension.is_empty() {
                if SONG_RE.is_match(extension) {
                    file_list.push((dir.clone(), metadata.len() as f64));
                }

                if PLAYLIST_RE.is_match(extension) {
                    playlist_list.push(dir);
                }
            }
            return Ok(FileList {
                file_list,
                playlist_list,
            });
        }
    }

    let dir_entries = fs::read_dir(dir)?;

    for entry in dir_entries {
        let Ok(entry) = entry else { continue };
        let path = entry.path();

        let res = get_files_recursively(path)?;
        file_list.extend_from_slice(&res.file_list);
        playlist_list.extend_from_slice(&res.playlist_list);
    }

    Ok(FileList {
        file_list,
        playlist_list,
    })
}

fn generate_image(data: &[u8], path: PathBuf, dimensions: u32) -> Result<()> {
    let img = image::load_from_memory(data)?;

    let width = NonZeroU32::new(img.width()).unwrap();
    let height = NonZeroU32::new(img.height()).unwrap();
    let src_image = fr::images::Image::from_vec_u8(
        width.into(),
        height.into(),
        img.as_rgba8().unwrap().to_vec(),
        fr::PixelType::U8x4,
    )
    .unwrap();

    // Create container for data of destination image
    let dst_width = NonZeroU32::new(dimensions).unwrap();
    let dst_height = NonZeroU32::new(dimensions).unwrap();
    let mut dst_image =
        fr::images::Image::new(dst_width.into(), dst_height.into(), src_image.pixel_type());

    // Get mutable view of destination image data

    // Create Resizer instance and resize source image
    // into buffer of destination image
    let mut resizer = fr::Resizer::new();
    resizer.resize(
        &src_image,
        &mut dst_image,
        Some(&ResizeOptions {
            algorithm: fast_image_resize::ResizeAlg::Nearest,
            ..Default::default()
        }),
    )?;

    image::save_buffer(
        path,
        dst_image.buffer(),
        dst_width.get(),
        dst_height.get(),
        ColorType::Rgba8,
    )?;

    Ok(())
}

fn store_picture(thumbnail_dir: &Path, picture: &Picture) -> Result<(PathBuf, PathBuf)> {
    let data = picture.data();
    let hash = blake3::hash(data).to_hex();
    let hash_str = hash.as_str();

    let low_path = thumbnail_dir.join(format!("{}-low.png", hash_str));
    let high_path = thumbnail_dir.join(format!("{}.png", hash_str));

    if !Path::new(high_path.to_str().unwrap()).exists() {
        generate_image(data, high_path.clone(), 400)?;
    }

    if !Path::new(low_path.to_str().unwrap()).exists() {
        generate_image(data, low_path.clone(), 80)?;
    }

    Ok((
        dunce::canonicalize(high_path)?,
        dunce::canonicalize(low_path)?,
    ))
}

fn scan_lrc(mut path: PathBuf) -> Option<String> {
    path.set_extension("lrc");
    if path.exists() {
        lazy_static! {
            static ref LRC_REGEX: Regex = Regex::new(r"\[\d{2}:\d{2}.\d{2}\]").unwrap();
        }

        let data = fs::read(path);
        if data.is_err() {
            return None;
        }

        let mut parsed_lyrics = "".to_string();
        let parsed = String::from_utf8_lossy(&data.unwrap()).to_string();
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

pub fn scan_file(
    path: &PathBuf,
    thumbnail_dir: &Path,
    size: f64,
    guess: bool,
    artist_split: &str,
) -> Result<Song> {
    let mut song: Song = Song {
        song: QueryableSong::empty(),
        album: None,
        artists: Some(vec![]),
        genre: Some(vec![]),
    };
    song.song._id = Some(Uuid::new_v4().to_string());
    song.song.title = Some(path.file_name().unwrap().to_string_lossy().to_string());
    song.song.path = Some(dunce::canonicalize(path)?.to_string_lossy().to_string());
    song.song.size = Some(size);
    song.song.duration = Some(0f64);
    song.song.type_ = SongType::LOCAL;

    let file = if guess {
        read_from_path(path.clone())?
    } else {
        let file_res = Probe::open(path.clone())?.guess_file_type()?.read();
        if file_res.is_err() {
            println!("Error reading file without guess {:?}", file_res.err());
            return Ok(song);
        }
        file_res.unwrap()
    };

    let properties = file.properties();
    let mut tags = file.primary_tag();
    if tags.is_none() {
        tags = file.first_tag();
    }
    song.song.bitrate = Some((properties.audio_bitrate().unwrap_or_default() * 1000) as f64);
    song.song.sample_rate = properties.sample_rate().map(|v| v as f64);
    song.song.duration = Some(properties.duration().as_secs() as f64);

    if tags.is_some() {
        let metadata = tags.unwrap();

        let picture = metadata.pictures().first();
        if picture.is_some() {
            if let Ok((high_path, low_path)) = store_picture(thumbnail_dir, picture.unwrap()) {
                song.song.song_cover_path_high = Some(high_path.to_string_lossy().to_string());
                song.song.song_cover_path_low = Some(low_path.to_string_lossy().to_string());
            }
        } else {
            let mut base_path = path.clone();
            base_path.pop();
            let files_res = base_path.read_dir();
            if let Ok(mut files) = files_res {
                song.song.song_cover_path_high = files.find_map(|e| {
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
                    None
                });
            }
        }

        let mut lyrics = metadata
            .get_string(&lofty::prelude::ItemKey::Lyrics)
            .map(str::to_string);

        if lyrics.is_none() {
            lyrics = scan_lrc(path.clone());
        }

        song.song.title = metadata
            .title()
            .map(|s| s.to_string())
            .or(path.file_name().map(|s| s.to_string_lossy().to_string()));
        // song.album = metadata.album().map(|s| s.to_string());
        let artists: Option<Vec<QueryableArtist>> = metadata.artist().map(|s| {
            s.split(artist_split)
                .map(|s| QueryableArtist {
                    artist_id: Some(Uuid::new_v4().to_string()),
                    artist_name: Some(s.trim().to_string()),
                    ..Default::default()
                })
                .collect()
        });

        let album = metadata.album();
        if album.is_some() {
            song.song.track_no = metadata
                .get_string(&lofty::prelude::ItemKey::TrackNumber)
                .map(|s| s.parse().unwrap_or_default());

            song.album = Some(QueryableAlbum {
                album_id: Some(Uuid::new_v4().to_string()),
                album_name: album.map(|v| v.to_string()),
                album_coverpath_high: song.song.song_cover_path_high.clone(),
                album_coverpath_low: song.song.song_cover_path_low.clone(),
                album_artist: metadata
                    .get_string(&lofty::prelude::ItemKey::AlbumArtist)
                    .map(|s| s.to_owned()),
                ..Default::default()
            })
        }

        song.artists = artists;

        song.song.year = metadata.year().map(|s| s.to_string());
        song.genre = metadata.genre().map(|s| {
            vec![QueryableGenre {
                genre_name: Some(s.to_string()),
                ..Default::default()
            }]
        });
        song.song.lyrics = lyrics;
    }

    Ok(song)
}
