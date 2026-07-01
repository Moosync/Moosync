use std::{borrow::Cow, error::Error, fmt::Display, path::PathBuf};

use songs_proto::moosync::types::Song;
use types::prelude::SongsExt;

use crate::error::PlayerError;

#[derive(Clone, Debug)]
pub(crate) enum ValidSrc<'a> {
    Path(PathBuf),
    Url(Cow<'a, str>),
}

impl Display for ValidSrc<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.clone().inner())
    }
}

impl ValidSrc<'_> {
    pub(crate) fn inner(&self) -> Cow<'_, str> {
        match self {
            ValidSrc::Path(path) => path.to_string_lossy(),
            ValidSrc::Url(url) => url.clone(),
        }
    }
}

pub(crate) fn get_valid_src(song: &'_ Song) -> Result<ValidSrc<'_>, PlayerError> {
    if let Some(path) = song.get_path() {
        let path = PathBuf::from(path.as_ref());
        if path.exists() {
            return Ok(ValidSrc::Path(path));
        }
    }
    if let Some(src) = song.get_playback_url() {
        return Ok(ValidSrc::Url(src));
    }
    Err(PlayerError::NoSrcFound(song.clone()))
}

pub type SourceResolverFn =
    Box<dyn Fn(&Song) -> Result<String, Box<dyn Error + Send + Sync>> + Send + Sync>;
pub(crate) struct SourceResolver {
    resolver: std::sync::Mutex<Option<SourceResolverFn>>,
}

impl SourceResolver {
    #[tracing::instrument(level = "debug", skip_all)]
    pub fn new() -> Self {
        Self {
            resolver: std::sync::Mutex::new(None),
        }
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub fn set_resolver(&self, resolver: SourceResolverFn) {
        let mut r = self.resolver.lock().unwrap();
        *r = Some(resolver);
    }

    // This method will ignore any existing playback url and path and try to find a
    // new one
    #[tracing::instrument(level = "debug", skip_all)]
    pub fn resolve_playback_url(&self, song: &mut Song) -> Result<(), PlayerError> {
        let resolver_lock = self.resolver.lock().unwrap();
        let playback_url = if let Some(ref resolver) = *resolver_lock {
            resolver(song).map_err(|e| PlayerError::PlaybackUrlResolutionFailed(e.to_string()))?
        } else {
            return Err(PlayerError::PlaybackUrlResolutionFailed(
                "Resolver not set".to_string(),
            ));
        };

        if let Some(inner_song) = song.song.as_mut() {
            inner_song.playback_url = Some(playback_url);
        };
        Err(PlayerError::InvalidSong)
    }
}
