use std::path::PathBuf;

use types::songs::Song;

#[derive(Debug)]
pub struct FileList {
    pub file_list: Vec<(PathBuf, f64)>,
    pub playlist_list: Vec<PathBuf>,
}
