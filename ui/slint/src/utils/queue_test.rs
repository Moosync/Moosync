use assertables::{assert_len_eq_x, assert_ok, assert_some_eq_x};
use songs_proto::moosync::types::{GetEntityOptions, InnerSong, Playlist, Song, entity_result};
use tracing_test::traced_test;

use super::save_queue;
use crate::test_utils::{TestSlintSmContext, state_manager_fixture};

#[tokio::test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_save_queue_creates_playlist_in_db() {
    let TestSlintSmContext { sm, .. } = state_manager_fixture();

    let song = Song {
        song: Some(InnerSong {
            id: Some("song_util_1".into()),
            title: Some("Queue Song".into()),
            path: Some("/music/test_u1.mp3".into()),
            ..Default::default()
        }),
        ..Default::default()
    };
    {
        let mut ph = sm.get_player_handler_mut().await;
        ph.add_to_queue(vec![song]);
    }

    save_queue(
        &sm,
        "Custom Playlist".to_string(),
        "Custom Desc".to_string(),
    )
    .await;

    let db = sm.get_database().await;
    let playlists_res = db.get_entity_by_options(GetEntityOptions {
        playlist: Some(Playlist::default()),
        ..Default::default()
    });

    assert_ok!(playlists_res.as_ref());
    let res = playlists_res.unwrap().result;
    match res {
        Some(entity_result::Result::Playlists(list)) => {
            assert_len_eq_x!(&list.playlists, 1);
            assert_eq!(list.playlists[0].playlist_name, "Custom Playlist");
            assert_some_eq_x!(list.playlists[0].playlist_desc.as_deref(), "Custom Desc");
        }
        _ => panic!("Expected playlists in entity result"),
    }
}
