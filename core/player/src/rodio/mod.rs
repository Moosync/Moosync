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

use extensions_proto::moosync::types::player_event::Event as PlayerEvent;
use rodio::{
    MixerDeviceSink, Player,
    cpal::{
        SupportedStreamConfig,
        traits::{DeviceTrait, HostTrait},
    },
    source::EmptyCallback,
};
use tokio::sync::mpsc::UnboundedSender;

use crate::{
    error::PlayerError, generic::PlayerExt, rodio::decoder::FFMPEGDecoder, source::ValidSrc,
};

mod decoder;
#[cfg(test)]
mod test;

#[cfg(target_os = "linux")]
mod pulse_monitor;

/// Get the system's preferred sample rate for audio output.
///
/// On Linux, queries PulseAudio/PipeWire directly because cpal's
/// `default_output_config()` is known to return incorrect rates on
/// PipeWire/Snapcast sinks. On other platforms or as a fallback,
/// uses cpal's reported default.
#[tracing::instrument(level = "debug", skip_all)]
pub fn get_system_sample_rate() -> u32 {
    #[cfg(target_os = "linux")]
    {
        if let Some(rate) = pulse_monitor::get_default_sample_rate() {
            tracing::trace!("PulseAudio: Detected sample rate: {} Hz", rate);
            return rate;
        }
        tracing::trace!("PulseAudio: Not available, falling back to cpal");
    }
    get_cpal_default_sample_rate()
}

#[tracing::instrument(level = "debug", skip_all)]
fn get_cpal_default_sample_rate() -> u32 {
    let Some(device) = rodio::cpal::default_host().default_output_device() else {
        tracing::trace!("cpal: No audio device found, using default 44100 Hz");
        return 44100;
    };
    let default_rate = device
        .default_output_config()
        .map(|c: SupportedStreamConfig| c.sample_rate())
        .unwrap_or(44100);
    tracing::trace!("cpal: Using sample rate: {} Hz", default_rate);
    default_rate
}

pub(crate) use decoder::DecoderError;

pub struct RodioPlayer {
    _sink: Mutex<Option<MixerDeviceSink>>,
    player: Arc<Mutex<Option<Player>>>,
    events_tx: UnboundedSender<PlayerEvent>,
}

impl RodioPlayer {
    #[tracing::instrument(level = "debug", skip_all)]
    pub fn new(events_tx: UnboundedSender<PlayerEvent>) -> Self {
        Self {
            _sink: Mutex::new(None),
            player: Arc::new(Mutex::new(None)),
            events_tx,
        }
    }

    #[tracing::instrument(level = "debug", skip_all)]
    fn send_event(events_tx: UnboundedSender<PlayerEvent>, event: PlayerEvent) {
        if let Err(e) = events_tx.send(event) {
            tracing::error!("Failed to send event: {:?}", e);
        }
    }
}

impl PlayerExt for RodioPlayer {
    #[tracing::instrument(level = "debug", skip_all)]
    fn play(&self) -> Result<(), PlayerError> {
        if let Some(player) = self.player.lock().unwrap().as_ref() {
            player.play();
        }
        Ok(())
    }

    #[tracing::instrument(level = "debug", skip_all)]
    fn pause(&self) -> Result<(), PlayerError> {
        if let Some(player) = self.player.lock().unwrap().as_ref() {
            player.pause();
        }
        Ok(())
    }

    #[tracing::instrument(level = "debug", skip_all)]
    fn stop(&self) -> Result<(), PlayerError> {
        if let Some(player) = self.player.lock().unwrap().as_ref() {
            player.stop();
        }
        Ok(())
    }

    #[tracing::instrument(level = "debug", skip_all)]
    fn set_volume(&self, volume: u8) -> Result<(), PlayerError> {
        if let Some(player) = self.player.lock().unwrap().as_ref() {
            player.set_volume(volume as f32 / 100f32);
        }

        Ok(())
    }

    #[tracing::instrument(level = "debug", skip_all)]
    fn seek(&self, pos: Duration) -> Result<(), PlayerError> {
        if let Some(player) = self.player.lock().unwrap().as_ref() {
            player.try_seek(pos)?;
        }
        Ok(())
    }

    #[tracing::instrument(level = "debug", skip_all)]
    fn set_src(&self, src: ValidSrc) -> Result<(), PlayerError> {
        let old_volume = self.get_volume().unwrap_or_else(|e| {
            tracing::error!("Failed to retrieve old volume: {:?}. Defaulting to 50", e);
            50
        });
        let events_tx = self.events_tx.clone();

        if let Some(player) = self.player.lock().unwrap().as_ref() {
            player.clear();
        }

        // Prefer PulseAudio's native sample rate over cpal's default,
        // since cpal's default_output_config() can lie on PipeWire sinks.
        let system_rate = get_system_sample_rate().max(1);
        let safe_rate = NonZero::new(system_rate).expect("Sample rate is clamped to 1");

        // Request the rate from the OS so cpal/PipeWire negotiate it explicitly
        // rather than picking whatever happens to be default. The actual
        // negotiated rate may differ; we use that for decoder output.
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

        let player = rodio::Player::connect_new(sink.mixer());
        player.append(FFMPEGDecoder::open(&src.inner(), output_sample_rate)?);
        player.append(EmptyCallback::new(Box::new(move || {
            let events_tx = events_tx.clone();
            Self::send_event(events_tx, PlayerEvent::Ended(true));
        })));
        player.set_volume(old_volume as f32 / 100f32);

        *self.player.lock().unwrap() = Some(player);
        *self._sink.lock().unwrap() = Some(sink);
        Ok(())
    }

    #[tracing::instrument(level = "debug", skip_all)]
    fn can_play(&self, src: ValidSrc) -> bool {
        match src {
            ValidSrc::Path(path) => path.exists(),
            ValidSrc::Url(url) => url.starts_with("http://") || url.starts_with("https://"),
        }
    }

    #[tracing::instrument(level = "debug", skip_all)]
    fn get_current_pos(&self) -> Result<Duration, PlayerError> {
        if let Some(player) = &self.player.lock().unwrap().as_ref() {
            return Ok(player.get_pos());
        }
        Ok(Duration::default())
    }

    #[tracing::instrument(level = "debug", skip_all)]
    fn get_volume(&self) -> Result<u8, PlayerError> {
        if let Some(player) = self.player.lock().unwrap().as_ref() {
            Ok((player.volume() * 100.0).round() as u8)
        } else {
            Ok(100)
        }
    }

    #[tracing::instrument(level = "debug", skip_all)]
    fn get_player_state(
        &self,
    ) -> Result<extensions_proto::moosync::types::PlayerState, PlayerError> {
        let guard = self.player.lock().unwrap();
        let Some(player) = guard.as_ref() else {
            return Ok(extensions_proto::moosync::types::PlayerState::Stopped);
        };

        if player.empty() {
            return Ok(extensions_proto::moosync::types::PlayerState::Stopped);
        }
        if player.is_paused() {
            return Ok(extensions_proto::moosync::types::PlayerState::Paused);
        }
        Ok(extensions_proto::moosync::types::PlayerState::Playing)
    }
}
