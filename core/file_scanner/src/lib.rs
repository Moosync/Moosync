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

use types::errors::{MoosyncError, Result};

use crate::context::ScannerContext;
use songs_proto::moosync::types::{Playlist, Song};
use std::{future::Future, path::PathBuf, pin::Pin};
pub use types::ScanProgress;

#[cfg(target_os = "android")]
use crate::context::android::AndroidScannerContext;
#[cfg(not(any(target_os = "android", target_os = "ios")))]
use crate::context::desktop::DesktopScannerContext;

mod context;

#[cfg(test)]
mod tests;

#[derive(Debug)]
pub struct FileList {
    pub file_list: Vec<(PathBuf, f64)>,
    pub playlist_list: Vec<PathBuf>,
}

pub type ScanProgressReceiver = tokio::sync::mpsc::UnboundedReceiver<ScanProgress>;
pub type OnSongScanned = Box<
    dyn Fn(Option<String>, Vec<Song>) -> Pin<Box<dyn Future<Output = ()> + Send>> + Send + Sync,
>;
pub type OnPlaylistScanned =
    Box<dyn Fn(Vec<Playlist>) -> Pin<Box<dyn Future<Output = ()> + Send>> + Send + Sync>;
pub type OnProgressUpdated = Box<dyn Fn(ScanProgress) + Send + Sync>;

pub struct ScannerHolder {
    scan_dir: Option<PathBuf>,
    thumbnail_dir: Option<PathBuf>,
    artist_split: Option<String>,
    on_song: Option<OnSongScanned>,
    on_playlist: Option<OnPlaylistScanned>,
    subscribers: std::sync::Mutex<Vec<tokio::sync::mpsc::UnboundedSender<ScanProgress>>>,
}

#[plugin_macro::generate]
impl ScannerHolder {
    #[tracing::instrument(level = "debug", skip())]
    pub fn new() -> Self {
        Self {
            scan_dir: None,
            thumbnail_dir: None,
            artist_split: None,
            on_song: None,
            on_playlist: None,
            subscribers: std::sync::Mutex::new(Vec::new()),
        }
    }

    pub fn set_scan_dir(&mut self, dir: PathBuf) {
        self.scan_dir = Some(dir);
    }

    pub fn set_thumbnail_dir(&mut self, dir: PathBuf) {
        self.thumbnail_dir = Some(dir);
    }

    pub fn set_artist_split(&mut self, split: String) {
        self.artist_split = Some(split);
    }

    pub fn set_on_song<F, Fut>(&mut self, cb: F)
    where
        F: Fn(Option<String>, Vec<Song>) -> Fut + Send + Sync + 'static,
        Fut: std::future::Future<Output = ()> + Send + 'static,
    {
        self.on_song = Some(Box::new(move |pl_id, songs| Box::pin(cb(pl_id, songs))));
    }

    pub fn set_on_playlist<F, Fut>(&mut self, cb: F)
    where
        F: Fn(Vec<Playlist>) -> Fut + Send + Sync + 'static,
        Fut: std::future::Future<Output = ()> + Send + 'static,
    {
        self.on_playlist = Some(Box::new(move |playlists| Box::pin(cb(playlists))));
    }

    #[tracing::instrument(level = "debug", skip(self))]
    pub fn add_subscriber(&self) -> ScanProgressReceiver {
        let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
        if let Ok(mut subs) = self.subscribers.lock() {
            subs.push(tx);
        }
        rx
    }

    #[tracing::instrument(level = "trace", skip(self))]
    pub async fn start_scan(&self) -> Result<()> {
        let scan_dir = self
            .scan_dir
            .clone()
            .ok_or_else(|| MoosyncError::String("scan_dir not set".into()))?;
        let thumbnail_dir = self
            .thumbnail_dir
            .clone()
            .ok_or_else(|| MoosyncError::String("thumbnail_dir not set".into()))?;
        let artist_split = self.artist_split.clone().unwrap_or_else(|| ";".to_string());

        let on_song = self
            .on_song
            .as_ref()
            .ok_or_else(|| MoosyncError::String("on_song callback not set".into()))?;
        let on_playlist = self
            .on_playlist
            .as_ref()
            .ok_or_else(|| MoosyncError::String("on_playlist callback not set".into()))?;

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

        #[cfg(not(any(target_os = "android", target_os = "ios")))]
        let context = DesktopScannerContext::new(scan_dir, thumbnail_dir, artist_split);

        #[cfg(target_os = "android")]
        let context = AndroidScannerContext::new(scan_dir, thumbnail_dir, artist_split);

        context.start_scan(on_song, on_playlist, &on_progress).await
    }
}

impl Default for ScannerHolder {
    fn default() -> Self {
        Self::new()
    }
}

impl types::plugin::Plugin for ScannerHolder {
    fn init(_context: &types::plugin::PluginContext) -> types::plugin::Arc<types::plugin::RwLock<Self>> {
        types::plugin::Arc::new(types::plugin::RwLock::new(ScannerHolder::new()))
    }
}
