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
use tokio::sync::mpsc::unbounded_channel;

use crate::{
    generic::PlayerExt,
    rodio::{RodioPlayer, get_system_sample_rate},
    source::ValidSrc,
};

const PATH_48K: &str = "core/player/src/rodio/test_data/LRMonoPhase4.mp3";

#[test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_rodio_player_initial_state() {
    let (tx, _rx) = unbounded_channel();
    let player = RodioPlayer::new(tx);

    let state = player.get_player_state().expect("Failed to get state");
    let pos = player.get_current_pos().expect("Failed to get pos");
    let vol = player.get_volume().expect("Failed to get volume");

    assert_eq!(state, PlayerState::Stopped);
    assert_eq!(pos, Duration::ZERO);
    assert_eq!(vol, 100);
}

#[test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_rodio_player_can_play() {
    let (tx, _rx) = unbounded_channel();
    let player = RodioPlayer::new(tx);

    assert!(!player.can_play(ValidSrc::Path("non_existent_audio_file.mp3".into())));
    assert!(player.can_play(ValidSrc::Path(PATH_48K.into())));
    assert!(player.can_play(ValidSrc::Url("http://stream.example.com/audio.mp3".into())));
    assert!(player.can_play(ValidSrc::Url("https://stream.example.com/audio.mp3".into())));
    assert!(!player.can_play(ValidSrc::Url("ftp://stream.example.com/audio.mp3".into())));
}

#[test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_rodio_player_volume_control() {
    let (tx, _rx) = unbounded_channel();
    let player = RodioPlayer::new(tx);

    // Unloaded player accepts volume changes without error
    assert!(player.set_volume(50).is_ok());
    assert_eq!(player.get_volume().unwrap(), 100);

    if player.set_src(ValidSrc::Path(PATH_48K.into())).is_ok() {
        assert!(player.set_volume(50).is_ok());
        assert_eq!(player.get_volume().unwrap(), 50);

        assert!(player.set_volume(0).is_ok());
        assert_eq!(player.get_volume().unwrap(), 0);

        assert!(player.set_volume(100).is_ok());
        assert_eq!(player.get_volume().unwrap(), 100);
    }
}

#[test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_get_system_sample_rate() {
    let rate = get_system_sample_rate();

    assert!(rate >= 8000, "Expected valid sample rate, got {}", rate);
}
