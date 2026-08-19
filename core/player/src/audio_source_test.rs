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

use extensions_proto::moosync::types::PlayerState;
use songs_proto::moosync::types::{InnerSong, Song};

use crate::audio_source::AudioSource;

#[tokio::test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_audio_source_load_and_methods() {
    let mut audio_src = AudioSource::new(Box::new(|| {}));

    let song = Song {
        song: Some(InnerSong {
            playback_url: Some("https://example.com/audio.mp3".to_string()),
            ..Default::default()
        }),
        ..Default::default()
    };

    let set_res = audio_src.set_src(song);
    let vol_res = audio_src.set_volume(85);
    let seek_res = audio_src.seek(Duration::from_secs(12));
    let pause_res = audio_src.pause();
    let stop_res = audio_src.stop();

    assert!(set_res.is_err());
    assert!(vol_res.is_ok());
    assert!(seek_res.is_ok());
    assert!(pause_res.is_ok());
    assert!(stop_res.is_ok());
    assert_eq!(audio_src.get_player_state(), PlayerState::Stopped);
}

#[tokio::test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_audio_source_two_pass_resolver() {
    let mut audio_src = AudioSource::new(Box::new(|| {}));

    audio_src.set_resolver(Box::new(|_s| {
        Ok("https://resolved.example.com/stream.mp3".to_string())
    }));

    let song = Song {
        song: Some(InnerSong {
            id: Some("stream_song".to_string()),
            ..Default::default()
        }),
        ..Default::default()
    };

    let res = audio_src.load_song(song);

    assert!(res.is_err());
    assert_eq!(audio_src.get_player_state(), PlayerState::Stopped);
}
