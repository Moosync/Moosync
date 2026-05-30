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

use songs_proto::moosync::types::Song;
use spotify_player::LibrespotHolder;

use crate::{
    error::PlayerError,
    generic::PlayerExt,
    mux_player::MuxPlayer,
    source::{SourceResolver, SourceResolverFn},
};

mod error;
mod generic;
mod mux_player;
mod queue;
mod rodio;
mod source;

#[cfg(test)]
mod test;

pub struct PlayerHandler {
    mux: MuxPlayer,
    source_resolver: SourceResolver,
}

#[plugin_macro::generate]
impl PlayerHandler {
    pub fn new() -> Self {
        Self {
            mux: MuxPlayer::new(),
            source_resolver: SourceResolver::new(),
        }
    }

    pub fn set_resolver(&self, f: SourceResolverFn) {
        self.source_resolver.set_resolver(f);
    }

    pub fn load_song(&mut self, mut song: Song) -> Result<(), PlayerError> {
        let res = self.mux.load(&song);

        // Try to resolve the playback url if no player was found
        // This basically runs 2 passes of load if no playback url was found
        if let Err(e) = res {
            match e {
                PlayerError::NoPlayerFound(_) | PlayerError::NoSrcFound(_) => {
                    self.source_resolver.resolve_playback_url(&mut song)?;
                    self.mux.load(&song)?;
                }
                _ => return Err(e),
            }
        }

        Ok(())
    }

    fn play(&self) -> Result<(), PlayerError> {
        self.mux.play()
    }

    fn pause(&self) -> Result<(), PlayerError> {
        self.mux.pause()
    }

    fn set_volume(&self, volume: f32) -> Result<(), PlayerError> {
        self.mux.set_volume(volume)
    }

    fn seek(&self, position: f64) -> Result<(), PlayerError> {
        self.mux.seek(position)
    }
}

impl types::plugin::Plugin for PlayerHandler {
    fn init(_context: &types::plugin::PluginContext) -> Self {
        PlayerHandler::new()
    }
}
