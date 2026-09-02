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
    num::NonZero,
    sync::{Arc, Mutex},
    time::Duration,
};

use extensions_proto::moosync::types::{PlayerState, player_event::Event as PlayerEvent};
use rodio::{MixerDeviceSink, Player, source::EmptyCallback};
use tokio::sync::mpsc::UnboundedSender;

use crate::{
    context::AudioPlayerContext,
    error::PlayerError,
    rodio::{decoder::FFMPEGDecoder, get_system_sample_rate},
    source::ValidSrc,
};

pub struct RodioPlayerContext {
    sink: Mutex<Option<MixerDeviceSink>>,
    player: Arc<Mutex<Option<Player>>>,
}

impl Default for RodioPlayerContext {
    #[tracing::instrument(level = "debug", skip_all)]
    fn default() -> Self { Self::new() }
}

impl RodioPlayerContext {
    #[tracing::instrument(level = "debug", skip_all)]
    pub fn new() -> Self {
        Self {
            sink: Mutex::new(None),
            player: Arc::new(Mutex::new(None)),
        }
    }
}

impl AudioPlayerContext for RodioPlayerContext {
    #[tracing::instrument(level = "debug", skip_all)]
    fn play(&self) -> Result<(), PlayerError> {
        let guard = self.player.lock().unwrap();
        if let Some(player) = guard.as_ref() {
            player.play();
        }
        Ok(())
    }

    #[tracing::instrument(level = "debug", skip_all)]
    fn pause(&self) -> Result<(), PlayerError> {
        let guard = self.player.lock().unwrap();
        if let Some(player) = guard.as_ref() {
            player.pause();
        }
        Ok(())
    }

    #[tracing::instrument(level = "debug", skip_all)]
    fn stop(&self) -> Result<(), PlayerError> {
        let guard = self.player.lock().unwrap();
        if let Some(player) = guard.as_ref() {
            player.stop();
        }
        Ok(())
    }

    #[tracing::instrument(level = "debug", skip_all)]
    fn set_volume(&self, volume: u8) -> Result<(), PlayerError> {
        let guard = self.player.lock().unwrap();
        if let Some(player) = guard.as_ref() {
            player.set_volume(volume as f32 / 100f32);
        }
        Ok(())
    }

    #[tracing::instrument(level = "debug", skip_all)]
    fn get_volume(&self) -> Result<u8, PlayerError> {
        let guard = self.player.lock().unwrap();
        let Some(player) = guard.as_ref() else {
            return Ok(100);
        };
        Ok((player.volume() * 100.0).round() as u8)
    }

    #[tracing::instrument(level = "debug", skip_all)]
    fn seek(&self, pos: Duration) -> Result<(), PlayerError> {
        let guard = self.player.lock().unwrap();
        if let Some(player) = guard.as_ref() {
            player.try_seek(pos)?;
        }
        Ok(())
    }

    #[tracing::instrument(level = "debug", skip_all)]
    fn get_current_pos(&self) -> Result<Duration, PlayerError> {
        let guard = self.player.lock().unwrap();
        if let Some(player) = guard.as_ref() {
            return Ok(player.get_pos());
        }
        Ok(Duration::default())
    }

    #[tracing::instrument(level = "debug", skip_all)]
    fn get_player_state(&self) -> Result<PlayerState, PlayerError> {
        let guard = self.player.lock().unwrap();
        let Some(player) = guard.as_ref() else {
            return Ok(PlayerState::Stopped);
        };

        if player.empty() {
            return Ok(PlayerState::Stopped);
        }
        if player.is_paused() {
            return Ok(PlayerState::Paused);
        }
        Ok(PlayerState::Playing)
    }

    #[tracing::instrument(level = "debug", skip_all)]
    fn set_src(
        &self,
        src: ValidSrc,
        events_tx: UnboundedSender<PlayerEvent>,
    ) -> Result<(), PlayerError> {
        let old_volume = self.get_volume().unwrap_or(50);

        if let Some(player) = self.player.lock().unwrap().as_ref() {
            player.clear();
        }

        let system_rate = get_system_sample_rate().max(1);
        let safe_rate = NonZero::new(system_rate).expect("Sample rate is clamped to 1");

        let device_builder = rodio::DeviceSinkBuilder::from_default_device()?;
        let mut sink = device_builder
            .with_sample_rate(safe_rate)
            .open_stream()
            .unwrap_or(rodio::DeviceSinkBuilder::open_default_sink()?);
        sink.log_on_drop(false);
        let cfg = sink.config();
        let output_sample_rate = cfg.sample_rate().get();
        tracing::trace!(
            "Sink requested rate={}, actual channels={}, sample_rate={}, format={:?}",
            system_rate,
            cfg.channel_count(),
            output_sample_rate,
            cfg.sample_format()
        );

        let decoder = FFMPEGDecoder::open(&src.inner(), output_sample_rate)?;
        let player = rodio::Player::connect_new(sink.mixer());
        player.append(decoder);
        player.append(EmptyCallback::new(Box::new(move || {
            let _ = events_tx.send(PlayerEvent::Ended(true));
        })));
        player.set_volume(old_volume as f32 / 100f32);

        *self.player.lock().unwrap() = Some(player);
        *self.sink.lock().unwrap() = Some(sink);
        Ok(())
    }
}
