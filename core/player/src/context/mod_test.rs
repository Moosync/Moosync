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

use assertables::assert_ok_eq_x;
use extensions_proto::moosync::types::PlayerState;
use rstest::{fixture, rstest};
use tokio::sync::mpsc::unbounded_channel;
use tracing_test::traced_test;

use super::{AudioPlayerContext, DummyAudioPlayerContext};
use crate::source::ValidSrc;

#[fixture]
#[tracing::instrument(level = "debug", skip_all)]
fn dummy_player_context() -> DummyAudioPlayerContext { DummyAudioPlayerContext::new() }

#[rstest]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_dummy_audio_player_context_operations(dummy_player_context: DummyAudioPlayerContext) {
    let (events_tx, _events_rx) = unbounded_channel();

    assert_ok_eq_x!(
        dummy_player_context.get_player_state(),
        PlayerState::Stopped
    );
    assert_ok_eq_x!(dummy_player_context.get_volume(), 100);

    dummy_player_context.set_volume(80).unwrap();
    assert_ok_eq_x!(dummy_player_context.get_volume(), 80);

    let src = ValidSrc::Url("https://example.com/test.mp3".into());
    dummy_player_context.set_src(src, events_tx).unwrap();
    assert_ok_eq_x!(
        dummy_player_context.get_player_state(),
        PlayerState::Playing
    );

    dummy_player_context.pause().unwrap();
    assert_ok_eq_x!(dummy_player_context.get_player_state(), PlayerState::Paused);

    dummy_player_context.play().unwrap();
    assert_ok_eq_x!(
        dummy_player_context.get_player_state(),
        PlayerState::Playing
    );

    dummy_player_context.seek(Duration::from_secs(45)).unwrap();
    assert_ok_eq_x!(
        dummy_player_context.get_current_pos(),
        Duration::from_secs(45)
    );

    dummy_player_context.stop().unwrap();
    assert_ok_eq_x!(
        dummy_player_context.get_player_state(),
        PlayerState::Stopped
    );
    assert_ok_eq_x!(dummy_player_context.get_current_pos(), Duration::default());
}
