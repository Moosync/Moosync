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
use player::PlayerHandler;
use preferences::keys::CLEAR_QUEUE;
use preferences_proto::moosync::types::{PreferenceItem, PreferenceValue, preference_value};
use rstest::{fixture, rstest};
use songs_proto::moosync::types::{InnerSong, Song, SongType};
use tempdir::TempDir;
use tracing_test::traced_test;
use types::plugin::PluginContext;

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
