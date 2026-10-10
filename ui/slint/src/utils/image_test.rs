use assertables::{assert_none, assert_some, assert_some_eq_x};
use rstest::{fixture, rstest};
use tempdir::TempDir;
use tracing_test::traced_test;

use super::cache_image;

#[fixture]
#[tracing::instrument(level = "debug", skip_all)]
fn temp_dir_fixture() -> TempDir {
    TempDir::new("moosync_cache_img_test").expect("failed to create temp dir")
}

#[rstest]
#[tokio::test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_cache_image_nonexistent_local(temp_dir_fixture: TempDir) {
    let res = cache_image("/non/existent/path.png", temp_dir_fixture.path()).await;

    assert_none!(res);
}

#[rstest]
#[tokio::test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_cache_image_existing_local(temp_dir_fixture: TempDir) {
    let local_file = temp_dir_fixture.path().join("test.txt");
    std::fs::write(&local_file, b"test").unwrap();

    let res = cache_image(local_file.to_str().unwrap(), temp_dir_fixture.path()).await;

    assert_some_eq_x!(&res, &local_file);
}

#[rstest]
#[tokio::test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_cache_image_migrates_unextended_file(temp_dir_fixture: TempDir) {
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

    let res = cache_image("https://example.com/image", temp_dir_fixture.path()).await;

    assert_some!(res.as_ref());
    let path = res.unwrap();
    assert_eq!(path.extension().and_then(|e| e.to_str()), Some("png"));
    assert!(path.exists());
    assert!(!unextended_file.exists());
}
