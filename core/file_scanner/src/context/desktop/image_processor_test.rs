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

use std::{env::temp_dir, fs, io::Cursor};

use image::{DynamicImage, ImageBuffer, ImageFormat, Rgba};
use uuid::Uuid;

use crate::{context::desktop::image_processor::ImageProcessor, error::ScannerError};

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

#[test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_image_processor_resize_and_save_synthetic_image() {
    let png_bytes = create_synthetic_png_bytes(500, 300);
    let out_dir = temp_dir().join(format!("moosync_img_test_{}", Uuid::new_v4()));
    fs::create_dir_all(&out_dir).unwrap();
    let out_path = out_dir.join("resized_cover.png");

    let processor = ImageProcessor::new(&png_bytes).resize(250).compress();
    let save_res = processor.save(&out_path);
    assert!(save_res.is_ok());

    let loaded = image::open(&out_path).unwrap();
    assert_eq!(loaded.width(), 250);
    assert_eq!(loaded.height(), 250);

    let _ = fs::remove_dir_all(out_dir);
}

#[test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_image_processor_from_image_default_dimension() {
    let imgbuf = ImageBuffer::from_pixel(100, 100, Rgba([255, 0, 0, 255]));
    let dynamic = DynamicImage::ImageRgba8(imgbuf);

    let out_dir = temp_dir().join(format!("moosync_img_test_{}", Uuid::new_v4()));
    fs::create_dir_all(&out_dir).unwrap();
    let out_path = out_dir.join("default_dim_cover.png");

    let processor = ImageProcessor::from_image(dynamic);
    let save_res = processor.save(&out_path);
    assert!(save_res.is_ok());

    let loaded = image::open(&out_path).unwrap();
    assert_eq!(loaded.width(), 400);
    assert_eq!(loaded.height(), 400);

    let _ = fs::remove_dir_all(out_dir);
}

#[test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_image_processor_invalid_bytes_returns_error() {
    let bad_bytes = b"definitely_not_valid_image_bytes_xyz";
    let out_path = temp_dir().join("bad.png");

    let processor = ImageProcessor::new(bad_bytes);
    let res = processor.save(&out_path);
    assert!(res.is_err());
    match res.unwrap_err() {
        ScannerError::Image(_) => {}
        err => panic!("Expected ScannerError::Image, got: {:?}", err),
    }
}

#[test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_image_processor_zero_dimension_returns_error() {
    let png_bytes = create_synthetic_png_bytes(50, 50);
    let out_path = temp_dir().join("zero.png");

    let processor = ImageProcessor::new(&png_bytes).resize(0);
    let res = processor.save(&out_path);
    assert!(res.is_err());
    match res.unwrap_err() {
        ScannerError::InvalidImageDimensions => {}
        err => panic!(
            "Expected ScannerError::InvalidImageDimensions, got: {:?}",
            err
        ),
    }
}
