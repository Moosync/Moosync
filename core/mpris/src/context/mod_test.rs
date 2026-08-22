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

use extensions_proto::moosync::types::PlayerState;

use crate::{
    MediaControlEvent, MediaPosition, MprisPlayerDetails, SeekDirection,
    context::{DummyContext, MprisContext, from_souvlaki_event},
};

#[test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_from_souvlaki_event_mapping() {
    assert_eq!(
        from_souvlaki_event(souvlaki::MediaControlEvent::Play),
        MediaControlEvent::Play
    );
    assert_eq!(
        from_souvlaki_event(souvlaki::MediaControlEvent::Pause),
        MediaControlEvent::Pause
    );
    assert_eq!(
        from_souvlaki_event(souvlaki::MediaControlEvent::Toggle),
        MediaControlEvent::Toggle
    );
    assert_eq!(
        from_souvlaki_event(souvlaki::MediaControlEvent::Next),
        MediaControlEvent::Next
    );
    assert_eq!(
        from_souvlaki_event(souvlaki::MediaControlEvent::Previous),
        MediaControlEvent::Previous
    );
    assert_eq!(
        from_souvlaki_event(souvlaki::MediaControlEvent::Stop),
        MediaControlEvent::Stop
    );
    assert_eq!(
        from_souvlaki_event(souvlaki::MediaControlEvent::Seek(
            souvlaki::SeekDirection::Forward
        )),
        MediaControlEvent::Seek(SeekDirection::Forward)
    );
    assert_eq!(
        from_souvlaki_event(souvlaki::MediaControlEvent::SeekBy(
            souvlaki::SeekDirection::Backward,
            Duration::from_secs(10)
        )),
        MediaControlEvent::SeekBy(SeekDirection::Backward, Duration::from_secs(10))
    );
    assert_eq!(
        from_souvlaki_event(souvlaki::MediaControlEvent::SetPosition(
            souvlaki::MediaPosition(Duration::from_millis(5000))
        )),
        MediaControlEvent::SetPosition(MediaPosition(Duration::from_millis(5000)))
    );
    assert_eq!(
        from_souvlaki_event(souvlaki::MediaControlEvent::SetVolume(0.75)),
        MediaControlEvent::SetVolume(0.75)
    );
    assert_eq!(
        from_souvlaki_event(souvlaki::MediaControlEvent::OpenUri(
            "file:///music.mp3".to_string()
        )),
        MediaControlEvent::OpenUri("file:///music.mp3".to_string())
    );
    assert_eq!(
        from_souvlaki_event(souvlaki::MediaControlEvent::Raise),
        MediaControlEvent::Raise
    );
    assert_eq!(
        from_souvlaki_event(souvlaki::MediaControlEvent::Quit),
        MediaControlEvent::Quit
    );
}

#[test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_dummy_context_operations() {
    let mut dummy = DummyContext {};
    let (tx, _rx) = mpsc::channel();
    assert!(dummy.attach(tx).is_ok());
    assert!(dummy.set_metadata(MprisPlayerDetails::default()).is_ok());
    assert!(
        dummy
            .set_playback_state(PlayerState::Playing, 12000)
            .is_ok()
    );
}
