use std::path::PathBuf;

#[derive(Debug)]
pub struct FileList {
    pub file_list: Vec<(PathBuf, f64)>,
    pub playlist_list: Vec<PathBuf>,
}
