use std::{error::Error, fmt::Display, path::PathBuf};

use songs_proto::moosync::types::Song;
use types::prelude::SongsExt;

use crate::error::PlayerError;

#[derive(Clone, Debug)]
pub(crate) enum ValidSrc {
    Path(PathBuf),
    Url(String),
}

impl Display for ValidSrc {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.clone().inner())
    }
}

impl ValidSrc {
    pub(crate) fn inner(self) -> String {
        match self {
            ValidSrc::Path(path) => path.to_str().unwrap().to_string(),
            ValidSrc::Url(url) => url,
        }
    }
}

pub(crate) fn get_valid_src(song: &Song) -> Result<ValidSrc, PlayerError> {
    if let Some(path) = song.get_path() {
        let path = PathBuf::from(&path);
        if path.exists() {
            return Ok(ValidSrc::Path(path));
        }
    }
    if let Some(src) = song.get_playback_url() {
        return Ok(ValidSrc::Url(src));
    }
    Err(PlayerError::NoSrcFound(song.clone()))
}

pub type SourceResolverFn = Box<dyn Fn(&Song) -> Result<String, Box<dyn Error + Send + Sync>>>;
pub(crate) struct SourceResolver {
    resolver: SourceResolverFn,
}

impl SourceResolver {
    pub fn new(resolver: SourceResolverFn) -> Self {
        Self { resolver }
    }

    // This method will ignore any existing playback url and path and try to find a new one
    pub fn resolve_playback_url(&self, song: &mut Song) -> Result<(), PlayerError> {
        let playback_url =
            (self.resolver)(song).map_err(|e| PlayerError::PlaybackUrlResolutionFailed(e))?;

        if let Some(inner_song) = song.song.as_mut() {
            inner_song.playback_url = Some(playback_url);
        };
        Err(PlayerError::InvalidSong)
    }
}
