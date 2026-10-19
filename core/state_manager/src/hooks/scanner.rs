use std::{
    env::temp_dir,
    error::Error,
    path::PathBuf,
    sync::{Arc, Mutex},
};

use async_trait::async_trait;
use database::Database;
use file_scanner::{PlaylistSongId, ScannerHolder};
use platform_dirs::UserDirs;
use preferences::{
    keys::{EXCLUDE_MUSIC_PATHS, MUSIC_PATHS, PreferenceItemExt, SCAN_INTERVAL, SCAN_THREADS},
    preferences::PreferenceConfig,
};
use songs_proto::moosync::types::{
    GetSongOptions, InnerSong, Playlist, SearchableSong, Song, SongType,
};
use tokio::{
    task::JoinHandle,
    time::{Duration, sleep},
};
use tracing::Instrument;

use super::Hook;
use crate::StateManager;

pub struct ScannerHook;

impl Default for ScannerHook {
    fn default() -> Self { Self::new() }
}

impl ScannerHook {
    #[tracing::instrument(level = "debug", skip_all)]
    pub fn new() -> Self { Self }
}

#[tracing::instrument(level = "debug", skip_all)]
async fn save_scanned_playlists(
    database: Arc<Database>,
    playlists: Vec<(Playlist, Vec<PlaylistSongId>)>,
) {
    for (playlist, song_identifiers) in playlists {
        let mut playlist_songs = Vec::new();
        for identifier in song_identifiers {
            if let Some(song) = resolve_or_create_playlist_song(&database, identifier) {
                playlist_songs.push(song);
            }
        }
        if let Err(e) = database.create_playlist_with_songs(playlist, &playlist_songs) {
            tracing::error!("Failed to create playlist with songs: {:?}", e);
        }
    }
}

#[tracing::instrument(level = "debug", skip_all)]
async fn save_scanned_songs(
    database: Arc<Database>,
    playlist_id: Option<String>,
    songs: Vec<Song>,
) {
    let songs = match database.insert_songs(songs) {
        Ok(songs) => songs,
        Err(e) => {
            tracing::error!("Failed to insert scanned songs: {:?}", e);
            return;
        }
    };
    if let Some(playlist_id) = playlist_id
        && let Err(e) = database.add_to_playlist(&playlist_id, &songs)
    {
        tracing::error!(
            "Failed to add scanned songs to playlist {}: {:?}",
            playlist_id,
            e
        );
    }
}

#[tracing::instrument(level = "debug", skip_all)]
async fn sync_music_paths(
    key: String,
    scanner: Arc<ScannerHolder>,
    preferences: Arc<PreferenceConfig>,
    database: Arc<Database>,
) {
    let mut scan_dirs = preferences
        .load(&MUSIC_PATHS)
        .value::<Vec<String>>()
        .unwrap_or_default()
        .into_iter()
        .map(PathBuf::from)
        .collect::<Vec<_>>();

    if scan_dirs.is_empty()
        && let Some(user_dirs) = UserDirs::new()
    {
        scan_dirs.push(user_dirs.music_dir);
    }

    let exclude_dirs = preferences
        .load(&EXCLUDE_MUSIC_PATHS)
        .value::<Vec<String>>()
        .unwrap_or_default()
        .into_iter()
        .map(PathBuf::from)
        .collect::<Vec<_>>();

    let threads = preferences.load(&SCAN_THREADS).value::<i32>().unwrap_or(0);

    scanner.set_scan_dirs(scan_dirs.clone());
    scanner.set_exclude_dirs(exclude_dirs);
    scanner.set_scan_threads(threads);

    if key == MUSIC_PATHS.id && !scan_dirs.is_empty() {
        if let Err(e) = database.remove_songs_outside_directories(&scan_dirs) {
            tracing::error!("Failed to clean up songs: {:?}", e);
        }
        if let Err(e) = scanner.start_scan().await {
            tracing::error!("Scan failed: {:?}", e);
        }
    }
}

#[tracing::instrument(level = "debug", skip_all)]
fn update_periodic_scan(
    interval_mins: i32,
    scanner: Arc<ScannerHolder>,
    periodic_task: &mut Option<JoinHandle<()>>,
) {
    if let Some(handle) = periodic_task.take() {
        handle.abort();
    }

    if interval_mins <= 0 {
        return;
    }

    *periodic_task = Some(tokio::spawn(
        async move {
            loop {
                sleep(Duration::from_secs(interval_mins as u64 * 60)).await;
                if let Err(e) = scanner.start_scan().await {
                    tracing::error!("Periodic scan failed: {:?}", e);
                }
            }
        }
        .in_current_span(),
    ));
}

#[async_trait]
impl Hook for ScannerHook {
    #[tracing::instrument(level = "debug", skip_all)]
    async fn on_startup(
        &self,
        state_manager: &StateManager,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        let scanner = state_manager.plugins.get::<ScannerHolder>();
        let preferences = state_manager.plugins.get::<PreferenceConfig>();
        let database = state_manager.plugins.get::<Database>();

        scanner.set_artist_split(",".into());
        scanner.set_thumbnail_dir(temp_dir());

        let db = database.clone();
        scanner.set_on_playlist(move |playlists| {
            save_scanned_playlists(db.clone(), playlists).in_current_span()
        });

        let db = database.clone();
        scanner.set_on_song(move |playlist_id, songs| {
            save_scanned_songs(db.clone(), playlist_id, songs).in_current_span()
        });

        preferences.on_preference_changed.listen_filtered_immediate(
            {
                let database = database.clone();
                let scanner = scanner.clone();
                let preferences = preferences.clone();
                move |key| {
                    tokio::spawn(
                        sync_music_paths(
                            key,
                            scanner.clone(),
                            preferences.clone(),
                            database.clone(),
                        )
                        .in_current_span(),
                    );
                }
            },
            vec![
                MUSIC_PATHS.id.clone(),
                EXCLUDE_MUSIC_PATHS.id.clone(),
                SCAN_THREADS.id.clone(),
            ],
        );

        let periodic_task = Arc::new(Mutex::new(None));
        preferences.on_preference_changed.listen_filtered_immediate(
            {
                let scanner = scanner.clone();
                let preferences = preferences.clone();
                move |_| {
                    let interval = preferences.load(&SCAN_INTERVAL).value::<i32>().unwrap_or(0);
                    if let Ok(mut guard) = periodic_task.lock() {
                        update_periodic_scan(interval, scanner.clone(), &mut guard);
                    }
                }
            },
            vec![SCAN_INTERVAL.id.clone()],
        );

        Ok(())
    }
}

#[tracing::instrument(level = "debug", skip_all)]
fn resolve_or_create_playlist_song(db: &Database, identifier: PlaylistSongId) -> Option<Song> {
    let opt = match &identifier {
        PlaylistSongId::Url(url) => GetSongOptions {
            song: Some(SearchableSong {
                playback_url: Some(url.clone()),
                ..Default::default()
            }),
            ..Default::default()
        },
        PlaylistSongId::Path(path) => GetSongOptions {
            song: Some(SearchableSong {
                path: Some(path.to_string_lossy().to_string()),
                ..Default::default()
            }),
            ..Default::default()
        },
    };

    let songs = match db.get_songs_by_options(opt) {
        Ok(songs) => songs,
        Err(e) => {
            tracing::error!("Failed to get song by options: {:?}", e);
            return None;
        }
    };
    if let Some(song) = songs.into_iter().next() {
        return Some(song);
    }

    let (song_type, playback_url, path, title) = match identifier {
        PlaylistSongId::Url(url) => (SongType::Url.into(), Some(url), None, None),
        PlaylistSongId::Path(p) => {
            let path_str = p.to_string_lossy().to_string();
            let title = p
                .file_stem()
                .map(|s| s.to_string_lossy().to_string())
                .unwrap_or_else(|| path_str.clone());
            (SongType::Local.into(), None, Some(path_str), Some(title))
        }
    };

    let inner_song = InnerSong {
        id: Some(uuid::Uuid::new_v4().to_string()),
        r#type: song_type,
        playback_url,
        path,
        title,
        ..Default::default()
    };

    let proto_song = Song {
        song: Some(inner_song),
        ..Default::default()
    };

    let inserted = match db.insert_songs(vec![proto_song]) {
        Ok(inserted) => inserted,
        Err(e) => {
            tracing::error!("Failed to insert resolved playlist song: {:?}", e);
            return None;
        }
    };
    inserted.into_iter().next()
}

#[cfg(test)]
#[path = "scanner_test.rs"]
mod scanner_test;
