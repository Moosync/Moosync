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

use assertables::{assert_len_eq_x, assert_not_empty, assert_ok};
use rodio::Source;
use rstest::rstest;
use tracing_test::traced_test;
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

#[rstest]
#[case(PATH_48K, 48000, 480_000)]
#[case(PATH_44K, 44100, 441_000)]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_decoder_raw_tracks(
    #[case] path: &str,
    #[case] sample_rate: u32,
    #[case] expected_samples: usize,
) {
    let decoder = FFMPEGDecoder::open(path, sample_rate).expect("Failed to open track");
    let output = capture_raw_decoder_output(decoder, sample_rate, 2);

    assert_len_eq_x!(&output, expected_samples);
    assert!(
        output.iter().any(|&s| s != 0.0),
        "Decoded stream is pure silence!"
    );
}

#[test]
#[traced_test]
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
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_decoder_http_stream_with_cache() {
    let (_mock_server, url) = spawn_audio_mock_server(PATH_48K).await;

    let decoder = FFMPEGDecoder::open(&url, 48000).expect("Failed to open http stream with cache");
    let output = capture_raw_decoder_output(decoder, 48000, 2);

    assert_len_eq_x!(&output, 480_000);
}

#[tokio::test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_decoder_http_stream_cache_seeking() {
    let (_mock_server, url) = spawn_audio_mock_server(PATH_48K).await;
    let mut decoder =
        FFMPEGDecoder::open(&url, 48000).expect("Failed to open http stream with cache");

    let first_chunk: Vec<f32> = decoder.by_ref().take(48_000).collect();
    let seek_res = Source::try_seek(&mut decoder, Duration::from_secs(0));
    let reseeked_chunk: Vec<f32> = decoder.take(48_000).collect();

    assert_len_eq_x!(&first_chunk, 48_000);
    assert!(first_chunk.iter().any(|&s| s != 0.0));
    assert_ok!(seek_res);
    assert_len_eq_x!(&reseeked_chunk, 48_000);
    assert!(reseeked_chunk.iter().any(|&s| s != 0.0));
}

#[test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_decoder_resampling_44100_to_48000() {
    let decoder =
        FFMPEGDecoder::open(PATH_44K, 48000).expect("Failed to open with 48k output rate");

    let samples: Vec<f32> = decoder.collect();

    assert_not_empty!(&samples);
    assert!(
        samples.iter().any(|&s| s != 0.0),
        "Resampled samples should contain audio data"
    );
}

#[test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_decoder_local_file_seek() {
    let mut decoder =
        FFMPEGDecoder::open(PATH_48K, 44100).expect("Failed to open local test audio file");

    let initial_chunk: Vec<f32> = decoder.by_ref().take(44_100).collect();
    let seek_2s_res = Source::try_seek(&mut decoder, Duration::from_secs(2));
    let seek_chunk: Vec<f32> = decoder.by_ref().take(44_100).collect();
    let seek_0s_res = Source::try_seek(&mut decoder, Duration::from_secs(0));
    let start_chunk: Vec<f32> = decoder.take(44_100).collect();

    assert_len_eq_x!(&initial_chunk, 44_100);
    assert_ok!(seek_2s_res);
    assert_len_eq_x!(&seek_chunk, 44_100);
    assert!(seek_chunk.iter().any(|&s| s != 0.0));
    assert_ok!(seek_0s_res);
    assert_len_eq_x!(&start_chunk, 44_100);
    assert!(start_chunk.iter().any(|&s| s != 0.0));
}
