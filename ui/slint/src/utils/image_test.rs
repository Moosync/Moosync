use tempdir::TempDir;

use super::{
    cache_image, default_empty_icon, default_folder_icon, default_song_cover, get_safe_name,
};

#[test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_get_safe_name_replaces_special_chars() {
    let result = get_safe_name("https://example.com/cover.jpg");

    assert_eq!(result, "https___example_com_cover_jpg");
}

#[test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_get_safe_name_replaces_hyphens() {
    let result = get_safe_name("abc-123_xyz");

    assert_eq!(result, "abc_123_xyz");
}

#[tokio::test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_cache_image_nonexistent_local() {
    let tmp = TempDir::new("moosync_cache_img_test").unwrap();

    let res = cache_image("/non/existent/path.png", tmp.path()).await;

    assert!(res.is_none());
}

#[tokio::test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_cache_image_existing_local() {
    let tmp = TempDir::new("moosync_cache_img_test2").unwrap();
    let local_file = tmp.path().join("test.txt");
    std::fs::write(&local_file, b"test").unwrap();

    let res = cache_image(local_file.to_str().unwrap(), tmp.path()).await;

    assert_eq!(res, Some(local_file));
}

#[test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_default_song_cover() {
    let cover = default_song_cover();

    assert_ne!(cover.size().width, 0);
    assert_ne!(cover.size().height, 0);
}

#[test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_default_empty_icon() {
    let icon = default_empty_icon();

    assert_ne!(icon.size().width, 0);
    assert_ne!(icon.size().height, 0);
}

#[test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_default_folder_icon() {
    let icon = default_folder_icon();

    assert_ne!(icon.size().width, 0);
    assert_ne!(icon.size().height, 0);
}
