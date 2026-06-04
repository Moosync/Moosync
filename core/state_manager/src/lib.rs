use std::{
    env::temp_dir,
    num::NonZeroUsize,
    sync::{Arc, Mutex},
};

use database::Database;
use extensions::ExtensionHandler;
use file_scanner::ScannerHolder;
use lru::LruCache;
use player::PlayerHandler;
use songs_proto::moosync::types::{GetSongOptions, SearchableSong, Song};
use tempdir;
use tokio::runtime::Handle;
use tracing::trace;
#[cfg(target_os = "android")]
use types::android::AndroidJNIContext;
use types::plugin::{PluginContext, PluginRegistry};

use crate::interceptors::database::CacheDatabaseInterceptor;

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

    song_cache: Arc<Mutex<LruCache<String, Song>>>,
    runtime: Handle,
    pub on_extensions_updated: Arc<std::sync::Mutex<Vec<Box<dyn Fn() + Send + Sync + 'static>>>>,
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

    fn create_song_cache() -> Arc<Mutex<LruCache<String, Song>>> {
        let cache_size: usize = (5usize * 1024usize * 1024usize).saturating_div(size_of::<Song>());
        Arc::new(Mutex::new(LruCache::new(
            NonZeroUsize::new(cache_size).unwrap(),
        )))
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

        let cache = Self::create_song_cache();
        let interceptors = Interceptors::default().with(CacheDatabaseInterceptor::new(&cache));
        let runtime = Handle::current();

        Ok(Self {
            plugins: Arc::new(plugins),
            interceptors: Arc::new(interceptors),
            cache_dir,
            song_cache: cache,
            runtime,
            on_extensions_updated: Arc::new(std::sync::Mutex::new(Vec::new())),
        })
    }

    pub fn on_extensions_updated<F>(&self, callback: F)
    where
        F: Fn() + Send + Sync + 'static,
    {
        self.on_extensions_updated
            .lock()
            .unwrap()
            .push(Box::new(callback));
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
        {
            let mut file_scanner = scanner.write().await;
            file_scanner.set_artist_split(",".into());
            file_scanner.set_thumbnail_dir(temp_dir());
            file_scanner
                .set_scan_dir("/home/ovenoboyo/Nextcloud/Sahil/Music/Music that heals".into());

            let database = self.plugins.get::<Database>();
            file_scanner.set_on_playlist(move |p| {
                let db = database.clone();
                async move {
                    for playlist in p {
                        let _ = db.read().await.create_playlist(playlist);
                    }
                }
            });

            let database = self.plugins.get::<Database>();
            file_scanner.set_on_song(move |pl_id: Option<String>, songs| {
                let db = database.clone();
                async move {
                    if let Ok(songs) = db.read().await.insert_songs(songs) {
                        if let Some(pl_id) = pl_id {
                            let _ = db.read().await.add_to_playlist(pl_id, songs);
                        }
                    }
                }
            });
        }

        tokio::task::spawn(async move {
            let scanner = scanner.read().await;
            scanner.start_scan().await.unwrap();
        });
    }

    pub async fn get_song_from_cache(&self, id: String) -> Option<Song> {
        let song = {
            let mut cache = self.song_cache.lock().unwrap();
            cache.get(&id).cloned()
        };

        if song.is_some() {
            trace!("Cache hit for {}", id);
            return song;
        }

        let database = self.get_database().await;
        let song = database
            .get_songs_by_options(GetSongOptions {
                song: Some(SearchableSong {
                    id: Some(id.clone()),
                    ..Default::default()
                }),
                ..Default::default()
            })
            .unwrap_or_default();
        let song = song.get(0);
        if song.is_some() {
            trace!("Cache miss. Got {} from database", id);
            return song.cloned();
        }

        // TODO: Qeury extensions for song by id

        None
    }
}
