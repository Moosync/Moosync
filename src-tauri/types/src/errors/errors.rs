use std::io;

use rusty_ytdl::VideoError;

#[derive(Debug, thiserror::Error)]
pub enum MoosyncError {
    #[error(transparent)]
    Tauri(#[from] tauri::Error),
    #[error(transparent)]
    Diesel(#[from] diesel::result::Error),
    #[error(transparent)]
    IO(#[from] io::Error),
    #[error(transparent)]
    Json(#[from] serde_json::Error),
    #[error(transparent)]
    Youtube(#[from] VideoError),
    #[error(transparent)]
    DotPaths(#[from] json_dotpath::Error),
    #[error("{0}")]
    String(String),
}

impl serde::Serialize for MoosyncError {
    fn serialize<S>(&self, serializer: S) -> core::result::Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer,
    {
        serializer.serialize_str(self.to_string().as_ref())
    }
}

pub type Result<T> = std::result::Result<T, MoosyncError>;
