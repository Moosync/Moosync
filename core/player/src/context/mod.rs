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
