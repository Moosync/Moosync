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

use extensions::ReplyHandler;
use songs_proto::moosync::types::{
    GetEntityOptions, GetSongOptions, InnerSong, Playlist, SearchableSong, Song,
};
use tempdir::TempDir;
use types::{plugin::PluginContext, prelude::SongsExt};

use crate::{StateManager, reply_handler::StateReplyHandler};

#[tracing::instrument(level = "debug", skip_all)]
fn create_test_state_reply_handler() -> (StateReplyHandler, TempDir) {
    let tmp = TempDir::new("moosync_sm_reply_test").unwrap();
    let test_dir = tmp.path().to_path_buf();

    let context = PluginContext {
        data_dir: test_dir.clone(),
        cache_dir: test_dir.clone(),
        tmp_dir: test_dir.clone(),
        #[cfg(target_os = "android")]
        android_context: types::android::AndroidJNIContext::default(),
    };

    let sm = StateManager::new_with_context(context).unwrap();
    let handler = StateReplyHandler::new(sm);
    (handler, tmp)
}

#[test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_reply_handler_version_and_player_queries() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let _guard = rt.enter();

    let (handler, _tmp) = create_test_state_reply_handler();

    let handle = std::thread::spawn(move || {
        let version = handler.get_app_version("pkg").unwrap();
        assert!(!version.is_empty());

        let player_state = handler.get_player_state("pkg").unwrap();
        assert_eq!(
            player_state,
            extensions_proto::moosync::types::PlayerState::Stopped as i32
        );

        let vol = handler.get_volume("pkg").unwrap();
        assert_eq!(vol, 100.0);

        let time = handler.get_time("pkg").unwrap();
        assert_eq!(time, 0.0);

        let cur_song = handler.get_current_song("pkg").unwrap();
        assert!(cur_song.is_none());

        let queue = handler.get_queue("pkg").unwrap();
        assert!(queue.0.is_empty());
    });

    handle.join().unwrap();
}

#[test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_reply_handler_preferences_and_secure_storage() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let _guard = rt.enter();

    let (handler, _tmp) = create_test_state_reply_handler();

    let handle = std::thread::spawn(move || {
        let pref_val = extensions_proto::struct_proto::google::protobuf::Value {
            kind: Some(
                extensions_proto::struct_proto::google::protobuf::value::Kind::StringValue(
                    "test_val".to_string(),
                ),
            ),
        };
        assert!(
            handler
                .set_preference("pkg", "my_key", pref_val.clone())
                .is_ok()
        );

        let loaded_pref = handler.get_preference("pkg", "my_key").unwrap();
        assert_eq!(loaded_pref.unwrap().kind, pref_val.kind);

        let sec_val = extensions_proto::struct_proto::google::protobuf::Value {
            kind: Some(
                extensions_proto::struct_proto::google::protobuf::value::Kind::StringValue(
                    "secret_123".to_string(),
                ),
            ),
        };
        assert!(
            handler
                .set_secure("pkg", "sec_key", sec_val.clone())
                .is_ok()
        );

        let loaded_sec = handler.get_secure("pkg", "sec_key").unwrap();
        assert_eq!(loaded_sec.unwrap().kind, sec_val.kind);
    });

    handle.join().unwrap();
}

#[test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_reply_handler_songs_crud() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let _guard = rt.enter();

    let (handler, _tmp) = create_test_state_reply_handler();

    let handle = std::thread::spawn(move || {
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
        assert_eq!(inserted.len(), 1);

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
        assert_eq!(songs.len(), 1);
        assert_eq!(songs[0].get_id().unwrap(), "rep_song_1");

        let mut updated = song.clone();
        updated.song.as_mut().unwrap().title = Some("Updated Title".to_string());
        assert!(handler.update_song("pkg", updated).is_ok());

        assert!(handler.remove_song("pkg", song).is_ok());
    });

    handle.join().unwrap();
}

#[test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_reply_handler_playlist_operations() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let _guard = rt.enter();

    let (handler, _tmp) = create_test_state_reply_handler();

    let handle = std::thread::spawn(move || {
        let playlist = Playlist {
            playlist_name: "Reply Playlist".to_string(),
            ..Default::default()
        };
        let pl_res = handler.add_playlist("pkg", playlist);
        assert!(pl_res.is_ok());
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
        assert!(handler.add_to_playlist("pkg", pl_id, vec![song]).is_ok());

        let entity = handler.get_entity("pkg", GetEntityOptions::default());
        assert!(entity.is_ok());
    });

    handle.join().unwrap();
}
