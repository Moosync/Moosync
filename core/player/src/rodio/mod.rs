use rodio::cpal::{
    SupportedStreamConfig,
    traits::{DeviceTrait, HostTrait},
};

pub(crate) mod decoder;
#[cfg(test)]
mod decoder_test;

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
pub(crate) fn get_cpal_default_sample_rate() -> u32 {
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
