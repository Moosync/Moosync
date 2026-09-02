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

use std::time::Duration;

use extensions_proto::moosync::types::{PlayerState, player_event::Event as PlayerEvent};
use tokio::sync::mpsc::UnboundedSender;

use crate::{error::PlayerError, source::ValidSrc};

pub mod dummy;
pub mod rodio;

#[cfg(test)]
mod mod_test;

pub use dummy::DummyAudioPlayerContext;
pub use rodio::RodioPlayerContext;

pub trait AudioPlayerContext: Send + Sync {
    fn play(&self) -> Result<(), PlayerError>;
    fn pause(&self) -> Result<(), PlayerError>;
    fn stop(&self) -> Result<(), PlayerError>;
    fn set_volume(&self, volume: u8) -> Result<(), PlayerError>;
    fn get_volume(&self) -> Result<u8, PlayerError>;
    fn seek(&self, pos: Duration) -> Result<(), PlayerError>;
    fn get_current_pos(&self) -> Result<Duration, PlayerError>;
    fn get_player_state(&self) -> Result<PlayerState, PlayerError>;
    fn set_src(
        &self,
        src: ValidSrc,
        events_tx: UnboundedSender<PlayerEvent>,
    ) -> Result<(), PlayerError>;
}
