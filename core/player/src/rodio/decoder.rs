// Adapted from https://github.com/tarkah/ffmpeg-decoder-rs

use std::{
    ffi::{CString, NulError, c_int},
    num::NonZero,
    str::FromStr,
    sync::Arc,
    time::Duration,
};

use rodio::{ChannelCount, Sample, SampleRate, Source, source::SeekError};
use rsmpeg::{
    avcodec::AVCodecContext,
    avformat::AVFormatContextInput,
    avutil::{AVFrame, AVSamples, err2str},
    error::RsmpegError,
    ffi::{AV_SAMPLE_FMT_FLT, AVMEDIA_TYPE_AUDIO, AVSampleFormat},
    swresample::SwrContext,
};
// Rodio needs f32 samples in non planar format
const DEFAULT_CONVERSION_FORMAT: AVSampleFormat = AV_SAMPLE_FMT_FLT;

use thiserror::Error;
use tracing::error;

#[derive(Error, Debug, PartialEq, Eq)]
pub enum DecoderError {
    #[error("Could not find any audio stream")]
    NoAudioStream,
    #[error("Wrong stream got selected somehow, expected {0}, got {1}")]
    WrongStream(usize, i32),
    #[error("Rsmpeg genric error: {0}")]
    Rsmpeg(#[from] rsmpeg::error::RsmpegError),
    #[error("Error parsing string: {0}")]
    String(#[from] NulError),
    #[error("AVERROR({code}): `{msg}`", code = .0, msg = err2str(*.0).unwrap_or_else(|| "Unknown error code.".to_string()))]
    AV(c_int),
}

impl From<DecoderError> for SeekError {
    fn from(e: DecoderError) -> SeekError { SeekError::Other(Arc::new(e)) }
}

pub struct FFMPEGDecoder {
    format_ctx: AVFormatContextInput,
    stream_idx: usize,
    codec_ctx: AVCodecContext,
    swr_ctx: SwrContext,
    current_frame: Vec<u8>, // holds interleaved f32 bytes ready to be consumed
    requested_seek_timestamp: i64,
    output_sample_rate: u32, // Target sample rate (matches audio device)
}

impl FFMPEGDecoder {
    #[tracing::instrument(level = "debug", skip_all)]
    fn initialize_swr_context(
        codec_ctx: &AVCodecContext,
        output_sample_rate: i32,
    ) -> Result<SwrContext, DecoderError> {
        // Always use SwrContext to handle format AND sample rate conversion.
        // Without this, decoder reports source rate to rodio instead of the
        // output rate, causing stale buffer reports and audible drift on
        // outputs where source rate != device rate.
        tracing::trace!(
            "DECODER: sample_fmt={}, sample_rate={} -> {}, channels={}",
            codec_ctx.sample_fmt,
            codec_ctx.sample_rate,
            output_sample_rate,
            codec_ctx.ch_layout.nb_channels
        );

        let mut ctx = SwrContext::new(
            &codec_ctx.ch_layout,
            DEFAULT_CONVERSION_FORMAT,
            output_sample_rate, // Output at device's native rate
            &codec_ctx.ch_layout,
            codec_ctx.sample_fmt,
            codec_ctx.sample_rate, // Input at source rate
        )?;
        ctx.init()?;
        Ok(ctx)
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub fn open(path: &str, output_sample_rate: u32) -> Result<FFMPEGDecoder, DecoderError> {
        let input_path = if path.starts_with("http") {
            CString::from_str(&format!("cache:{}", path))?
        } else {
            CString::from_str(&format!("file:{}", path))?
        };
        // https://www.ffmpeg.org/ffmpeg-protocols.html#cache
        let format_ctx = AVFormatContextInput::builder()
            .url(&input_path)
            .open()
            .map_err(|e| DecoderError::AV(e.raw_error().unwrap_or_default()))?;

        let stream = format_ctx.find_best_stream(AVMEDIA_TYPE_AUDIO)?;
        if let Some((stream_idx, codec)) = stream {
            // Get the streams codec
            let mut codec_ctx = AVCodecContext::new(&codec);
            codec_ctx.open(None)?;
            codec_ctx.apply_codecpar(&format_ctx.streams().get(stream_idx).unwrap().codecpar())?;

            let swr_ctx = Self::initialize_swr_context(&codec_ctx, output_sample_rate as i32)?;
            tracing::trace!(
                "Stream details: bitrate: {}, channels: {}, codec: {:?}, source_rate: {}, output_rate: {}",
                codec_ctx.bit_rate,
                codec_ctx.ch_layout.nb_channels,
                codec_ctx.codec,
                codec_ctx.sample_rate,
                output_sample_rate
            );

            return Ok(FFMPEGDecoder {
                format_ctx,
                stream_idx,
                codec_ctx,
                swr_ctx,
                current_frame: Vec::new(),
                requested_seek_timestamp: 0,
                output_sample_rate,
            });
        }
        Err(DecoderError::NoAudioStream)
    }

    #[tracing::instrument(level = "debug", skip_all)]
    fn convert_and_store_frame(&mut self, frame: &AVFrame) -> Result<(), DecoderError> {
        let num_samples = frame.nb_samples;
        let num_channels = self.codec_ctx.ch_layout.nb_channels;

        // Get pointer to extended_data (frame plane pointers)
        let extended_data_ptr = frame.extended_data.cast();

        // Convert using SwrContext (handles format + sample rate conversion).
        let out_samples = self.swr_ctx.get_out_samples(num_samples);

        let mut samples = AVSamples::new(num_channels, out_samples, DEFAULT_CONVERSION_FORMAT, 0)
            .expect("AVSamples allocation failed");

        let converted = unsafe {
            self.swr_ctx.convert(
                samples.audio_data.as_mut_ptr(),
                out_samples,
                extended_data_ptr,
                num_samples,
            )?
        };

        let (_, dst_bufsize) =
            AVSamples::get_buffer_size(num_channels, converted, DEFAULT_CONVERSION_FORMAT, 0)
                .unwrap();

        // Copy converted samples to current_frame.
        // SwrContext outputs to AV_SAMPLE_FMT_FLT (interleaved float), so all data
        // is in audio_data[0] as a single contiguous buffer. This works regardless
        // of whether the input was planar or interleaved.
        let p = samples.audio_data[0] as *const u8;
        let slice = unsafe { std::slice::from_raw_parts(p, dst_bufsize as usize) };
        self.current_frame.clear();
        self.current_frame.extend_from_slice(slice);

        Ok(())
    }

    #[tracing::instrument(level = "debug", skip_all)]
    fn decode_next_packet(&mut self) -> Result<Option<AVFrame>, DecoderError> {
        // Read the next packet
        let packet_opt = self.format_ctx.read_packet()?;

        let packet = match packet_opt {
            Some(p) => p,
            None => {
                return Ok(None);
            } // EOF
        };

        // Only handle our chosen stream
        if (packet.stream_index as usize) != self.stream_idx {
            return Err(DecoderError::WrongStream(
                /* expected= */ self.stream_idx,
                /* got= */ packet.stream_index,
            ));
        }

        // Send packet to decoder
        self.codec_ctx.send_packet(Some(&packet))?;

        // Attempt to receive one decoded frame
        match self.codec_ctx.receive_frame() {
            Ok(frame) => Ok(Some(frame)),
            Err(RsmpegError::DecoderDrainError) => Ok(None), /* We sent what we had, probably */
            // can't decode anymore
            Err(e) => Err(DecoderError::Rsmpeg(e)),
        }
    }

    #[tracing::instrument(level = "debug", skip_all)]
    fn process_next_packet(&mut self) -> Result<(), DecoderError> {
        if !self.current_frame.is_empty() {
            return Ok(());
        }

        // Try decoding one packet/frame
        match self.decode_next_packet()? {
            Some(frame) => {
                self.convert_and_store_frame(&frame)?;
                Ok(())
            }
            None => {
                // EOF
                self.flush_buffers();
                Ok(())
            }
        }
    }

    #[tracing::instrument(level = "debug", skip_all)]
    fn flush_buffers(&mut self) { self.current_frame.clear(); }

    #[tracing::instrument(level = "debug", skip_all)]
    fn resync_after_seek(&mut self) -> Result<(), DecoderError> {
        loop {
            match self.decode_next_packet() {
                Ok(Some(frame)) => {
                    if frame.best_effort_timestamp >= self.requested_seek_timestamp {
                        self.convert_and_store_frame(&frame)?;
                        break;
                    }
                }
                Err(DecoderError::WrongStream(_, _)) => {
                    continue;
                }
                Err(e) => return Err(e),
                _ => {
                    break;
                }
            }
        }

        Ok(())
    }
}

unsafe impl Send for FFMPEGDecoder {}

impl Iterator for FFMPEGDecoder {
    type Item = Sample;

    #[inline]
    #[tracing::instrument(level = "debug", skip_all)]
    fn next(&mut self) -> Option<Self::Item> {
        if !self.current_frame.is_empty() {
            return Some(self.next_sample());
        }

        match self.process_next_packet() {
            Err(DecoderError::WrongStream(expected, got)) => {
                tracing::debug!("Tried to decode stream {}, expected {}", got, expected);
                return self.next();
            }
            Err(e) => {
                error!("Error filling buffer: {:?}", e);
                self.flush_buffers();
                return None;
            }
            _ => (),
        }

        if !self.current_frame.is_empty() {
            Some(self.next_sample())
        } else {
            None
        }
    }
}

impl FFMPEGDecoder {
    // Helper to read next sample as f32 from current_frame bytes.
    // We assume output format is interleaved f32 (AV_SAMPLE_FMT_FLT).
    #[tracing::instrument(level = "debug", skip_all)]
    fn next_sample(&mut self) -> Sample {
        if self.current_frame.is_empty() {
            return 0f32;
        }
        // pop 4 bytes (f32 LE) and convert
        let b0 = self.current_frame.remove(0);
        let b1 = self.current_frame.remove(0);
        let b2 = self.current_frame.remove(0);
        let b3 = self.current_frame.remove(0);
        let bytes = [b0, b1, b2, b3];
        let sample = f32::from_le_bytes(bytes);

        sample.clamp(-1.0, 1.0)
    }
}

impl Source for FFMPEGDecoder {
    #[inline]
    #[tracing::instrument(level = "debug", skip_all)]
    fn channels(&self) -> ChannelCount {
        NonZero::new(self.codec_ctx.ch_layout.nb_channels as u16).unwrap()
    }

    #[inline]
    #[tracing::instrument(level = "debug", skip_all)]
    fn sample_rate(&self) -> SampleRate {
        // Report the output rate (what rodio will consume), not the source rate.
        // Otherwise rodio computes stale buffer/pos data when source rate != device
        // rate.
        NonZero::new(self.output_sample_rate)
            .unwrap_or_else(|| NonZero::new(44100).expect("44100 is non-zero"))
    }

    #[inline]
    #[tracing::instrument(level = "debug", skip_all)]
    fn total_duration(&self) -> Option<Duration> {
        let stream = &self.format_ctx.streams()[self.stream_idx];

        if stream.duration <= 0 {
            return None;
        }

        let time_base = stream.time_base;

        let micros = (stream.duration as u64)
            .saturating_mul(time_base.num as u64)
            .saturating_mul(1_000_000)
            / (time_base.den as u64);

        Some(Duration::from_micros(micros))
    }

    #[tracing::instrument(level = "debug", skip_all)]
    fn current_span_len(&self) -> Option<usize> { None }

    #[tracing::instrument(level = "debug", skip_all)]
    fn try_seek(&mut self, pos: Duration) -> Result<(), SeekError> {
        let stream = &self.format_ctx.streams()[self.stream_idx];
        let time_base = stream.time_base;

        // Convert Duration -> timestamp in stream time_base units
        let timestamp =
            (pos.as_secs_f64() * time_base.den as f64 / time_base.num as f64).round() as i64;

        self.flush_buffers();

        self.format_ctx
            .seek(
                self.stream_idx as i32,
                timestamp,
                rsmpeg::ffi::AVSEEK_FLAG_BACKWARD as i32,
            )
            .map_err(|e| Into::<SeekError>::into(Into::<DecoderError>::into(e)))?;

        self.requested_seek_timestamp = timestamp;
        self.resync_after_seek().map_err(Into::<SeekError>::into)?;

        Ok(())
    }
}

impl Drop for FFMPEGDecoder {
    fn drop(&mut self) {
        tracing::trace!("Dropping ffmpeg decoder");
        let _ = self.codec_ctx.send_packet(None);
    }
}
