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

use std::{
    sync::{
        Arc, RwLock,
        atomic::{AtomicU8, Ordering},
    },
    time::Duration,
};

use assertables::assert_ok;
use extensions_proto::moosync::types::{PlayerState, player_event::Event as PlayerEvent};
use player::PlayerHandler;
use preferences::keys::CLEAR_QUEUE;
use preferences_proto::moosync::types::{PreferenceItem, PreferenceValue, preference_value};
use rstest::{fixture, rstest};
use songs_proto::moosync::types::{InnerSong, Song, SongType};
use tempdir::TempDir;
use tokio::sync::mpsc::UnboundedSender;
use tracing_test::traced_test;
use types::{
    player::{AudioPlayerContext, PlayerError, ValidSrc},
    plugin::PluginContext,
};

use crate::{
    StateManager,
    hooks::{Hook, player::PlayerHook},
};

struct TestSmContext {
    pub _temp_dir: TempDir,
    pub sm: StateManager,
}

#[fixture]
#[tracing::instrument(level = "debug", skip_all)]
fn sm_context() -> TestSmContext {
    let temp_dir = TempDir::new("moosync_sm_player_hook_test").expect("failed to create temp dir");
    let test_dir = temp_dir.path().to_path_buf();
    let context = PluginContext {
        data_dir: test_dir.clone(),
        cache_dir: test_dir.clone(),
        tmp_dir: test_dir.clone(),
        #[cfg(target_os = "android")]
        android_context: types::android::AndroidJNIContext::default(),
    };
    let sm = StateManager::new_with_context(context).expect("failed to create state manager");
    TestSmContext {
        _temp_dir: temp_dir,
        sm,
    }
}

#[rstest]
#[tokio::test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_player_hook_on_startup(sm_context: TestSmContext) {
    let TestSmContext { sm, .. } = sm_context;
    let hook = PlayerHook::new();

    assert_ok!(hook.on_startup(&sm).await);

    let player_handler = sm.plugins.get::<PlayerHandler>();
    let test_song = Song {
        song: Some(InnerSong {
            id: Some("hook_test_song".to_string()),
            title: Some("Resolver Test".to_string()),
            path: Some("/test/resolver.mp3".to_string()),
            r#type: SongType::Local.into(),
            ..Default::default()
        }),
        ..Default::default()
    };
    player_handler.play_now(vec![test_song]).await;
    player_handler.set_volume(55).await;
    player_handler.seek(Duration::from_secs(12)).await;
    let _ = player_handler.pause().await;
    let current_song = player_handler.get_current_song();
    assert!(current_song.is_some());
    assert_eq!(
        current_song.unwrap().song.as_ref().unwrap().id.as_deref(),
        Some("hook_test_song")
    );
}

#[rstest]
#[tokio::test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_player_hook_on_exit_clear_queue_enabled(sm_context: TestSmContext) {
    let TestSmContext { sm, .. } = sm_context;
    let hook = PlayerHook::new();
    let preference_config = sm.get_preference_config();
    let player_handler = sm.plugins.get::<PlayerHandler>();

    preference_config
        .save(PreferenceItem {
            id: CLEAR_QUEUE.id.clone(),
            value: Some(PreferenceValue {
                value: Some(preference_value::Value::BoolValue(true)),
            }),
            ..CLEAR_QUEUE.clone()
        })
        .unwrap();

    let test_song_1 = Song {
        song: Some(InnerSong {
            id: Some("song_1".to_string()),
            title: Some("Title 1".to_string()),
            path: Some("/test/1.mp3".to_string()),
            r#type: SongType::Local.into(),
            ..Default::default()
        }),
        ..Default::default()
    };
    let test_song_2 = Song {
        song: Some(InnerSong {
            id: Some("song_2".to_string()),
            title: Some("Title 2".to_string()),
            path: Some("/test/2.mp3".to_string()),
            r#type: SongType::Local.into(),
            ..Default::default()
        }),
        ..Default::default()
    };
    player_handler
        .add_to_queue(vec![test_song_1, test_song_2])
        .await;
    assert_eq!(player_handler.get_queue().len(), 2);

    assert_ok!(hook.on_exit(&sm).await);

    assert!(player_handler.get_queue().is_empty());
}

#[rstest]
#[tokio::test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_player_hook_on_exit_clear_queue_disabled(sm_context: TestSmContext) {
    let TestSmContext { sm, .. } = sm_context;
    let hook = PlayerHook::new();
    let preference_config = sm.get_preference_config();
    let player_handler = sm.plugins.get::<PlayerHandler>();

    preference_config
        .save(PreferenceItem {
            id: CLEAR_QUEUE.id.clone(),
            value: Some(PreferenceValue {
                value: Some(preference_value::Value::BoolValue(false)),
            }),
            ..CLEAR_QUEUE.clone()
        })
        .unwrap();

    let test_song = Song {
        song: Some(InnerSong {
            id: Some("song_1".to_string()),
            title: Some("Title 1".to_string()),
            path: Some("/test/1.mp3".to_string()),
            r#type: SongType::Local.into(),
            ..Default::default()
        }),
        ..Default::default()
    };
    player_handler.add_to_queue(vec![test_song]).await;
    assert_eq!(player_handler.get_queue().len(), 1);

    assert_ok!(hook.on_exit(&sm).await);

    assert_eq!(player_handler.get_queue().len(), 1);
}

struct CustomSchemePlayer {
    volume: AtomicU8,
    state: RwLock<PlayerState>,
}

#[async_trait::async_trait]
impl AudioPlayerContext for CustomSchemePlayer {
    #[tracing::instrument(level = "debug", skip_all)]
    fn get_id(&self) -> &str { "extension:custom_ext" }

    #[tracing::instrument(level = "debug", skip_all)]
    async fn play(&self) -> Result<(), PlayerError> {
        *self.state.write().unwrap() = PlayerState::Playing;
        Ok(())
    }

    #[tracing::instrument(level = "debug", skip_all)]
    async fn pause(&self) -> Result<(), PlayerError> {
        *self.state.write().unwrap() = PlayerState::Paused;
        Ok(())
    }

    #[tracing::instrument(level = "debug", skip_all)]
    async fn stop(&self) -> Result<(), PlayerError> {
        *self.state.write().unwrap() = PlayerState::Stopped;
        Ok(())
    }

    #[tracing::instrument(level = "debug", skip_all)]
    async fn set_volume(&self, volume: u8) -> Result<(), PlayerError> {
        self.volume.store(volume, Ordering::SeqCst);
        Ok(())
    }

    #[tracing::instrument(level = "debug", skip_all)]
    fn get_volume(&self) -> Result<u8, PlayerError> { Ok(self.volume.load(Ordering::SeqCst)) }

    #[tracing::instrument(level = "debug", skip_all)]
    async fn seek(&self, _pos: Duration) -> Result<(), PlayerError> { Ok(()) }

    #[tracing::instrument(level = "debug", skip_all)]
    fn get_current_pos(&self) -> Result<Duration, PlayerError> { Ok(Duration::ZERO) }

    #[tracing::instrument(level = "debug", skip_all)]
    fn get_player_state(&self) -> Result<PlayerState, PlayerError> {
        Ok(*self.state.read().unwrap())
    }

    #[tracing::instrument(level = "debug", skip_all)]
    async fn can_play(&self, src: &ValidSrc<'_>) -> bool {
        src.inner().starts_with("custom_scheme://")
    }

    #[tracing::instrument(level = "debug", skip_all)]
    async fn set_src<'a>(
        &self,
        _src: ValidSrc<'a>,
        _events_tx: UnboundedSender<PlayerEvent>,
    ) -> Result<(), PlayerError> {
        *self.state.write().unwrap() = PlayerState::Playing;
        Ok(())
    }
}

#[rstest]
#[tokio::test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_player_hook_dynamic_extension_player(sm_context: TestSmContext) {
    let TestSmContext { sm, .. } = sm_context;
    let hook = PlayerHook::new();
    assert_ok!(hook.on_startup(&sm).await);

    let player_handler = sm.plugins.get::<PlayerHandler>();

    let custom_song = Song {
        song: Some(InnerSong {
            id: Some("custom_stream_song".to_string()),
            title: Some("Custom Stream".to_string()),
            playback_url: Some("custom_scheme://track/123".to_string()),
            ..Default::default()
        }),
        ..Default::default()
    };

    let player = Arc::new(CustomSchemePlayer {
        volume: AtomicU8::new(100),
        state: RwLock::new(PlayerState::Stopped),
    });

    player_handler.add_context(player);
    assert!(
        player_handler
            .get_player_ids()
            .contains(&"extension:custom_ext".to_string())
    );

    player_handler.play_now(vec![custom_song]).await;
    let current = player_handler.get_current_song();
    assert!(current.is_some());
    assert_eq!(
        current.unwrap().song.unwrap().id,
        Some("custom_stream_song".to_string())
    );

    player_handler.remove_context("extension:custom_ext");
    assert!(
        !player_handler
            .get_player_ids()
            .contains(&"extension:custom_ext".to_string())
    );
}
