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

use std::{fs, time::Duration};

use rodio::Source;
use wiremock::{Mock, MockServer, ResponseTemplate, matchers::method};

use crate::rodio::decoder::FFMPEGDecoder;

const CAPTURE_DURATION_SECS: u64 = 5;
const PATH_48K: &str = "core/player/src/rodio/test_data/LRMonoPhase4.mp3";
const PATH_44K: &str = "core/player/src/rodio/test_data/ff-16b-2c-44100hz.mp3";

#[tracing::instrument(level = "debug", skip_all)]
fn capture_raw_decoder_output(decoder: FFMPEGDecoder, sample_rate: u32, channels: u16) -> Vec<f32> {
    let samples_to_capture =
        (sample_rate as u64 * channels as u64 * CAPTURE_DURATION_SECS) as usize;
    decoder.take(samples_to_capture).collect()
}

#[tracing::instrument(level = "debug", skip_all)]
async fn spawn_audio_mock_server(path: &str) -> (MockServer, String) {
    let mock_server = MockServer::start().await;
    let audio_bytes = fs::read(path).expect("Failed to read test audio file");

    Mock::given(method("GET"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_bytes(audio_bytes.clone())
                .insert_header("content-type", "audio/mpeg")
                .insert_header("content-length", audio_bytes.len().to_string().as_str())
                .insert_header("accept-ranges", "bytes"),
        )
        .mount(&mock_server)
        .await;

    let url = format!("{}/audio.mp3", mock_server.uri());
    (mock_server, url)
}

#[tracing::instrument(level = "debug", skip_all)]
fn assert_exact_bytes(expected: &[f32], actual: &[f32], track_name: &str) {
    assert_eq!(
        expected.len(),
        actual.len(),
        "Length mismatch for {}: Expected {} samples, got {}",
        track_name,
        expected.len(),
        actual.len()
    );

    for (i, (&e, &a)) in expected.iter().zip(actual.iter()).enumerate() {
        assert_eq!(
            e.to_bits(),
            a.to_bits(),
            "Exact bytestream mismatch in {} at sample {}!\nExpected: {}\nActual:   {}",
            track_name,
            i,
            e,
            a
        );
    }
}

#[test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_decoder_48000_track_raw() {
    let decoder =
        FFMPEGDecoder::open(PATH_48K, 48000).expect("Failed to open 48k track from Bazel runfiles");
    let output = capture_raw_decoder_output(decoder, 48000, 2);

    assert_eq!(
        output.len(),
        480_000,
        "Decoder failed to yield exactly 480,000 samples"
    );
    assert!(
        output.iter().any(|&s| s != 0.0),
        "Decoded stream is pure silence!"
    );
}

#[test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_decoder_44100_track_raw() {
    let decoder = FFMPEGDecoder::open(PATH_44K, 44100)
        .expect("Failed to open 44.1k track from Bazel runfiles");
    let output = capture_raw_decoder_output(decoder, 44100, 2);

    assert_eq!(
        output.len(),
        441_000,
        "Decoder failed to yield exactly 441,000 samples"
    );
    assert!(
        output.iter().any(|&s| s != 0.0),
        "Decoded stream is pure silence!"
    );
}

#[test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_raw_decoder_succession() {
    let expected_48k =
        capture_raw_decoder_output(FFMPEGDecoder::open(PATH_48K, 48000).unwrap(), 48000, 2);
    let expected_44k =
        capture_raw_decoder_output(FFMPEGDecoder::open(PATH_44K, 44100).unwrap(), 44100, 2);

    let decoder_48k = FFMPEGDecoder::open(PATH_48K, 48000).unwrap().take(480_000);
    let decoder_44k = FFMPEGDecoder::open(PATH_44K, 44100).unwrap().take(441_000);

    let mut succession_output: Vec<f32> = decoder_48k.chain(decoder_44k).collect();

    let succession_44k = succession_output.split_off(expected_48k.len());
    let succession_48k = succession_output;

    assert_exact_bytes(&expected_48k, &succession_48k, "Track 1 (48k)");
    assert_exact_bytes(&expected_44k, &succession_44k, "Track 2 (44.1k)");
}

#[tokio::test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_decoder_http_stream_with_cache() {
    let (_mock_server, url) = spawn_audio_mock_server(PATH_48K).await;

    let decoder = FFMPEGDecoder::open(&url, 48000).expect("Failed to open http stream with cache");
    let output = capture_raw_decoder_output(decoder, 48000, 2);

    assert_eq!(output.len(), 480_000);
}

#[tokio::test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_decoder_http_stream_cache_seeking() {
    let (_mock_server, url) = spawn_audio_mock_server(PATH_48K).await;

    let mut decoder =
        FFMPEGDecoder::open(&url, 48000).expect("Failed to open http stream with cache");

    let first_chunk: Vec<f32> = decoder.by_ref().take(48_000).collect();
    assert_eq!(first_chunk.len(), 48_000);
    assert!(first_chunk.iter().any(|&s| s != 0.0));

    Source::try_seek(&mut decoder, Duration::from_secs(0))
        .expect("Failed to seek back using cache");

    let reseeked_chunk: Vec<f32> = decoder.take(48_000).collect();
    assert_eq!(reseeked_chunk.len(), 48_000);
    assert!(reseeked_chunk.iter().any(|&s| s != 0.0));
}

#[test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_decoder_resampling_44100_to_48000() {
    let decoder =
        FFMPEGDecoder::open(PATH_44K, 48000).expect("Failed to open with 48k output rate");
    let samples: Vec<f32> = decoder.collect();

    assert!(
        !samples.is_empty(),
        "Decoder should produce resampled samples"
    );
    assert!(
        samples.iter().any(|&s| s != 0.0),
        "Resampled samples should contain audio data"
    );
}

#[test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_decoder_local_file_seek() {
    let mut decoder =
        FFMPEGDecoder::open(PATH_48K, 44100).expect("Failed to open local test audio file");

    let initial_chunk: Vec<f32> = decoder.by_ref().take(44_100).collect();
    assert_eq!(initial_chunk.len(), 44_100);

    // Seek forward to 2 seconds
    Source::try_seek(&mut decoder, Duration::from_secs(2)).expect("Failed to seek to 2 seconds");
    let seek_chunk: Vec<f32> = decoder.by_ref().take(44_100).collect();
    assert_eq!(seek_chunk.len(), 44_100);
    assert!(seek_chunk.iter().any(|&s| s != 0.0));

    // Seek back to 0 seconds
    Source::try_seek(&mut decoder, Duration::from_secs(0))
        .expect("Failed to seek back to 0 seconds");
    let start_chunk: Vec<f32> = decoder.take(44_100).collect();
    assert_eq!(start_chunk.len(), 44_100);
    assert!(start_chunk.iter().any(|&s| s != 0.0));
}
