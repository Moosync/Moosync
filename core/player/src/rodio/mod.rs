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
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};

use extensions_proto::moosync::types::player_event::Event as PlayerEvent;
use rodio::{MixerDeviceSink, Player, source::EmptyCallback};
use tokio::sync::mpsc::UnboundedSender;

use crate::{
    error::PlayerError, generic::PlayerExt, rodio::decoder::FFMPEGDecoder, source::ValidSrc,
};

mod decoder;
#[cfg(test)]
mod test;

pub(crate) use decoder::DecoderError;

pub struct RodioPlayer {
    _sink: Mutex<Option<MixerDeviceSink>>,
    player: Arc<Mutex<Option<Player>>>,
    events_tx: UnboundedSender<PlayerEvent>,
}

impl RodioPlayer {
    #[tracing::instrument(level = "debug", skip())]
    pub fn new(events_tx: UnboundedSender<PlayerEvent>) -> Self {
        Self {
            _sink: Mutex::new(None),
            player: Arc::new(Mutex::new(None)),
            events_tx,
        }
    }

    fn send_event(events_tx: UnboundedSender<PlayerEvent>, event: PlayerEvent) {
        if let Err(e) = events_tx.send(event) {
            tracing::error!("Failed to send event: {:?}", e);
        }
    }

    #[tracing::instrument(level = "debug", skip(self))]
    pub async fn get_volume(&self) -> Result<f32, PlayerError> {
        Ok(0f32)
    }
}

impl PlayerExt for RodioPlayer {
    #[tracing::instrument(level = "debug", skip(self))]
    fn play(&self) -> Result<(), PlayerError> {
        if let Some(player) = self.player.lock().unwrap().as_ref() {
            player.play();
        }
        Ok(())
    }

    #[tracing::instrument(level = "debug", skip(self))]
    fn pause(&self) -> Result<(), PlayerError> {
        if let Some(player) = self.player.lock().unwrap().as_ref() {
            player.pause();
        }
        Ok(())
    }

    #[tracing::instrument(level = "debug", skip(self))]
    fn stop(&self) -> Result<(), PlayerError> {
        if let Some(player) = self.player.lock().unwrap().as_ref() {
            player.stop();
        }
        Ok(())
    }

    #[tracing::instrument(level = "debug", skip(self))]
    fn set_volume(&self, volume: u8) -> Result<(), PlayerError> {
        if let Some(player) = self.player.lock().unwrap().as_ref() {
            player.set_volume(volume as f32 / 100f32);
        }

        Ok(())
    }

    #[tracing::instrument(level = "debug", skip(self))]
    fn seek(&self, pos: Duration) -> Result<(), PlayerError> {
        if let Some(player) = self.player.lock().unwrap().as_ref() {
            player.try_seek(pos)?;
        }
        Ok(())
    }

    fn set_src(&self, src: ValidSrc) -> Result<(), PlayerError> {
        let events_tx = self.events_tx.clone();

        if let Some(player) = self.player.lock().unwrap().as_ref() {
            player.clear();
        }

        let _sink = rodio::DeviceSinkBuilder::open_default_sink().unwrap();
        let player = rodio::Player::connect_new(_sink.mixer());

        tracing::trace!("Sink sample rate {}", _sink.config().sample_rate());

        player.append(FFMPEGDecoder::open(&src.inner())?);
        player.append(EmptyCallback::new(Box::new(move || {
            let events_tx = events_tx.clone();
            Self::send_event(events_tx, PlayerEvent::Ended(true));
        })));

        tracing::trace!("sink sample rate: {}", _sink.config().sample_rate());

        *self.player.lock().unwrap() = Some(player);
        *self._sink.lock().unwrap() = Some(_sink);
        Ok(())
    }

    fn can_play(&self, src: ValidSrc) -> bool {
        match src {
            ValidSrc::Path(path) => path.exists(),
            ValidSrc::Url(url) => url.starts_with("http://") || url.starts_with("https://"),
        }
    }

    fn get_current_pos(&self) -> Result<Duration, PlayerError> {
        if let Some(player) = &self.player.lock().unwrap().as_ref() {
            return Ok(player.get_pos());
        }
        Ok(Duration::default())
    }
}
