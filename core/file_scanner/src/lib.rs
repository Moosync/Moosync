// Moosync
// Copyright (C) 2024, 2025  Moosync <support@moosync.app>
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <http://www.gnu.org/licenses/>.

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
#[cfg(target_os = "android")]
use crate::context::android::JniScannerBridge;
#[cfg(not(target_os = "android"))]
use crate::context::desktop::DesktopScannerContext;
pub use crate::context::{
    ScannerContext,
    android::{AndroidScannerBridge, AndroidScannerContext},
};
use crate::error::ScannerError;

mod context;

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
    android_bridge: Option<Arc<dyn AndroidScannerBridge>>,
}

pub struct ScannerHolder {
    config: Mutex<ScannerConfig>,
    subscribers: Mutex<Vec<tokio::sync::mpsc::UnboundedSender<ScanProgress>>>,
}

#[plugin_macro::generate]
impl ScannerHolder {
    #[tracing::instrument(level = "debug", skip_all)]
    pub fn new() -> Self {
        Self {
            config: Mutex::new(ScannerConfig::default()),
            subscribers: Mutex::new(Vec::new()),
        }
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub fn set_android_bridge(&self, bridge: Arc<dyn AndroidScannerBridge>) {
        self.config.lock().unwrap().android_bridge = Some(bridge);
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
            _scan_threads,
            on_song,
            on_playlist,
            android_bridge,
        ) = {
            let config = self.config.lock().unwrap();
            let uses_android = cfg!(target_os = "android") || config.android_bridge.is_some();
            if !uses_android && config.scan_dirs.is_empty() {
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
                config.android_bridge.clone(),
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

        if let Some(bridge) = android_bridge {
            let context = AndroidScannerContext::new_with_bridge(
                bridge,
                scan_dirs,
                exclude_dirs,
                thumbnail_dir,
                artist_split,
            );
            return context
                .start_scan(&on_song, &on_playlist, &on_progress)
                .await;
        }

        #[cfg(not(target_os = "android"))]
        {
            let context = DesktopScannerContext::new(
                scan_dirs,
                thumbnail_dir,
                artist_split,
                exclude_dirs,
                _scan_threads,
            );
            return context
                .start_scan(&on_song, &on_playlist, &on_progress)
                .await;
        }

        #[cfg(target_os = "android")]
        Err(ScannerError::Jni(
            "Android scanner bridge not initialized".to_string(),
        ))
    }
}

impl Default for ScannerHolder {
    fn default() -> Self { Self::new() }
}

impl Plugin for ScannerHolder {
    #[tracing::instrument(level = "debug", skip_all)]
    fn init(_context: &PluginContext) -> Arc<Self> {
        let holder = ScannerHolder::new();
        #[cfg(target_os = "android")]
        {
            holder.set_android_bridge(Arc::new(JniScannerBridge::new(
                _context.android_context.clone(),
            )));
        }
        Arc::new(holder)
    }
}
