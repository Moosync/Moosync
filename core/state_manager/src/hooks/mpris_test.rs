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
use extensions_proto::moosync::types::{PlayerEvent, player_event::Event};
use player::PlayerHandler;
use rstest::{fixture, rstest};
use songs_proto::moosync::types::{Album, Artist, InnerSong, Song};
use tempdir::TempDir;
use tracing_test::traced_test;
use types::plugin::PluginContext;

use crate::{
    StateManager,
    hooks::{Hook, mpris::MprisHook},
};

struct TestStateManagerContext {
    pub _temp_dir: TempDir,
    pub state_manager: StateManager,
}

#[fixture]
#[tracing::instrument(level = "debug", skip_all)]
fn state_manager_context() -> TestStateManagerContext {
    let temp_dir = TempDir::new("moosync_sm_mpris_hook_test").expect("failed to create temp dir");
    let test_dir = temp_dir.path().to_path_buf();
    let context = PluginContext {
        data_dir: test_dir.clone(),
        cache_dir: test_dir.clone(),
        tmp_dir: test_dir,
        #[cfg(target_os = "android")]
        android_context: types::android::AndroidJNIContext::default(),
    };
    let state_manager =
        StateManager::new_with_context(context).expect("failed to create state manager");
    TestStateManagerContext {
        _temp_dir: temp_dir,
        state_manager,
    }
}

#[rstest]
#[tokio::test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_mpris_hook_on_startup(state_manager_context: TestStateManagerContext) {
    let TestStateManagerContext { state_manager, .. } = state_manager_context;
    let hook = MprisHook::new();

    let startup_result = hook.on_startup(&state_manager).await;

    assert_ok!(startup_result);
}

#[rstest]
#[tokio::test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_mpris_hook_song_changed(state_manager_context: TestStateManagerContext) {
    let TestStateManagerContext { state_manager, .. } = state_manager_context;
    let hook = MprisHook::new();
    hook.on_startup(&state_manager)
        .await
        .expect("on_startup failed");
    let player_handler = state_manager.plugins.get::<PlayerHandler>();
    let song = Song {
        song: Some(InnerSong {
            title: Some("MPRIS Test Title".into()),
            ..Default::default()
        }),
        artists: vec![Artist {
            artist_name: Some("MPRIS Test Artist".into()),
            ..Default::default()
        }],
        album: Some(Album {
            album_name: Some("MPRIS Test Album".into()),
            ..Default::default()
        }),
        ..Default::default()
    };

    player_handler.on_song_changed.emit(Some(song));
}

#[rstest]
#[tokio::test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_mpris_hook_player_events(state_manager_context: TestStateManagerContext) {
    let TestStateManagerContext { state_manager, .. } = state_manager_context;
    let hook = MprisHook::new();
    hook.on_startup(&state_manager)
        .await
        .expect("on_startup failed");
    let player_handler = state_manager.plugins.get::<PlayerHandler>();

    player_handler.on_player_event.emit(PlayerEvent {
        event: Some(Event::Play(true)),
    });
    player_handler.on_player_event.emit(PlayerEvent {
        event: Some(Event::Pause(true)),
    });
    player_handler.on_player_event.emit(PlayerEvent {
        event: Some(Event::TimeUpdate(Default::default())),
    });
}
