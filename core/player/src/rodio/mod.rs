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

use extensions_proto::moosync::types::player_event::Event as PlayerEvent;
use rodio::cpal::{
    SupportedStreamConfig,
    traits::{DeviceTrait, HostTrait},
};
use tokio::sync::mpsc::UnboundedSender;

use crate::{
    context::{AudioPlayerContext, RodioPlayerContext},
    error::PlayerError,
    generic::PlayerExt,
    source::ValidSrc,
};

pub(crate) mod decoder;
#[cfg(test)]
mod decoder_test;
#[cfg(test)]
mod mod_test;

#[cfg(target_os = "linux")]
mod pulse_monitor;

#[cfg(all(test, target_os = "linux"))]
mod pulse_monitor_test;

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
    context: Box<dyn AudioPlayerContext>,
    events_tx: UnboundedSender<PlayerEvent>,
}

impl RodioPlayer {
    #[tracing::instrument(level = "debug", skip_all)]
    pub fn new(events_tx: UnboundedSender<PlayerEvent>) -> Self {
        Self {
            context: Box::new(RodioPlayerContext::new()),
            events_tx,
        }
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub fn new_with_context(
        context: Box<dyn AudioPlayerContext>,
        events_tx: UnboundedSender<PlayerEvent>,
    ) -> Self {
        Self { context, events_tx }
    }
}

impl PlayerExt for RodioPlayer {
    #[tracing::instrument(level = "debug", skip_all)]
    fn play(&self) -> Result<(), PlayerError> { self.context.play() }

    #[tracing::instrument(level = "debug", skip_all)]
    fn pause(&self) -> Result<(), PlayerError> { self.context.pause() }

    #[tracing::instrument(level = "debug", skip_all)]
    fn stop(&self) -> Result<(), PlayerError> { self.context.stop() }

    #[tracing::instrument(level = "debug", skip_all)]
    fn set_volume(&self, volume: u8) -> Result<(), PlayerError> { self.context.set_volume(volume) }

    #[tracing::instrument(level = "debug", skip_all)]
    fn seek(&self, pos: Duration) -> Result<(), PlayerError> { self.context.seek(pos) }

    #[tracing::instrument(level = "debug", skip_all)]
    fn set_src(&self, src: ValidSrc) -> Result<(), PlayerError> {
        self.context.set_src(src, self.events_tx.clone())
    }

    #[tracing::instrument(level = "debug", skip_all)]
    fn can_play(&self, src: ValidSrc) -> bool {
        match src {
            ValidSrc::Path(path) => path.exists(),
            ValidSrc::Url(url) => url.starts_with("http://") || url.starts_with("https://"),
        }
    }

    #[tracing::instrument(level = "debug", skip_all)]
    fn get_current_pos(&self) -> Result<Duration, PlayerError> { self.context.get_current_pos() }

    #[tracing::instrument(level = "debug", skip_all)]
    fn get_volume(&self) -> Result<u8, PlayerError> { self.context.get_volume() }

    #[tracing::instrument(level = "debug", skip_all)]
    fn get_player_state(
        &self,
    ) -> Result<extensions_proto::moosync::types::PlayerState, PlayerError> {
        self.context.get_player_state()
    }
}
