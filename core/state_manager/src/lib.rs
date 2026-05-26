use std::sync::{Arc, mpsc};

use database::database::Database;
use extensions::ExtensionHandler;
use extensions_proto::moosync::types::{MainCommand, MainCommandResponse};
use file_scanner::ScannerHolder;
use lyrics::LyricsFetcher;
use mpris::MprisHolder;
#[cfg(target_os = "android")]
use mpris::AndroidMprisContext;
use platform_dirs;
use preferences::preferences::PreferenceConfig;
use rodio_player::RodioPlayer;
use tempdir;
use themes::themes::ThemeHolder;
use tokio::runtime::Handle;
use types::errors::MoosyncError;

#[derive(Debug, thiserror::Error)]
pub enum StateManagerError {
    #[error("State initialization error: {0}")]
    InitializeError(Box<dyn std::error::Error + Send + Sync>),
}

pub struct StateManager {
    preferences: PreferenceConfig,
    database: Database,
    file_scanner: ScannerHolder,
    lyrics: LyricsFetcher,
    extensions: ExtensionHandler,
    rodio_player: RodioPlayer,
    themes: ThemeHolder,
    mpris: MprisHolder,
}

impl StateManager {
    pub fn new(
        #[cfg(target_os = "android")] jvm: Arc<jni::JavaVM>,
        #[cfg(target_os = "android")] activity: jni::objects::GlobalRef,
        #[cfg(target_os = "android")] service_class: jni::objects::GlobalRef,
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

        #[cfg(not(target_os = "android"))]
        let mpris = MprisHolder::new()
            .map_err(|e| StateManagerError::InitializeError(Box::new(e)))?;

        #[cfg(target_os = "android")]
        let mpris = {
            let context = Box::new(AndroidMprisContext::new(jvm, activity, service_class));
            MprisHolder::new_with_context(context)
                .map_err(|e| StateManagerError::InitializeError(Box::new(e)))?
        };

        let tmp = tempdir::TempDir::new("moosync")
            .expect("Failed to create tmp dir")
            .into_path();
        let runtime = Handle::current();

        let extensions_dir = data_dir.join("extensions");
        let theme_dir = data_dir.join("themes");

        let (themes_changed_tx, _themes_changed_rx) = mpsc::channel();

        Ok(Self {
            preferences: PreferenceConfig::new(data_dir.clone())
                .map_err(|e| StateManagerError::InitializeError(Box::new(e)))?,
            database: Database::new(data_dir.clone()),
            file_scanner: ScannerHolder::new(),
            lyrics: LyricsFetcher::new(),
            extensions: ExtensionHandler::new(
                extensions_dir,
                tmp.clone(),
                cache_dir,
                Arc::new(Box::new(move |ext, command| {
                    runtime.block_on(handle_request(ext, command))
                })),
            ),
            rodio_player: RodioPlayer::new(),
            themes: ThemeHolder::new(theme_dir, tmp, themes_changed_tx),
            mpris,
        })
    }

    pub fn mpris(&self) -> &MprisHolder {
        &self.mpris
    }
}

pub async fn handle_request(
    _ext: &str,
    _command: MainCommand,
) -> Result<MainCommandResponse, MoosyncError> {
    todo!("Not implemented yet");
}
