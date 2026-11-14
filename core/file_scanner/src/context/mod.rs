use std::path::PathBuf;

use async_trait::async_trait;

use crate::{OnPlaylistScanned, OnProgressUpdated, OnSongScanned, error::ScannerError};

#[cfg(target_os = "android")]
pub mod android;
#[cfg(not(target_os = "android"))]
pub mod desktop;

#[cfg(target_os = "android")]
pub use android::AndroidScannerContext;
#[cfg(not(target_os = "android"))]
pub use desktop::DesktopScannerContext;

#[derive(Clone, Debug, Default)]
pub struct ScanParameters {
    pub scan_dirs: Vec<PathBuf>,
    pub exclude_dirs: Vec<PathBuf>,
    pub thumbnail_dir: PathBuf,
    pub artist_split: String,
    pub scan_threads: Option<i32>,
}

#[async_trait]
#[cfg_attr(test, mockall::automock)]
pub trait ScannerContext: Send + Sync {
    async fn scan(
        &self,
        params: ScanParameters,
        on_song: &OnSongScanned,
        on_playlist: &OnPlaylistScanned,
        on_progress: &OnProgressUpdated,
    ) -> Result<(), ScannerError>;
}
