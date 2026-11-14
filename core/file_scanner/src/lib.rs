use std::{
    future::Future,
    path::PathBuf,
    pin::Pin,
    sync::{Arc, Mutex},
};

use songs_proto::moosync::types::{Playlist, Song};
pub use types::ScanProgress;
use types::plugin::{Plugin, PluginContext};

pub mod error;
use crate::error::ScannerError;

pub mod context;
#[cfg(target_os = "android")]
pub use crate::context::AndroidScannerContext;
#[cfg(not(target_os = "android"))]
pub use crate::context::DesktopScannerContext;
pub use crate::context::{ScanParameters, ScannerContext};

#[cfg(test)]
mod lib_test;
#[cfg(test)]
mod lib_test_smoke;

#[derive(Debug)]
pub struct FileList {
    pub file_list: Vec<(PathBuf, f64)>,
    pub playlist_list: Vec<PathBuf>,
}

pub type ScanProgressReceiver = tokio::sync::mpsc::UnboundedReceiver<ScanProgress>;
pub type OnSongScanned = Arc<
    dyn Fn(Option<String>, Vec<Song>) -> Pin<Box<dyn Future<Output = ()> + Send>> + Send + Sync,
>;
#[derive(Debug, Clone, PartialEq)]
pub enum PlaylistSongId {
    Path(PathBuf),
    Url(String),
}

pub type OnPlaylistScanned = Arc<
    dyn Fn(Vec<(Playlist, Vec<PlaylistSongId>)>) -> Pin<Box<dyn Future<Output = ()> + Send>>
        + Send
        + Sync,
>;
pub type OnProgressUpdated = Box<dyn Fn(ScanProgress) + Send + Sync>;

#[derive(Default)]
struct ScannerConfig {
    scan_dirs: Vec<PathBuf>,
    thumbnail_dir: Option<PathBuf>,
    artist_split: Option<String>,
    exclude_dirs: Vec<PathBuf>,
    scan_threads: Option<i32>,
    on_song: Option<OnSongScanned>,
    on_playlist: Option<OnPlaylistScanned>,
}

pub struct ScannerHolder {
    context: Box<dyn ScannerContext>,
    config: Mutex<ScannerConfig>,
    subscribers: Mutex<Vec<tokio::sync::mpsc::UnboundedSender<ScanProgress>>>,
}

#[plugin_macro::generate]
impl ScannerHolder {
    #[tracing::instrument(level = "debug", skip_all)]
    pub fn new(
        #[cfg(target_os = "android")] android_context: types::android::AndroidJNIContext,
    ) -> Self {
        #[cfg(not(target_os = "android"))]
        let context = Box::new(DesktopScannerContext::new());
        #[cfg(target_os = "android")]
        let context = Box::new(AndroidScannerContext::new(android_context));
        Self::new_with_context(context)
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub fn new_with_context(context: Box<dyn ScannerContext>) -> Self {
        Self {
            context,
            config: Mutex::new(ScannerConfig::default()),
            subscribers: Mutex::new(Vec::new()),
        }
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub fn set_scan_dirs(&self, dirs: Vec<PathBuf>) {
        self.config.lock().unwrap().scan_dirs = dirs;
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub fn set_exclude_dirs(&self, dirs: Vec<PathBuf>) {
        self.config.lock().unwrap().exclude_dirs = dirs;
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub fn set_scan_threads(&self, threads: i32) {
        self.config.lock().unwrap().scan_threads = Some(threads);
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub fn get_scan_threads(&self) -> Option<i32> { self.config.lock().unwrap().scan_threads }

    #[tracing::instrument(level = "debug", skip_all)]
    pub fn get_scan_dirs(&self) -> Vec<PathBuf> { self.config.lock().unwrap().scan_dirs.clone() }

    #[tracing::instrument(level = "debug", skip_all)]
    pub fn set_thumbnail_dir(&self, dir: PathBuf) {
        self.config.lock().unwrap().thumbnail_dir = Some(dir);
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub fn set_artist_split(&self, split: String) {
        self.config.lock().unwrap().artist_split = Some(split);
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub fn set_on_song<F, Fut>(&self, cb: F)
    where
        F: Fn(Option<String>, Vec<Song>) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = ()> + Send + 'static,
    {
        self.config.lock().unwrap().on_song =
            Some(Arc::new(move |pl_id, songs| Box::pin(cb(pl_id, songs))));
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub fn set_on_playlist<F, Fut>(&self, cb: F)
    where
        F: Fn(Vec<(Playlist, Vec<PlaylistSongId>)>) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = ()> + Send + 'static,
    {
        self.config.lock().unwrap().on_playlist =
            Some(Arc::new(move |playlists| Box::pin(cb(playlists))));
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub fn add_subscriber(&self) -> ScanProgressReceiver {
        let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
        if let Ok(mut subs) = self.subscribers.lock() {
            subs.push(tx);
        }
        rx
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub async fn start_scan(&self) -> Result<(), ScannerError> {
        let (
            scan_dirs,
            thumbnail_dir,
            artist_split,
            exclude_dirs,
            scan_threads,
            on_song,
            on_playlist,
        ) = {
            let config = self.config.lock().unwrap();
            #[cfg(not(target_os = "android"))]
            if config.scan_dirs.is_empty() {
                return Err(ScannerError::ScanDirsNotConfigured);
            }
            let thumbnail_dir = config
                .thumbnail_dir
                .clone()
                .ok_or(ScannerError::ThumbnailDirNotConfigured)?;
            let artist_split = config
                .artist_split
                .clone()
                .unwrap_or_else(|| ";".to_string());
            let on_song = config
                .on_song
                .clone()
                .ok_or(ScannerError::SongCallbackNotConfigured)?;
            let on_playlist = config
                .on_playlist
                .clone()
                .ok_or(ScannerError::PlaylistCallbackNotConfigured)?;
            (
                config.scan_dirs.clone(),
                thumbnail_dir,
                artist_split,
                config.exclude_dirs.clone(),
                config.scan_threads,
                on_song,
                on_playlist,
            )
        };

        let subscribers = if let Ok(subs) = self.subscribers.lock() {
            subs.clone()
        } else {
            Vec::new()
        };

        let on_progress: OnProgressUpdated = Box::new(move |progress| {
            for tx in &subscribers {
                let _ = tx.send(progress);
            }
        });

        let params = ScanParameters {
            scan_dirs,
            exclude_dirs,
            thumbnail_dir,
            artist_split,
            scan_threads,
        };

        self.context
            .scan(params, &on_song, &on_playlist, &on_progress)
            .await
    }
}

#[cfg(not(target_os = "android"))]
impl Default for ScannerHolder {
    fn default() -> Self { Self::new() }
}

impl Plugin for ScannerHolder {
    #[tracing::instrument(level = "debug", skip_all)]
    fn init(_context: &PluginContext) -> Arc<Self> {
        let holder = ScannerHolder::new(
            #[cfg(target_os = "android")]
            _context.android_context.clone(),
        );
        Arc::new(holder)
    }
}
