use std::{
    fs::{copy, create_dir_all},
    io::Cursor,
    path::{Path, PathBuf},
    time::Duration,
};

use extensions::ExtensionInfo;
use image::{DynamicImage, ImageFormat, RgbImage};
use rstest::rstest;
use songs_proto::moosync::types::{Album, Artist, GetSongOptions, InnerSong, SearchableSong, Song};
use tempdir::TempDir;
use tokio::time::sleep;
use tracing_test::traced_test;
use types::plugin::PluginContext;
use wiremock::{
    Mock, MockServer, ResponseTemplate,
    matchers::{method, path},
};

use crate::StateManager;

struct TestSmContext {
    pub _temp_dir: TempDir,
    pub sm: StateManager,
}

#[tracing::instrument(level = "debug", skip_all)]
fn create_test_image_bytes(width: u32, height: u32) -> Vec<u8> {
    let image = DynamicImage::ImageRgb8(RgbImage::new(width, height));
    let mut bytes = Cursor::new(Vec::new());
    image.write_to(&mut bytes, ImageFormat::Png).unwrap();
    bytes.into_inner()
}

#[tracing::instrument(level = "debug", skip_all)]
fn find_sample_extension_msxt() -> PathBuf {
    let runfiles_dir = std::env::var("TEST_SRCDIR").unwrap_or_else(|_| ".".to_string());
    let candidates = [
        PathBuf::from(&runfiles_dir).join("_main/sample_extensions/default/sample_extension.msxt"),
        PathBuf::from(&runfiles_dir)
            .join("moosync/sample_extensions/default/sample_extension.msxt"),
        PathBuf::from("bazel-bin/sample_extensions/default/sample_extension.msxt"),
    ];
    candidates
        .iter()
        .find(|p| p.exists())
        .cloned()
        .unwrap_or_else(|| {
            panic!("sample_extension.msxt not found in runfiles (searched in {candidates:?})")
        })
}

#[tracing::instrument(level = "debug", skip_all)]
fn find_sample_extension_dir() -> PathBuf {
    let runfiles_dir = std::env::var("TEST_SRCDIR").unwrap_or_else(|_| ".".to_string());
    let candidates = [
        PathBuf::from(&runfiles_dir).join("_main/sample_extensions/default"),
        PathBuf::from(&runfiles_dir).join("moosync/sample_extensions/default"),
        PathBuf::from("sample_extensions/default"),
    ];
    candidates
        .iter()
        .find(|p| p.join("sample_extension.wasm").exists())
        .cloned()
        .unwrap_or_else(|| {
            panic!("sample_extension directory not found in runfiles (searched in {candidates:?})")
        })
}

#[tracing::instrument(level = "debug", skip_all)]
fn create_test_sm() -> TestSmContext {
    let temp_dir = TempDir::new("moosync_cover_art_integration_test").unwrap();
    let base_dir = temp_dir.path().to_path_buf();
    let data_dir = base_dir.join("data");
    let cache_dir = base_dir.join("cache");
    let tmp_dir = base_dir.join("tmp");
    create_dir_all(&data_dir).unwrap();
    create_dir_all(&cache_dir).unwrap();
    create_dir_all(&tmp_dir).unwrap();

    let context = PluginContext {
        data_dir,
        cache_dir,
        tmp_dir,
        #[cfg(target_os = "android")]
        android_context: types::android::AndroidJNIContext::default(),
    };
    let sm = StateManager::new_with_context(context).unwrap();
    TestSmContext {
        _temp_dir: temp_dir,
        sm,
    }
}

#[tracing::instrument(level = "debug", skip_all)]
async fn setup_mock_cover_server() -> (MockServer, String) {
    let mock_server = MockServer::start().await;
    let png_bytes = create_test_image_bytes(200, 200);

    Mock::given(method("GET"))
        .and(path("/covers/song_1.png"))
        .respond_with(ResponseTemplate::new(200).set_body_bytes(png_bytes.clone()))
        .mount(&mock_server)
        .await;

    Mock::given(method("GET"))
        .and(path("/covers/album_1.png"))
        .respond_with(ResponseTemplate::new(200).set_body_bytes(png_bytes.clone()))
        .mount(&mock_server)
        .await;

    Mock::given(method("GET"))
        .and(path("/covers/artist_1.png"))
        .respond_with(ResponseTemplate::new(200).set_body_bytes(png_bytes))
        .mount(&mock_server)
        .await;

    let uri = mock_server.uri();
    (mock_server, uri)
}

#[tracing::instrument(level = "debug", skip_all)]
async fn insert_test_entities(sm: &StateManager, server_uri: &str) {
    let db = sm.get_database();
    let song = Song {
        song: Some(InnerSong {
            id: None,
            title: Some("Integration Test Song".to_string()),
            path: None,
            playback_url: Some(format!("{server_uri}/covers/song_1.png")),
            duration: Some(songs_proto::duration_proto::google::protobuf::Duration {
                seconds: 240,
                nanos: 0,
            }),
            r#type: songs_proto::moosync::types::SongType::Url.into(),
            song_cover_path_high: None,
            song_cover_path_low: None,
            ..Default::default()
        }),
        album: Some(Album {
            album_id: None,
            album_name: Some("Integration Test Album".to_string()),
            album_artist: Some(format!("{server_uri}/covers/album_1.png")),
            album_coverpath_high: None,
            album_coverpath_low: None,
            ..Default::default()
        }),
        artists: vec![Artist {
            artist_id: None,
            artist_name: Some(format!("{server_uri}/covers/artist_1.png")),
            artist_coverpath: None,
            ..Default::default()
        }],
        genre: vec![],
    };
    db.insert_songs(vec![song]).await.unwrap();
}

#[tracing::instrument(level = "debug", skip_all)]
async fn wait_for_covers_to_show_up(sm: &StateManager) {
    let db = sm.get_database();
    let mut resolved = false;

    for _ in 0..100 {
        sleep(Duration::from_millis(100)).await;

        let songs = db
            .get_songs_by_options(GetSongOptions {
                song: Some(SearchableSong::default()),
                ..Default::default()
            })
            .await
            .unwrap_or_default();
        let song_has_cover = songs.iter().any(|s| {
            if let Some(inner) = &s.song
                && let Some(high) = &inner.song_cover_path_high
                && !high.is_empty()
                && Path::new(high).exists()
            {
                true
            } else {
                false
            }
        });

        let albums_res = db.get_albums(Album::default(), false).await;
        let album_has_cover = match albums_res {
            Ok(albums) => albums.iter().any(|a| {
                if let Some(high) = &a.album_coverpath_high
                    && !high.is_empty()
                    && Path::new(high).exists()
                {
                    true
                } else {
                    false
                }
            }),
            _ => false,
        };

        let artists_res = db.get_artists(Artist::default(), false).await;
        let artist_has_cover = match artists_res {
            Ok(artists) => artists.iter().any(|a| {
                if let Some(cover) = &a.artist_coverpath
                    && !cover.is_empty()
                    && Path::new(cover).exists()
                {
                    true
                } else {
                    false
                }
            }),
            _ => false,
        };

        if song_has_cover && album_has_cover && artist_has_cover {
            resolved = true;
            break;
        }
    }

    assert!(
        resolved,
        "Timed out waiting for cover arts to show up for song, album, and artist"
    );
}

#[rstest]
#[tokio::test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_user_journey_1_existing_extension_on_startup() {
    let (_mock_server, server_uri) = setup_mock_cover_server().await;
    let TestSmContext { _temp_dir, sm } = create_test_sm();

    let ext_handler = sm.get_extension_handler();
    let sample_src = find_sample_extension_dir();
    let target_dir = ext_handler.extensions_dir.join("rs.sample");
    create_dir_all(&target_dir).unwrap();
    copy(
        sample_src.join("sample_extension.wasm"),
        target_dir.join("sample_extension.wasm"),
    )
    .unwrap();
    copy(
        sample_src.join("manifest.pb"),
        target_dir.join("manifest.pb"),
    )
    .unwrap();

    insert_test_entities(&sm, &server_uri).await;

    sm.setup().await;

    wait_for_covers_to_show_up(&sm).await;
}

#[rstest]
#[tokio::test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_user_journey_2_install_extension_dynamically() {
    let (_mock_server, server_uri) = setup_mock_cover_server().await;
    let TestSmContext { _temp_dir, sm } = create_test_sm();

    sm.setup().await;
    insert_test_entities(&sm, &server_uri).await;

    let db = sm.get_database();
    let initial_songs = db
        .get_songs_by_options(GetSongOptions {
            song: Some(SearchableSong::default()),
            ..Default::default()
        })
        .await
        .unwrap();
    assert!(
        initial_songs[0]
            .song
            .as_ref()
            .unwrap()
            .song_cover_path_high
            .is_none()
    );

    let msxt_path = find_sample_extension_msxt();
    let ext_handler = sm.get_extension_handler();
    ext_handler
        .install_extension(ExtensionInfo::LocalPath(msxt_path))
        .await
        .expect("Failed to install extension");

    wait_for_covers_to_show_up(&sm).await;
}
