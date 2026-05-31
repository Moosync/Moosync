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

use crate::{OnPlaylistScanned, OnProgressUpdated, OnSongScanned};
use types::errors::Result;

pub struct AndroidScannerContext {
    #[allow(dead_code)]
    scan_dir: PathBuf,
    #[allow(dead_code)]
    thumbnail_dir: PathBuf,
    #[allow(dead_code)]
    artist_split: String,
}

impl AndroidScannerContext {
    pub fn new(scan_dir: PathBuf, thumbnail_dir: PathBuf, artist_split: String) -> Self {
        Self {
            scan_dir,
            thumbnail_dir,
            artist_split,
        }
    }
}

impl super::ScannerContext for AndroidScannerContext {
    async fn start_scan(
        &self,
        _on_song: &OnSongScanned,
        _on_playlist: &OnPlaylistScanned,
        _on_progress: &OnProgressUpdated,
    ) -> Result<()> {
        Ok(())
    }
}
