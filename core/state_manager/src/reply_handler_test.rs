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
async fn test_reply_handler_get_app_version(reply_context: TestReplyContext) {
    let TestReplyContext { handler, .. } = reply_context;

    let version = tokio::task::spawn_blocking(move || handler.get_app_version("pkg"))
        .await
        .unwrap();

    assert_ok!(version.as_ref());
    assert!(!version.unwrap().is_empty());
}

#[rstest]
#[tokio::test(flavor = "multi_thread")]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_reply_handler_get_player_state(reply_context: TestReplyContext) {
    let TestReplyContext { handler, .. } = reply_context;

    let state = tokio::task::spawn_blocking(move || handler.get_player_state("pkg"))
        .await
        .unwrap();

    assert_ok!(state.as_ref());
    assert_eq!(
        state.unwrap(),
        extensions_proto::moosync::types::PlayerState::Stopped as i32
    );
}

#[rstest]
#[tokio::test(flavor = "multi_thread")]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_reply_handler_get_volume(reply_context: TestReplyContext) {
    let TestReplyContext { handler, .. } = reply_context;

    let volume = tokio::task::spawn_blocking(move || handler.get_volume("pkg"))
        .await
        .unwrap();

    assert_ok!(volume.as_ref());
    assert_eq!(volume.unwrap(), 100.0);
}

#[rstest]
#[tokio::test(flavor = "multi_thread")]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_reply_handler_get_time(reply_context: TestReplyContext) {
    let TestReplyContext { handler, .. } = reply_context;

    let time = tokio::task::spawn_blocking(move || handler.get_time("pkg"))
        .await
        .unwrap();

    assert_ok!(time.as_ref());
    assert_eq!(time.unwrap(), 0.0);
}

#[rstest]
#[tokio::test(flavor = "multi_thread")]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_reply_handler_get_current_song_when_empty(reply_context: TestReplyContext) {
    let TestReplyContext { handler, .. } = reply_context;

    let song = tokio::task::spawn_blocking(move || handler.get_current_song("pkg"))
        .await
        .unwrap();

    assert_ok!(song.as_ref());
    assert_none!(song.unwrap());
}

#[rstest]
#[tokio::test(flavor = "multi_thread")]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_reply_handler_get_queue_when_empty(reply_context: TestReplyContext) {
    let TestReplyContext { handler, .. } = reply_context;

    let queue = tokio::task::spawn_blocking(move || handler.get_queue("pkg"))
        .await
        .unwrap();

    assert_ok!(queue.as_ref());
    assert_is_empty!(&queue.unwrap().0);
}

#[rstest]
#[tokio::test(flavor = "multi_thread")]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_reply_handler_set_and_get_preference(reply_context: TestReplyContext) {
    let TestReplyContext { handler, .. } = reply_context;
    let pref_val = extensions_proto::struct_proto::google::protobuf::Value {
        kind: Some(
            extensions_proto::struct_proto::google::protobuf::value::Kind::StringValue(
                "test_val".to_string(),
            ),
        ),
    };
    let pref_clone = pref_val.clone();

    let loaded = tokio::task::spawn_blocking(move || {
        handler.set_preference("pkg", "my_key", pref_clone).unwrap();
        handler.get_preference("pkg", "my_key")
    })
    .await
    .unwrap();

    assert_ok!(loaded.as_ref());
    assert_eq!(loaded.unwrap().unwrap().kind, pref_val.kind);
}

#[rstest]
#[tokio::test(flavor = "multi_thread")]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_reply_handler_set_and_get_secure(reply_context: TestReplyContext) {
    let TestReplyContext { handler, .. } = reply_context;
    let sec_val = extensions_proto::struct_proto::google::protobuf::Value {
        kind: Some(
            extensions_proto::struct_proto::google::protobuf::value::Kind::StringValue(
                "secret_123".to_string(),
            ),
        ),
    };
    let sec_clone = sec_val.clone();

    let loaded = tokio::task::spawn_blocking(move || {
        handler.set_secure("pkg", "sec_key", sec_clone).unwrap();
        handler.get_secure("pkg", "sec_key")
    })
    .await
    .unwrap();

    assert_ok!(loaded.as_ref());
    assert_eq!(loaded.unwrap().unwrap().kind, sec_val.kind);
}

#[rstest]
#[tokio::test(flavor = "multi_thread")]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_reply_handler_add_and_get_songs(reply_context: TestReplyContext) {
    let TestReplyContext { handler, .. } = reply_context;
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

    let result = tokio::task::spawn_blocking(move || {
        let inserted = handler.add_songs("pkg", vec![song]).unwrap();
        let fetched = handler.get_song(
            "pkg",
            GetSongOptions {
                song: Some(SearchableSong {
                    id: Some("rep_song_1".to_string()),
                    ..Default::default()
                }),
                ..Default::default()
            },
        );
        (inserted, fetched)
    })
    .await
    .unwrap();

    assert_len_eq_x!(&result.0, 1);
    assert_ok!(result.1.as_ref());
    let fetched = result.1.unwrap();
    assert_len_eq_x!(&fetched, 1);
    assert_some_eq_x!(fetched[0].get_id(), "rep_song_1");
}

#[rstest]
#[tokio::test(flavor = "multi_thread")]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_reply_handler_update_song(reply_context: TestReplyContext) {
    let TestReplyContext { handler, .. } = reply_context;
    let song = Song {
        song: Some(InnerSong {
            id: Some("rep_song_update".to_string()),
            title: Some("Original Title".to_string()),
            path: Some("/music/rep_update.mp3".to_string()),
            duration: Some(songs_proto::duration_proto::google::protobuf::Duration {
                seconds: 120,
                nanos: 0,
            }),
            r#type: songs_proto::moosync::types::SongType::Local as i32,
            ..Default::default()
        }),
        ..Default::default()
    };
    let mut updated = song.clone();
    updated.song.as_mut().unwrap().title = Some("Updated Title".to_string());

    let result = tokio::task::spawn_blocking(move || {
        handler.add_songs("pkg", vec![song]).unwrap();
        handler.update_song("pkg", updated)
    })
    .await
    .unwrap();

    assert_ok!(result);
}

#[rstest]
#[tokio::test(flavor = "multi_thread")]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_reply_handler_remove_song(reply_context: TestReplyContext) {
    let TestReplyContext { handler, .. } = reply_context;
    let song = Song {
        song: Some(InnerSong {
            id: Some("rep_song_remove".to_string()),
            title: Some("Remove Song".to_string()),
            path: Some("/music/rep_remove.mp3".to_string()),
            duration: Some(songs_proto::duration_proto::google::protobuf::Duration {
                seconds: 120,
                nanos: 0,
            }),
            r#type: songs_proto::moosync::types::SongType::Local as i32,
            ..Default::default()
        }),
        ..Default::default()
    };
    let song_clone = song.clone();

    let result = tokio::task::spawn_blocking(move || {
        handler.add_songs("pkg", vec![song]).unwrap();
        handler.remove_song("pkg", song_clone)
    })
    .await
    .unwrap();

    assert_ok!(result);
}

#[rstest]
#[tokio::test(flavor = "multi_thread")]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_reply_handler_add_playlist(reply_context: TestReplyContext) {
    let TestReplyContext { handler, .. } = reply_context;
    let playlist = Playlist {
        playlist_name: "Reply Playlist".to_string(),
        ..Default::default()
    };

    let pl_res = tokio::task::spawn_blocking(move || handler.add_playlist("pkg", playlist))
        .await
        .unwrap();

    assert_ok!(pl_res.as_ref());
    assert!(!pl_res.unwrap().is_empty());
}

#[rstest]
#[tokio::test(flavor = "multi_thread")]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_reply_handler_add_to_playlist(reply_context: TestReplyContext) {
    let TestReplyContext { handler, .. } = reply_context;
    let playlist = Playlist {
        playlist_name: "Reply Playlist With Songs".to_string(),
        ..Default::default()
    };
    let song = Song {
        song: Some(InnerSong {
            id: Some("pl_song_1".to_string()),
            title: Some("Playlist Song".to_string()),
            path: Some("/music/pl_song.mp3".to_string()),
            ..Default::default()
        }),
        ..Default::default()
    };

    let result = tokio::task::spawn_blocking(move || {
        let pl_id = handler.add_playlist("pkg", playlist).unwrap();
        handler.add_to_playlist("pkg", pl_id, vec![song])
    })
    .await
    .unwrap();

    assert_ok!(result);
}

#[rstest]
#[tokio::test(flavor = "multi_thread")]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_reply_handler_get_entity(reply_context: TestReplyContext) {
    let TestReplyContext { handler, .. } = reply_context;

    let result =
        tokio::task::spawn_blocking(move || handler.get_entity("pkg", GetEntityOptions::default()))
            .await
            .unwrap();

    assert_ok!(result);
}
