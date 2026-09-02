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

use super::{AudioPlayerContext, DummyAudioPlayerContext};
use crate::source::ValidSrc;

#[test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_dummy_audio_player_context_operations() {
    let context = DummyAudioPlayerContext::new();
    let (events_tx, _events_rx) = unbounded_channel();

    assert_eq!(context.get_player_state().unwrap(), PlayerState::Stopped);
    assert_eq!(context.get_volume().unwrap(), 100);

    context.set_volume(80).unwrap();
    assert_eq!(context.get_volume().unwrap(), 80);

    let src = ValidSrc::Url("https://example.com/test.mp3".into());
    context.set_src(src, events_tx).unwrap();
    assert_eq!(context.get_player_state().unwrap(), PlayerState::Playing);

    context.pause().unwrap();
    assert_eq!(context.get_player_state().unwrap(), PlayerState::Paused);

    context.play().unwrap();
    assert_eq!(context.get_player_state().unwrap(), PlayerState::Playing);

    context.seek(Duration::from_secs(45)).unwrap();
    assert_eq!(context.get_current_pos().unwrap(), Duration::from_secs(45));

    context.stop().unwrap();
    assert_eq!(context.get_player_state().unwrap(), PlayerState::Stopped);
    assert_eq!(context.get_current_pos().unwrap(), Duration::default());
}
