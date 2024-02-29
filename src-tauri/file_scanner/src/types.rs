use std::path::PathBuf;

use types::songs::Song;

#[derive(Debug)]
pub struct FileList {
    pub file_list: Vec<(PathBuf, f64)>,
    pub playlist_list: Vec<PathBuf>,
}

#[derive(Debug)]
pub struct SongWithLen {
    pub song: Song,
    pub size: u32,
    pub current: u32,
}
