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

use std::{sync::Mutex, time::Duration};

use extensions_proto::moosync::types::{PlayerState, player_event::Event as PlayerEvent};
use tokio::sync::mpsc::UnboundedSender;

use crate::{context::AudioPlayerContext, error::PlayerError, source::ValidSrc};

struct DummyState {
    volume: u8,
    state: PlayerState,
    position: Duration,
    current_src: Option<String>,
}

pub struct DummyAudioPlayerContext {
    state: Mutex<DummyState>,
}

impl Default for DummyAudioPlayerContext {
    #[tracing::instrument(level = "debug", skip_all)]
    fn default() -> Self { Self::new() }
}

impl DummyAudioPlayerContext {
    #[tracing::instrument(level = "debug", skip_all)]
    pub fn new() -> Self {
        Self {
            state: Mutex::new(DummyState {
                volume: 100,
                state: PlayerState::Stopped,
                position: Duration::default(),
                current_src: None,
            }),
        }
    }
}

impl AudioPlayerContext for DummyAudioPlayerContext {
    #[tracing::instrument(level = "debug", skip_all)]
    fn play(&self) -> Result<(), PlayerError> {
        let mut guard = self.state.lock().unwrap();
        guard.state = PlayerState::Playing;
        Ok(())
    }

    #[tracing::instrument(level = "debug", skip_all)]
    fn pause(&self) -> Result<(), PlayerError> {
        let mut guard = self.state.lock().unwrap();
        guard.state = PlayerState::Paused;
        Ok(())
    }

    #[tracing::instrument(level = "debug", skip_all)]
    fn stop(&self) -> Result<(), PlayerError> {
        let mut guard = self.state.lock().unwrap();
        guard.state = PlayerState::Stopped;
        guard.position = Duration::default();
        Ok(())
    }

    #[tracing::instrument(level = "debug", skip_all)]
    fn set_volume(&self, volume: u8) -> Result<(), PlayerError> {
        let mut guard = self.state.lock().unwrap();
        guard.volume = volume;
        Ok(())
    }

    #[tracing::instrument(level = "debug", skip_all)]
    fn get_volume(&self) -> Result<u8, PlayerError> {
        let guard = self.state.lock().unwrap();
        Ok(guard.volume)
    }

    #[tracing::instrument(level = "debug", skip_all)]
    fn seek(&self, pos: Duration) -> Result<(), PlayerError> {
        let mut guard = self.state.lock().unwrap();
        guard.position = pos;
        Ok(())
    }

    #[tracing::instrument(level = "debug", skip_all)]
    fn get_current_pos(&self) -> Result<Duration, PlayerError> {
        let guard = self.state.lock().unwrap();
        Ok(guard.position)
    }

    #[tracing::instrument(level = "debug", skip_all)]
    fn get_player_state(&self) -> Result<PlayerState, PlayerError> {
        let guard = self.state.lock().unwrap();
        Ok(guard.state)
    }

    #[tracing::instrument(level = "debug", skip_all)]
    fn set_src(
        &self,
        src: ValidSrc,
        _events_tx: UnboundedSender<PlayerEvent>,
    ) -> Result<(), PlayerError> {
        let mut guard = self.state.lock().unwrap();
        guard.current_src = Some(src.inner().to_string());
        guard.state = PlayerState::Playing;
        guard.position = Duration::default();
        Ok(())
    }
}
