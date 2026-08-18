use std::fmt::Debug;

use rodio::source::SeekError;
use songs_proto::moosync::types::Song;
use thiserror::Error;

use crate::rodio::DecoderError;

#[derive(Debug, Error)]
pub enum PlayerError {
    #[error("Could not find a player to play song: {0:?}")]
    NoPlayerFound(Song),
    #[error("Could not find a src for song: {0:?}")]
    NoSrcFound(Song),
    #[error("Could not resolve playback url for song: {0}")]
    PlaybackUrlResolutionFailed(String),
    #[error("Invalid song")]
    InvalidSong,
    #[error("Failed to seek: {0:?}")]
    SeekError(SeekError),
    #[error("Failed to create ffmpeg decoder: {0:?}")]
    DecoderError(DecoderError),
    #[error("Audio device error: {0}")]
    AudioDevice(String),
}

impl From<DecoderError> for PlayerError {
    fn from(value: DecoderError) -> Self { Self::DecoderError(value) }
}

impl From<SeekError> for PlayerError {
    fn from(value: SeekError) -> Self { Self::SeekError(value) }
}
