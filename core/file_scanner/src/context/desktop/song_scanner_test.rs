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
    sync::{Arc, Mutex},
};

use assertables::{assert_len_eq_x, assert_none, assert_ok, assert_some};
use lofty::{
    picture::{MimeType, Picture, PictureType},
    prelude::{Accessor, ItemKey},
    tag::{Tag, TagType},
};
use rstest::{fixture, rstest};
use tempdir::TempDir;
use tracing_test::traced_test;

use crate::{
    FileList, OnProgressUpdated, OnSongScanned, ScanProgress,
    context::desktop::song_scanner::{
        SongScanner, check_directory, extract_cover_art, extract_metadata, parse_artists_string,
        process_thumbnail, scan_directory_for_cover, update_scan_progress,
    },
};

const TINY_PNG: &[u8] = &[
    0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, 0x00, 0x00, 0x00, 0x0D, 0x49, 0x48, 0x44, 0x52,
    0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x08, 0x06, 0x00, 0x00, 0x00, 0x1F, 0x15, 0xC4,
    0x89, 0x00, 0x00, 0x00, 0x0A, 0x49, 0x44, 0x41, 0x54, 0x78, 0x9C, 0x63, 0x00, 0x01, 0x00, 0x00,
    0x05, 0x00, 0x01, 0x0D, 0x0A, 0x2D, 0xB4, 0x00, 0x00, 0x00, 0x00, 0x49, 0x45, 0x4E, 0x44, 0xAE,
    0x42, 0x60, 0x82,
];

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
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_parse_artists_string() {
    let parsed = parse_artists_string("Alice, Bob , Charlie", ",");
    assert_len_eq_x!(&parsed, 3);
    assert_eq!(parsed[0].artist_name.as_deref(), Some("Alice"));
    assert_eq!(parsed[1].artist_name.as_deref(), Some("Bob"));
    assert_eq!(parsed[2].artist_name.as_deref(), Some("Charlie"));
}

#[rstest]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_update_scan_progress() {
    let progress_val = Arc::new(Mutex::new(None));
    let progress_clone = progress_val.clone();
    let on_progress: OnProgressUpdated = Box::new(move |p| {
        if let ScanProgress::PROGRESS(val) = p {
            *progress_clone.lock().unwrap() = Some(val);
        }
    });

    update_scan_progress(100, 50, &on_progress);
    assert_eq!(*progress_val.lock().unwrap(), Some(50));

    // Zero total handled gracefully
    update_scan_progress(0, 0, &on_progress);
}

#[rstest]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_scan_directory_for_cover(temp_dir_fixture: TempDir) {
    let song_dir = temp_dir_fixture.path().join("album_dir");
    fs::create_dir_all(&song_dir).unwrap();
    let cover_path = song_dir.join("cover.png");
    fs::write(&cover_path, TINY_PNG).unwrap();
    let song_file = song_dir.join("track.wav");

    let found = scan_directory_for_cover(&song_file);
    assert_some!(&found);
    assert!(found.unwrap().ends_with("cover.png"));

    let empty_dir = temp_dir_fixture.path().join("empty_dir");
    fs::create_dir_all(&empty_dir).unwrap();
    let not_found = scan_directory_for_cover(&empty_dir.join("track.wav"));
    assert_none!(&not_found);
}

#[rstest]
#[tokio::test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_process_thumbnail_cache(temp_dir_fixture: TempDir) {
    let target = temp_dir_fixture.path().join("thumb_test.png");
    assert!(!target.exists());

    let res = process_thumbnail(TINY_PNG, &target, 100).await;
    assert_ok!(res);
    assert!(target.exists());

    // Second call hits cache early return
    let res_cached = process_thumbnail(TINY_PNG, &target, 100).await;
    assert_ok!(res_cached);
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
async fn test_song_scanner_extract_metadata_and_cover_art(temp_dir_fixture: TempDir) {
    let thumb_dir = temp_dir_fixture.path().join("thumbs");
    fs::create_dir_all(&thumb_dir).unwrap();
    let song_path = temp_dir_fixture.path().join("track.mp3");

    let mut tag = Tag::new(TagType::Id3v2);
    tag.set_title("Direct Title".to_string());
    tag.set_artist("Artist A; Artist B".to_string());
    tag.set_album("Direct Album".to_string());
    tag.set_year(2023);
    tag.set_genre("Jazz".to_string());
    tag.insert_text(ItemKey::TrackNumber, "5".to_string());
    tag.insert_text(ItemKey::AlbumArtist, "Album Artist Direct".to_string());
    let pic = Picture::new_unchecked(
        PictureType::CoverFront,
        Some(MimeType::Png),
        None,
        TINY_PNG.to_vec(),
    );
    tag.push_picture(pic);

    let mut inner_song = songs_proto::moosync::types::InnerSong::default();
    let mut song = songs_proto::moosync::types::Song::default();

    extract_cover_art(&tag, &song_path, &thumb_dir, &mut inner_song).await;
    assert_some!(&inner_song.song_cover_path_high);
    assert_some!(&inner_song.song_cover_path_low);

    extract_metadata(&tag, &song_path, ";", &mut song, &mut inner_song);
    assert_eq!(inner_song.title.as_deref(), Some("Direct Title"));
    assert_eq!(inner_song.track_no, Some(5.0));
    assert_eq!(inner_song.year.as_deref(), Some("2023"));

    assert_len_eq_x!(&song.artists, 2);
    assert_eq!(song.artists[0].artist_name.as_deref(), Some("Artist A"));
    assert_eq!(song.artists[1].artist_name.as_deref(), Some("Artist B"));

    assert_some!(&song.album);
    let album = song.album.as_ref().unwrap();
    assert_eq!(album.album_name.as_deref(), Some("Direct Album"));
    assert_eq!(album.album_artist.as_deref(), Some("Album Artist Direct"));

    assert_len_eq_x!(&song.genre, 1);
    assert_eq!(song.genre[0].genre_name.as_deref(), Some("Jazz"));

    // Test fallback when tag has no picture
    let empty_tag = Tag::new(TagType::Id3v2);
    let mut inner_fallback = songs_proto::moosync::types::InnerSong::default();
    extract_cover_art(&empty_tag, &song_path, &thumb_dir, &mut inner_fallback).await;
    assert_none!(&inner_fallback.song_cover_path_high);

    // Test fallback when tag has no title (uses file name)
    let mut song_fallback = songs_proto::moosync::types::Song::default();
    extract_metadata(
        &empty_tag,
        &song_path,
        ";",
        &mut song_fallback,
        &mut inner_fallback,
    );
    assert_eq!(inner_fallback.title.as_deref(), Some("track.mp3"));
}

#[rstest]
#[tokio::test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_song_scanner_lifecycle(temp_dir_fixture: TempDir) {
    let thumb_dir = temp_dir_fixture.path().join("thumbs");
    fs::create_dir_all(&thumb_dir).unwrap();
    let song_a = temp_dir_fixture.path().join("song_a.mp3");
    fs::write(&song_a, b"data1").unwrap();

    let song_b = temp_dir_fixture.path().join("song_b.mp3");
    fs::write(&song_b, b"data2").unwrap();

    let file_list = FileList {
        file_list: vec![(song_a, 1000.0), (song_b, 2000.0)],
        playlist_list: vec![],
    };
    let scanner = SongScanner::new(&file_list, thumb_dir, ";".to_string(), Some(2));
    let mut scanned = 0;
    let scanned_songs = Arc::new(Mutex::new(Vec::new()));
    let scanned_songs_clone = scanned_songs.clone();
    let on_song: OnSongScanned = Arc::new(move |_pl_id, songs| {
        let cloned = scanned_songs_clone.clone();
        Box::pin(async move {
            cloned.lock().unwrap().extend(songs);
        })
    });
    let on_progress: OnProgressUpdated = Box::new(|_p: ScanProgress| {});

    assert_ok!(scanner.scan(&mut scanned, 2, &on_song, &on_progress).await);
    assert_eq!(scanned, 2);
    assert_len_eq_x!(&*scanned_songs.lock().unwrap(), 2);
}
