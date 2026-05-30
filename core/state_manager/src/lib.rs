use std::{
    env::temp_dir,
    sync::{Arc, mpsc},
};

use database::Database;
use extensions::ExtensionHandler;
use extensions_proto::moosync::types::{MainCommand, MainCommandResponse};
use file_scanner::ScannerHolder;
use lyrics::LyricsFetcher;
use mpris::MprisHolder;
use platform_dirs;
use player::PlayerHandler;
use preferences::preferences::PreferenceConfig;
use queue_manager::QueueManager;
use tempdir;
use themes::themes::ThemeHolder;
use tokio::{runtime::Handle, sync::OwnedRwLockReadGuard};
use tracing::{debug, info};
#[cfg(target_os = "android")]
use types::android::AndroidJNIContext;
use types::errors::MoosyncError;
use types::init_plugin;
use types::plugin::{Plugin, PluginContext, PluginRegistry};

use crate::interceptors::database::DummyDatabaseInterceptor;

pub mod interceptors;

plugin_macro::generate_plugin_system!(
    preferences::preferences::PreferenceConfig,
    database::Database,
    file_scanner::ScannerHolder,
    lyrics::LyricsFetcher,
    extensions::ExtensionHandler,
    player::PlayerHandler,
    themes::themes::ThemeHolder,
    mpris::MprisHolder,
    queue_manager::QueueManager
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
}

impl StateManager {
    pub fn new(
        #[cfg(target_os = "android")] android_context: AndroidJNIContext,
    ) -> Result<Self, StateManagerError> {
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
        let runtime = Handle::current();

        let extensions_dir = data_dir.join("extensions");
        let theme_dir = data_dir.join("themes");

        let (themes_changed_tx, _themes_changed_rx) = mpsc::channel();

        let mut plugins = PluginRegistry::new();

        // 1. Preferences Config
        let context = PluginContext {
            data_dir: data_dir.clone(),
            cache_dir,
            tmp_dir: tmp.clone(),
            #[cfg(target_os = "android")]
            android_context: android_context.clone(),
            reply_handler: Some(Arc::new(move |ext, command| {
                runtime.block_on(handle_request(ext, command))
            })),
            themes_changed_tx: Some(themes_changed_tx),
            player_resolver: Some(Arc::new(|_| Ok("".into()))),
        };

        init_all_plugins(&mut plugins, &context);

        let interceptors = Interceptors::default().with(DummyDatabaseInterceptor);

        Ok(Self {
            plugins: Arc::new(plugins),
            interceptors: Arc::new(interceptors),
        })
    }

    pub async fn setup(&self) {
        self.setup_scanner().await;
        let scanner = self.plugins.get::<ScannerHolder>();
        tokio::task::spawn(async move {
            let scanner = scanner.read().await;
            scanner.start_scan().await.unwrap();
        });
    }

    async fn setup_scanner(&self) {
        let scanner = self.plugins.get::<ScannerHolder>();
        let mut file_scanner = scanner.write().await;
        file_scanner.set_artist_split(",".into());
        file_scanner.set_thumbnail_dir(temp_dir());
        file_scanner.set_scan_dir("/home/ovenoboyo/Nextcloud/Sahil/Music/Music that heals".into());

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
}

pub async fn handle_request(
    _ext: &str,
    _command: MainCommand,
) -> Result<MainCommandResponse, MoosyncError> {
    todo!("Not implemented yet");
}
