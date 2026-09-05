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

use std::{borrow::Cow, time::Duration};

use assertables::{assert_err, assert_ok};
use extensions_proto::moosync::types::PlayerState;
use rstest::{fixture, rstest};
use songs_proto::moosync::types::{InnerSong, Song};
use tokio::sync::mpsc::unbounded_channel;
use tracing_test::traced_test;

use crate::{mux_player::MuxPlayer, source::ValidSrc};

#[fixture]
#[tracing::instrument(level = "debug", skip_all)]
fn mux_player() -> MuxPlayer {
    let (tx, _rx) = unbounded_channel();
    MuxPlayer::new(tx)
}

#[rstest]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_mux_player_initial_state(mux_player: MuxPlayer) {
    let state = mux_player.get_player_state();

    assert_eq!(state, PlayerState::Stopped);
}

#[rstest]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_mux_player_can_play_unsupported_url(mux_player: MuxPlayer) {
    let can_play_ftp = mux_player.can_play(ValidSrc::Url(Cow::Borrowed("ftp://example.com")));

    assert!(!can_play_ftp);
}

#[rstest]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_mux_player_load_unsupported_url(mut mux_player: MuxPlayer) {
    let song = Song {
        song: Some(InnerSong {
            playback_url: Some("ftp://example.com/audio.mp3".to_string()),
            ..Default::default()
        }),
        ..Default::default()
    };

    let result = mux_player.load(&song);

    assert_err!(result.as_ref());
}

#[rstest]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_mux_player_pause(mux_player: MuxPlayer) {
    let result = mux_player.pause();

    assert_ok!(result);
}

#[rstest]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_mux_player_stop(mux_player: MuxPlayer) {
    let result = mux_player.stop();

    assert_ok!(result);
}

#[rstest]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_mux_player_set_volume(mux_player: MuxPlayer) {
    let result = mux_player.set_volume(90);

    assert_ok!(result);
}

#[rstest]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_mux_player_seek(mux_player: MuxPlayer) {
    let result = mux_player.seek(Duration::from_secs(5));

    assert_ok!(result);
}
