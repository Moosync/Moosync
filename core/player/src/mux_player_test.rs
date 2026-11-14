use std::{error::Error, sync::Arc, time::Duration};

use assertables::{assert_err, assert_ok};
use async_trait::async_trait;
use extensions_proto::moosync::types::{PlaybackDetailsRequestedResponse, PlayerState};
use rstest::{fixture, rstest};
use songs_proto::moosync::types::{InnerSong, Song};
use tracing_test::traced_test;
use types::subscription::SubscriberList;

use crate::{
    context::DummyAudioPlayerContext,
    mux_player::MuxPlayer,
    rodio::{get_cpal_default_sample_rate, get_system_sample_rate},
    source::{SourceResolver, ValidSrc},
};

#[fixture]
#[tracing::instrument(level = "debug", skip_all)]
fn mux_player() -> MuxPlayer {
    MuxPlayer::new_with_contexts(
        SubscriberList::new(),
        vec![Arc::new(DummyAudioPlayerContext::new())],
    )
}

#[rstest]
#[tokio::test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_mux_player_initial_state(mux_player: MuxPlayer) {
    let state = mux_player.get_player_state();

    assert_eq!(state, PlayerState::Stopped);
}

#[rstest]
#[tokio::test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_mux_player_can_play_unsupported_url(mux_player: MuxPlayer) {
    let can_play_ftp = mux_player
        .can_play(ValidSrc::new_url("ftp://example.com"))
        .await;

    assert!(!can_play_ftp);
}

#[rstest]
#[tokio::test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_mux_player_load_unsupported_url(mux_player: MuxPlayer) {
    let song = Song {
        song: Some(InnerSong {
            playback_url: Some("ftp://example.com/audio.mp3".to_string()),
            ..Default::default()
        }),
        ..Default::default()
    };

    let result = mux_player.load(&song).await;

    assert_err!(result.as_ref());
}

#[rstest]
#[tokio::test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_mux_player_pause(mux_player: MuxPlayer) {
    let result = mux_player.pause().await;

    assert_ok!(result);
}

#[rstest]
#[tokio::test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_mux_player_stop(mux_player: MuxPlayer) {
    let result = mux_player.stop().await;

    assert_ok!(result);
}

#[rstest]
#[tokio::test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_mux_player_set_volume(mux_player: MuxPlayer) {
    let result = mux_player.set_volume(90).await;

    assert_ok!(result);
}

#[rstest]
#[tokio::test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_mux_player_seek(mux_player: MuxPlayer) {
    let result = mux_player.seek(Duration::from_secs(5)).await;

    assert_ok!(result);
}

#[rstest]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_get_system_sample_rate() {
    let rate = get_system_sample_rate();
    assert!(rate >= 8000, "Expected valid sample rate, got {}", rate);

    let cpal_rate = get_cpal_default_sample_rate();
    assert!(
        cpal_rate >= 8000,
        "Expected valid cpal rate, got {}",
        cpal_rate
    );
}

#[derive(Default)]
struct RejectAudioPlayer;

#[async_trait]
impl crate::context::AudioPlayerContext for RejectAudioPlayer {
    #[tracing::instrument(level = "debug", skip_all)]
    fn get_id(&self) -> &str { "reject" }
    #[tracing::instrument(level = "debug", skip_all)]
    async fn can_play(&self, _src: &ValidSrc<'_>) -> bool { false }
    #[tracing::instrument(level = "debug", skip_all)]
    async fn set_src<'a>(
        &self,
        _src: ValidSrc<'a>,
        _events_tx: tokio::sync::mpsc::UnboundedSender<
            extensions_proto::moosync::types::player_event::Event,
        >,
    ) -> Result<(), crate::error::PlayerError> {
        Err(crate::error::PlayerError::NoPlayerFound(Box::default()))
    }
    #[tracing::instrument(level = "debug", skip_all)]
    async fn play(&self) -> Result<(), crate::error::PlayerError> { Ok(()) }
    #[tracing::instrument(level = "debug", skip_all)]
    async fn pause(&self) -> Result<(), crate::error::PlayerError> { Ok(()) }
    #[tracing::instrument(level = "debug", skip_all)]
    async fn stop(&self) -> Result<(), crate::error::PlayerError> { Ok(()) }
    #[tracing::instrument(level = "debug", skip_all)]
    async fn set_volume(&self, _vol: u8) -> Result<(), crate::error::PlayerError> { Ok(()) }
    #[tracing::instrument(level = "debug", skip_all)]
    async fn seek(&self, _pos: Duration) -> Result<(), crate::error::PlayerError> { Ok(()) }
    #[tracing::instrument(level = "debug", skip_all)]
    fn get_volume(&self) -> Result<u8, crate::error::PlayerError> { Ok(100) }
    #[tracing::instrument(level = "debug", skip_all)]
    fn get_current_pos(&self) -> Result<Duration, crate::error::PlayerError> {
        Ok(Duration::from_secs(0))
    }
    #[tracing::instrument(level = "debug", skip_all)]
    fn get_player_state(&self) -> Result<PlayerState, crate::error::PlayerError> {
        Ok(PlayerState::Stopped)
    }
}

#[rstest]
#[tokio::test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_mux_player_no_player_found() {
    let mux =
        MuxPlayer::new_with_contexts(SubscriberList::new(), vec![Arc::new(RejectAudioPlayer)]);

    let song = Song {
        song: Some(InnerSong {
            playback_url: Some("https://example.com/audio.mp3".to_string()),
            ..Default::default()
        }),
        ..Default::default()
    };

    let result = mux.load(&song).await;

    assert_err!(result.as_ref());
}

#[rstest]
#[tokio::test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_mux_player_play_success(mux_player: MuxPlayer) {
    let song = Song {
        song: Some(InnerSong {
            playback_url: Some("https://example.com/audio.mp3".to_string()),
            ..Default::default()
        }),
        ..Default::default()
    };

    assert_ok!(mux_player.load(&song).await);
    let play_res = mux_player.play().await;
    assert_ok!(play_res);
    assert_eq!(mux_player.get_player_state(), PlayerState::Playing);
}

#[rstest]
#[tokio::test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_mux_player_load_song_resolution() {
    let mux = MuxPlayer::new_with_contexts(
        SubscriberList::new(),
        vec![Arc::new(DummyAudioPlayerContext::new())],
    );

    struct MockMuxResolver;

    #[async_trait]
    impl SourceResolver for MockMuxResolver {
        #[tracing::instrument(level = "debug", skip_all)]
        async fn resolve(
            &self,
            _song: &Song,
        ) -> Result<PlaybackDetailsRequestedResponse, Box<dyn Error + Send + Sync>> {
            Ok(PlaybackDetailsRequestedResponse {
                url: "https://resolved.example.com/stream.mp3".to_string(),
                duration: None,
                decryption_key: None,
            })
        }

        #[tracing::instrument(level = "debug", skip_all)]
        async fn resolve_fallback(
            &self,
            _song: &Song,
        ) -> Result<PlaybackDetailsRequestedResponse, Box<dyn Error + Send + Sync>> {
            Ok(PlaybackDetailsRequestedResponse {
                url: "https://resolved.example.com/stream.mp3".to_string(),
                duration: None,
                decryption_key: None,
            })
        }
    }

    mux.set_resolver(Arc::new(MockMuxResolver));

    let stream_song = Song {
        song: Some(InnerSong {
            id: Some("ext:456".to_string()),
            ..Default::default()
        }),
        ..Default::default()
    };

    let res = mux.load_song(&stream_song).await;
    assert_ok!(res);
}

#[rstest]
#[tokio::test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_mux_player_fallback_switching() {
    let mux = MuxPlayer::new_with_contexts(
        SubscriberList::new(),
        vec![
            Arc::new(RejectAudioPlayer),
            Arc::new(DummyAudioPlayerContext::new()),
        ],
    );

    let valid_src = ValidSrc::new_url("https://example.com/stream.mp3");
    let load_res = mux.load_src(valid_src).await;

    assert_ok!(load_res);
}

struct FailingSetSrcPlayer;

#[async_trait]
impl crate::context::AudioPlayerContext for FailingSetSrcPlayer {
    #[tracing::instrument(level = "debug", skip_all)]
    fn get_id(&self) -> &str { "failing" }
    #[tracing::instrument(level = "debug", skip_all)]
    async fn can_play(&self, _src: &ValidSrc<'_>) -> bool { true }
    #[tracing::instrument(level = "debug", skip_all)]
    async fn set_src<'a>(
        &self,
        _src: ValidSrc<'a>,
        _events_tx: tokio::sync::mpsc::UnboundedSender<
            extensions_proto::moosync::types::player_event::Event,
        >,
    ) -> Result<(), crate::error::PlayerError> {
        Err(crate::error::PlayerError::NoPlayerFound(Box::default()))
    }
    #[tracing::instrument(level = "debug", skip_all)]
    async fn play(&self) -> Result<(), crate::error::PlayerError> { Ok(()) }
    #[tracing::instrument(level = "debug", skip_all)]
    async fn pause(&self) -> Result<(), crate::error::PlayerError> { Ok(()) }
    #[tracing::instrument(level = "debug", skip_all)]
    async fn stop(&self) -> Result<(), crate::error::PlayerError> { Ok(()) }
    #[tracing::instrument(level = "debug", skip_all)]
    async fn set_volume(&self, _vol: u8) -> Result<(), crate::error::PlayerError> { Ok(()) }
    #[tracing::instrument(level = "debug", skip_all)]
    async fn seek(&self, _pos: Duration) -> Result<(), crate::error::PlayerError> { Ok(()) }
    #[tracing::instrument(level = "debug", skip_all)]
    fn get_volume(&self) -> Result<u8, crate::error::PlayerError> { Ok(100) }
    #[tracing::instrument(level = "debug", skip_all)]
    fn get_current_pos(&self) -> Result<Duration, crate::error::PlayerError> {
        Ok(Duration::from_secs(0))
    }
    #[tracing::instrument(level = "debug", skip_all)]
    fn get_player_state(&self) -> Result<PlayerState, crate::error::PlayerError> {
        Ok(PlayerState::Stopped)
    }
}

#[rstest]
#[tokio::test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_mux_player_active_player_failure_fallback() {
    let mux = MuxPlayer::new_with_contexts(
        SubscriberList::new(),
        vec![
            Arc::new(FailingSetSrcPlayer),
            Arc::new(DummyAudioPlayerContext::new()),
        ],
    );

    let valid_src = ValidSrc::new_url("https://example.com/stream.mp3");
    let load_res = mux.load_src(valid_src).await;
    assert_ok!(load_res);
}

#[rstest]
#[tokio::test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_mux_player_all_players_failing() {
    let mux = MuxPlayer::new_with_contexts(
        SubscriberList::new(),
        vec![Arc::new(FailingSetSrcPlayer), Arc::new(RejectAudioPlayer)],
    );

    let valid_src = ValidSrc::new_url("https://example.com/stream.mp3");
    let load_res = mux.load_src(valid_src).await;
    assert_err!(load_res);

    let dummy_song = Song::default();
    let song_load_res = mux.load_song(&dummy_song).await;
    assert_err!(song_load_res);
}

#[rstest]
#[tokio::test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_mux_player_ended_event_propagation() {
    use std::sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    };

    let ended_flag = Arc::new(AtomicBool::new(false));
    let ended_flag_clone = ended_flag.clone();
    let on_ended_list = SubscriberList::new();
    on_ended_list.listen(move |()| {
        ended_flag_clone.store(true, Ordering::SeqCst);
    });

    let mux = MuxPlayer::new_with_contexts(
        on_ended_list,
        vec![Arc::new(DummyAudioPlayerContext::new())],
    );

    let valid_src = ValidSrc::new_url("https://example.com/stream.mp3");
    assert_ok!(mux.load_src(valid_src).await);

    let events_tx = mux.create_new_events_tx();
    let _ = events_tx.send(extensions_proto::moosync::types::player_event::Event::Ended(true));

    tokio::time::sleep(Duration::from_millis(50)).await;
    assert!(ended_flag.load(Ordering::SeqCst));
}

#[rstest]
#[tokio::test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_mux_player_dynamic_registration() {
    let mux =
        MuxPlayer::new_with_contexts(SubscriberList::new(), vec![Arc::new(RejectAudioPlayer)]);

    let valid_src = ValidSrc::new_url("https://example.com/stream.mp3");
    assert_err!(mux.load_src(valid_src.clone()).await);

    // Dynamically add dummy player
    mux.add_player(Arc::new(DummyAudioPlayerContext::new()));
    assert_ok!(mux.load_src(valid_src.clone()).await);

    // Dynamically remove dummy player
    mux.remove_player("dummy");
    assert_err!(mux.load_src(valid_src).await);
}

struct AdvancedMockResolver {
    ext_a_url: Option<String>,
    fallback_url: Option<String>,
}

#[async_trait]
impl SourceResolver for AdvancedMockResolver {
    #[tracing::instrument(level = "debug", skip_all)]
    async fn resolve(
        &self,
        song: &Song,
    ) -> Result<PlaybackDetailsRequestedResponse, Box<dyn Error + Send + Sync>> {
        let ext = song.song.as_ref().and_then(|s| s.extension.as_deref());
        if ext == Some("ext_a")
            && let Some(ref url) = self.ext_a_url
        {
            return Ok(PlaybackDetailsRequestedResponse {
                url: url.clone(),
                duration: None,
                decryption_key: None,
            });
        }
        Err(Box::new(crate::error::PlayerError::NoSrcFound(Box::new(
            song.clone(),
        ))))
    }

    #[tracing::instrument(level = "debug", skip_all)]
    async fn resolve_fallback(
        &self,
        _song: &Song,
    ) -> Result<PlaybackDetailsRequestedResponse, Box<dyn Error + Send + Sync>> {
        if let Some(ref url) = self.fallback_url {
            return Ok(PlaybackDetailsRequestedResponse {
                url: url.clone(),
                duration: None,
                decryption_key: None,
            });
        }
        Err(Box::new(crate::error::PlayerError::NoSrcFound(Box::new(
            _song.clone(),
        ))))
    }
}

struct MockExtensionPlayer {
    id: String,
    prefix: String,
}

#[async_trait]
impl crate::context::AudioPlayerContext for MockExtensionPlayer {
    #[tracing::instrument(level = "debug", skip_all)]
    fn get_id(&self) -> &str { &self.id }

    #[tracing::instrument(level = "debug", skip_all)]
    async fn can_play(&self, src: &ValidSrc<'_>) -> bool { src.inner().starts_with(&self.prefix) }

    #[tracing::instrument(level = "debug", skip_all)]
    async fn set_src<'a>(
        &self,
        _src: ValidSrc<'a>,
        _events_tx: tokio::sync::mpsc::UnboundedSender<
            extensions_proto::moosync::types::player_event::Event,
        >,
    ) -> Result<(), crate::error::PlayerError> {
        Ok(())
    }

    #[tracing::instrument(level = "debug", skip_all)]
    async fn play(&self) -> Result<(), crate::error::PlayerError> { Ok(()) }

    #[tracing::instrument(level = "debug", skip_all)]
    async fn pause(&self) -> Result<(), crate::error::PlayerError> { Ok(()) }

    #[tracing::instrument(level = "debug", skip_all)]
    async fn stop(&self) -> Result<(), crate::error::PlayerError> { Ok(()) }

    #[tracing::instrument(level = "debug", skip_all)]
    async fn set_volume(&self, _vol: u8) -> Result<(), crate::error::PlayerError> { Ok(()) }

    #[tracing::instrument(level = "debug", skip_all)]
    async fn seek(&self, _pos: Duration) -> Result<(), crate::error::PlayerError> { Ok(()) }

    #[tracing::instrument(level = "debug", skip_all)]
    fn get_volume(&self) -> Result<u8, crate::error::PlayerError> { Ok(100) }

    #[tracing::instrument(level = "debug", skip_all)]
    fn get_current_pos(&self) -> Result<Duration, crate::error::PlayerError> { Ok(Duration::ZERO) }

    #[tracing::instrument(level = "debug", skip_all)]
    fn get_player_state(&self) -> Result<PlayerState, crate::error::PlayerError> {
        Ok(PlayerState::Stopped)
    }
}

#[rstest]
#[tokio::test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_url_resolution_rule_1_existing_playback_url() {
    let mux = MuxPlayer::new_with_contexts(
        SubscriberList::new(),
        vec![Arc::new(DummyAudioPlayerContext::new())],
    );
    let resolver = Arc::new(AdvancedMockResolver {
        ext_a_url: None,
        fallback_url: None,
    });
    mux.set_resolver(resolver);

    let song = Song {
        song: Some(InnerSong {
            playback_url: Some("https://example.com/rule1.mp3".to_string()),
            extension: Some("ext_a".to_string()),
            ..Default::default()
        }),
        ..Default::default()
    };

    let result = mux.load_song(&song).await;

    assert_ok!(result);
}

#[rstest]
#[tokio::test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_url_resolution_rule_2_same_extension_resolves_url() {
    let mux = MuxPlayer::new_with_contexts(
        SubscriberList::new(),
        vec![Arc::new(DummyAudioPlayerContext::new())],
    );
    let resolver = Arc::new(AdvancedMockResolver {
        ext_a_url: Some("https://example.com/resolved_by_a.mp3".to_string()),
        fallback_url: None,
    });
    mux.set_resolver(resolver);

    let song = Song {
        song: Some(InnerSong {
            playback_url: None,
            extension: Some("ext_a".to_string()),
            ..Default::default()
        }),
        ..Default::default()
    };

    let result = mux.load_song(&song).await;

    assert_ok!(result);
}

#[rstest]
#[tokio::test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_url_resolution_rule_3_same_extension_player_provider() {
    let custom_player = Arc::new(MockExtensionPlayer {
        id: "extension:ext_player".to_string(),
        prefix: "custom_scheme://".to_string(),
    });
    let mux = MuxPlayer::new_with_contexts(
        SubscriberList::new(),
        vec![Arc::new(RejectAudioPlayer), custom_player],
    );
    let resolver = Arc::new(AdvancedMockResolver {
        ext_a_url: None,
        fallback_url: None,
    });
    mux.set_resolver(resolver);

    let song = Song {
        song: Some(InnerSong {
            playback_url: None,
            url: Some("custom_scheme://track/789".to_string()),
            extension: Some("ext_player".to_string()),
            ..Default::default()
        }),
        ..Default::default()
    };

    let result = mux.load_song(&song).await;

    assert_ok!(result);
}

#[rstest]
#[tokio::test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_url_resolution_rule_4_fallback_url_resolution() {
    let mux = MuxPlayer::new_with_contexts(
        SubscriberList::new(),
        vec![Arc::new(DummyAudioPlayerContext::new())],
    );
    let resolver = Arc::new(AdvancedMockResolver {
        ext_a_url: None,
        fallback_url: Some("https://example.com/fallback_resolved.mp3".to_string()),
    });
    mux.set_resolver(resolver);

    let song = Song {
        song: Some(InnerSong {
            playback_url: None,
            extension: Some("ext_unsupported".to_string()),
            ..Default::default()
        }),
        ..Default::default()
    };

    let result = mux.load_song(&song).await;

    assert_ok!(result);
}

#[rstest]
#[tokio::test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_url_resolution_rule_4_fallback_player_provider() {
    let fallback_player = Arc::new(MockExtensionPlayer {
        id: "extension:ext_other".to_string(),
        prefix: "fallback_player://".to_string(),
    });
    let mux = MuxPlayer::new_with_contexts(
        SubscriberList::new(),
        vec![Arc::new(RejectAudioPlayer), fallback_player],
    );
    let resolver = Arc::new(AdvancedMockResolver {
        ext_a_url: None,
        fallback_url: None,
    });
    mux.set_resolver(resolver);

    let song = Song {
        song: Some(InnerSong {
            playback_url: None,
            url: Some("fallback_player://track/999".to_string()),
            extension: Some("ext_unsupported".to_string()),
            ..Default::default()
        }),
        ..Default::default()
    };

    let result = mux.load_song(&song).await;

    assert_ok!(result);
}
