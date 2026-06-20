use std::{env::temp_dir, sync::Arc};

use database::Database;
use extensions::ExtensionHandler;
use file_scanner::{PlaylistSongId, ScannerHolder};
use player::PlayerHandler;
use songs_proto::moosync::types::Song;
use tempdir;
use tokio::runtime::Handle;
#[cfg(target_os = "android")]
use types::android::AndroidJNIContext;
use types::{
    plugin::{PluginContext, PluginRegistry},
    subscription::SubscriberList,
};

pub mod interceptors;
mod reply_handler;
use crate::reply_handler::StateReplyHandler;

plugin_macro::generate_plugin_system!(
    preferences::preferences::PreferenceConfig,
    database::Database,
    file_scanner::ScannerHolder,
    lyrics::LyricsFetcher,
    extensions::ExtensionHandler,
    player::PlayerHandler,
    themes::themes::ThemeHolder,
    mpris::MprisHolder
);

#[derive(Debug, thiserror::Error)]
pub enum StateManagerError {
    #[error("State initialization error: {0}")]
    InitializeError(Box<dyn std::error::Error + Send + Sync>),
}

#[derive(Clone)]
pub struct StateManager {
    pub plugins: Arc<PluginRegistry>,
    pub interceptors: Arc<Interceptors>,
    pub cache_dir: std::path::PathBuf,

    runtime: Handle,
    pub on_extensions_updated: SubscriberList<Box<dyn Fn(()) + Send + Sync + 'static>>,
}

impl StateManager {
    pub fn get_cache_dir(&self) -> std::path::PathBuf { self.cache_dir.clone() }

    fn get_dirs() -> (std::path::PathBuf, std::path::PathBuf, std::path::PathBuf) {
        #[cfg(not(target_os = "android"))]
        let (data_dir, cache_dir) = {
            let platform = platform_dirs::AppDirs::new(Some("moosync"), false)
                .expect("Could not retrieve directory to store data");
            (platform.data_dir, platform.cache_dir)
        };

        #[cfg(target_os = "android")]
        let (data_dir, cache_dir) = {
            let data_dir = std::path::PathBuf::from("/data/data/app.moosync.android/files");
            let cache_dir = std::path::PathBuf::from("/data/data/app.moosync.android/cache");
            std::fs::create_dir_all(&data_dir).ok();
            std::fs::create_dir_all(&cache_dir).ok();
            (data_dir, cache_dir)
        };

        let tmp = tempdir::TempDir::new("moosync")
            .expect("Failed to create tmp dir")
            .into_path();

        (data_dir, cache_dir, tmp)
    }

    fn generate_context(
        data_dir: std::path::PathBuf,
        cache_dir: std::path::PathBuf,
        tmp_dir: std::path::PathBuf,
        #[cfg(target_os = "android")] android_context: AndroidJNIContext,
    ) -> PluginContext {
        PluginContext {
            data_dir,
            cache_dir,
            tmp_dir,
            #[cfg(target_os = "android")]
            android_context,
        }
    }

    pub fn new(
        #[cfg(target_os = "android")] android_context: AndroidJNIContext,
    ) -> Result<Self, StateManagerError> {
        let (data_dir, cache_dir, tmp_dir) = Self::get_dirs();

        let context = Self::generate_context(
            data_dir,
            cache_dir.clone(),
            tmp_dir,
            #[cfg(target_os = "android")]
            android_context,
        );

        let mut plugins = PluginRegistry::new();
        init_all_plugins(&mut plugins, &context);

        let interceptors = Interceptors::default();
        let runtime = Handle::current();

        Ok(Self {
            plugins: Arc::new(plugins),
            interceptors: Arc::new(interceptors),
            cache_dir,
            runtime,
            on_extensions_updated: SubscriberList::new(),
        })
    }

    pub async fn setup(&self) {
        self.setup_extensions().await;
        self.setup_themes().await;
        self.setup_player().await;
        self.setup_scanner().await;
    }

    async fn setup_extensions(&self) {
        // Set reply_handler in ExtensionHandler
        let extensions = self.plugins.get::<ExtensionHandler>();
        let extensions_cl = extensions.clone();

        let reply_handler = Arc::new(StateReplyHandler::new(self.clone()));
        let mut ext_handle = extensions.write().await;
        ext_handle.set_reply_handler(reply_handler);

        tokio::spawn(async move {
            let ext_handle = extensions_cl.read().await;
            if let Err(e) = ext_handle.find_new_extensions() {
                tracing::error!("Failed to find new extensions: {:?}", e);
            }
        });
    }

    async fn setup_themes(&self) {}

    async fn setup_player(&self) {
        // TODO: create source resolver for player
        let player_handler = self.plugins.get::<PlayerHandler>();
        player_handler
            .read()
            .await
            .set_resolver(Box::new(|_| Ok("".into())));
    }

    async fn setup_scanner(&self) {
        let scanner = self.plugins.get::<ScannerHolder>();
        let preferences = self
            .plugins
            .get::<preferences::preferences::PreferenceConfig>();
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

            let database = self.plugins.get::<Database>();
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
    }
}

types::generate_on_event_impl!(
    StateManager;
    on_extensions_updated, ();
);

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
