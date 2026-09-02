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
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    time::Duration,
};

use assertables::{assert_err, assert_ok};
use extensions_proto::moosync::types::PlayerState;
use rstest::{fixture, rstest};
use songs_proto::moosync::types::{InnerSong, Song};
use tracing_test::traced_test;

use crate::audio_source::AudioSource;

#[fixture]
#[tracing::instrument(level = "debug", skip_all)]
fn audio_source() -> AudioSource { AudioSource::new(Box::new(|| {})) }

#[rstest]
#[tokio::test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_audio_source_load_and_methods(mut audio_source: AudioSource) {
    let mut song = Song {
        song: Some(InnerSong {
            playback_url: Some("https://example.com/audio.mp3".to_string()),
            ..Default::default()
        }),
        ..Default::default()
    };

    assert_err!(audio_source.set_src(&mut song).as_ref());
    assert_ok!(audio_source.set_volume(85));
    assert_ok!(audio_source.seek(Duration::from_secs(12)));
    assert_ok!(audio_source.pause());
    assert_ok!(audio_source.stop());
    assert_eq!(audio_source.get_player_state(), PlayerState::Stopped);
}

#[rstest]
#[tokio::test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_audio_source_two_pass_resolver(mut audio_source: AudioSource) {
    let resolver_called = Arc::new(AtomicBool::new(false));
    let flag = resolver_called.clone();
    audio_source.set_resolver(Box::new(move |_s| {
        flag.store(true, Ordering::SeqCst);
        Ok("https://resolved.example.com/stream.mp3".to_string())
    }));
    let mut song = Song {
        song: Some(InnerSong {
            id: Some("stream_song".to_string()),
            ..Default::default()
        }),
        ..Default::default()
    };

    assert_err!(audio_source.load_song(&mut song).as_ref());
    assert!(resolver_called.load(Ordering::SeqCst));
    assert_eq!(audio_source.get_player_state(), PlayerState::Stopped);
}
