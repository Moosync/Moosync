#[derive(Debug, thiserror::Error)]
pub enum MprisError {
    #[error("Media controls initialization failed: {0}")]
    InitFailed(String),

    #[error("Failed to attach media controls: {0}")]
    AttachFailed(String),

    #[error("Failed to set metadata: {0}")]
    SetMetadataFailed(String),

    #[error("Failed to set playback state: {0}")]
    SetPlaybackFailed(String),

    #[error("Platform not supported")]
    Unsupported,

    #[cfg(not(target_os = "android"))]
    #[error("Souvlaki integration error: {0:?}")]
    Souvlaki(souvlaki::Error),
}

#[cfg(not(target_os = "android"))]
impl From<souvlaki::Error> for MprisError {
    #[tracing::instrument(level = "trace", skip_all)]
    fn from(err: souvlaki::Error) -> Self { MprisError::Souvlaki(err) }
}
