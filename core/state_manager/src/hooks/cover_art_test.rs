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

use std::io::Cursor;

use assertables::{assert_none, assert_ok, assert_some};
use image::{DynamicImage, ImageFormat, RgbImage};
use player::PlayerHandler;
use rstest::{fixture, rstest};
use songs_proto::moosync::types::{Album, Artist, InnerSong, Song};
use tempdir::TempDir;
use tracing_test::traced_test;
use types::{
    image::{
        HIGH_RES_THUMBNAIL_SIZE, LOW_RES_THUMBNAIL_SIZE, download_and_save_image,
        get_extension_from_bytes, get_safe_name, resize_image,
    },
    plugin::PluginContext,
};
use wiremock::{
    Mock, MockServer, ResponseTemplate,
    matchers::{method, path},
};

use crate::{
    StateManager,
    hooks::{Hook, cover_art::CoverArtHook},
};

struct TestSmContext {
    pub _temp_dir: TempDir,
    pub sm: StateManager,
}

#[fixture]
#[tracing::instrument(level = "debug", skip_all)]
fn sm_context() -> TestSmContext {
    let temp_dir =
        TempDir::new("moosync_sm_cover_art_hook_test").expect("failed to create temp dir");
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

#[tracing::instrument(level = "debug", skip_all)]
fn create_test_image_bytes(width: u32, height: u32) -> Vec<u8> {
    let image = DynamicImage::ImageRgb8(RgbImage::new(width, height));
    let mut bytes = Cursor::new(Vec::new());
    image.write_to(&mut bytes, ImageFormat::Png).unwrap();
    bytes.into_inner()
}

#[rstest]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_safe_name() {
    let url = "https://api.deezer.com/cover/123?w=500&h=500";

    let safe = get_safe_name(url);

    assert_eq!(safe, "https___api_deezer_com_cover_123_w_500_h_500");
}

#[rstest]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_get_extension_from_bytes() {
    let png_bytes = create_test_image_bytes(10, 10);
    let mut jpg_cursor = Cursor::new(Vec::new());
    DynamicImage::ImageRgb8(RgbImage::new(10, 10))
        .write_to(&mut jpg_cursor, ImageFormat::Jpeg)
        .unwrap();
    let jpg_bytes = jpg_cursor.into_inner();

    let png_ext = get_extension_from_bytes(&png_bytes);
    let jpg_ext = get_extension_from_bytes(&jpg_bytes);
    let unknown_ext = get_extension_from_bytes(b"invalid data");

    assert_eq!(png_ext, "png");
    assert_eq!(jpg_ext, "jpg");
    assert_eq!(unknown_ext, "png");
}

#[rstest]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_resize_image() {
    let large_image = DynamicImage::ImageRgb8(RgbImage::new(1200, 900));
    let small_image = DynamicImage::ImageRgb8(RgbImage::new(50, 50));

    let high_resized = resize_image(large_image.clone(), HIGH_RES_THUMBNAIL_SIZE);
    let low_resized = resize_image(large_image, LOW_RES_THUMBNAIL_SIZE);
    let untouched = resize_image(small_image, LOW_RES_THUMBNAIL_SIZE);

    assert_eq!(high_resized.width(), 800);
    assert_eq!(high_resized.height(), 600);
    assert_eq!(low_resized.width(), 80);
    assert_eq!(low_resized.height(), 60);
    assert_eq!(untouched.width(), 50);
    assert_eq!(untouched.height(), 50);
}

#[rstest]
#[tokio::test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_download_and_save_image_success() {
    let temp_dir = TempDir::new("cover_art_test_download").unwrap();
    let cache_dir = temp_dir.path().join("image_cache");
    let mock_server = MockServer::start().await;
    let png_bytes = create_test_image_bytes(200, 200);

    Mock::given(method("GET"))
        .and(path("/cover.png"))
        .respond_with(ResponseTemplate::new(200).set_body_bytes(png_bytes))
        .mount(&mock_server)
        .await;

    let target_url = format!("{}/cover.png", mock_server.uri());
    let result = download_and_save_image(&[target_url], &cache_dir).await;

    assert_some!(&result);
    let (high_path, low_path) = result.unwrap();
    assert!(high_path.exists());
    assert!(low_path.exists());
    let loaded_high = image::open(&high_path).unwrap();
    let loaded_low = image::open(&low_path).unwrap();
    assert_eq!(loaded_high.width(), 200);
    assert_eq!(loaded_high.height(), 200);
    assert_eq!(loaded_low.width(), 80);
    assert_eq!(loaded_low.height(), 80);
}

#[rstest]
#[tokio::test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_download_and_save_image_fallback() {
    let temp_dir = TempDir::new("cover_art_test_fallback").unwrap();
    let cache_dir = temp_dir.path().join("image_cache");
    let mock_server = MockServer::start().await;
    let png_bytes = create_test_image_bytes(100, 100);

    Mock::given(method("GET"))
        .and(path("/fail.png"))
        .respond_with(ResponseTemplate::new(404))
        .mount(&mock_server)
        .await;

    Mock::given(method("GET"))
        .and(path("/success.png"))
        .respond_with(ResponseTemplate::new(200).set_body_bytes(png_bytes))
        .mount(&mock_server)
        .await;

    let fail_url = format!("{}/fail.png", mock_server.uri());
    let success_url = format!("{}/success.png", mock_server.uri());
    let result = download_and_save_image(&[fail_url, success_url], &cache_dir).await;

    assert_some!(&result);
    let (high_path, low_path) = result.unwrap();
    assert!(high_path.exists());
    assert!(low_path.exists());
}

#[rstest]
#[tokio::test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_download_and_save_image_all_fail() {
    let temp_dir = TempDir::new("cover_art_test_fail").unwrap();
    let cache_dir = temp_dir.path().join("image_cache");
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/bad_500.png"))
        .respond_with(ResponseTemplate::new(500))
        .mount(&mock_server)
        .await;

    let bad_url = format!("{}/bad_500.png", mock_server.uri());
    let result = download_and_save_image(&[bad_url], &cache_dir).await;

    assert_none!(result);
}

#[rstest]
#[tokio::test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_resolve_song_cover_skips_when_cover_exists(sm_context: TestSmContext) {
    let TestSmContext { sm, .. } = sm_context;
    let hook = CoverArtHook::new();
    let song = Song {
        song: Some(InnerSong {
            id: Some("song_with_cover_123".to_string()),
            title: Some("Song With Cover".to_string()),
            song_cover_path_high: Some("/path/to/existing_high.png".to_string()),
            song_cover_path_low: Some("/path/to/existing_low.png".to_string()),
            ..Default::default()
        }),
        ..Default::default()
    };

    hook.check_and_resolve_song_cover(&sm, &song, true).await;

    let in_flight = hook.in_flight.lock().await;
    assert!(in_flight.is_empty());
}

#[rstest]
#[tokio::test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_cover_art_hook_on_startup(sm_context: TestSmContext) {
    let TestSmContext { sm, .. } = sm_context;
    let hook = CoverArtHook::new();

    let startup_res = hook.on_startup(&sm).await;

    assert_ok!(startup_res);

    let player_handler = sm.plugins.get::<PlayerHandler>();
    player_handler.on_song_changed.emit(Some(Song::default()));

    let ext_handler = sm.get_extension_handler();
    ext_handler.trigger_extensions_updated();
}

#[rstest]
#[tokio::test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_resolve_album_cover_skips_when_cover_exists(sm_context: TestSmContext) {
    let TestSmContext { sm, .. } = sm_context;
    let hook = CoverArtHook::new();
    let album = Album {
        album_id: Some("album_123".to_string()),
        album_name: Some("Test Album".to_string()),
        album_coverpath_high: Some("/path/to/high.png".to_string()),
        album_coverpath_low: Some("/path/to/low.png".to_string()),
        ..Default::default()
    };

    hook.check_and_resolve_album_cover(&sm, &album).await;

    let in_flight = hook.in_flight.lock().await;
    assert!(in_flight.is_empty());
}

#[rstest]
#[tokio::test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_resolve_artist_cover_skips_when_cover_exists(sm_context: TestSmContext) {
    let TestSmContext { sm, .. } = sm_context;
    let hook = CoverArtHook::new();
    let artist = Artist {
        artist_id: Some("artist_123".to_string()),
        artist_name: Some("Test Artist".to_string()),
        artist_coverpath: Some("/path/to/high.png".to_string()),
        ..Default::default()
    };

    hook.check_and_resolve_artist_cover(&sm, &artist).await;

    let in_flight = hook.in_flight.lock().await;
    assert!(in_flight.is_empty());
}

#[rstest]
#[tokio::test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_resolve_all_missing_skips_when_no_extensions(sm_context: TestSmContext) {
    let TestSmContext { sm, .. } = sm_context;
    let hook = CoverArtHook::new();

    hook.resolve_all_missing(&sm).await;

    let in_flight = hook.in_flight.lock().await;
    assert!(in_flight.is_empty());
}
