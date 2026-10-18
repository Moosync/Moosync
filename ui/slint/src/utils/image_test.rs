use assertables::{assert_none, assert_some, assert_some_eq_x};
use extensions_proto::moosync::types::ExtensionDetail;
use rstest::{fixture, rstest};
use tempdir::TempDir;
use tracing_test::traced_test;
use wiremock::{Mock, MockServer, ResponseTemplate, matchers::method};

use super::{
    find_cached_or_local_image, find_cached_or_local_path, load_image, load_image_opt,
    resolve_extension_icon, resolve_image,
};

#[fixture]
#[tracing::instrument(level = "debug", skip_all)]
fn temp_dir_fixture() -> TempDir {
    TempDir::new("moosync_cache_img_test").expect("failed to create temp dir")
}

#[rstest]
#[tokio::test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_resolve_image_nonexistent_local(temp_dir_fixture: TempDir) {
    let res = resolve_image("/non/existent/path.png", temp_dir_fixture.path()).await;

    assert_none!(res);
}

#[rstest]
#[tokio::test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_resolve_image_existing_local(temp_dir_fixture: TempDir) {
    let local_file = temp_dir_fixture.path().join("test.txt");
    std::fs::write(&local_file, b"test").unwrap();

    let res = resolve_image(local_file.to_str().unwrap(), temp_dir_fixture.path()).await;

    assert_some_eq_x!(&res, &local_file);
}

#[rstest]
#[tokio::test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_resolve_image_migrates_unextended_file(temp_dir_fixture: TempDir) {
    let img_cache_dir = temp_dir_fixture.path().join("image_cache");
    std::fs::create_dir_all(&img_cache_dir).unwrap();
    let png_bytes = [
        0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a, 0x00, 0x00, 0x00, 0x0d, 0x49, 0x48, 0x44,
        0x52, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x08, 0x06, 0x00, 0x00, 0x00, 0x1f,
        0x15, 0xc4, 0x89, 0x00, 0x00, 0x00, 0x0a, 0x49, 0x44, 0x41, 0x54, 0x78, 0x9c, 0x63, 0x00,
        0x01, 0x00, 0x00, 0x05, 0x00, 0x01, 0x0d, 0x0a, 0x2d, 0xb4, 0x00, 0x00, 0x00, 0x00, 0x49,
        0x45, 0x4e, 0x44, 0xae, 0x42, 0x60, 0x82,
    ];
    let safe_name = "https___example_com_image";
    let unextended_file = img_cache_dir.join(safe_name);
    std::fs::write(&unextended_file, png_bytes).unwrap();

    let res = resolve_image("https://example.com/image", temp_dir_fixture.path()).await;

    assert_some!(res.as_ref());
    let path = res.unwrap();
    assert_eq!(path.extension().and_then(|e| e.to_str()), Some("png"));
    assert!(path.exists());
    assert!(!unextended_file.exists());
}

#[rstest]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_find_cached_or_local_path_and_image(temp_dir_fixture: TempDir) {
    let empty_path = find_cached_or_local_path("", temp_dir_fixture.path());
    assert_none!(empty_path);
    let empty_res = find_cached_or_local_image("", temp_dir_fixture.path());
    assert_none!(empty_res);

    let nonexistent = find_cached_or_local_image("/non/existent.png", temp_dir_fixture.path());
    assert_none!(nonexistent);

    let img_cache_dir = temp_dir_fixture.path().join("image_cache");
    std::fs::create_dir_all(&img_cache_dir).unwrap();
    let png_bytes = [
        0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a, 0x00, 0x00, 0x00, 0x0d, 0x49, 0x48, 0x44,
        0x52, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x08, 0x06, 0x00, 0x00, 0x00, 0x1f,
        0x15, 0xc4, 0x89, 0x00, 0x00, 0x00, 0x0a, 0x49, 0x44, 0x41, 0x54, 0x78, 0x9c, 0x63, 0x00,
        0x01, 0x00, 0x00, 0x05, 0x00, 0x01, 0x0d, 0x0a, 0x2d, 0xb4, 0x00, 0x00, 0x00, 0x00, 0x49,
        0x45, 0x4e, 0x44, 0xae, 0x42, 0x60, 0x82,
    ];
    let cached_file = img_cache_dir.join("https___example_com_cached.png");
    std::fs::write(&cached_file, png_bytes).unwrap();

    let found_path =
        find_cached_or_local_path("https://example.com/cached", temp_dir_fixture.path());
    assert_some_eq_x!(&found_path, &cached_file);

    let cached_res =
        find_cached_or_local_image("https://example.com/cached", temp_dir_fixture.path());
    assert_some!(cached_res.as_ref());
    assert_eq!(cached_res.unwrap().size().width, 1);
}

#[rstest]
#[tokio::test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_resolve_image_local_and_cached(temp_dir_fixture: TempDir) {
    let empty_res = resolve_image("", temp_dir_fixture.path()).await;
    assert_none!(empty_res);

    let png_bytes = [
        0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a, 0x00, 0x00, 0x00, 0x0d, 0x49, 0x48, 0x44,
        0x52, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x08, 0x06, 0x00, 0x00, 0x00, 0x1f,
        0x15, 0xc4, 0x89, 0x00, 0x00, 0x00, 0x0a, 0x49, 0x44, 0x41, 0x54, 0x78, 0x9c, 0x63, 0x00,
        0x01, 0x00, 0x00, 0x05, 0x00, 0x01, 0x0d, 0x0a, 0x2d, 0xb4, 0x00, 0x00, 0x00, 0x00, 0x49,
        0x45, 0x4e, 0x44, 0xae, 0x42, 0x60, 0x82,
    ];
    let local_file = temp_dir_fixture.path().join("local.png");
    std::fs::write(&local_file, png_bytes).unwrap();

    let local_res = resolve_image(local_file.to_str().unwrap(), temp_dir_fixture.path()).await;
    assert_some!(local_res.as_ref());
    assert_eq!(load_image(local_res.unwrap()).size().width, 1);
}

#[rstest]
#[tokio::test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_resolve_image_network(temp_dir_fixture: TempDir) {
    let server = MockServer::start().await;
    let png_bytes = [
        0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a, 0x00, 0x00, 0x00, 0x0d, 0x49, 0x48, 0x44,
        0x52, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x08, 0x06, 0x00, 0x00, 0x00, 0x1f,
        0x15, 0xc4, 0x89, 0x00, 0x00, 0x00, 0x0a, 0x49, 0x44, 0x41, 0x54, 0x78, 0x9c, 0x63, 0x00,
        0x01, 0x00, 0x00, 0x05, 0x00, 0x01, 0x0d, 0x0a, 0x2d, 0xb4, 0x00, 0x00, 0x00, 0x00, 0x49,
        0x45, 0x4e, 0x44, 0xae, 0x42, 0x60, 0x82,
    ];

    Mock::given(method("GET"))
        .respond_with(ResponseTemplate::new(200).set_body_bytes(png_bytes.to_vec()))
        .mount(&server)
        .await;

    let image_url = format!("{}/cover.png", server.uri());
    let res = resolve_image(&image_url, temp_dir_fixture.path()).await;
    assert_some!(res.as_ref());
    assert_eq!(load_image(res.unwrap()).size().width, 1);
}

#[rstest]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_load_image_and_opt(temp_dir_fixture: TempDir) {
    assert_eq!(load_image("").size().width, 0);
    assert_none!(load_image_opt(""));

    let png_bytes = [
        0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a, 0x00, 0x00, 0x00, 0x0d, 0x49, 0x48, 0x44,
        0x52, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x08, 0x06, 0x00, 0x00, 0x00, 0x1f,
        0x15, 0xc4, 0x89, 0x00, 0x00, 0x00, 0x0a, 0x49, 0x44, 0x41, 0x54, 0x78, 0x9c, 0x63, 0x00,
        0x01, 0x00, 0x00, 0x05, 0x00, 0x01, 0x0d, 0x0a, 0x2d, 0xb4, 0x00, 0x00, 0x00, 0x00, 0x49,
        0x45, 0x4e, 0x44, 0xae, 0x42, 0x60, 0x82,
    ];
    let local_file = temp_dir_fixture.path().join("icon.png");
    std::fs::write(&local_file, png_bytes).unwrap();

    assert_eq!(load_image(&local_file).size().width, 1);
    assert_eq!(load_image_opt(&local_file).unwrap().size().width, 1);
}

#[rstest]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_resolve_extension_icon(temp_dir_fixture: TempDir) {
    let empty_icon = resolve_extension_icon(None, None);
    assert_eq!(empty_icon.size().width, 0);

    let png_bytes = [
        0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a, 0x00, 0x00, 0x00, 0x0d, 0x49, 0x48, 0x44,
        0x52, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x08, 0x06, 0x00, 0x00, 0x00, 0x1f,
        0x15, 0xc4, 0x89, 0x00, 0x00, 0x00, 0x0a, 0x49, 0x44, 0x41, 0x54, 0x78, 0x9c, 0x63, 0x00,
        0x01, 0x00, 0x00, 0x05, 0x00, 0x01, 0x0d, 0x0a, 0x2d, 0xb4, 0x00, 0x00, 0x00, 0x00, 0x49,
        0x45, 0x4e, 0x44, 0xae, 0x42, 0x60, 0x82,
    ];
    let local_file = temp_dir_fixture.path().join("ext_icon.png");
    std::fs::write(&local_file, png_bytes).unwrap();

    let detail = ExtensionDetail {
        extension_icon: Some(local_file.to_str().unwrap().to_string()),
        ..Default::default()
    };
    let detail_icon = resolve_extension_icon(Some(&detail), None);
    assert_eq!(detail_icon.size().width, 1);

    let fallback_icon = resolve_extension_icon(None, Some(local_file.to_str().unwrap()));
    assert_eq!(fallback_icon.size().width, 1);
}
