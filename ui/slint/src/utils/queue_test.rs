use songs_proto::moosync::types::{GetEntityOptions, InnerSong, Playlist, Song, entity_result};
use state_manager::StateManager;
use tempdir::TempDir;
use types::plugin::PluginContext;

use super::save_queue;

#[tokio::test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_save_queue_creates_playlist_in_db() {
    let tmp = TempDir::new("utils_save_queue_test").unwrap();
    let test_dir = tmp.path().to_path_buf();
    let context = PluginContext {
        data_dir: test_dir.clone(),
        cache_dir: test_dir.clone(),
        tmp_dir: test_dir.clone(),
        #[cfg(target_os = "android")]
        android_context: types::android::AndroidJNIContext::default(),
    };
    let state_manager = StateManager::new_with_context(context).unwrap();

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
        let mut ph = state_manager.get_player_handler_mut().await;
        ph.add_to_queue(vec![song]);
    }

    save_queue(
        &state_manager,
        "Custom Playlist".to_string(),
        "Custom Desc".to_string(),
    )
    .await;

    let db = state_manager.get_database().await;
    let playlists_res = db.get_entity_by_options(GetEntityOptions {
        playlist: Some(Playlist::default()),
        ..Default::default()
    });

    assert!(playlists_res.is_ok());
    let res = playlists_res.unwrap().result;
    match res {
        Some(entity_result::Result::Playlists(list)) => {
            assert_eq!(list.playlists.len(), 1);
            assert_eq!(list.playlists[0].playlist_name, "Custom Playlist");
            assert_eq!(list.playlists[0].playlist_desc, Some("Custom Desc".into()));
        }
        _ => panic!("Expected playlists in entity result"),
    }
}
