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

use assertables::assert_ok;
use extensions_proto::moosync::types::PlayerState;
use rstest::{fixture, rstest};
use tracing_test::traced_test;
use types::plugin::{Plugin, PluginContext};

use crate::{
    MediaControlEvent, MprisHolder, MprisPlayerDetails, context::MockMprisContext,
    error::MprisError,
};

#[fixture]
#[tracing::instrument(level = "debug", skip_all)]
fn mock_mpris_context() -> Box<MockMprisContext> {
    let mut mock = Box::new(MockMprisContext::new());
    mock.expect_attach().returning(|_| Ok(()));
    mock
}

#[rstest]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_mpris_holder_new(mock_mpris_context: Box<MockMprisContext>) {
    let holder = MprisHolder::new_with_context(mock_mpris_context);

    assert!(holder.is_ok());
}

#[rstest]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_mpris_holder_set_metadata(mut mock_mpris_context: Box<MockMprisContext>) {
    mock_mpris_context
        .expect_set_metadata()
        .times(1)
        .returning(|_| Ok(()));
    let holder = MprisHolder::new_with_context(mock_mpris_context).unwrap();
    let metadata = MprisPlayerDetails {
        title: Some("Title".to_string()),
        album_name: Some("Album".to_string()),
        artist_name: Some("Artist".to_string()),
        thumbnail: Some("http://cover.url".to_string()),
        duration: Some(120.0),
        ..Default::default()
    };

    let result = holder.set_metadata(metadata);

    assert_ok!(result);
}

#[rstest]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_mpris_holder_set_playback_state(mut mock_mpris_context: Box<MockMprisContext>) {
    mock_mpris_context
        .expect_set_playback_state()
        .with(
            mockall::predicate::eq(PlayerState::Playing),
            mockall::predicate::eq(0),
        )
        .times(1)
        .returning(|_, _| Ok(()));
    let holder = MprisHolder::new_with_context(mock_mpris_context).unwrap();

    let result = holder.set_playback_state(PlayerState::Playing);

    assert_ok!(result);
}

#[rstest]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_mpris_holder_set_position(mut mock_mpris_context: Box<MockMprisContext>) {
    mock_mpris_context
        .expect_set_playback_state()
        .with(
            mockall::predicate::eq(PlayerState::Stopped),
            mockall::predicate::eq(45000),
        )
        .times(1)
        .returning(|_, _| Ok(()));
    let holder = MprisHolder::new_with_context(mock_mpris_context).unwrap();

    let result = holder.set_position(45.0);

    assert_ok!(result);
}

#[rstest]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_mpris_holder_event_channel() {
    let mut mock = Box::new(MockMprisContext::new());
    mock.expect_attach().returning(|tx| {
        tx.send(MediaControlEvent::Play).unwrap();
        tx.send(MediaControlEvent::Next).unwrap();
        tx.send(MediaControlEvent::Pause).unwrap();
        Ok(())
    });

    let holder = MprisHolder::new_with_context(mock).unwrap();
    let rx = holder.event_rx.lock().unwrap();

    assert_eq!(rx.recv().unwrap(), MediaControlEvent::Play);
    assert_eq!(rx.recv().unwrap(), MediaControlEvent::Next);
    assert_eq!(rx.recv().unwrap(), MediaControlEvent::Pause);
}

#[rstest]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_mpris_holder_attach_error() {
    let mut mock = Box::new(MockMprisContext::new());
    mock.expect_attach()
        .returning(|_| Err(MprisError::AttachFailed("cannot attach".to_string())));

    let res = MprisHolder::new_with_context(mock);
    assert!(res.is_err());
}

#[rstest]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_mpris_holder_context_errors() {
    let mut mock = Box::new(MockMprisContext::new());
    mock.expect_attach().returning(|_| Ok(()));
    mock.expect_set_metadata()
        .returning(|_| Err(MprisError::SetMetadataFailed("failed".to_string())));
    mock.expect_set_playback_state()
        .returning(|_, _| Err(MprisError::SetPlaybackFailed("failed".to_string())));

    let holder = MprisHolder::new_with_context(mock).unwrap();
    assert!(holder.set_metadata(MprisPlayerDetails::default()).is_err());
    assert!(holder.set_playback_state(PlayerState::Playing).is_err());
}

#[test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_mpris_holder_new_and_plugin_init() {
    let holder_res = MprisHolder::new();
    assert!(holder_res.is_ok());

    let ctx = PluginContext {
        data_dir: std::env::temp_dir().join("moosync_test_data"),
        cache_dir: std::env::temp_dir().join("moosync_test_cache"),
        tmp_dir: std::env::temp_dir().join("moosync_test_tmp"),
    };
    let plugin = <MprisHolder as Plugin>::init(&ctx);
    assert_ok!(plugin.set_playback_state(PlayerState::Paused));
}
