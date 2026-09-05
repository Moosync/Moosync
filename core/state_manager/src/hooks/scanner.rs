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
use preferences::preferences::PreferenceConfig;
use songs_proto::moosync::types::{GetSongOptions, InnerSong, SearchableSong, Song, SongType};
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

        {
            let mut file_scanner = scanner.write().await;
            file_scanner.set_artist_split(",".into());
            file_scanner.set_thumbnail_dir(temp_dir());

            let db_playlist = database.clone();
            file_scanner.set_on_playlist(move |playlists_with_songs| {
                let db = db_playlist.clone();
                async move {
                    let db_read = db.read().await;
                    for (playlist, song_identifiers) in playlists_with_songs {
                        let mut playlist_songs = Vec::new();
                        for identifier in song_identifiers {
                            if let Some(song) =
                                resolve_or_create_playlist_song(&db_read, identifier)
                            {
                                playlist_songs.push(song);
                            }
                        }
                        if let Err(e) =
                            db_read.create_playlist_with_songs(playlist, &playlist_songs)
                        {
                            tracing::error!("Failed to create playlist with songs: {:?}", e);
                        }
                    }
                }
                .in_current_span()
            });

            let db_song = database.clone();
            file_scanner.set_on_song(move |pl_id: Option<String>, songs| {
                let db = db_song.clone();
                async move {
                    let songs = match db.read().await.insert_songs(songs) {
                        Ok(songs) => songs,
                        Err(e) => {
                            tracing::error!("Failed to insert scanned songs: {:?}", e);
                            return;
                        }
                    };
                    if let Some(pl_id) = pl_id
                        && let Err(e) = db.read().await.add_to_playlist(&pl_id, &songs)
                    {
                        tracing::error!(
                            "Failed to add scanned songs to playlist {}: {:?}",
                            pl_id,
                            e
                        );
                    }
                }
                .in_current_span()
            });
        }

        preferences.read().await.on_preference_changed_immediate(
            {
                let database = database.clone();
                let scanner = scanner.clone();
                let preferences = preferences.clone();
                move |key| {
                    let database = database.clone();
                    let scanner = scanner.clone();
                    let preferences = preferences.clone();
                    tokio::spawn(
                        async move {
                            let prefs_read = preferences.read().await;
                            let mut scan_dirs = prefs_read
                                .load(preferences::keys::MusicPaths)
                                .unwrap_or_default()
                                .into_iter()
                                .map(PathBuf::from)
                                .collect::<Vec<_>>();

                            if scan_dirs.is_empty()
                                && let Some(user_dirs) = UserDirs::new()
                            {
                                scan_dirs.push(user_dirs.music_dir);
                            }

                            let exclude_dirs = prefs_read
                                .load(preferences::keys::ExcludeMusicPaths)
                                .unwrap_or_default()
                                .into_iter()
                                .map(PathBuf::from)
                                .collect::<Vec<_>>();

                            let threads =
                                prefs_read.load(preferences::keys::ScanThreads).unwrap_or(0);

                            {
                                let mut scanner_write = scanner.write().await;
                                scanner_write.set_scan_dirs(scan_dirs.clone());
                                scanner_write.set_exclude_dirs(exclude_dirs);
                                scanner_write.set_scan_threads(threads);
                            }

                            if key == preferences::keys::MusicPaths && !scan_dirs.is_empty() {
                                let db_read = database.read().await;
                                if let Err(e) = db_read.remove_songs_outside_directories(&scan_dirs)
                                {
                                    tracing::error!("Failed to clean up songs: {:?}", e);
                                }
                                let scanner_read = scanner.read().await;
                                if let Err(e) = scanner_read.start_scan().await {
                                    tracing::error!("Scan failed: {:?}", e);
                                }
                            }
                        }
                        .in_current_span(),
                    );
                }
            },
            vec![
                preferences::keys::MusicPaths.into(),
                preferences::keys::ExcludeMusicPaths.into(),
                preferences::keys::ScanThreads.into(),
            ],
        );

        let periodic_task = Arc::new(Mutex::new(None::<JoinHandle<()>>));

        preferences.read().await.on_preference_changed_immediate(
            {
                let scanner = scanner.clone();
                let preferences = preferences.clone();
                let periodic_task = periodic_task.clone();
                move |key| {
                    if key != preferences::keys::ScanInterval {
                        return;
                    }
                    let scanner = scanner.clone();
                    let preferences = preferences.clone();
                    let periodic_task = periodic_task.clone();
                    tokio::spawn(
                        async move {
                            let interval_mins = preferences
                                .read()
                                .await
                                .load(preferences::keys::ScanInterval)
                                .unwrap_or(0);

                            if let Ok(mut guard) = periodic_task.lock()
                                && let Some(handle) = guard.take()
                            {
                                handle.abort();
                            }

                            if interval_mins <= 0 {
                                return;
                            }

                            let scanner = scanner.clone();
                            let handle = tokio::spawn(
                                async move {
                                    loop {
                                        sleep(Duration::from_secs(interval_mins as u64 * 60)).await;
                                        let scanner = scanner.read().await;
                                        if let Err(e) = scanner.start_scan().await {
                                            tracing::error!("Periodic scan failed: {:?}", e);
                                        }
                                    }
                                }
                                .in_current_span(),
                            );
                            if let Ok(mut guard) = periodic_task.lock() {
                                *guard = Some(handle);
                            }
                        }
                        .in_current_span(),
                    );
                }
            },
            preferences::keys::ScanInterval,
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
