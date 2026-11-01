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
