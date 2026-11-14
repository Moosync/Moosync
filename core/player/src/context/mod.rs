use player_proto::moosync::types::PlayerData;
pub use types::player::AudioPlayerContext;

use crate::error::PlayerError;

#[cfg(any(test, feature = "test"))]
pub mod dummy;
pub mod file_persist;
pub mod rodio;

#[cfg(test)]
mod mod_test;

#[cfg(any(test, feature = "test"))]
pub use dummy::{DummyAudioPlayerContext, DummyPersistContext};
pub use rodio::RodioPlayerContext;

pub trait PersistContext: Send + Sync {
    fn persist(&self, player_data: &PlayerData) -> Result<(), PlayerError>;
    fn load(&self) -> Result<PlayerData, PlayerError>;
}
