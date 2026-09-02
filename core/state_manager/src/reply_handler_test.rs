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

use assertables::{assert_is_empty, assert_len_eq_x, assert_none, assert_ok, assert_some_eq_x};
use extensions::ReplyHandler;
use rstest::{fixture, rstest};
use songs_proto::moosync::types::{
    GetEntityOptions, GetSongOptions, InnerSong, Playlist, SearchableSong, Song,
};
use tempdir::TempDir;
use tracing_test::traced_test;
use types::{plugin::PluginContext, prelude::SongsExt};

use crate::{StateManager, reply_handler::StateReplyHandler};

struct TestReplyContext {
    pub _temp_dir: TempDir,
    pub handler: StateReplyHandler,
}

#[fixture]
#[tracing::instrument(level = "debug", skip_all)]
fn reply_context() -> TestReplyContext {
    let temp_dir = TempDir::new("moosync_sm_reply_test").expect("failed to create temp dir");
    let test_dir = temp_dir.path().to_path_buf();

    let context = PluginContext {
        data_dir: test_dir.clone(),
        cache_dir: test_dir.clone(),
        tmp_dir: test_dir.clone(),
        #[cfg(target_os = "android")]
        android_context: types::android::AndroidJNIContext::default(),
    };

    let sm = StateManager::new_with_context(context).expect("failed to create state manager");
    let handler = StateReplyHandler::new(sm);
    TestReplyContext {
        _temp_dir: temp_dir,
        handler,
    }
}

#[rstest]
#[tokio::test(flavor = "multi_thread")]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_reply_handler_version_and_player_queries(reply_context: TestReplyContext) {
    let TestReplyContext { handler, .. } = reply_context;

    let res = tokio::task::spawn_blocking(move || {
        let version = handler.get_app_version("pkg").unwrap();
        let player_state = handler.get_player_state("pkg").unwrap();
        let vol = handler.get_volume("pkg").unwrap();
        let time = handler.get_time("pkg").unwrap();
        let cur_song = handler.get_current_song("pkg").unwrap();
        let queue = handler.get_queue("pkg").unwrap();

        assert!(!version.is_empty());
        assert_eq!(
            player_state,
            extensions_proto::moosync::types::PlayerState::Stopped as i32
        );
        assert_eq!(vol, 100.0);
        assert_eq!(time, 0.0);
        assert_none!(cur_song);
        assert_is_empty!(&queue.0);
    })
    .await;

    assert_ok!(res);
}

#[rstest]
#[tokio::test(flavor = "multi_thread")]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_reply_handler_preferences_and_secure_storage(reply_context: TestReplyContext) {
    let TestReplyContext { handler, .. } = reply_context;

    let res = tokio::task::spawn_blocking(move || {
        let pref_val = extensions_proto::struct_proto::google::protobuf::Value {
            kind: Some(
                extensions_proto::struct_proto::google::protobuf::value::Kind::StringValue(
                    "test_val".to_string(),
                ),
            ),
        };
        assert_ok!(handler.set_preference("pkg", "my_key", pref_val.clone()));
        let loaded_pref = handler.get_preference("pkg", "my_key").unwrap();
        assert_eq!(loaded_pref.unwrap().kind, pref_val.kind);

        let sec_val = extensions_proto::struct_proto::google::protobuf::Value {
            kind: Some(
                extensions_proto::struct_proto::google::protobuf::value::Kind::StringValue(
                    "secret_123".to_string(),
                ),
            ),
        };
        assert_ok!(handler.set_secure("pkg", "sec_key", sec_val.clone()));
        let loaded_sec = handler.get_secure("pkg", "sec_key").unwrap();
        assert_eq!(loaded_sec.unwrap().kind, sec_val.kind);
    })
    .await;

    assert_ok!(res);
}

#[rstest]
#[tokio::test(flavor = "multi_thread")]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_reply_handler_songs_crud(reply_context: TestReplyContext) {
    let TestReplyContext { handler, .. } = reply_context;

    let res = tokio::task::spawn_blocking(move || {
        let song = Song {
            song: Some(InnerSong {
                id: Some("rep_song_1".to_string()),
                title: Some("Reply Song".to_string()),
                path: Some("/music/rep.mp3".to_string()),
                duration: Some(songs_proto::duration_proto::google::protobuf::Duration {
                    seconds: 120,
                    nanos: 0,
                }),
                r#type: songs_proto::moosync::types::SongType::Local as i32,
                ..Default::default()
            }),
            ..Default::default()
        };

        let inserted = handler.add_songs("pkg", vec![song.clone()]).unwrap();
        assert_len_eq_x!(&inserted, 1);

        let songs = handler
            .get_song(
                "pkg",
                GetSongOptions {
                    song: Some(SearchableSong {
                        id: Some("rep_song_1".to_string()),
                        ..Default::default()
                    }),
                    ..Default::default()
                },
            )
            .unwrap();
        assert_len_eq_x!(&songs, 1);
        assert_some_eq_x!(songs[0].get_id(), "rep_song_1");

        let mut updated = song.clone();
        updated.song.as_mut().unwrap().title = Some("Updated Title".to_string());
        assert_ok!(handler.update_song("pkg", updated));
        assert_ok!(handler.remove_song("pkg", song));
    })
    .await;

    assert_ok!(res);
}

#[rstest]
#[tokio::test(flavor = "multi_thread")]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_reply_handler_playlist_operations(reply_context: TestReplyContext) {
    let TestReplyContext { handler, .. } = reply_context;

    let res = tokio::task::spawn_blocking(move || {
        let playlist = Playlist {
            playlist_name: "Reply Playlist".to_string(),
            ..Default::default()
        };
        let pl_res = handler.add_playlist("pkg", playlist);
        assert_ok!(pl_res.as_ref());
        let pl_id = pl_res.unwrap();
        assert!(!pl_id.is_empty());

        let song = Song {
            song: Some(InnerSong {
                id: Some("pl_song_1".to_string()),
                title: Some("Playlist Song".to_string()),
                path: Some("/music/pl_song.mp3".to_string()),
                ..Default::default()
            }),
            ..Default::default()
        };
        assert_ok!(handler.add_to_playlist("pkg", pl_id, vec![song]));
        assert_ok!(handler.get_entity("pkg", GetEntityOptions::default()));
    })
    .await;

    assert_ok!(res);
}
