use std::{
    borrow::Cow,
    error::Error,
    fmt::Display,
    path::PathBuf,
    sync::{Arc, Mutex},
};

use async_trait::async_trait;
use extensions_proto::moosync::types::PlaybackDetailsRequestedResponse;
use songs_proto::moosync::types::Song;
use types::prelude::SongsExt;

use crate::error::PlayerError;

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
        write!(f, "{}", self.clone().inner())
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
    pub(crate) fn inner(&self) -> Cow<'_, str> {
        match self {
            ValidSrc::Path(path) => path.to_string_lossy(),
            ValidSrc::Url { url, .. } => url.clone(),
        }
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub(crate) fn decryption_key(&self) -> Option<Cow<'_, str>> {
        match self {
            ValidSrc::Path(_) => None,
            ValidSrc::Url { decryption_key, .. } => decryption_key.clone(),
        }
    }
}

#[tracing::instrument(level = "debug", skip_all)]
pub(crate) fn get_valid_src(song: &'_ Song) -> Result<ValidSrc<'_>, PlayerError> {
    let path_opt = song
        .get_path()
        .map(|p| PathBuf::from(p.as_ref()))
        .filter(|p| p.exists());
    if let Some(path) = path_opt {
        return Ok(ValidSrc::Path(path));
    }
    if let Some(src) = song.get_playback_url() {
        return Ok(ValidSrc::new_url(src));
    }
    Err(PlayerError::NoSrcFound(Box::new(song.clone())))
}

#[async_trait]
pub trait SourceResolver: Send + Sync {
    async fn resolve(
        &self,
        song: &Song,
    ) -> Result<PlaybackDetailsRequestedResponse, Box<dyn Error + Send + Sync>>;
}

pub(crate) struct SourceResolverHolder {
    resolver: Mutex<Option<Arc<dyn SourceResolver>>>,
}

impl SourceResolverHolder {
    #[tracing::instrument(level = "debug", skip_all)]
    pub fn new() -> Self {
        Self {
            resolver: Mutex::new(None),
        }
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub fn set_resolver(&self, resolver: Arc<dyn SourceResolver>) {
        let mut r = self.resolver.lock().unwrap();
        *r = Some(resolver);
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub async fn resolve(
        &self,
        song: &Song,
    ) -> Result<PlaybackDetailsRequestedResponse, PlayerError> {
        let resolver = {
            let resolver_lock = self.resolver.lock().unwrap();
            let Some(ref resolver) = *resolver_lock else {
                return Err(PlayerError::PlaybackUrlResolutionFailed(
                    "Resolver not set".to_string(),
                ));
            };
            resolver.clone()
        };
        tracing::debug!("Resolving playback details for song: {:?}", song);
        resolver.resolve(song).await.map_err(|e| {
            PlayerError::PlaybackUrlResolutionFailed(format!("Resolver failed: {}", e))
        })
    }
}
