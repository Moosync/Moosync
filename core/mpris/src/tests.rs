use super::{MprisHolder, context::MockMprisContext};
use types::mpris::MprisPlayerDetails;
use types::ui::player_details::PlayerState;

#[test]
fn test_mpris_holder_new() {
    // This tests the real constructor (with real context), might fail if no dbus
    // But since we installed dbus-dev and run in environment, it might pass or fail depending on session bus.
    // However, we should focus on testing logic via mock.
    let mut mock = Box::new(MockMprisContext::new());
    mock.expect_attach().returning(|_| Ok(()));

    let holder = MprisHolder::new_with_context(mock);
    assert!(holder.is_ok());
}

#[test]
fn test_set_metadata() {
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
fn test_set_playback_state() {
    let mut mock = Box::new(MockMprisContext::new());
    mock.expect_attach().returning(|_| Ok(()));
    mock.expect_set_playback_state()
        .times(1)
        .returning(|_, _| Ok(()));

    let holder = MprisHolder::new_with_context(mock).unwrap();
    let res = holder.set_playback_state(PlayerState::Playing);
    assert!(res.is_ok());
}
