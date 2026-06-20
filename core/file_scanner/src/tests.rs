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
    env,
    fs::{self, File},
    io::Write,
    sync::{Arc, Mutex},
};

use songs_proto::moosync::types::{Playlist, Song};

use crate::{PlaylistSongId, ScanProgress, ScannerHolder};

#[tokio::test]
async fn test_playlist_scan() {
    let playlist_contents = r#"
#EXTM3U
#EXTINF:0,stream
#EXTVLCOPT:network-caching=1000
https://cast.animu.com.br:9079/stream
#EXTINF:0,320
#EXTVLCOPT:network-caching=1000
https://radio.stereoanime.net/listen/stereoanime/320
#EXTINF:0,stream.flac
#EXTVLCOPT:network-caching=1000
https://chiru.no/stream.flac"#;

    let test_out_dir = env::temp_dir().join("moosync-test-out");
    let test_in_dir = env::temp_dir().join("moosync-test-in");

    fs::create_dir_all(test_out_dir.clone()).unwrap();
    fs::create_dir_all(test_in_dir.clone()).unwrap();

    let mut input = File::create(test_in_dir.join("playlist.m3u")).unwrap();
    input.write_all(playlist_contents.as_bytes()).unwrap();

    let playlist_count = Arc::new(Mutex::new(0));
    let playlist_count_clone = playlist_count.clone();

    let mut scanner = ScannerHolder::new();
    scanner.set_scan_dirs(vec![test_in_dir.clone()]);
    scanner.set_thumbnail_dir(test_out_dir.clone());
    scanner.set_artist_split("".to_string());

    scanner.set_on_song(move |_playlist_id: Option<String>, _songs: Vec<Song>| async move {});

    scanner.set_on_playlist(move |playlists: Vec<(Playlist, Vec<PlaylistSongId>)>| {
        let playlist_count_clone = playlist_count_clone.clone();
        async move {
            let mut count = playlist_count_clone.lock().unwrap();
            for (playlist, songs) in playlists {
                assert_eq!(playlist.playlist_name, "");
                assert_eq!(songs.len(), 3);
                assert_eq!(
                    songs[0],
                    PlaylistSongId::Url("cast.animu.com.br:9079/stream".to_string())
                );
                assert_eq!(
                    songs[1],
                    PlaylistSongId::Url("radio.stereoanime.net/listen/stereoanime/320".to_string())
                );
                assert_eq!(
                    songs[2],
                    PlaylistSongId::Url("chiru.no/stream.flac".to_string())
                );
                *count += 1;
            }
        }
    });

    let mut progress_rx = scanner.add_subscriber();

    scanner.start_scan().await.unwrap();

    let mut progress_events = Vec::new();
    while let Ok(evt) = progress_rx.try_recv() {
        progress_events.push(evt);
    }
    assert!(!progress_events.is_empty());
    assert_eq!(*progress_events.last().unwrap(), ScanProgress::STOPPED);

    for _ in 0..100 {
        if *playlist_count.lock().unwrap() == 1 {
            break;
        }
        tokio::time::sleep(std::time::Duration::from_millis(5)).await;
    }
    assert_eq!(*playlist_count.lock().unwrap(), 1);

    fs::remove_dir_all(test_in_dir).unwrap();
    fs::remove_dir_all(test_out_dir).unwrap();
}

#[tokio::test]
async fn test_playlist_scan_with_extra_comments() {
    let playlist_contents = r#"
#EXTM3U
# This is an extra comment line
#EXTINF:0,track1
https://example.com/track1
# Another comment
#EXTINF:0,track2
https://example.com/track2"#;

    let test_out_dir = env::temp_dir().join("moosync-test-out-comments");
    let test_in_dir = env::temp_dir().join("moosync-test-in-comments");

    fs::create_dir_all(test_out_dir.clone()).unwrap();
    fs::create_dir_all(test_in_dir.clone()).unwrap();

    let mut input = File::create(test_in_dir.join("playlist.m3u")).unwrap();
    input.write_all(playlist_contents.as_bytes()).unwrap();

    let playlist_count = Arc::new(Mutex::new(0));
    let playlist_count_clone = playlist_count.clone();

    let mut scanner = ScannerHolder::new();
    scanner.set_scan_dirs(vec![test_in_dir.clone()]);
    scanner.set_thumbnail_dir(test_out_dir.clone());
    scanner.set_artist_split("".to_string());

    scanner.set_on_song(move |_playlist_id: Option<String>, _songs: Vec<Song>| async move {});

    scanner.set_on_playlist(move |playlists: Vec<(Playlist, Vec<PlaylistSongId>)>| {
        let playlist_count_clone = playlist_count_clone.clone();
        async move {
            let mut count = playlist_count_clone.lock().unwrap();
            for (playlist, songs) in playlists {
                assert_eq!(playlist.playlist_name, "");
                assert_eq!(songs.len(), 2);
                assert_eq!(
                    songs[0],
                    PlaylistSongId::Url("example.com/track1".to_string())
                );
                assert_eq!(
                    songs[1],
                    PlaylistSongId::Url("example.com/track2".to_string())
                );
                *count += 1;
            }
        }
    });

    let mut progress_rx = scanner.add_subscriber();

    scanner.start_scan().await.unwrap();

    let mut progress_events = Vec::new();
    while let Ok(evt) = progress_rx.try_recv() {
        progress_events.push(evt);
    }
    assert!(!progress_events.is_empty());
    assert_eq!(*progress_events.last().unwrap(), ScanProgress::STOPPED);

    for _ in 0..100 {
        if *playlist_count.lock().unwrap() == 1 {
            break;
        }
        tokio::time::sleep(std::time::Duration::from_millis(5)).await;
    }
    assert_eq!(*playlist_count.lock().unwrap(), 1);

    fs::remove_dir_all(test_in_dir).unwrap();
    fs::remove_dir_all(test_out_dir).unwrap();
}

#[tokio::test]
async fn test_playlist_scan_single_entry() {
    let playlist_contents = r#"
#EXTM3U
#EXTINF:0,lonely_track
https://example.com/lonely_track"#;

    let test_out_dir = env::temp_dir().join("moosync-test-out-single");
    let test_in_dir = env::temp_dir().join("moosync-test-in-single");

    fs::create_dir_all(test_out_dir.clone()).unwrap();
    fs::create_dir_all(test_in_dir.clone()).unwrap();

    let mut input = File::create(test_in_dir.join("playlist.m3u")).unwrap();
    input.write_all(playlist_contents.as_bytes()).unwrap();

    let playlist_count = Arc::new(Mutex::new(0));
    let playlist_count_clone = playlist_count.clone();

    let mut scanner = ScannerHolder::new();
    scanner.set_scan_dirs(vec![test_in_dir.clone()]);
    scanner.set_thumbnail_dir(test_out_dir.clone());
    scanner.set_artist_split("".to_string());

    scanner.set_on_song(move |_playlist_id: Option<String>, _songs: Vec<Song>| async move {});

    scanner.set_on_playlist(move |playlists: Vec<(Playlist, Vec<PlaylistSongId>)>| {
        let playlist_count_clone = playlist_count_clone.clone();
        async move {
            let mut count = playlist_count_clone.lock().unwrap();
            for (playlist, songs) in playlists {
                assert_eq!(playlist.playlist_name, "");
                assert_eq!(songs.len(), 1);
                assert_eq!(
                    songs[0],
                    PlaylistSongId::Url("example.com/lonely_track".to_string())
                );
                *count += 1;
            }
        }
    });

    let mut progress_rx = scanner.add_subscriber();

    scanner.start_scan().await.unwrap();

    let mut progress_events = Vec::new();
    while let Ok(evt) = progress_rx.try_recv() {
        progress_events.push(evt);
    }
    assert!(!progress_events.is_empty());
    assert_eq!(*progress_events.last().unwrap(), ScanProgress::STOPPED);

    for _ in 0..100 {
        if *playlist_count.lock().unwrap() == 1 {
            break;
        }
        tokio::time::sleep(std::time::Duration::from_millis(5)).await;
    }
    assert_eq!(*playlist_count.lock().unwrap(), 1);

    fs::remove_dir_all(test_in_dir).unwrap();
    fs::remove_dir_all(test_out_dir).unwrap();
}
