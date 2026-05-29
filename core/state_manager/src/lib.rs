use std::{
    env::temp_dir,
    sync::{Arc, mpsc},
};

use database::database::Database;
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
use tokio::{runtime::Handle, sync::RwLock, sync::RwLockReadGuard};
use tracing::debug;
#[cfg(target_os = "android")]
use types::android::AndroidJNIContext;
use types::errors::MoosyncError;

#[derive(Debug, thiserror::Error)]
pub enum StateManagerError {
    #[error("State initialization error: {0}")]
    InitializeError(Box<dyn std::error::Error + Send + Sync>),
}

#[derive(Clone)]
pub struct StateManager {
    preferences: Arc<RwLock<PreferenceConfig>>,
    database: Arc<RwLock<Database>>,
    file_scanner: Arc<RwLock<ScannerHolder>>,
    lyrics: Arc<RwLock<LyricsFetcher>>,
    extensions: Arc<RwLock<ExtensionHandler>>,
    player: Arc<RwLock<PlayerHandler>>,
    themes: Arc<RwLock<ThemeHolder>>,
    mpris: Arc<RwLock<MprisHolder>>,
    queue_manager: Arc<RwLock<QueueManager>>,
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

        Ok(Self {
            preferences: Arc::new(RwLock::new(
                PreferenceConfig::new(data_dir.clone())
                    .map_err(|e| StateManagerError::InitializeError(Box::new(e)))?,
            )),
            database: Arc::new(RwLock::new(Database::new(data_dir.clone()))),
            file_scanner: Arc::new(RwLock::new(ScannerHolder::new())),
            lyrics: Arc::new(RwLock::new(LyricsFetcher::new())),
            extensions: Arc::new(RwLock::new(ExtensionHandler::new(
                extensions_dir,
                tmp.clone(),
                cache_dir,
                Arc::new(Box::new(move |ext, command| {
                    runtime.block_on(handle_request(ext, command))
                })),
            ))),
            player: Arc::new(RwLock::new(PlayerHandler::new(Box::new(|_| Ok("".into()))))),
            themes: Arc::new(RwLock::new(ThemeHolder::new(
                theme_dir,
                tmp,
                themes_changed_tx,
            ))),
            mpris: Arc::new(RwLock::new(
                MprisHolder::new(
                    #[cfg(target_os = "android")]
                    android_context,
                )
                .map_err(|e| StateManagerError::InitializeError(Box::new(e)))?,
            )),
            queue_manager: Arc::new(RwLock::new(QueueManager::new())),
        })
    }

    pub async fn setup(&self) {
        self.setup_scanner().await;
        let scanner = self.file_scanner.clone();
        tokio::task::spawn(async move {
            let scanner = scanner.read().await;
            scanner.start_scan().await.unwrap();
        });
    }

    async fn setup_scanner(&self) {
        let mut file_scanner = self.file_scanner.write().await;
        file_scanner.set_artist_split(",".into());
        file_scanner.set_thumbnail_dir(temp_dir());
        file_scanner.set_scan_dir("/home/ovenoboyo/Nextcloud/Sahil/Music/Music that heals".into());

        let database = self.database.clone();
        file_scanner.set_on_playlist(move |p| {
            let db = database.clone();
            async move {
                for playlist in p {
                    let _ = db.read().await.create_playlist(playlist);
                }
            }
        });

        let database = self.database.clone();
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

    pub async fn get_scanner(&self) -> RwLockReadGuard<'_, ScannerHolder> {
        self.file_scanner.read().await
    }

    pub async fn get_database(&self) -> RwLockReadGuard<'_, Database> {
        self.database.read().await
    }
}

pub async fn handle_request(
    _ext: &str,
    _command: MainCommand,
) -> Result<MainCommandResponse, MoosyncError> {
    todo!("Not implemented yet");
}
