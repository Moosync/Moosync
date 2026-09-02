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

use std::{sync::mpsc, time::Duration};

use assertables::assert_ok;
use extensions_proto::moosync::types::PlayerState;
use rstest::rstest;
use tracing_test::traced_test;

use crate::{
    MediaControlEvent, MediaPosition, MprisPlayerDetails, SeekDirection,
    context::{DummyContext, MprisContext, from_souvlaki_event},
};

#[rstest]
#[case(souvlaki::MediaControlEvent::Play, MediaControlEvent::Play)]
#[case(souvlaki::MediaControlEvent::Pause, MediaControlEvent::Pause)]
#[case(souvlaki::MediaControlEvent::Toggle, MediaControlEvent::Toggle)]
#[case(souvlaki::MediaControlEvent::Next, MediaControlEvent::Next)]
#[case(souvlaki::MediaControlEvent::Previous, MediaControlEvent::Previous)]
#[case(souvlaki::MediaControlEvent::Stop, MediaControlEvent::Stop)]
#[case(
    souvlaki::MediaControlEvent::Seek(souvlaki::SeekDirection::Forward),
    MediaControlEvent::Seek(SeekDirection::Forward)
)]
#[case(
    souvlaki::MediaControlEvent::SeekBy(
        souvlaki::SeekDirection::Backward,
        Duration::from_secs(10)
    ),
    MediaControlEvent::SeekBy(SeekDirection::Backward, Duration::from_secs(10))
)]
#[case(
    souvlaki::MediaControlEvent::SetPosition(souvlaki::MediaPosition(Duration::from_millis(
        5000
    ))),
    MediaControlEvent::SetPosition(MediaPosition(Duration::from_millis(5000)))
)]
#[case(
    souvlaki::MediaControlEvent::SetVolume(0.75),
    MediaControlEvent::SetVolume(0.75)
)]
#[case(
    souvlaki::MediaControlEvent::OpenUri("file:///music.mp3".to_string()),
    MediaControlEvent::OpenUri("file:///music.mp3".to_string())
)]
#[case(souvlaki::MediaControlEvent::Raise, MediaControlEvent::Raise)]
#[case(souvlaki::MediaControlEvent::Quit, MediaControlEvent::Quit)]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_from_souvlaki_event_mapping(
    #[case] souvlaki_event: souvlaki::MediaControlEvent,
    #[case] expected: MediaControlEvent,
) {
    let mapped = from_souvlaki_event(souvlaki_event);

    assert_eq!(mapped, expected);
}

#[test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_dummy_context_operations() {
    let mut dummy = DummyContext {};
    let (tx, _rx) = mpsc::channel();

    assert_ok!(dummy.attach(tx));
    assert_ok!(dummy.set_metadata(MprisPlayerDetails::default()));
    assert_ok!(dummy.set_playback_state(PlayerState::Playing, 12000));
}
