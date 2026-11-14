use std::path::PathBuf;

use crate::player::*;

#[test]
fn test_valid_src_url() {
    let src = ValidSrc::new_url("https://example.com/audio.mp3");
    assert_eq!(src.inner(), "https://example.com/audio.mp3");
    assert_eq!(src.decryption_key(), None);
    assert_eq!(src.to_string(), "https://example.com/audio.mp3");

    let owned = src.into_owned();
    assert_eq!(owned.inner(), "https://example.com/audio.mp3");
}

#[test]
fn test_valid_src_url_with_key() {
    let src = ValidSrc::new_url_with_key("https://example.com/stream", Some("secret_key"));
    assert_eq!(src.inner(), "https://example.com/stream");
    assert_eq!(src.decryption_key().as_deref(), Some("secret_key"));

    let owned = src.into_owned();
    assert_eq!(owned.inner(), "https://example.com/stream");
    assert_eq!(owned.decryption_key().as_deref(), Some("secret_key"));
}

#[test]
fn test_valid_src_path() {
    let path = PathBuf::from("/music/song.flac");
    let src = ValidSrc::Path(path.clone());
    assert_eq!(src.inner(), "/music/song.flac");
    assert_eq!(src.decryption_key(), None);

    let owned = src.into_owned();
    assert_eq!(owned, ValidSrc::Path(path));
}

#[test]
fn test_player_error_display() {
    let err = PlayerError::PlaybackUrlResolutionFailed("not found".to_string());
    assert!(err.to_string().contains("not found"));

    let custom_err = PlayerError::Custom("something went wrong".to_string());
    assert!(custom_err.to_string().contains("something went wrong"));
}
