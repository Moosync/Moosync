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

use extensions_proto::moosync::types::player_event::Event::Ended;
use songs_proto::moosync::types::Song;
use tokio::sync::mpsc::unbounded_channel;

use crate::{
    OnEndedCallback,
    error::PlayerError,
    generic::PlayerExt,
    mux_player::MuxPlayer,
    source::{SourceResolver, SourceResolverFn},
};

pub struct AudioSource {
    mux: MuxPlayer,
    source_resolver: SourceResolver,
}

impl AudioSource {
    pub fn new(on_ended_callback: OnEndedCallback) -> Self {
        let (events_tx, mut events_rx) = unbounded_channel();
        tokio::spawn(async move {
            while let Some(event) = events_rx.recv().await {
                match event {
                    Ended(_) => on_ended_callback(),
                    _ => {}
                }
            }
        });
        Self {
            mux: MuxPlayer::new(events_tx),
            source_resolver: SourceResolver::new(),
        }
    }

    pub fn set_resolver(&self, f: SourceResolverFn) {
        self.source_resolver.set_resolver(f);
    }

    pub fn set_src(&mut self, song: Song) -> Result<(), PlayerError> {
        self.load_song(song)
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

    pub fn play(&self) -> Result<(), PlayerError> {
        self.mux.play()
    }

    pub fn pause(&self) -> Result<(), PlayerError> {
        self.mux.pause()
    }

    pub fn stop(&self) -> Result<(), PlayerError> {
        self.mux.stop()
    }

    pub fn set_volume(&self, volume: u8) -> Result<(), PlayerError> {
        self.mux.set_volume(volume)
    }

    pub fn seek(&self, position: Duration) -> Result<(), PlayerError> {
        self.mux.seek(position)
    }

    pub fn get_current_pos(&self) -> Result<Duration, PlayerError> {
        self.mux.get_current_pos()
    }

    pub fn get_player_state(&self) -> extensions_proto::moosync::types::PlayerState {
        self.mux.get_player_state()
    }

    pub fn get_volume(&self) -> u8 {
        self.mux.get_volume()
    }
}
