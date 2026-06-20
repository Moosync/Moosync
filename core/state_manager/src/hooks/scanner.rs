use std::{env::temp_dir, error::Error};

use async_trait::async_trait;
use database::Database;
use file_scanner::PlaylistSongId;
use songs_proto::moosync::types::Song;

use super::Hook;
use crate::StateManager;

pub struct ScannerHook;

impl ScannerHook {
    pub fn new() -> Self { Self }
}

#[async_trait]
impl Hook for ScannerHook {
    async fn on_startup(
        &self,
        state_manager: &StateManager,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        let scanner = state_manager.plugins.get::<file_scanner::ScannerHolder>();
        let preferences = state_manager
            .plugins
            .get::<preferences::preferences::PreferenceConfig>();
        let database = state_manager.plugins.get::<Database>();

        {
            let mut file_scanner = scanner.write().await;
            file_scanner.set_artist_split(",".into());
            file_scanner.set_thumbnail_dir(temp_dir());

            let prefs_read = preferences.read().await;
            let mut scan_dirs = prefs_read
                .load_selective::<Vec<String>>("music_paths".to_string())
                .unwrap_or_default()
                .into_iter()
                .map(std::path::PathBuf::from)
                .collect::<Vec<_>>();

            if scan_dirs.is_empty() {
                if let Some(user_dirs) = platform_dirs::UserDirs::new() {
                    scan_dirs.push(user_dirs.music_dir);
                }
            }

            if !scan_dirs.is_empty() {
                file_scanner.set_scan_dirs(scan_dirs.clone());
            }

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
            });

            let db_song = database.clone();
            file_scanner.set_on_song(move |pl_id: Option<String>, songs| {
                let db = db_song.clone();
                async move {
                    if let Ok(songs) = db.read().await.insert_songs(songs) {
                        if let Some(pl_id) = pl_id {
                            let _ = db.read().await.add_to_playlist(&pl_id, &songs);
                        }
                    }
                }
            });

            if !scan_dirs.is_empty() {
                let db = database.clone();
                let scan_dirs_startup = scan_dirs.clone();
                tokio::spawn(async move {
                    let db_read = db.read().await;
                    if let Err(e) = db_read.remove_songs_outside_directories(&scan_dirs_startup) {
                        tracing::error!(
                            "Failed to clean up songs outside scan directories on startup: {:?}",
                            e
                        );
                    }
                });
            }

            let db = database.clone();
            let preferences_cb = preferences.clone();
            let _handle = prefs_read.on_preference_changed(move |key| {
                if key == "music_paths" {
                    let db = db.clone();
                    let preferences_inner = preferences_cb.clone();
                    tokio::spawn(async move {
                        let prefs = preferences_inner.read().await;
                        let mut scan_dirs = prefs
                            .load_selective::<Vec<String>>("music_paths".to_string())
                            .unwrap_or_default()
                            .into_iter()
                            .map(std::path::PathBuf::from)
                            .collect::<Vec<_>>();

                        if scan_dirs.is_empty() {
                            if let Some(user_dirs) = platform_dirs::UserDirs::new() {
                                scan_dirs.push(user_dirs.music_dir);
                            }
                        }

                        if !scan_dirs.is_empty() {
                            let db_read = db.read().await;
                            if let Err(e) = db_read.remove_songs_outside_directories(&scan_dirs) {
                                tracing::error!("Failed to clean up songs outside scan directories on preference change: {:?}", e);
                            }
                        }
                    });
                }
            });
        }

        tokio::task::spawn(async move {
            let scanner = scanner.read().await;
            if let Err(e) = scanner.start_scan().await {
                tracing::error!("Failed to start scan: {:?}", e);
            }
        });

        Ok(())
    }
}

fn resolve_or_create_playlist_song(db: &Database, identifier: PlaylistSongId) -> Option<Song> {
    let opt = match &identifier {
        PlaylistSongId::Url(url) => songs_proto::moosync::types::GetSongOptions {
            song: Some(songs_proto::moosync::types::SearchableSong {
                playback_url: Some(url.clone()),
                ..Default::default()
            }),
            ..Default::default()
        },
        PlaylistSongId::Path(path) => songs_proto::moosync::types::GetSongOptions {
            song: Some(songs_proto::moosync::types::SearchableSong {
                path: Some(path.to_string_lossy().to_string()),
                ..Default::default()
            }),
            ..Default::default()
        },
    };

    let songs = db.get_songs_by_options(opt).ok()?;
    if let Some(song) = songs.into_iter().next() {
        return Some(song);
    }

    let mut inner_song = songs_proto::moosync::types::InnerSong::default();
    inner_song.id = Some(uuid::Uuid::new_v4().to_string());
    match identifier {
        PlaylistSongId::Url(url) => {
            inner_song.r#type = songs_proto::moosync::types::SongType::Url.into();
            inner_song.playback_url = Some(url);
        }
        PlaylistSongId::Path(path) => {
            inner_song.r#type = songs_proto::moosync::types::SongType::Local.into();
            let path_str = path.to_string_lossy().to_string();
            inner_song.path = Some(path_str.clone());
            let title = path
                .file_stem()
                .map(|s| s.to_string_lossy().to_string())
                .unwrap_or(path_str);
            inner_song.title = Some(title);
        }
    }

    let proto_song = Song {
        song: Some(inner_song),
        ..Default::default()
    };

    let inserted = db.insert_songs(vec![proto_song]).ok()?;
    inserted.into_iter().next()
}
