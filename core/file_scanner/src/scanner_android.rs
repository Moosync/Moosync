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

use std::sync::{Mutex, mpsc::Sender};

use types::errors::Result;
use types::{entities::Playlist, songs::Song};

#[derive(Debug, PartialEq, Eq)]
pub enum ScanState {
    UNDEFINED,
    SCANNING,
    QUEUED,
}

#[derive(Debug)]
pub struct ScannerHolder {}

impl Default for ScannerHolder {
    #[tracing::instrument(level = "debug", skip())]
    fn default() -> Self {
        Self::new()
    }
}

impl ScannerHolder {
    #[tracing::instrument(level = "debug", skip())]
    pub fn new() -> Self {
        Self {}
    }

    #[tracing::instrument(level = "debug", skip(self))]
    pub fn get_progress(&self) -> u8 {
        0
    }

    #[tracing::instrument(
        level = "trace",
        skip(
            self,
            dir,
            thumbnail_dir,
            artist_split,
            scan_threads,
            song_tx,
            playlist_tx
        )
    )]
    pub fn start_scan(
        &self,
        dir: String,
        thumbnail_dir: String,
        artist_split: String,
        scan_threads: f64,
        song_tx: Sender<(Option<String>, Vec<Song>)>,
        playlist_tx: Sender<Vec<Playlist>>,
    ) -> Result<()> {
        Ok(())
    }
}
