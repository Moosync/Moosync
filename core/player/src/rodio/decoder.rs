// Adapted from https://github.com/tarkah/ffmpeg-decoder-rs

use std::{
    ffi::{CString, NulError, c_int},
    num::NonZero,
    ptr, slice,
    str::FromStr,
    sync::Arc,
    time::Duration,
};

use rodio::{ChannelCount, Sample, SampleRate, Source, source::SeekError};
use rsmpeg::{
    avcodec::AVCodecContext,
    avformat::AVFormatContextInput,
    avutil::{AVFrame, AVRational, AVSamples, err2str},
    error::RsmpegError,
    ffi::{
        AV_NOPTS_VALUE, AV_SAMPLE_FMT_FLT, AVMEDIA_TYPE_AUDIO, AVSEEK_FLAG_BACKWARD, AVSampleFormat,
    },
    swresample::SwrContext,
};
use thiserror::Error;
use tracing::{error, trace};

const DEFAULT_CONVERSION_FORMAT: AVSampleFormat = AV_SAMPLE_FMT_FLT;
const DEFAULT_FALLBACK_SAMPLE_RATE: u32 = 44100;

#[derive(Error, Debug, PartialEq, Eq)]
pub enum DecoderError {
    #[error("Could not find any audio stream")]
    NoAudioStream,
    #[error("Rsmpeg generic error: {0}")]
    Rsmpeg(#[from] RsmpegError),
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
    current_frame: Vec<f32>,
    current_frame_cursor: usize,
    requested_seek_timestamp: i64,
    output_sample_rate: u32,
    demuxer_eof: bool,
}

#[tracing::instrument(level = "debug", skip_all)]
fn format_input_url(path: &str) -> Result<CString, NulError> {
    if path.starts_with("http") {
        return CString::from_str(&format!("cache:{}", path));
    }
    CString::from_str(&format!("file:{}", path))
}

#[tracing::instrument(level = "debug", skip_all)]
fn calculate_seek_timestamp(pos: Duration, time_base: AVRational) -> i64 {
    if time_base.den <= 0 || time_base.num <= 0 {
        return 0;
    }
    (pos.as_secs_f64() * time_base.den as f64 / time_base.num as f64).round() as i64
}

impl FFMPEGDecoder {
    /// Initializes the libswresample context for converting audio format and
    /// sample rate.
    #[tracing::instrument(level = "debug", skip_all)]
    fn initialize_swr_context(
        codec_ctx: &AVCodecContext,
        output_sample_rate: i32,
    ) -> Result<SwrContext, DecoderError> {
        trace!(
            "DECODER: sample_fmt={}, sample_rate={} -> {}, channels={}",
            codec_ctx.sample_fmt,
            codec_ctx.sample_rate,
            output_sample_rate,
            codec_ctx.ch_layout.nb_channels
        );

        let mut ctx = SwrContext::new(
            &codec_ctx.ch_layout,
            DEFAULT_CONVERSION_FORMAT,
            output_sample_rate,
            &codec_ctx.ch_layout,
            codec_ctx.sample_fmt,
            codec_ctx.sample_rate,
        )?;
        ctx.init()?;
        Ok(ctx)
    }

    /// Opens an audio file or stream from the given path/URL and initializes
    /// the decoder.
    #[tracing::instrument(level = "debug", skip_all)]
    pub fn open(path: &str, output_sample_rate: u32) -> Result<FFMPEGDecoder, DecoderError> {
        let input_path = format_input_url(path)?;

        let format_ctx = AVFormatContextInput::builder()
            .url(&input_path)
            .open()
            .map_err(|e| DecoderError::AV(e.raw_error().unwrap_or_default()))?;

        let stream = format_ctx.find_best_stream(AVMEDIA_TYPE_AUDIO)?;
        let Some((stream_idx, codec)) = stream else {
            return Err(DecoderError::NoAudioStream);
        };

        let mut codec_ctx = AVCodecContext::new(&codec);
        codec_ctx.apply_codecpar(&format_ctx.streams().get(stream_idx).unwrap().codecpar())?;
        codec_ctx.open(None)?;

        let target_sample_rate = if output_sample_rate > 0 {
            output_sample_rate
        } else {
            codec_ctx
                .sample_rate
                .max(DEFAULT_FALLBACK_SAMPLE_RATE as i32) as u32
        };

        let swr_ctx = Self::initialize_swr_context(&codec_ctx, target_sample_rate as i32)?;
        trace!(
            "Stream details: bitrate: {}, channels: {}, codec: {:?}, source_rate: {}, output_rate: {}",
            codec_ctx.bit_rate,
            codec_ctx.ch_layout.nb_channels,
            codec_ctx.codec,
            codec_ctx.sample_rate,
            target_sample_rate
        );

        Ok(FFMPEGDecoder {
            format_ctx,
            stream_idx,
            codec_ctx,
            swr_ctx,
            current_frame: Vec::new(),
            current_frame_cursor: 0,
            requested_seek_timestamp: 0,
            output_sample_rate: target_sample_rate,
            demuxer_eof: false,
        })
    }

    /// Resamples audio samples from `in_data` and appends them to
    /// `self.current_frame`. Pass `in_data = null` and `in_count = 0` to
    /// drain trailing buffered samples from `SwrContext` at EOF.
    #[tracing::instrument(level = "debug", skip_all)]
    fn convert_and_store_samples(
        &mut self,
        in_data: *const *const u8,
        in_count: i32,
    ) -> Result<bool, DecoderError> {
        let out_samples = self.swr_ctx.get_out_samples(in_count);
        if out_samples <= 0 {
            return Ok(false);
        }

        let num_channels = self.codec_ctx.ch_layout.nb_channels;
        let mut samples = AVSamples::new(num_channels, out_samples, DEFAULT_CONVERSION_FORMAT, 0)
            .expect("AVSamples allocation failed");

        let converted = unsafe {
            self.swr_ctx.convert(
                samples.audio_data.as_mut_ptr(),
                out_samples,
                in_data,
                in_count,
            )?
        };

        if converted <= 0 {
            return Ok(false);
        }

        let total_samples = (converted * num_channels) as usize;
        let sample_slice =
            unsafe { slice::from_raw_parts(samples.audio_data[0] as *const f32, total_samples) };
        self.current_frame.extend_from_slice(sample_slice);
        Ok(true)
    }

    /// Reads demuxer packets until finding the next packet for our stream,
    /// sending it to the codec. Sends `None` to the codec once when EOF is
    /// reached to initiate codec flushing.
    #[tracing::instrument(level = "debug", skip_all)]
    fn read_and_send_packet(&mut self) -> Result<bool, DecoderError> {
        if self.demuxer_eof {
            return Ok(false);
        }

        while let Some(packet) = self.format_ctx.read_packet()? {
            if packet.stream_index as usize == self.stream_idx {
                self.codec_ctx.send_packet(Some(&packet))?;
                return Ok(true);
            }
        }

        self.demuxer_eof = true;
        self.codec_ctx.send_packet(None)?;
        Ok(false)
    }

    /// Pulls a decoded frame from the codec, feeding packets as needed until a
    /// frame is produced or EOF.
    #[tracing::instrument(level = "debug", skip_all)]
    fn receive_frame(&mut self) -> Result<Option<AVFrame>, DecoderError> {
        loop {
            match self.codec_ctx.receive_frame() {
                Ok(frame) => return Ok(Some(frame)),
                Err(RsmpegError::DecoderDrainError) => {
                    let has_more = self.read_and_send_packet()?;
                    if !has_more {
                        match self.codec_ctx.receive_frame() {
                            Ok(frame) => return Ok(Some(frame)),
                            Err(
                                RsmpegError::DecoderDrainError | RsmpegError::DecoderFlushedError,
                            ) => {
                                return Ok(None);
                            }
                            Err(e) => return Err(DecoderError::Rsmpeg(e)),
                        }
                    }
                }
                Err(RsmpegError::DecoderFlushedError) => return Ok(None),
                Err(e) => return Err(DecoderError::Rsmpeg(e)),
            }
        }
    }

    /// Decodes the next available audio frame, or drains the resampler if at
    /// EOF.
    #[tracing::instrument(level = "debug", skip_all)]
    fn decode_next_frame(&mut self) -> Result<bool, DecoderError> {
        if let Some(frame) = self.receive_frame()? {
            return self.convert_and_store_samples(frame.extended_data.cast(), frame.nb_samples);
        }

        self.convert_and_store_samples(ptr::null(), 0)
    }

    /// Refills `self.current_frame` when all samples from the current batch
    /// have been consumed.
    #[tracing::instrument(level = "debug", skip_all)]
    fn process_next_frame(&mut self) -> Result<bool, DecoderError> {
        if self.current_frame_cursor < self.current_frame.len() {
            return Ok(true);
        }

        self.flush_buffers();

        while self.current_frame.is_empty() {
            let has_more = self.decode_next_frame()?;
            if !has_more && self.current_frame.is_empty() {
                return Ok(false);
            }
        }

        Ok(true)
    }

    /// Clears internal decoded sample buffers and resets the read cursor.
    #[tracing::instrument(level = "debug", skip_all)]
    fn flush_buffers(&mut self) {
        self.current_frame.clear();
        self.current_frame_cursor = 0;
    }

    /// Skips decoded frames after a seek until reaching the target timestamp.
    #[tracing::instrument(level = "debug", skip_all)]
    fn resync_after_seek(&mut self) -> Result<(), DecoderError> {
        while let Some(frame) = self.receive_frame()? {
            if frame.best_effort_timestamp == AV_NOPTS_VALUE
                || frame.best_effort_timestamp >= self.requested_seek_timestamp
            {
                self.convert_and_store_samples(frame.extended_data.cast(), frame.nb_samples)?;
                return Ok(());
            }
        }

        Ok(())
    }

    /// Yields the next individual f32 sample from the buffer.
    #[inline]
    #[tracing::instrument(level = "debug", skip_all)]
    fn next_sample(&mut self) -> Sample {
        if self.current_frame_cursor >= self.current_frame.len() {
            return 0f32;
        }
        let sample = self.current_frame[self.current_frame_cursor];
        self.current_frame_cursor += 1;
        sample.clamp(-1.0, 1.0)
    }
}

unsafe impl Send for FFMPEGDecoder {}

impl Iterator for FFMPEGDecoder {
    type Item = Sample;

    #[inline]
    #[tracing::instrument(level = "debug", skip_all)]
    fn next(&mut self) -> Option<Self::Item> {
        if self.current_frame_cursor < self.current_frame.len() {
            return Some(self.next_sample());
        }

        match self.process_next_frame() {
            Ok(true) => Some(self.next_sample()),
            Ok(false) => None,
            Err(e) => {
                error!("Error filling buffer: {:?}", e);
                self.flush_buffers();
                None
            }
        }
    }
}

impl Source for FFMPEGDecoder {
    #[inline]
    #[tracing::instrument(level = "debug", skip_all)]
    fn channels(&self) -> ChannelCount {
        NonZero::new(self.codec_ctx.ch_layout.nb_channels.max(1) as u16).unwrap()
    }

    #[inline]
    #[tracing::instrument(level = "debug", skip_all)]
    fn sample_rate(&self) -> SampleRate {
        NonZero::new(self.output_sample_rate)
            .unwrap_or_else(|| NonZero::new(DEFAULT_FALLBACK_SAMPLE_RATE).expect("non-zero"))
    }

    #[inline]
    #[tracing::instrument(level = "debug", skip_all)]
    fn total_duration(&self) -> Option<Duration> {
        let stream = &self.format_ctx.streams()[self.stream_idx];
        let time_base = stream.time_base;

        if stream.duration > 0 && time_base.den > 0 && time_base.num > 0 {
            let micros = (stream.duration as u64)
                .saturating_mul(time_base.num as u64)
                .saturating_mul(1_000_000)
                / (time_base.den as u64);
            return Some(Duration::from_micros(micros));
        }

        let duration = self.format_ctx.duration();
        if duration > 0 {
            return Some(Duration::from_micros(duration as u64));
        }

        None
    }

    #[tracing::instrument(level = "debug", skip_all)]
    fn current_span_len(&self) -> Option<usize> { None }

    #[tracing::instrument(level = "debug", skip_all)]
    fn try_seek(&mut self, pos: Duration) -> Result<(), SeekError> {
        let stream = &self.format_ctx.streams()[self.stream_idx];
        let timestamp = calculate_seek_timestamp(pos, stream.time_base);

        self.flush_buffers();

        self.format_ctx
            .seek(
                self.stream_idx as i32,
                timestamp,
                AVSEEK_FLAG_BACKWARD as i32,
            )
            .map_err(|e| SeekError::from(DecoderError::from(e)))?;

        self.codec_ctx.flush_buffers();
        self.demuxer_eof = false;
        self.swr_ctx =
            Self::initialize_swr_context(&self.codec_ctx, self.output_sample_rate as i32)
                .map_err(SeekError::from)?;
        self.requested_seek_timestamp = timestamp;
        self.resync_after_seek().map_err(SeekError::from)?;

        Ok(())
    }
}

impl Drop for FFMPEGDecoder {
    fn drop(&mut self) {
        trace!("Dropping ffmpeg decoder");
    }
}
