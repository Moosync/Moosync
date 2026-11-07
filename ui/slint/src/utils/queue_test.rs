use std::fs;

use assertables::{assert_len_eq_x, assert_ok, assert_some, assert_some_eq_x};
use image::{DynamicImage, ImageFormat, RgbaImage};
use songs_proto::moosync::types::{GetEntityOptions, InnerSong, Playlist, Song, entity_result};
use state_manager::StateManager;
use tempdir::TempDir;
use tracing_test::traced_test;

use super::save_queue;
use crate::test_utils::{TestSlintSmContext, state_manager_fixture};

#[tracing::instrument(level = "debug", skip_all)]
fn fetch_local_playlists(sm: &StateManager) -> Vec<Playlist> {
    let res = sm
        .get_database()
        .get_entity_by_options(GetEntityOptions {
            playlist: Some(Playlist::default()),
            ..Default::default()
        })
        .unwrap()
        .result;
    match res {
        Some(entity_result::Result::Playlists(list)) => list.playlists,
        _ => panic!("Expected playlists in entity result"),
    }
}

#[tokio::test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_save_queue_creates_empty_playlist_without_songs() {
    let TestSlintSmContext { sm, .. } = state_manager_fixture();

    save_queue(
        &sm,
        "Empty Playlist".to_string(),
        String::new(),
        String::new(),
        Vec::new(),
    )
    .await;

    let playlists = fetch_local_playlists(&sm);
    assert_len_eq_x!(&playlists, 1);
    assert_eq!(playlists[0].playlist_name, "Empty Playlist");
    assert_eq!(playlists[0].playlist_coverpath, None);
}

#[tokio::test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_save_queue_skips_without_name_and_songs() {
    let TestSlintSmContext { sm, .. } = state_manager_fixture();

    save_queue(
        &sm,
        "  ".to_string(),
        String::new(),
        String::new(),
        Vec::new(),
    )
    .await;

    let playlists = fetch_local_playlists(&sm);
    assert_len_eq_x!(&playlists, 0);
}

#[tokio::test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_save_queue_converts_cover_to_webp() {
    let TestSlintSmContext { sm, .. } = state_manager_fixture();
    let source_dir = TempDir::new("playlist_cover").unwrap();
    let source_path = source_dir.path().join("cover.png");
    DynamicImage::ImageRgba8(RgbaImage::new(4, 4))
        .save_with_format(&source_path, ImageFormat::Png)
        .unwrap();

    save_queue(
        &sm,
        "Covered".to_string(),
        String::new(),
        source_path.to_string_lossy().to_string(),
        Vec::new(),
    )
    .await;

    let playlists = fetch_local_playlists(&sm);
    assert_len_eq_x!(&playlists, 1);
    let cover = playlists[0].playlist_coverpath.clone();
    assert_some!(&cover);
    let cover = cover.unwrap();
    assert!(cover.ends_with(".webp"));
    let bytes = fs::read(&cover).unwrap();
    assert_eq!(&bytes[0..4], b"RIFF");
    assert_eq!(&bytes[8..12], b"WEBP");
}

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
        let ph = sm.get_player_handler();
        ph.add_to_queue(vec![song]).await;
    }

    let queue = sm.get_player_handler().get_queue().to_vec();
    save_queue(
        &sm,
        "Custom Playlist".to_string(),
        "Custom Desc".to_string(),
        String::new(),
        queue,
    )
    .await;

    let db = sm.get_database();
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
