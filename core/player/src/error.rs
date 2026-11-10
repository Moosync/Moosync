pub use types::player::PlayerError;

use crate::rodio::DecoderError;

impl From<DecoderError> for PlayerError {
    #[tracing::instrument(level = "debug", skip_all)]
    fn from(e: DecoderError) -> Self { PlayerError::DecoderError(format!("{e:?}")) }
}
