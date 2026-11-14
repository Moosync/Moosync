use std::io::Cursor;

use assertables::{assert_err, assert_matches, assert_ok};
use image::{DynamicImage, ImageBuffer, ImageFormat, Rgba};
use rstest::{fixture, rstest};
use tempdir::TempDir;
use tracing_test::traced_test;

use crate::{context::desktop::image_processor::ImageProcessor, error::ScannerError};

#[fixture]
#[tracing::instrument(level = "debug", skip_all)]
fn temp_dir_fixture() -> TempDir {
    TempDir::new("moosync_img_test").expect("failed to create temp dir")
}

#[tracing::instrument(level = "debug", skip_all)]
fn create_synthetic_png_bytes(width: u32, height: u32) -> Vec<u8> {
    let mut imgbuf = ImageBuffer::new(width, height);
    for (x, y, pixel) in imgbuf.enumerate_pixels_mut() {
        let r = (x % 256) as u8;
        let g = (y % 256) as u8;
        let b = ((x + y) % 256) as u8;
        *pixel = Rgba([r, g, b, 255]);
    }
    let dynamic = DynamicImage::ImageRgba8(imgbuf);
    let mut bytes: Vec<u8> = Vec::new();
    dynamic
        .write_to(&mut Cursor::new(&mut bytes), ImageFormat::Png)
        .unwrap();
    bytes
}

#[rstest]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_image_processor_resize_and_save_synthetic_image(temp_dir_fixture: TempDir) {
    let png_bytes = create_synthetic_png_bytes(500, 300);
    let out_path = temp_dir_fixture.path().join("resized_cover.webp");
    let processor = ImageProcessor::new(&png_bytes).resize(250);

    assert_ok!(processor.save(&out_path));
    let saved_bytes = std::fs::read(&out_path).unwrap();
    assert_eq!(
        image::guess_format(&saved_bytes).unwrap(),
        ImageFormat::WebP
    );
    let loaded = image::open(&out_path).unwrap();
    assert_eq!(loaded.width(), 250);
    assert_eq!(loaded.height(), 250);
}

#[rstest]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_image_processor_from_image_default_dimension(temp_dir_fixture: TempDir) {
    let imgbuf = ImageBuffer::from_pixel(100, 100, Rgba([255, 0, 0, 255]));
    let dynamic = DynamicImage::ImageRgba8(imgbuf);
    let out_path = temp_dir_fixture.path().join("default_dim_cover.webp");
    let processor = ImageProcessor::from_image(dynamic);

    assert_ok!(processor.save(&out_path));
    let loaded = image::open(&out_path).unwrap();
    assert_eq!(loaded.width(), 400);
    assert_eq!(loaded.height(), 400);
}

#[rstest]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_image_processor_invalid_bytes_returns_error(temp_dir_fixture: TempDir) {
    let bad_bytes = b"definitely_not_valid_image_bytes_xyz";
    let out_path = temp_dir_fixture.path().join("bad.png");
    let processor = ImageProcessor::new(bad_bytes);

    let res = processor.save(&out_path);

    assert_err!(res.as_ref());
    assert_matches!(res.unwrap_err(), ScannerError::Image(_));
}

#[rstest]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_image_processor_zero_dimension_returns_error(temp_dir_fixture: TempDir) {
    let png_bytes = create_synthetic_png_bytes(50, 50);
    let out_path = temp_dir_fixture.path().join("zero.png");
    let processor = ImageProcessor::new(&png_bytes).resize(0);

    let res = processor.save(&out_path);

    assert_err!(res.as_ref());
    assert_matches!(res.unwrap_err(), ScannerError::InvalidImageDimensions);
}
