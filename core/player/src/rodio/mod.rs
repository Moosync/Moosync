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

use std::{
    sync::{
        Arc, Mutex,
        mpsc::{Receiver, Sender, channel},
    },
    thread,
    time::Duration,
};

use extensions_proto::moosync::types::player_event::Event as PlayerEvent;
use rodio::{MixerDeviceSink, Player};
use tracing::{debug, error, info};
use types::errors::MoosyncError;

use crate::{
    error::PlayerError, generic::PlayerExt, rodio::decoder::FFMPEGDecoder, source::ValidSrc,
};

mod decoder;
pub(crate) use decoder::DecoderError;

pub struct RodioPlayer {
    sink: MixerDeviceSink,
    player: Player,
}

enum RodioCommand {
    SetSrc(String),
    Play,
    Pause,
    Stop,
    SetVolume(f32),
    Seek(u64),
}

impl RodioPlayer {
    #[tracing::instrument(level = "debug", skip())]
    pub fn new() -> Self {
        let sink = rodio::DeviceSinkBuilder::open_default_sink().unwrap();
        let player = rodio::Player::connect_new(sink.mixer());

        Self { sink, player }
    }

    fn send_event(events_tx: Sender<PlayerEvent>, event: PlayerEvent) {
        events_tx.send(event).unwrap();
    }

    #[tracing::instrument(level = "debug", skip(self))]
    pub async fn get_volume(&self) -> Result<f32, PlayerError> {
        Ok(0f32)
    }
}

impl PlayerExt for RodioPlayer {
    #[tracing::instrument(level = "debug", skip(self))]
    fn play(&self) -> Result<(), PlayerError> {
        self.player.play();
        Ok(())
    }

    #[tracing::instrument(level = "debug", skip(self))]
    fn pause(&self) -> Result<(), PlayerError> {
        self.player.pause();
        Ok(())
    }

    #[tracing::instrument(level = "debug", skip(self))]
    fn stop(&self) -> Result<(), PlayerError> {
        self.player.stop();
        Ok(())
    }

    #[tracing::instrument(level = "debug", skip(self))]
    fn set_volume(&self, volume: f32) -> Result<(), PlayerError> {
        self.player.set_volume(volume);
        Ok(())
    }

    #[tracing::instrument(level = "debug", skip(self))]
    fn seek(&self, pos: f64) -> Result<(), PlayerError> {
        self.player.try_seek(Duration::from_secs_f64(pos))?;
        Ok(())
    }

    fn set_src(&self, src: ValidSrc) -> Result<(), PlayerError> {
        self.player.append(FFMPEGDecoder::open(&src.inner())?);
        Ok(())
    }

    fn can_play(&self, src: ValidSrc) -> bool {
        match src {
            ValidSrc::Path(path) => path.exists(),
            ValidSrc::Url(url) => url.starts_with("http://") || url.starts_with("https://"),
        }
    }
}

impl Default for RodioPlayer {
    fn default() -> Self {
        Self::new()
    }
}
