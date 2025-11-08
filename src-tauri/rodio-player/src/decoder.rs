// Adapted from https://github.com/tarkah/ffmpeg-decoder-rs

use std::fmt::Display;
use std::num::NonZero;
use std::str::FromStr;
use std::sync::Arc;

use rodio::source::SeekError;
use rodio::{Sample, Source};
use rsmpeg::avformat::{self, AVFormatContextInput};
use rsmpeg::error::RsmpegError;
use tracing::error;
use types::errors::MoosyncError;

use std::ffi::{c_int, CString, NulError};
use std::time::Duration;

use rsmpeg::ffi::{AVSampleFormat, AVMEDIA_TYPE_AUDIO, AV_SAMPLE_FMT_FLT};

use rsmpeg::avcodec::AVCodecContext;
use rsmpeg::avutil::{err2str, sample_fmt_is_planar, AVFrame, AVSamples};
use rsmpeg::swresample::SwrContext;

// Rodio needs f32 samples in non planar format
const DEFAULT_CONVERSION_FORMAT: AVSampleFormat = AV_SAMPLE_FMT_FLT;

use thiserror::Error;

#[derive(Error, Debug, PartialEq, Eq)]
pub enum Error {
    #[error("Could not find any audio stream")]
    NoAudioStream,
    #[error("Wrong stream got selected somehow, expected {0}, got {1}")]
    WrongStream(usize, i32),
    #[error("Rsmpeg genric error: {0}")]
    RsmpegError(#[from] rsmpeg::error::RsmpegError),
    #[error("Error parsing string: {0}")]
    StringError(#[from] NulError),
    #[error("AVERROR({code}): `{msg}`", code = .0, msg = err2str(*.0).unwrap_or_else(|| "Unknown error code.".to_string()))]
    AVError(c_int),
}

impl Into<SeekError> for Error {
    fn into(self) -> SeekError {
        let arc = Arc::new(self);
        SeekError::Other(arc)
    }
}

impl Into<MoosyncError> for Error {
    fn into(self) -> MoosyncError {
        MoosyncError::PlaybackError(Box::new(self))
    }
}

pub struct FFMPEGDecoder {
    format_ctx: AVFormatContextInput,
    stream_idx: usize,
    codec_ctx: AVCodecContext,
    swr_ctx: Option<SwrContext>,
    current_frame: Vec<u8>, // holds interleaved f32 bytes ready to be consumed
    requested_seek_timestamp: i64,
}

impl FFMPEGDecoder {
    fn initialize_swr_context(codec_ctx: &AVCodecContext) -> Result<Option<SwrContext>, Error> {
        // Initialize swr context if conversion is needed OR if the decoded format is planar.
        // (Planar -> interleaved needs SwrContext even if sample formats are both float)
        let need_swr = codec_ctx.sample_fmt != DEFAULT_CONVERSION_FORMAT
            || sample_fmt_is_planar(codec_ctx.sample_fmt);

        if need_swr {
            let mut ctx = SwrContext::new(
                &codec_ctx.ch_layout,
                DEFAULT_CONVERSION_FORMAT,
                codec_ctx.sample_rate,
                &codec_ctx.ch_layout,
                codec_ctx.sample_fmt,
                codec_ctx.sample_rate,
            )?;
            ctx.init()?;
            Ok(Some(ctx))
        } else {
            Ok(None)
        }
    }

    pub fn open<'b, S: Display>(path: S) -> Result<FFMPEGDecoder, Error> {
        // https://www.ffmpeg.org/ffmpeg-protocols.html#cache
        let format_ctx = AVFormatContextInput::builder()
            .url(&CString::from_str(&format!("cache:{}", path))?)
            .open()?;

        let stream = format_ctx.find_best_stream(AVMEDIA_TYPE_AUDIO)?;
        if let Some((stream_idx, codec)) = stream {
            // Get the streams codec
            let mut codec_ctx = AVCodecContext::new(&codec);
            codec_ctx.apply_codecpar(&format_ctx.streams().get(stream_idx).unwrap().codecpar())?;
            codec_ctx.open(None)?;

            let swr_ctx = Self::initialize_swr_context(&codec_ctx)?;
            return Ok(FFMPEGDecoder {
                format_ctx,
                stream_idx,
                codec_ctx,
                swr_ctx,
                current_frame: Vec::new(),
                requested_seek_timestamp: 0,
            });
        }
        return Err(Error::NoAudioStream);
    }

    fn convert_and_store_frame(&mut self, frame: &AVFrame) -> Result<(), Error> {
        let num_samples = frame.nb_samples as i32;
        let num_channels = self.codec_ctx.ch_layout.nb_channels as i32;

        // Get pointer to extended_data (frame plane pointers)
        let extended_data_ptr = frame.extended_data.cast();

        if let Some(swr_ctx) = &mut self.swr_ctx {
            // Convert (this will handle planar -> interleaved and type conversion)
            let out_samples = swr_ctx.get_out_samples(num_samples);

            // Many rsmpeg/swresample wrappers expect you to provide buffers.
            // Use AVSamples to allocate the output buffer and call convert with its pointer(s).
            let mut samples = AVSamples::new(
                num_channels as i32,
                out_samples,
                DEFAULT_CONVERSION_FORMAT,
                0,
            )
            .expect("AVSamples allocation failed");

            let converted = unsafe {
                // Call convert with allocated output buffers
                swr_ctx.convert(
                    samples.audio_data.as_mut_ptr(),
                    out_samples,
                    extended_data_ptr,
                    num_samples,
                )?
            };

            // `converted` is number of samples output per channel
            let (_, dst_bufsize) = AVSamples::get_buffer_size(
                num_channels as i32,
                converted,
                DEFAULT_CONVERSION_FORMAT,
                0,
            )
            .unwrap();

            // Create a slice referencing the buffer and copy into current_frame
            let p = samples.audio_data[0] as *const u8;
            let slice = unsafe { std::slice::from_raw_parts(p, dst_bufsize as usize) };
            self.current_frame.clear();
            self.current_frame.extend_from_slice(slice);
        } else {
            // Assume interleaved and already in desired format - take contiguous buffer
            // frame.linesize[0] holds the size in bytes of the first buffer
            let size = frame.linesize[0] as usize;
            let p: *const u8 = frame.extended_data.cast::<u8>();
            let slice = unsafe { std::slice::from_raw_parts(p, size) };
            self.current_frame.clear();
            self.current_frame.extend_from_slice(slice);
        }

        Ok(())
    }

    fn decode_next_packet(&mut self) -> Result<Option<AVFrame>, Error> {
        // Read the next packet
        let packet_opt = self.format_ctx.read_packet()?;

        let packet = match packet_opt {
            Some(p) => p,
            None => return Ok(None), // EOF
        };

        // Only handle our chosen stream
        if (packet.stream_index as usize) != self.stream_idx {
            return Err(Error::WrongStream(
                /*expected=*/ self.stream_idx,
                /*got=*/ packet.stream_index,
            ));
        }

        // Send packet to decoder
        self.codec_ctx.send_packet(Some(&packet))?;

        // Attempt to receive one decoded frame
        match self.codec_ctx.receive_frame() {
            Ok(frame) => Ok(Some(frame)),
            Err(RsmpegError::DecoderDrainError) => Ok(None), // We sent what we had, probably can't decode anymore
            Err(e) => Err(Error::RsmpegError(e)),
        }
    }

    fn process_next_packet(&mut self) -> Result<(), Error> {
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

    fn flush_buffers(&mut self) {
        self.current_frame.clear();
    }

    fn resync_after_seek(&mut self) -> Result<(), Error> {
        println!("Resyncing to {}", self.requested_seek_timestamp);

        loop {
            match self.decode_next_packet() {
                Ok(Some(frame)) => {
                    if frame.best_effort_timestamp >= self.requested_seek_timestamp {
                        break;
                    }
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
    fn next(&mut self) -> Option<Self::Item> {
        if !self.current_frame.is_empty() {
            return Some(self.next_sample());
        }

        match self.process_next_packet() {
            Err(Error::WrongStream(expected, got)) => {
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
    fn next_sample(&mut self) -> Sample {
        if self.current_frame.len() == 0 {
            return 0f32;
        }
        // pop 4 bytes (f32 LE) and convert
        let b0 = self.current_frame.remove(0);
        let b1 = self.current_frame.remove(0);
        let b2 = self.current_frame.remove(0);
        let b3 = self.current_frame.remove(0);
        let bytes = [b0, b1, b2, b3];
        f32::from_le_bytes(bytes)
    }
}

impl Source for FFMPEGDecoder {
    #[inline]
    fn channels(&self) -> NonZero<u16> {
        NonZero::new(self.codec_ctx.ch_layout.nb_channels as u16).unwrap()
    }

    #[inline]
    fn sample_rate(&self) -> NonZero<u32> {
        NonZero::new(self.codec_ctx.sample_rate as u32).unwrap()
    }

    #[inline]
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

    fn current_span_len(&self) -> Option<usize> {
        None
    }

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
            .map_err(|e| Into::<SeekError>::into(Into::<Error>::into(e)))?;

        self.requested_seek_timestamp = timestamp;
        self.resync_after_seek().map_err(Into::<SeekError>::into)?;

        Ok(())
    }
}
