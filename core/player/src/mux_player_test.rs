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
fn test_mux_player_initial_state_and_can_play(mux_player: MuxPlayer) {
    let state = mux_player.get_player_state();
    let can_play_ftp = mux_player.can_play(ValidSrc::Url(Cow::Borrowed("ftp://example.com")));

    assert_eq!(state, PlayerState::Stopped);
    assert!(!can_play_ftp);
}

#[rstest]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_mux_player_load_unsupported_url_and_controls(mut mux_player: MuxPlayer) {
    let song = Song {
        song: Some(InnerSong {
            playback_url: Some("ftp://example.com/audio.mp3".to_string()),
            ..Default::default()
        }),
        ..Default::default()
    };

    assert_err!(mux_player.load(&song).as_ref());
    assert_ok!(mux_player.pause());
    assert_ok!(mux_player.stop());
    assert_ok!(mux_player.set_volume(90));
    assert_ok!(mux_player.seek(Duration::from_secs(5)));
    assert_eq!(mux_player.get_player_state(), PlayerState::Stopped);
}
