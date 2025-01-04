use std::{path::PathBuf, sync::mpsc::Sender};

use crate::utils::{check_directory, get_files_recursively, scan_file};
use threadpool::ThreadPool;
use types::errors::Result;
use types::songs::Song;

pub struct SongScanner<'a> {
    dir: PathBuf,
    pool: &'a mut ThreadPool,
    thumbnail_dir: PathBuf,
    artist_split: String,
}

impl<'a> SongScanner<'a> {
    #[tracing::instrument(level = "trace", skip(dir, pool, thumbnail_dir, artist_split))]
    pub fn new(
        dir: PathBuf,
        pool: &'a mut ThreadPool,
        thumbnail_dir: PathBuf,
        artist_split: String,
    ) -> Self {
        Self {
            dir,
            pool,
            thumbnail_dir,
            artist_split,
        }
    }

    #[tracing::instrument(level = "trace", skip(self))]
    fn check_dirs(&self) -> Result<()> {
        check_directory(self.thumbnail_dir.clone())?;

        Ok(())
    }

    #[tracing::instrument(level = "trace", skip(self, tx, size, path, playlist_id))]
    pub fn scan_in_pool(
        &self,
        tx: Sender<(Option<String>, Result<Song>)>,
        size: f64,
        path: PathBuf,
        playlist_id: Option<String>,
    ) {
        let thumbnail_dir = self.thumbnail_dir.clone();
        let artist_split = self.artist_split.clone();
        self.pool.execute(move || {
            let mut metadata = scan_file(&path, &thumbnail_dir, size, false, &artist_split);
            if metadata.is_err() {
                metadata = scan_file(&path, &thumbnail_dir, size, true, &artist_split);
            }

            tx.send((playlist_id, metadata))
                .expect("channel will be there waiting for the pool");
        });
    }

    #[tracing::instrument(level = "trace", skip(self, tx_song))]
    pub fn start(&self, tx_song: Sender<(Option<String>, Result<Song>)>) -> Result<usize> {
        self.check_dirs()?;

        let file_list = get_files_recursively(self.dir.clone())?;

        let song_list = file_list.file_list;

        let len = song_list.len();

        for (file_path, size) in song_list {
            self.scan_in_pool(tx_song.clone(), size, file_path, None);
        }

        drop(tx_song);

        Ok(len)
    }
}
