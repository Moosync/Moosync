use std::{io, num::ParseFloatError, string::FromUtf8Error, time::SystemTimeError};

use fast_image_resize::{DifferentTypesOfPixelsError, ImageBufferError};
use image::ImageError;
use librespot::{core::Error as LibrespotError};
use lofty::LoftyError;
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
    #[error(transparent)]
    SystemTimeError(#[from] SystemTimeError),
    #[error(transparent)]
    ImageBufferError(#[from] ImageBufferError),
    #[error(transparent)]
    ImageError(#[from] ImageError),
    #[error(transparent)]
    DifferentTypesOfPixelsError(#[from] DifferentTypesOfPixelsError),
    #[error(transparent)]
    LoftyError(#[from] LoftyError),
    #[error(transparent)]
    ParseFloatError(#[from] ParseFloatError),
    #[error(transparent)]
    JWalkError(#[from] jwalk::Error),
    #[error(transparent)]
    Librespot(#[from] LibrespotError),
    #[error(transparent)]
    UTF8(#[from] FromUtf8Error),
    #[error(transparent)]
    Reqwest(#[from] reqwest::Error),
    #[error(transparent)]
    ProtoBuf(#[from] protobuf::Error),
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
