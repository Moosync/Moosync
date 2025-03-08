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
    #[tracing::instrument(level = "debug", skip(dir, pool, thumbnail_dir, artist_split))]
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

    #[tracing::instrument(level = "debug", skip(self))]
    fn check_dirs(&self) -> Result<()> {
        check_directory(self.thumbnail_dir.clone())?;

        Ok(())
    }

    #[tracing::instrument(level = "debug", skip(self, tx, size, path, playlist_id))]
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

    #[tracing::instrument(level = "debug", skip(self, tx_song))]
    pub fn start(&self, tx_song: Sender<(Option<String>, Result<Song>)>) -> Result<usize> {
        tracing::debug!("Satrting scan");
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
