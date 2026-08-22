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

use crate::{FileList, PlaylistSongId, context::desktop::playlist_scanner::PlaylistScanner};

#[tracing::instrument(level = "debug", skip_all)]
fn get_test_dir() -> PathBuf {
    let dir = temp_dir().join(format!("moosync_m3u_test_{}", Uuid::new_v4()));
    fs::create_dir_all(&dir).unwrap();
    dir
}

#[test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_parse_line_url_or_file_variants() {
    let dir = get_test_dir();
    let s1 = dir.join("song.flac");
    fs::write(&s1, b"fake flac content").unwrap();

    let m3u_path = dir.join("variants.m3u");
    let content =
        "http://example.com/stream.mp3\nhttps://example.com/secure_stream.mp3\nsong.flac\n";
    fs::write(&m3u_path, content).unwrap();

    let file_list = FileList {
        file_list: vec![],
        playlist_list: vec![m3u_path],
    };

    let scanner = PlaylistScanner::new(&file_list);
    let playlists = scanner.scan().unwrap();
    assert_eq!(playlists.len(), 1);
    let songs = &playlists[0].1;
    assert_eq!(songs.len(), 3);

    assert_eq!(
        songs[0],
        PlaylistSongId::Url("example.com/stream.mp3".to_string())
    );
    assert_eq!(
        songs[1],
        PlaylistSongId::Url("example.com/secure_stream.mp3".to_string())
    );
    assert_eq!(songs[2], PlaylistSongId::Path(s1.canonicalize().unwrap()));

    let _ = fs::remove_dir_all(dir);
}

#[test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_resolve_playlist_song_path() {
    let dir = get_test_dir();
    let playlist_path = dir.join("test.m3u");
    let song_file = dir.join("song.mp3");
    fs::write(&song_file, b"fake audio content").unwrap();
    fs::write(&playlist_path, "song.mp3\n").unwrap();

    let file_list = FileList {
        file_list: vec![],
        playlist_list: vec![playlist_path],
    };

    let scanner = PlaylistScanner::new(&file_list);
    let playlists = scanner.scan().unwrap();
    assert_eq!(playlists.len(), 1);
    assert_eq!(
        playlists[0].1,
        vec![PlaylistSongId::Path(song_file.canonicalize().unwrap())]
    );

    let _ = fs::remove_dir_all(dir);
}

#[test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_playlist_scanner_standard_and_extended_m3u() {
    let dir = get_test_dir();
    let s1 = dir.join("track1.mp3");
    let s2 = dir.join("track2.mp3");
    fs::write(&s1, b"audio 1").unwrap();
    fs::write(&s2, b"audio 2").unwrap();

    let m3u_path = dir.join("playlist.m3u");
    let content = "#EXTM3U\n#PLAYLIST:Rock Classics\n#EXTINF:240.5,Queen - Bohemian Rhapsody\ntrack1.mp3\n#EXTINF:180,Pink Floyd-Time\ntrack2.mp3\nhttps://radio.example.com/live\n";
    fs::write(&m3u_path, content).unwrap();

    let file_list = FileList {
        file_list: vec![],
        playlist_list: vec![m3u_path.clone()],
    };

    let scanner = PlaylistScanner::new(&file_list);
    let playlists = scanner.scan().unwrap();

    assert_eq!(playlists.len(), 1);
    let (pl, songs) = &playlists[0];
    assert_eq!(pl.playlist_name, "Rock Classics");
    assert_eq!(songs.len(), 3);

    match &songs[0] {
        PlaylistSongId::Path(p) => assert_eq!(p, &s1.canonicalize().unwrap()),
        _ => panic!("Expected Path"),
    }
    match &songs[1] {
        PlaylistSongId::Path(p) => assert_eq!(p, &s2.canonicalize().unwrap()),
        _ => panic!("Expected Path"),
    }
    match &songs[2] {
        PlaylistSongId::Url(u) => assert_eq!(u, "radio.example.com/live"),
        _ => panic!("Expected Url"),
    }

    let _ = fs::remove_dir_all(dir);
}

#[test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_playlist_scanner_crlf_and_cr_line_endings() {
    let dir = get_test_dir();
    let s1 = dir.join("crlf_song.mp3");
    fs::write(&s1, b"audio").unwrap();

    let m3u_path = dir.join("crlf_playlist.m3u");
    let content =
        "#EXTM3U\r\n#PLAYLIST:CRLF Playlist\r\n#EXTINF:120,Artist - Song\r\ncrlf_song.mp3\r\n";
    fs::write(&m3u_path, content).unwrap();

    let file_list = FileList {
        file_list: vec![],
        playlist_list: vec![m3u_path],
    };

    let scanner = PlaylistScanner::new(&file_list);
    let playlists = scanner.scan().unwrap();
    assert_eq!(playlists.len(), 1);
    assert_eq!(playlists[0].0.playlist_name, "CRLF Playlist");
    assert_eq!(playlists[0].1.len(), 1);

    let _ = fs::remove_dir_all(dir);
}

#[test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_playlist_scanner_unicode_and_special_tags() {
    let dir = get_test_dir();
    let s1 = dir.join("japanese_song.mp3");
    fs::write(&s1, b"audio").unwrap();

    let m3u_path = dir.join("unicode_playlist.m3u8");
    let content = "#EXTM3U\n#PLAYLIST:J-Pop Hits 🌸\n#MOOSINF:SPOTIFY\n#EXTINF:200,初音ミク - メルト\njapanese_song.mp3\n#EXTVLCOPT:network-caching=1000\n# comment line\n";
    fs::write(&m3u_path, content).unwrap();

    let file_list = FileList {
        file_list: vec![],
        playlist_list: vec![m3u_path],
    };

    let scanner = PlaylistScanner::new(&file_list);
    let playlists = scanner.scan().unwrap();
    assert_eq!(playlists.len(), 1);
    assert_eq!(playlists[0].0.playlist_name, "J-Pop Hits 🌸");
    assert_eq!(playlists[0].1.len(), 1);

    let _ = fs::remove_dir_all(dir);
}

#[test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_playlist_scanner_empty_or_whitespace_file() {
    let dir = get_test_dir();
    let m3u_path = dir.join("empty.m3u");
    fs::write(&m3u_path, "#EXTM3U\n#PLAYLIST:Empty\n").unwrap();

    let file_list = FileList {
        file_list: vec![],
        playlist_list: vec![m3u_path],
    };

    let scanner = PlaylistScanner::new(&file_list);
    let playlists = scanner.scan().unwrap();
    assert_eq!(playlists.len(), 1);
    assert_eq!(playlists[0].1.len(), 0);

    let _ = fs::remove_dir_all(dir);
}

#[test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_playlist_scanner_duplicate_songs_preserves_multiplicity() {
    let dir = get_test_dir();
    let s1 = dir.join("repeat_song.mp3");
    fs::write(&s1, b"audio").unwrap();

    let m3u_path = dir.join("dups.m3u");
    let content =
        "#EXTM3U\n#PLAYLIST:Repeat PL\nrepeat_song.mp3\nrepeat_song.mp3\nrepeat_song.mp3\n";
    fs::write(&m3u_path, content).unwrap();

    let file_list = FileList {
        file_list: vec![],
        playlist_list: vec![m3u_path],
    };

    let scanner = PlaylistScanner::new(&file_list);
    let playlists = scanner.scan().unwrap();
    assert_eq!(playlists.len(), 1);
    assert_eq!(playlists[0].1.len(), 3);

    let _ = fs::remove_dir_all(dir);
}
