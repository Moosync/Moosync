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

use std::{path::PathBuf, time::Duration};

use assertables::assert_ok;
use database::Database;
use file_scanner::{PlaylistSongId, ScannerHolder};
use preferences::keys::SCAN_THREADS;
use preferences_proto::moosync::types::{PreferenceItem, PreferenceValue, preference_value};
use rstest::{fixture, rstest};
use songs_proto::moosync::types::{
    GetEntityOptions, GetSongOptions, InnerSong, Playlist, SearchableSong, Song, SongType,
    entity_result::Result as EntityResultVariant,
};
use tempdir::TempDir;
use tracing_test::traced_test;
use types::plugin::PluginContext;

use crate::{
    StateManager,
    hooks::{Hook, scanner::ScannerHook},
};

struct TestSmContext {
    pub _temp_dir: TempDir,
    pub sm: StateManager,
}

#[fixture]
#[tracing::instrument(level = "debug", skip_all)]
fn sm_context() -> TestSmContext {
    let temp_dir = TempDir::new("moosync_sm_scanner_hook_test").expect("failed to create temp dir");
    let test_dir = temp_dir.path().to_path_buf();
    let context = PluginContext {
        data_dir: test_dir.clone(),
        cache_dir: test_dir.clone(),
        tmp_dir: test_dir.clone(),
        #[cfg(target_os = "android")]
        android_context: types::android::AndroidJNIContext::default(),
    };
    let sm = StateManager::new_with_context(context).expect("failed to create state manager");
    TestSmContext {
        _temp_dir: temp_dir,
        sm,
    }
}

#[rstest]
#[tokio::test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_scanner_hook_on_startup(sm_context: TestSmContext) {
    let TestSmContext { sm, .. } = sm_context;
    let hook = ScannerHook::new();

    assert_ok!(hook.on_startup(&sm).await);

    let pref = sm.get_preference_config();
    pref.save(PreferenceItem {
        id: SCAN_THREADS.id.clone(),
        value: Some(PreferenceValue {
            value: Some(preference_value::Value::NumberValue(4.0)),
        }),
        ..SCAN_THREADS.clone()
    })
    .unwrap();
    tokio::time::sleep(Duration::from_millis(50)).await;

    let scanner = sm.get_scanner_holder();
    assert_eq!(scanner.get_scan_threads(), Some(4));
}

#[rstest]
#[tokio::test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_save_scanned_songs_without_playlist(sm_context: TestSmContext) {
    let TestSmContext { sm, .. } = sm_context;
    let db = sm.plugins.get::<Database>();
    let test_song = Song {
        song: Some(InnerSong {
            id: Some("scanner_song_1".to_string()),
            title: Some("Song 1".to_string()),
            path: Some("/music/song1.mp3".to_string()),
            r#type: SongType::Local.into(),
            ..Default::default()
        }),
        ..Default::default()
    };

    super::save_scanned_songs(db.clone(), None, vec![test_song]).await;

    let songs = db
        .get_songs_by_options(GetSongOptions {
            song: Some(SearchableSong {
                title: Some("Song 1".to_string()),
                ..Default::default()
            }),
            ..Default::default()
        })
        .expect("failed to get songs");
    assert_eq!(songs.len(), 1);
    assert_eq!(
        songs[0].song.as_ref().unwrap().title.as_deref(),
        Some("Song 1")
    );
}

#[rstest]
#[tokio::test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_save_scanned_songs_with_playlist(sm_context: TestSmContext) {
    let TestSmContext { sm, .. } = sm_context;
    let db = sm.plugins.get::<Database>();
    let playlist_id = db
        .create_playlist(Playlist {
            playlist_name: "Test Playlist".to_string(),
            ..Default::default()
        })
        .expect("failed to create playlist");

    let test_song = Song {
        song: Some(InnerSong {
            id: Some("scanner_song_2".to_string()),
            title: Some("Song 2".to_string()),
            path: Some("/music/song2.mp3".to_string()),
            r#type: SongType::Local.into(),
            ..Default::default()
        }),
        ..Default::default()
    };

    super::save_scanned_songs(db.clone(), Some(playlist_id.clone()), vec![test_song]).await;

    let songs = db
        .get_songs_by_options(GetSongOptions {
            playlist: Some(Playlist {
                playlist_id: Some(playlist_id),
                ..Default::default()
            }),
            ..Default::default()
        })
        .expect("failed to get playlist songs");
    assert_eq!(songs.len(), 1);
    assert_eq!(
        songs[0].song.as_ref().unwrap().title.as_deref(),
        Some("Song 2")
    );
}

#[rstest]
#[tokio::test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_resolve_or_create_playlist_song_path_new(sm_context: TestSmContext) {
    let TestSmContext { sm, .. } = sm_context;
    let db = sm.plugins.get::<Database>();
    let path = PathBuf::from("/music/resolved_song.flac");

    let resolved = super::resolve_or_create_playlist_song(&db, PlaylistSongId::Path(path.clone()));

    assert!(resolved.is_some());
    let song = resolved.unwrap();
    let inner = song.song.expect("inner song is missing");
    assert_eq!(inner.path.as_deref(), Some("/music/resolved_song.flac"));
    assert_eq!(inner.title.as_deref(), Some("resolved_song"));
}

#[rstest]
#[tokio::test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_resolve_or_create_playlist_song_url_new(sm_context: TestSmContext) {
    let TestSmContext { sm, .. } = sm_context;
    let db = sm.plugins.get::<Database>();
    let url = "https://example.com/stream.mp3".to_string();

    let resolved = super::resolve_or_create_playlist_song(&db, PlaylistSongId::Url(url.clone()));

    assert!(resolved.is_some());
    let song = resolved.unwrap();
    let inner = song.song.expect("inner song is missing");
    assert_eq!(
        inner.playback_url.as_deref(),
        Some("https://example.com/stream.mp3")
    );
}

#[rstest]
#[tokio::test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_resolve_or_create_playlist_song_existing(sm_context: TestSmContext) {
    let TestSmContext { sm, .. } = sm_context;
    let db = sm.plugins.get::<Database>();
    let existing_song = Song {
        song: Some(InnerSong {
            id: Some("existing_1".to_string()),
            title: Some("Existing Song".to_string()),
            path: Some("/music/existing.mp3".to_string()),
            r#type: SongType::Local.into(),
            ..Default::default()
        }),
        ..Default::default()
    };
    db.insert_songs(vec![existing_song]).expect("insert failed");

    let resolved = super::resolve_or_create_playlist_song(
        &db,
        PlaylistSongId::Path(PathBuf::from("/music/existing.mp3")),
    );

    assert!(resolved.is_some());
    let song = resolved.unwrap();
    assert_eq!(
        song.song.as_ref().unwrap().id.as_deref(),
        Some("existing_1")
    );
}

#[rstest]
#[tokio::test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_save_scanned_playlists(sm_context: TestSmContext) {
    let TestSmContext { sm, .. } = sm_context;
    let db = sm.plugins.get::<Database>();
    let playlist = Playlist {
        playlist_name: "Imported Playlist".to_string(),
        ..Default::default()
    };
    let identifiers = vec![
        PlaylistSongId::Path(PathBuf::from("/music/song_a.mp3")),
        PlaylistSongId::Url("https://moosync.app/stream.ogg".to_string()),
    ];

    super::save_scanned_playlists(db.clone(), vec![(playlist, identifiers)]).await;

    let res = db
        .get_entity_by_options(GetEntityOptions {
            playlist: Some(Playlist::default()),
            ..Default::default()
        })
        .expect("failed to get entities");
    let Some(EntityResultVariant::Playlists(playlists_list)) = res.result else {
        panic!("Expected Playlists variant");
    };
    assert_eq!(playlists_list.playlists.len(), 1);
    assert_eq!(
        playlists_list.playlists[0].playlist_name,
        "Imported Playlist"
    );

    let playlist_id = playlists_list.playlists[0].playlist_id.as_ref().unwrap();
    let songs = db
        .get_songs_by_options(GetSongOptions {
            playlist: Some(Playlist {
                playlist_id: Some(playlist_id.clone()),
                ..Default::default()
            }),
            ..Default::default()
        })
        .expect("failed to get playlist songs");
    assert_eq!(songs.len(), 2);
}

#[rstest]
#[tokio::test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_update_periodic_scan(sm_context: TestSmContext) {
    let TestSmContext { sm, .. } = sm_context;
    let scanner = sm.plugins.get::<ScannerHolder>();
    let mut task_handle = None;

    super::update_periodic_scan(0, scanner.clone(), &mut task_handle);
    assert!(task_handle.is_none());

    super::update_periodic_scan(10, scanner.clone(), &mut task_handle);
    assert!(task_handle.is_some());

    super::update_periodic_scan(0, scanner.clone(), &mut task_handle);
    assert!(task_handle.is_none());
}

#[rstest]
#[tokio::test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_sync_music_paths(sm_context: TestSmContext) {
    let TestSmContext { sm, .. } = sm_context;
    let scanner = sm.plugins.get::<ScannerHolder>();
    let preferences = sm
        .plugins
        .get::<preferences::preferences::PreferenceConfig>();
    let database = sm.plugins.get::<Database>();

    super::sync_music_paths(
        "other_key".to_string(),
        scanner.clone(),
        preferences.clone(),
        database.clone(),
    )
    .await;

    super::sync_music_paths(
        preferences::keys::MUSIC_PATHS.id.clone(),
        scanner.clone(),
        preferences.clone(),
        database.clone(),
    )
    .await;

    assert!(!scanner.get_scan_dirs().is_empty());
}
