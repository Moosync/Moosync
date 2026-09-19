use std::fmt::Debug;

use prost::DecodeError;
use rodio::{DeviceSinkError, source::SeekError};
use songs_proto::moosync::types::Song;
use thiserror::Error;

use crate::rodio::DecoderError;

#[derive(Debug, Error)]
pub enum PlayerError {
    #[error("Could not find a player to play song: {0:?}")]
    NoPlayerFound(Box<Song>),
    #[error("Could not find a src for song: {0:?}")]
    NoSrcFound(Box<Song>),
    #[error("Could not resolve playback url for song: {0}")]
    PlaybackUrlResolutionFailed(String),
    #[error("Invalid song")]
    InvalidSong,
    #[error("Failed to seek: {0:?}")]
    SeekError(#[from] SeekError),
    #[error("Failed to create ffmpeg decoder: {0:?}")]
    DecoderError(#[from] DecoderError),
    #[error("Audio device error: {0}")]
    AudioDevice(#[from] DeviceSinkError),
    #[error("IO Error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("Failed to parse stored player data: {0}")]
    ParseError(#[from] DecodeError),
}
