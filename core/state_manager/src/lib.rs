use std::sync::Arc;

use tempdir;
use tokio::runtime::Handle;
#[cfg(target_os = "android")]
use types::android::AndroidJNIContext;
use types::plugin::{PluginContext, PluginRegistry};

pub mod error;
pub use crate::error::StateManagerError;

pub mod hooks;
pub mod interceptors;
mod reply_handler;

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

#[derive(Clone)]
pub struct StateManager {
    pub(crate) plugins: Arc<PluginRegistry>,
    pub interceptors: Arc<Interceptors>,
    pub cache_dir: std::path::PathBuf,

    runtime: Handle,
    pub hooks: Arc<tokio::sync::Mutex<Vec<Arc<dyn hooks::Hook>>>>,
}

impl StateManager {
    #[tracing::instrument(level = "debug", skip_all)]
    pub fn get_cache_dir(&self) -> std::path::PathBuf { self.cache_dir.clone() }

    #[tracing::instrument(level = "debug", skip_all)]
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

    #[tracing::instrument(level = "debug", skip_all)]
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

    #[tracing::instrument(level = "debug", skip_all)]
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

        let hooks_vec: Vec<Arc<dyn hooks::Hook>> = vec![
            Arc::new(hooks::extensions::ExtensionsHook::new()),
            Arc::new(hooks::player::PlayerHook::new()),
            Arc::new(hooks::scanner::ScannerHook::new()),
        ];
        let hooks = Arc::new(tokio::sync::Mutex::new(hooks_vec));

        Ok(Self {
            plugins: Arc::new(plugins),
            interceptors: Arc::new(interceptors),
            cache_dir,
            runtime,
            hooks,
        })
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub fn register_hook(&self, hook: Arc<dyn hooks::Hook>) {
        self.hooks.blocking_lock().push(hook);
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub async fn setup(&self) {
        for hook in self.hooks.lock().await.clone() {
            if let Err(e) = hook.on_startup(self).await {
                tracing::error!("Hook on_startup error: {:?}", e);
            }
        }
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub async fn delayed_setup(&self) {
        for hook in self.hooks.lock().await.clone() {
            if let Err(e) = hook.on_delayed_startup(self).await {
                tracing::error!("Hook on_delayed_startup error: {:?}", e);
            }
        }
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub async fn shutdown(&self) {
        for hook in self.hooks.lock().await.clone() {
            if let Err(e) = hook.on_exit(self).await {
                tracing::error!("Hook on_exit error: {:?}", e);
            }
        }
    }
}
