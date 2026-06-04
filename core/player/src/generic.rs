use std::time::Duration;

use crate::{error::PlayerError, source::ValidSrc};

pub(crate) trait PlayerExt: Send + Sync {
    fn play(&self) -> Result<(), PlayerError>;
    fn pause(&self) -> Result<(), PlayerError>;
    fn stop(&self) -> Result<(), PlayerError>;
    fn set_volume(&self, volume: u8) -> Result<(), PlayerError>;
    fn seek(&self, position: Duration) -> Result<(), PlayerError>;
    fn set_src(&self, src: ValidSrc) -> Result<(), PlayerError>;
    fn get_current_pos(&self) -> Result<Duration, PlayerError>;
    fn get_volume(&self) -> Result<u8, PlayerError>;
    fn get_player_state(&self) -> Result<extensions_proto::moosync::types::PlayerState, PlayerError>;

    fn can_play(&self, src: ValidSrc) -> bool;
}

