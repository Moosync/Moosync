use assertables::{assert_none, assert_some_eq_x};
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
