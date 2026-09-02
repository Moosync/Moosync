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

use assertables::{assert_ok, assert_ok_eq_x};
use extensions_proto::moosync::types::PlayerState;
use rstest::{fixture, rstest};
use tokio::sync::mpsc::unbounded_channel;
use tracing_test::traced_test;

use crate::{
    generic::PlayerExt,
    rodio::{RodioPlayer, get_system_sample_rate},
    source::ValidSrc,
};

const PATH_48K: &str = "core/player/src/rodio/test_data/LRMonoPhase4.mp3";

#[fixture]
#[tracing::instrument(level = "debug", skip_all)]
fn rodio_player() -> RodioPlayer {
    let (tx, _rx) = unbounded_channel();
    RodioPlayer::new(tx)
}

#[rstest]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_rodio_player_initial_state(rodio_player: RodioPlayer) {
    let state = rodio_player.get_player_state();
    let pos = rodio_player.get_current_pos();
    let vol = rodio_player.get_volume();

    assert_ok_eq_x!(state, PlayerState::Stopped);
    assert_ok_eq_x!(pos, Duration::ZERO);
    assert_ok_eq_x!(vol, 100);
}

#[rstest]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_rodio_player_can_play(rodio_player: RodioPlayer) {
    let can_play_non_existent =
        rodio_player.can_play(ValidSrc::Path("non_existent_audio_file.mp3".into()));
    let can_play_48k = rodio_player.can_play(ValidSrc::Path(PATH_48K.into()));
    let can_play_http =
        rodio_player.can_play(ValidSrc::Url("http://stream.example.com/audio.mp3".into()));
    let can_play_https =
        rodio_player.can_play(ValidSrc::Url("https://stream.example.com/audio.mp3".into()));
    let can_play_ftp =
        rodio_player.can_play(ValidSrc::Url("ftp://stream.example.com/audio.mp3".into()));

    assert!(!can_play_non_existent);
    assert!(can_play_48k);
    assert!(can_play_http);
    assert!(can_play_https);
    assert!(!can_play_ftp);
}

#[rstest]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_rodio_player_volume_control(rodio_player: RodioPlayer) {
    let set_vol_unloaded = rodio_player.set_volume(50);
    let vol_unloaded = rodio_player.get_volume();

    assert_ok!(set_vol_unloaded);
    assert_ok_eq_x!(vol_unloaded, 100);

    let src_res = rodio_player.set_src(ValidSrc::Path(PATH_48K.into()));
    if src_res.is_ok() {
        assert_ok!(rodio_player.set_volume(50));
        assert_ok_eq_x!(rodio_player.get_volume(), 50);

        assert_ok!(rodio_player.set_volume(0));
        assert_ok_eq_x!(rodio_player.get_volume(), 0);

        assert_ok!(rodio_player.set_volume(100));
        assert_ok_eq_x!(rodio_player.get_volume(), 100);
    }
}

#[test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_get_system_sample_rate() {
    let rate = get_system_sample_rate();

    assert!(rate >= 8000, "Expected valid sample rate, got {}", rate);
}
