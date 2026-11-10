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

use std::{borrow::Cow, fmt::Display, path::PathBuf, time::Duration};

use async_trait::async_trait;
use extensions_proto::moosync::types::{PlayerState, player_event::Event as PlayerEvent};
use prost::DecodeError;
use songs_proto::moosync::types::Song;
use thiserror::Error;
use tokio::sync::mpsc::UnboundedSender;

#[derive(Clone, Debug, PartialEq)]
pub enum ValidSrc<'a> {
    Path(PathBuf),
    Url {
        url: Cow<'a, str>,
        decryption_key: Option<Cow<'a, str>>,
    },
}

impl Display for ValidSrc<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.inner())
    }
}

impl<'a> ValidSrc<'a> {
    #[tracing::instrument(level = "debug", skip_all)]
    pub fn new_url(url: impl Into<Cow<'a, str>>) -> Self {
        Self::Url {
            url: url.into(),
            decryption_key: None,
        }
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub fn new_url_with_key(
        url: impl Into<Cow<'a, str>>,
        decryption_key: Option<impl Into<Cow<'a, str>>>,
    ) -> Self {
        Self::Url {
            url: url.into(),
            decryption_key: decryption_key.map(Into::into),
        }
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub fn inner(&self) -> Cow<'_, str> {
        match self {
            ValidSrc::Path(path) => path.to_string_lossy(),
            ValidSrc::Url { url, .. } => url.clone(),
        }
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub fn decryption_key(&self) -> Option<Cow<'_, str>> {
        match self {
            ValidSrc::Path(_) => None,
            ValidSrc::Url { decryption_key, .. } => decryption_key.clone(),
        }
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub fn into_owned(self) -> ValidSrc<'static> {
        match self {
            ValidSrc::Path(path) => ValidSrc::Path(path),
            ValidSrc::Url {
                url,
                decryption_key,
            } => ValidSrc::Url {
                url: Cow::Owned(url.into_owned()),
                decryption_key: decryption_key.map(|k| Cow::Owned(k.into_owned())),
            },
        }
    }
}

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
    #[error("Failed to seek: {0}")]
    SeekError(String),
    #[error("Failed to create ffmpeg decoder: {0}")]
    DecoderError(String),
    #[error("Audio device error: {0}")]
    AudioDevice(String),
    #[error("IO Error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("Failed to parse stored player data: {0}")]
    ParseError(#[from] DecodeError),
    #[error("Custom player error: {0}")]
    Custom(String),
}

#[async_trait]
pub trait AudioPlayerContext: Send + Sync {
    async fn play(&self) -> Result<(), PlayerError>;
    async fn pause(&self) -> Result<(), PlayerError>;
    async fn stop(&self) -> Result<(), PlayerError>;
    async fn set_volume(&self, volume: u8) -> Result<(), PlayerError>;
    fn get_volume(&self) -> Result<u8, PlayerError>;
    async fn seek(&self, pos: Duration) -> Result<(), PlayerError>;
    fn get_current_pos(&self) -> Result<Duration, PlayerError>;
    fn get_player_state(&self) -> Result<PlayerState, PlayerError>;
    async fn can_play(&self, src: &ValidSrc<'_>) -> bool;
    async fn set_src(
        &self,
        src: ValidSrc<'static>,
        events_tx: UnboundedSender<PlayerEvent>,
    ) -> Result<(), PlayerError>;
}
