use std::sync::{mpsc::Sender, Mutex};

use types::errors::Result;
use types::{entities::QueryablePlaylist, songs::Song};

#[derive(Debug, PartialEq, Eq)]
pub enum ScanState {
    UNDEFINED,
    SCANNING,
    QUEUED,
}

#[derive(Debug)]
pub struct ScannerHolder {}

impl Default for ScannerHolder {
    #[tracing::instrument(level = "trace", skip())]
    fn default() -> Self {
        Self::new()
    }
}

impl ScannerHolder {
    #[tracing::instrument(level = "trace", skip())]
    pub fn new() -> Self {
        Self {}
    }

    #[tracing::instrument(level = "trace", skip(self))]
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
        playlist_tx: Sender<Vec<QueryablePlaylist>>,
    ) -> Result<()> {
        Ok(())
    }
}
