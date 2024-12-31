use std::sync::Mutex;

use database::database::Database;
use types::errors::Result;

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
        skip(self, database, dir, thumbnail_dir, artist_split, scan_threads, force)
    )]
    pub fn start_scan(
        &self,
        database: &Database,
        dir: String,
        thumbnail_dir: String,
        artist_split: String,
        scan_threads: f64,
        force: bool,
    ) -> Result<()> {
        Ok(())
    }
}
