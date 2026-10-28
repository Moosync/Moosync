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
    env, fs,
    path::PathBuf,
    process::Command,
    sync::{Arc, Mutex},
};

use android_scanner_proto::moosync::types::{ScanRequest, ScanResult, ScannedPlaylist};
use assertables::{assert_err, assert_matches, assert_ok};
use prost::Message;
use rstest::{fixture, rstest};
use songs_proto::moosync::types::{
    Album, Artist, Genre, InnerSong, LyricLine, Lyrics, Playlist, Song, SongType,
};
use tempdir::TempDir;
use tracing_test::traced_test;

use crate::{
    OnPlaylistScanned, OnProgressUpdated, OnSongScanned, PlaylistSongId, ScanProgress,
    context::{
        ScannerContext,
        android::{AndroidScannerBridge, AndroidScannerContext, MockAndroidScannerBridge},
    },
    error::ScannerError,
};

struct KotlinProcessBridge {
    temp_dir: PathBuf,
}

impl KotlinProcessBridge {
    #[tracing::instrument(level = "debug", skip_all)]
    fn new(temp_dir: PathBuf) -> Self { Self { temp_dir } }
}

use async_trait::async_trait;

#[async_trait]
impl AndroidScannerBridge for KotlinProcessBridge {
    #[tracing::instrument(level = "debug", skip_all)]
    fn invoke_scan(&self, request_bytes: Vec<u8>) -> Result<Vec<u8>, ScannerError> {
        let req_path = self.temp_dir.join("scan_request.pb");
        let res_path = self.temp_dir.join("scan_result.pb");
        fs::write(&req_path, &request_bytes)?;
        let runner = env::var("SCANNER_KT_RUNNER")
            .map_err(|e| ScannerError::Jni(format!("SCANNER_KT_RUNNER env missing: {e}")))?;
        let status = Command::new(runner)
            .arg(&req_path)
            .arg(&res_path)
            .status()?;
        if !status.success() {
            return Err(ScannerError::Jni(format!(
                "Kotlin scanner process failed: {status}"
            )));
        }
        let result_bytes = fs::read(&res_path)?;
        Ok(result_bytes)
    }
}

#[fixture]
#[tracing::instrument(level = "debug", skip_all)]
fn temp_dir() -> TempDir {
    TempDir::new("android_scanner_test").expect("failed to create temp dir")
}

#[tracing::instrument(level = "debug", skip_all)]
fn sample_song(id: &str, title: &str, path: &str) -> Song {
    Song {
        song: Some(InnerSong {
            id: Some(id.to_string()),
            title: Some(title.to_string()),
            path: Some(path.to_string()),
            size: Some(1024.0),
            r#type: SongType::Local.into(),
            song_cover_path_high: Some("/thumbs/high.png".to_string()),
            song_cover_path_low: Some("/thumbs/low.png".to_string()),
            lyrics: Some(Lyrics {
                lines: vec![LyricLine {
                    text: "Line 1".to_string(),
                    time_ms: 1500,
                }],
                is_synced: true,
            }),
            ..Default::default()
        }),
        album: Some(Album {
            album_id: Some("album_1".to_string()),
            album_name: Some("Test Album".to_string()),
            ..Default::default()
        }),
        artists: vec![Artist {
            artist_id: Some("artist_1".to_string()),
            artist_name: Some("Test Artist".to_string()),
            ..Default::default()
        }],
        genre: vec![Genre {
            genre_name: Some("Rock".to_string()),
            ..Default::default()
        }],
    }
}

#[tracing::instrument(level = "debug", skip_all)]
fn noop_playlist_cb() -> OnPlaylistScanned { Arc::new(|_playlists| Box::pin(async {})) }

#[tracing::instrument(level = "debug", skip_all)]
fn noop_song_cb() -> OnSongScanned { Arc::new(|_playlist_id, _songs| Box::pin(async {})) }

#[tracing::instrument(level = "debug", skip_all)]
fn noop_progress_cb() -> OnProgressUpdated { Box::new(|_progress| {}) }

#[rstest]
#[tokio::test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_android_scanner_encodes_request_and_dispatches_songs(temp_dir: TempDir) {
    let expected_song = sample_song("s1", "Android Track", "/storage/emulated/0/Music/track.mp3");
    let expected_clone = expected_song.clone();
    let thumb_dir = temp_dir.path().join("thumbs");
    let thumb_dir_str = thumb_dir.to_string_lossy().to_string();
    let mut mock_bridge = MockAndroidScannerBridge::new();
    mock_bridge
        .expect_ensure_permission()
        .times(1)
        .returning(|| Box::pin(async { Ok(true) }));
    mock_bridge
        .expect_invoke_scan()
        .times(1)
        .returning(move |req_bytes| {
            let decoded_req = ScanRequest::decode(req_bytes.as_slice()).unwrap();
            assert_eq!(decoded_req.scan_dirs, vec!["/storage/emulated/0/Music"]);
            assert_eq!(
                decoded_req.exclude_dirs,
                vec!["/storage/emulated/0/Music/Trash"]
            );
            assert_eq!(decoded_req.thumbnail_dir, thumb_dir_str);
            assert_eq!(decoded_req.artist_split, ";");
            let res = ScanResult {
                songs: vec![expected_clone.clone()],
                playlists: vec![],
            };
            let mut out = Vec::new();
            res.encode(&mut out).unwrap();
            Ok(out)
        });
    let received_songs = Arc::new(Mutex::new(Vec::new()));
    let received_clone = received_songs.clone();
    let on_song: OnSongScanned = Arc::new(move |_pl, songs| {
        let store = received_clone.clone();
        Box::pin(async move {
            store.lock().unwrap().extend(songs);
        })
    });
    let context = AndroidScannerContext::new_with_bridge(
        Arc::new(mock_bridge),
        vec![PathBuf::from("/storage/emulated/0/Music")],
        vec![PathBuf::from("/storage/emulated/0/Music/Trash")],
        thumb_dir,
        ";".to_string(),
    );

    let result = context
        .start_scan(&on_song, &noop_playlist_cb(), &noop_progress_cb())
        .await;

    assert_ok!(result);
    let songs = received_songs.lock().unwrap();
    assert_eq!(songs.len(), 1);
    assert_eq!(songs[0], expected_song);
}

#[rstest]
#[tokio::test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_android_scanner_dispatches_playlists_with_paths_and_urls(temp_dir: TempDir) {
    let mut mock_bridge = MockAndroidScannerBridge::new();
    mock_bridge
        .expect_ensure_permission()
        .times(1)
        .returning(|| Box::pin(async { Ok(true) }));
    mock_bridge.expect_invoke_scan().times(1).returning(|_| {
        let res = ScanResult {
            songs: vec![],
            playlists: vec![ScannedPlaylist {
                playlist: Some(Playlist {
                    playlist_id: Some("pl_1".to_string()),
                    playlist_name: "Favorites".to_string(),
                    ..Default::default()
                }),
                song_paths: vec![
                    "/storage/emulated/0/Music/a.mp3".to_string(),
                    "https://stream.example.com/live.ogg".to_string(),
                ],
            }],
        };
        let mut out = Vec::new();
        res.encode(&mut out).unwrap();
        Ok(out)
    });
    let received_playlists = Arc::new(Mutex::new(Vec::new()));
    let received_clone = received_playlists.clone();
    let on_playlist: OnPlaylistScanned = Arc::new(move |playlists| {
        let store = received_clone.clone();
        Box::pin(async move {
            store.lock().unwrap().extend(playlists);
        })
    });
    let context = AndroidScannerContext::new_with_bridge(
        Arc::new(mock_bridge),
        vec![],
        vec![],
        temp_dir.path().to_path_buf(),
        ";".to_string(),
    );

    let result = context
        .start_scan(&noop_song_cb(), &on_playlist, &noop_progress_cb())
        .await;

    assert_ok!(result);
    let playlists = received_playlists.lock().unwrap();
    assert_eq!(playlists.len(), 1);
    assert_eq!(playlists[0].0.playlist_name, "Favorites");
    assert_eq!(
        playlists[0].1,
        vec![
            PlaylistSongId::Path(PathBuf::from("/storage/emulated/0/Music/a.mp3")),
            PlaylistSongId::Url("https://stream.example.com/live.ogg".to_string()),
        ]
    );
}

#[rstest]
#[tokio::test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_android_scanner_reports_progress_to_completion(temp_dir: TempDir) {
    let mut mock_bridge = MockAndroidScannerBridge::new();
    mock_bridge
        .expect_ensure_permission()
        .times(1)
        .returning(|| Box::pin(async { Ok(true) }));
    mock_bridge.expect_invoke_scan().times(1).returning(|_| {
        let res = ScanResult {
            songs: vec![
                sample_song("1", "One", "/music/1.mp3"),
                sample_song("2", "Two", "/music/2.mp3"),
            ],
            playlists: vec![],
        };
        let mut out = Vec::new();
        res.encode(&mut out).unwrap();
        Ok(out)
    });
    let events = Arc::new(Mutex::new(Vec::new()));
    let events_clone = events.clone();
    let on_progress: OnProgressUpdated = Box::new(move |progress| {
        events_clone.lock().unwrap().push(progress);
    });
    let context = AndroidScannerContext::new_with_bridge(
        Arc::new(mock_bridge),
        vec![],
        vec![],
        temp_dir.path().to_path_buf(),
        ";".to_string(),
    );

    let result = context
        .start_scan(&noop_song_cb(), &noop_playlist_cb(), &on_progress)
        .await;

    assert_ok!(result);
    let recorded = events.lock().unwrap().clone();
    assert_eq!(
        recorded,
        vec![
            ScanProgress::PROGRESS(50),
            ScanProgress::PROGRESS(100),
            ScanProgress::STOPPED,
        ]
    );
}

#[rstest]
#[tokio::test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_android_scanner_invalid_protobuf_returns_decode_error(temp_dir: TempDir) {
    let mut mock_bridge = MockAndroidScannerBridge::new();
    mock_bridge
        .expect_ensure_permission()
        .times(1)
        .returning(|| Box::pin(async { Ok(true) }));
    mock_bridge
        .expect_invoke_scan()
        .times(1)
        .returning(|_| Ok(vec![0xFF, 0xFF, 0xFF]));
    let context = AndroidScannerContext::new_with_bridge(
        Arc::new(mock_bridge),
        vec![],
        vec![],
        temp_dir.path().to_path_buf(),
        ";".to_string(),
    );

    let result = context
        .start_scan(&noop_song_cb(), &noop_playlist_cb(), &noop_progress_cb())
        .await;

    assert_err!(result.as_ref());
    assert_matches!(result.unwrap_err(), ScannerError::Decode(_));
}

#[rstest]
#[tokio::test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_android_scanner_bridge_error_propagates(temp_dir: TempDir) {
    let mut mock_bridge = MockAndroidScannerBridge::new();
    mock_bridge
        .expect_ensure_permission()
        .times(1)
        .returning(|| Box::pin(async { Ok(true) }));
    mock_bridge
        .expect_invoke_scan()
        .times(1)
        .returning(|_| Err(ScannerError::Jni("JNI failure".to_string())));
    let context = AndroidScannerContext::new_with_bridge(
        Arc::new(mock_bridge),
        vec![],
        vec![],
        temp_dir.path().to_path_buf(),
        ";".to_string(),
    );

    let result = context
        .start_scan(&noop_song_cb(), &noop_playlist_cb(), &noop_progress_cb())
        .await;

    assert_err!(result.as_ref());
    assert_matches!(result.unwrap_err(), ScannerError::Jni(_));
}

#[rstest]
#[tokio::test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_android_scanner_permission_denied_aborts_scan(temp_dir: TempDir) {
    let mut mock_bridge = MockAndroidScannerBridge::new();
    mock_bridge
        .expect_ensure_permission()
        .times(1)
        .returning(|| Box::pin(async { Ok(false) }));
    let context = AndroidScannerContext::new_with_bridge(
        Arc::new(mock_bridge),
        vec![],
        vec![],
        temp_dir.path().to_path_buf(),
        ";".to_string(),
    );

    let result = context
        .start_scan(&noop_song_cb(), &noop_playlist_cb(), &noop_progress_cb())
        .await;

    assert_err!(result.as_ref());
    assert_matches!(result.unwrap_err(), ScannerError::PermissionDenied);
}

#[rstest]
#[tokio::test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_android_scanner_kotlin_end_to_end(temp_dir: TempDir) {
    let music_dir = temp_dir.path().join("Music");
    let excluded_dir = music_dir.join("Excluded");
    let thumb_dir = temp_dir.path().join("Thumbs");
    fs::create_dir_all(&music_dir).unwrap();
    fs::create_dir_all(&excluded_dir).unwrap();
    fs::create_dir_all(&thumb_dir).unwrap();
    let song_path = music_dir.join("E2E_Track.mp3");
    fs::write(&song_path, vec![1u8, 2, 3, 4, 5]).unwrap();
    fs::write(
        music_dir.join("E2E_Track.lrc"),
        "[00:03.25]Synced Kotlin Lyric",
    )
    .unwrap();
    fs::write(music_dir.join("cover.png"), vec![10u8, 20, 30, 40]).unwrap();
    fs::write(excluded_dir.join("Skip.mp3"), vec![9u8, 9, 9]).unwrap();
    fs::write(
        music_dir.join("e2e_list.m3u8"),
        "E2E_Track.mp3\nhttps://example.com/live.mp3\n",
    )
    .unwrap();
    let scanned_songs = Arc::new(Mutex::new(Vec::new()));
    let songs_clone = scanned_songs.clone();
    let on_song: OnSongScanned = Arc::new(move |_pl, songs| {
        let store = songs_clone.clone();
        Box::pin(async move {
            store.lock().unwrap().extend(songs);
        })
    });
    let scanned_playlists = Arc::new(Mutex::new(Vec::new()));
    let playlists_clone = scanned_playlists.clone();
    let on_playlist: OnPlaylistScanned = Arc::new(move |playlists| {
        let store = playlists_clone.clone();
        Box::pin(async move {
            store.lock().unwrap().extend(playlists);
        })
    });
    let bridge = Arc::new(KotlinProcessBridge::new(temp_dir.path().to_path_buf()));
    let context = AndroidScannerContext::new_with_bridge(
        bridge,
        vec![music_dir],
        vec![excluded_dir],
        thumb_dir,
        ";".to_string(),
    );

    let result = context
        .start_scan(&on_song, &on_playlist, &noop_progress_cb())
        .await;

    assert_ok!(result);
    let songs = scanned_songs.lock().unwrap();
    assert_eq!(songs.len(), 1);
    let inner = songs[0].song.as_ref().unwrap();
    assert_eq!(inner.title.as_deref(), Some("E2E_Track"));
    assert!(inner.song_cover_path_high.is_some());
    assert!(inner.song_cover_path_low.is_some());
    let lyrics = inner.lyrics.as_ref().unwrap();
    assert!(lyrics.is_synced);
    assert_eq!(lyrics.lines.len(), 1);
    assert_eq!(lyrics.lines[0].text, "Synced Kotlin Lyric");
    assert_eq!(lyrics.lines[0].time_ms, 3250);
    let playlists = scanned_playlists.lock().unwrap();
    assert_eq!(playlists.len(), 1);
    assert_eq!(playlists[0].0.playlist_name, "e2e_list");
    assert_eq!(playlists[0].1.len(), 2);
}
