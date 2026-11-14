use std::path::PathBuf;

use image::{DynamicImage, ImageFormat, RgbaImage};

use super::image::{
    find_cached_low_path, find_cached_or_local_path, find_cached_path, get_extension_from_bytes,
    get_safe_name, resize_image, save_local_image_as_webp,
};

#[test]
fn test_get_safe_name() {
    assert_eq!(
        get_safe_name("https://example.com/art.jpg"),
        "https___example_com_art_jpg"
    );
    assert_eq!(get_safe_name("simple_name"), "simple_name");
}

#[test]
fn test_get_extension_from_bytes() {
    assert_eq!(get_extension_from_bytes(&[0xFF, 0xD8, 0xFF]), "jpg");
    assert_eq!(
        get_extension_from_bytes(&[0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A]),
        "png"
    );
    assert_eq!(get_extension_from_bytes(&[0, 0, 0]), "png");
}

#[test]
fn test_resize_image() {
    let img = DynamicImage::ImageRgba8(RgbaImage::new(1000, 500));
    let resized = resize_image(img, 200);
    assert_eq!(resized.width(), 200);
    assert_eq!(resized.height(), 100);

    let small = DynamicImage::ImageRgba8(RgbaImage::new(50, 50));
    let not_resized = resize_image(small, 200);
    assert_eq!(not_resized.width(), 50);
    assert_eq!(not_resized.height(), 50);
}

#[test]
fn test_find_cached_path_empty() {
    let cache_dir = PathBuf::from("/tmp");
    assert_eq!(find_cached_path("", &cache_dir, false), None);
    assert_eq!(find_cached_or_local_path("", &cache_dir), None);
    assert_eq!(find_cached_low_path("", &cache_dir), None);
}

#[test]
fn test_save_local_image_as_webp() {
    let base_dir = std::env::temp_dir().join("moosync_test_local_webp");
    let _ = std::fs::remove_dir_all(&base_dir);
    std::fs::create_dir_all(&base_dir).unwrap();
    let src_path = base_dir.join("input.png");
    let out_dir = base_dir.join("out");
    let img = DynamicImage::ImageRgba8(RgbaImage::new(1000, 500));
    img.save_with_format(&src_path, ImageFormat::Png).unwrap();

    let saved = save_local_image_as_webp(&src_path, &out_dir);

    assert!(saved.is_some());
    let saved_path = saved.unwrap();
    assert_eq!(
        saved_path.extension().and_then(|e| e.to_str()),
        Some("webp")
    );
    let loaded = image::open(&saved_path).unwrap();
    assert_eq!(loaded.width(), 800);
    assert_eq!(loaded.height(), 400);
    let _ = std::fs::remove_dir_all(&base_dir);
}
