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

use extensions_proto::moosync::types::PlayerState;

use crate::{MprisHolder, MprisPlayerDetails, context::MockMprisContext};

#[test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_mpris_holder_new() {
    let mut mock = Box::new(MockMprisContext::new());
    mock.expect_attach().returning(|_| Ok(()));

    let holder = MprisHolder::new_with_context(mock);
    assert!(holder.is_ok());
}

#[test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_mpris_holder_set_metadata() {
    let mut mock = Box::new(MockMprisContext::new());
    mock.expect_attach().returning(|_| Ok(()));
    mock.expect_set_metadata().times(1).returning(|_| Ok(()));

    let holder = MprisHolder::new_with_context(mock).unwrap();
    let metadata = MprisPlayerDetails {
        title: Some("Title".to_string()),
        album_name: Some("Album".to_string()),
        artist_name: Some("Artist".to_string()),
        thumbnail: Some("http://cover.url".to_string()),
        duration: Some(120.0),
        ..Default::default()
    };

    let res = holder.set_metadata(metadata);
    assert!(res.is_ok());
}

#[test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_mpris_holder_set_playback_state_and_position() {
    let mut mock = Box::new(MockMprisContext::new());
    mock.expect_attach().returning(|_| Ok(()));
    mock.expect_set_playback_state()
        .with(
            mockall::predicate::eq(PlayerState::Playing),
            mockall::predicate::eq(0),
        )
        .times(1)
        .returning(|_, _| Ok(()));
    mock.expect_set_playback_state()
        .with(
            mockall::predicate::eq(PlayerState::Playing),
            mockall::predicate::eq(45000),
        )
        .times(1)
        .returning(|_, _| Ok(()));

    let holder = MprisHolder::new_with_context(mock).unwrap();
    let res = holder.set_playback_state(PlayerState::Playing);
    assert!(res.is_ok());

    let pos_res = holder.set_position(45.0);
    assert!(pos_res.is_ok());
}
