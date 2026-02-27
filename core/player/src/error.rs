use std::fmt::Debug;

use songs_proto::moosync::types::Song;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum PlayerError {
    #[error("Could not find a player to play song: {0}")]
    NoPlayerFound(String),
    #[error("Could not find a src for song: {0:?}")]
    NoSrcFound(Song),
    #[error("Could not resolve playback url for song: {0:?}")]
    PlaybackUrlResolutionFailed(Box<dyn std::error::Error + Send + Sync>),
    #[error("Invalid song")]
    InvalidSong,
}
