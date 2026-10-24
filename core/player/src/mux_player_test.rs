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
        vec![Box::new(DummyAudioPlayerContext::new())],
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
    let can_play_ftp = mux_player.can_play(ValidSrc::new_url("ftp://example.com"));

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

    let result = mux_player.load(&song);

    assert_err!(result.as_ref());
}

#[rstest]
#[tokio::test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_mux_player_pause(mux_player: MuxPlayer) {
    let result = mux_player.pause();

    assert_ok!(result);
}

#[rstest]
#[tokio::test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_mux_player_stop(mux_player: MuxPlayer) {
    let result = mux_player.stop();

    assert_ok!(result);
}

#[rstest]
#[tokio::test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_mux_player_set_volume(mux_player: MuxPlayer) {
    let result = mux_player.set_volume(90);

    assert_ok!(result);
}

#[rstest]
#[tokio::test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_mux_player_seek(mux_player: MuxPlayer) {
    let result = mux_player.seek(Duration::from_secs(5));

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

impl crate::context::AudioPlayerContext for RejectAudioPlayer {
    #[tracing::instrument(level = "debug", skip_all)]
    fn can_play(&self, _src: &ValidSrc) -> bool { false }
    #[tracing::instrument(level = "debug", skip_all)]
    fn set_src(
        &self,
        _src: ValidSrc,
        _events_tx: tokio::sync::mpsc::UnboundedSender<
            extensions_proto::moosync::types::player_event::Event,
        >,
    ) -> Result<(), crate::error::PlayerError> {
        Err(crate::error::PlayerError::NoPlayerFound(Box::default()))
    }
    #[tracing::instrument(level = "debug", skip_all)]
    fn play(&self) -> Result<(), crate::error::PlayerError> { Ok(()) }
    #[tracing::instrument(level = "debug", skip_all)]
    fn pause(&self) -> Result<(), crate::error::PlayerError> { Ok(()) }
    #[tracing::instrument(level = "debug", skip_all)]
    fn stop(&self) -> Result<(), crate::error::PlayerError> { Ok(()) }
    #[tracing::instrument(level = "debug", skip_all)]
    fn set_volume(&self, _vol: u8) -> Result<(), crate::error::PlayerError> { Ok(()) }
    #[tracing::instrument(level = "debug", skip_all)]
    fn seek(&self, _pos: Duration) -> Result<(), crate::error::PlayerError> { Ok(()) }
    #[tracing::instrument(level = "debug", skip_all)]
    fn get_volume(&self) -> Result<u8, crate::error::PlayerError> { Ok(100) }
    #[tracing::instrument(level = "debug", skip_all)]
    fn get_current_pos(&self) -> Result<Duration, crate::error::PlayerError> {
        Ok(Duration::from_secs(12))
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
async fn test_mux_player_playback_and_metrics(mux_player: MuxPlayer) {
    let play_res = mux_player.play();
    assert_ok!(play_res);

    let vol = mux_player.get_volume();
    assert_eq!(vol, 100);

    let pos_res = mux_player.get_current_pos();
    assert_ok!(pos_res);

    let valid_song = Song {
        song: Some(InnerSong {
            playback_url: Some("https://example.com/audio.mp3".to_string()),
            ..Default::default()
        }),
        ..Default::default()
    };
    let load_res = mux_player.set_src(&valid_song).await;
    assert_ok!(load_res);
}

#[rstest]
#[tokio::test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_mux_player_source_resolver() {
    let mux = MuxPlayer::new_with_contexts(
        SubscriberList::new(),
        vec![Box::new(DummyAudioPlayerContext::new())],
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
            Box::new(RejectAudioPlayer),
            Box::new(DummyAudioPlayerContext::new()),
        ],
    );

    let valid_src = ValidSrc::new_url("https://example.com/stream.mp3");
    let load_res = mux.load_src(valid_src);

    assert_ok!(load_res);
}

struct FailingSetSrcPlayer;

impl crate::context::AudioPlayerContext for FailingSetSrcPlayer {
    #[tracing::instrument(level = "debug", skip_all)]
    fn can_play(&self, _src: &ValidSrc) -> bool { true }
    #[tracing::instrument(level = "debug", skip_all)]
    fn set_src(
        &self,
        _src: ValidSrc,
        _events_tx: tokio::sync::mpsc::UnboundedSender<
            extensions_proto::moosync::types::player_event::Event,
        >,
    ) -> Result<(), crate::error::PlayerError> {
        Err(crate::error::PlayerError::NoPlayerFound(Box::default()))
    }
    #[tracing::instrument(level = "debug", skip_all)]
    fn play(&self) -> Result<(), crate::error::PlayerError> { Ok(()) }
    #[tracing::instrument(level = "debug", skip_all)]
    fn pause(&self) -> Result<(), crate::error::PlayerError> { Ok(()) }
    #[tracing::instrument(level = "debug", skip_all)]
    fn stop(&self) -> Result<(), crate::error::PlayerError> { Ok(()) }
    #[tracing::instrument(level = "debug", skip_all)]
    fn set_volume(&self, _vol: u8) -> Result<(), crate::error::PlayerError> { Ok(()) }
    #[tracing::instrument(level = "debug", skip_all)]
    fn seek(&self, _pos: Duration) -> Result<(), crate::error::PlayerError> { Ok(()) }
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
            Box::new(FailingSetSrcPlayer),
            Box::new(DummyAudioPlayerContext::new()),
        ],
    );

    let valid_src = ValidSrc::new_url("https://example.com/stream.mp3");
    let load_res = mux.load_src(valid_src);
    assert_ok!(load_res);
}

#[rstest]
#[tokio::test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_mux_player_all_players_failing() {
    let mux = MuxPlayer::new_with_contexts(
        SubscriberList::new(),
        vec![Box::new(FailingSetSrcPlayer), Box::new(RejectAudioPlayer)],
    );

    let valid_src = ValidSrc::new_url("https://example.com/stream.mp3");
    let load_res = mux.load_src(valid_src);
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
        vec![Box::new(DummyAudioPlayerContext::new())],
    );

    let valid_src = ValidSrc::new_url("https://example.com/stream.mp3");
    assert_ok!(mux.load_src(valid_src));

    let events_tx = mux.create_new_events_tx();
    let _ = events_tx.send(extensions_proto::moosync::types::player_event::Event::Ended(true));

    tokio::time::sleep(Duration::from_millis(50)).await;
    assert!(ended_flag.load(Ordering::SeqCst));
}
