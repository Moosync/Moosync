use assertables::{assert_none, assert_some_eq_x};
use rstest::{fixture, rstest};
use tempdir::TempDir;
use tracing_test::traced_test;

use super::{cache_image, get_safe_name};

#[fixture]
#[tracing::instrument(level = "debug", skip_all)]
fn temp_dir_fixture() -> TempDir {
    TempDir::new("moosync_cache_img_test").expect("failed to create temp dir")
}

#[rstest]
#[case("https://example.com/cover.jpg", "https___example_com_cover_jpg")]
#[case("abc-123_xyz", "abc_123_xyz")]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_get_safe_name(#[case] input: &str, #[case] expected: &str) {
    let result = get_safe_name(input);

    assert_eq!(result, expected);
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
