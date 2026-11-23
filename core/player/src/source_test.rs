use std::{error::Error, fs, path::PathBuf, sync::Arc};

use assertables::{assert_err, assert_matches, assert_ok};
use async_trait::async_trait;
use extensions_proto::moosync::types::PlaybackDetailsRequestedResponse;
use rstest::{fixture, rstest};
use songs_proto::moosync::types::{InnerSong, PlaybackUrl, Song};
use tempdir::TempDir;
use tracing_test::traced_test;

use crate::{
    error::PlayerError,
    source::{SourceResolver, SourceResolverHolder, ValidSrc, get_valid_src},
};

#[fixture]
#[tracing::instrument(level = "debug", skip_all)]
fn temp_dir_fixture() -> TempDir {
    TempDir::new("moosync_src_test").expect("failed to create temp dir")
}

#[test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_valid_src_inner_and_display() {
    let path_src = ValidSrc::Path(PathBuf::from("/music/song.mp3"));
    let url_src = ValidSrc::new_url("https://stream.org/audio.mp3");
    let key_src = ValidSrc::new_url_with_key("https://stream.org/audio.mp3", Some("mykey"));

    assert_eq!(path_src.inner(), "/music/song.mp3");
    assert_eq!(format!("{}", path_src), "/music/song.mp3");
    assert_eq!(url_src.inner(), "https://stream.org/audio.mp3");
    assert_eq!(format!("{}", url_src), "https://stream.org/audio.mp3");
    assert_eq!(url_src.decryption_key(), None);
    assert_eq!(key_src.decryption_key().as_deref(), Some("mykey"));
}

#[rstest]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_get_valid_src_file_exists(temp_dir_fixture: TempDir) {
    let test_file = temp_dir_fixture.path().join("song.mp3");
    fs::write(&test_file, b"audio").unwrap();
    let song = Song {
        song: Some(InnerSong {
            path: Some(test_file.to_string_lossy().to_string()),
            ..Default::default()
        }),
        ..Default::default()
    };

    let valid_src = get_valid_src(&song);

    assert_ok!(valid_src.as_ref());
    assert_matches!(valid_src.unwrap(), ValidSrc::Path(p) if p == test_file);
}

#[test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_get_valid_src_playback_url_when_file_absent() {
    let song = Song {
        song: Some(InnerSong {
            path: Some("/non_existent_file.mp3".to_string()),
            playback_url: Some(songs_proto::moosync::types::PlaybackUrl {
                url: "https://example.com/audio".to_string(),
                ..Default::default()
            }),
            ..Default::default()
        }),
        ..Default::default()
    };

    let valid_src = get_valid_src(&song);

    assert_ok!(valid_src.as_ref());
    assert_matches!(valid_src.unwrap(), ValidSrc::Url { url, .. } if url == "https://example.com/audio");
}

#[test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_get_valid_src_error_when_none_present() {
    let song = Song {
        song: Some(InnerSong {
            path: None,
            playback_url: None,
            ..Default::default()
        }),
        ..Default::default()
    };

    let valid_src = get_valid_src(&song);

    assert_err!(valid_src.as_ref());
    assert_matches!(valid_src.unwrap_err(), PlayerError::NoSrcFound(_));
}

struct MockResolver;

#[async_trait]
impl SourceResolver for MockResolver {
    #[tracing::instrument(level = "debug", skip_all)]
    async fn resolve(
        &self,
        _song: &Song,
        _is_prefetch: bool,
    ) -> Result<PlaybackDetailsRequestedResponse, Box<dyn Error + Send + Sync>> {
        Ok(PlaybackDetailsRequestedResponse {
            url: "https://resolved.stream/123".to_string(),
            duration: None,
            decryption_key: Some("0123456789abcdef0123456789abcdef".to_string()),
            ..Default::default()
        })
    }

    #[tracing::instrument(level = "debug", skip_all)]
    async fn resolve_fallback(
        &self,
        _song: &Song,
        _is_prefetch: bool,
    ) -> Result<PlaybackDetailsRequestedResponse, Box<dyn Error + Send + Sync>> {
        Ok(PlaybackDetailsRequestedResponse {
            url: "https://resolved.stream/fallback".to_string(),
            duration: None,
            decryption_key: None,
            ..Default::default()
        })
    }
}

#[tokio::test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_source_resolver_resolve() {
    let resolver = SourceResolverHolder::new();
    let song = Song {
        song: Some(InnerSong {
            id: Some("123".to_string()),
            ..Default::default()
        }),
        ..Default::default()
    };

    assert_err!(resolver.resolve(&song, false).await.as_ref());
    assert_err!(resolver.resolve_fallback(&song, false).await.as_ref());

    resolver.set_resolver(Arc::new(MockResolver));
    let res = resolver.resolve(&song, false).await;
    assert_ok!(res.as_ref());
    let resp = res.unwrap();
    assert_eq!(resp.url, "https://resolved.stream/123");
    assert_eq!(
        resp.decryption_key.as_deref(),
        Some("0123456789abcdef0123456789abcdef")
    );

    let res_fallback = resolver.resolve_fallback(&song, false).await;
    assert_ok!(res_fallback.as_ref());
    let resp_fallback = res_fallback.unwrap();
    assert_eq!(resp_fallback.url, "https://resolved.stream/fallback");
}

#[test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_get_valid_src_playback_url_expiration() {
    let now_secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;

    let mut valid_playback_url = PlaybackUrl {
        url: "https://stream.example/valid.mp3".to_string(),
        expires_at: Some(Default::default()),
        decryption_key: None,
    };
    if let Some(ts) = valid_playback_url.expires_at.as_mut() {
        ts.seconds = now_secs + 3600;
    }

    let valid_song = Song {
        song: Some(InnerSong {
            id: Some("valid_ttl".to_string()),
            playback_url: Some(valid_playback_url),
            ..Default::default()
        }),
        ..Default::default()
    };

    let valid = get_valid_src(&valid_song);
    assert_ok!(valid.as_ref());
    assert_eq!(
        valid.unwrap(),
        ValidSrc::new_url("https://stream.example/valid.mp3")
    );

    let mut expired_playback_url = PlaybackUrl {
        url: "https://stream.example/expired.mp3".to_string(),
        expires_at: Some(Default::default()),
        decryption_key: None,
    };
    if let Some(ts) = expired_playback_url.expires_at.as_mut() {
        ts.seconds = now_secs - 60;
    }

    let expired_song = Song {
        song: Some(InnerSong {
            id: Some("expired_ttl".to_string()),
            playback_url: Some(expired_playback_url),
            ..Default::default()
        }),
        ..Default::default()
    };

    let expired = get_valid_src(&expired_song);
    assert_err!(expired.as_ref());
    assert_matches!(expired.unwrap_err(), PlayerError::NoSrcFound(_));
}
