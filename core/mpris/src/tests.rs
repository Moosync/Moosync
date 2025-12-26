use super::MprisHolder;
use types::mpris::MprisPlayerDetails;
use types::ui::player_details::PlayerState;

#[test]
fn test_mpris_holder_new() {
    let holder = MprisHolder::new();
    assert!(holder.is_ok());
}

#[test]
fn test_set_metadata() {
    let holder = MprisHolder::new().unwrap();
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
    let holder = MprisHolder::new().unwrap();
    let res = holder.set_playback_state(PlayerState::Playing);
    assert!(res.is_ok());
}
