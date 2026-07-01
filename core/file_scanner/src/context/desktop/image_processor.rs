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

use std::{num::NonZeroU32, path::Path};

use fast_image_resize::{self as fr, FilterType, ResizeAlg::Convolution, ResizeOptions};
use image::ColorType;

use crate::error::ScannerError;

pub struct ImageProcessor {
    raw_image: Result<image::DynamicImage, ScannerError>,
    dimensions: Option<u32>,
    compressed: bool,
}

impl ImageProcessor {
    #[tracing::instrument(level = "debug", skip_all)]
    pub fn new(data: &[u8]) -> Self {
        let raw_image = image::load_from_memory(data).map_err(ScannerError::Image);
        Self {
            raw_image,
            dimensions: None,
            compressed: false,
        }
    }

    #[allow(dead_code)]
    #[tracing::instrument(level = "debug", skip_all)]
    pub fn from_image(raw_image: image::DynamicImage) -> Self {
        Self {
            raw_image: Ok(raw_image),
            dimensions: None,
            compressed: false,
        }
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub fn resize(mut self, size: u32) -> Self {
        self.dimensions = Some(size);
        self
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub fn compress(mut self) -> Self {
        self.compressed = true;
        self
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub fn save(self, path: &Path) -> Result<(), ScannerError> {
        let raw_image = self.raw_image?;
        let dimensions = self.dimensions.unwrap_or(400);
        let src_image = Self::to_src_image(&raw_image)?;
        let dst_image = Self::resize_image(&src_image, dimensions)?;
        Self::save_image_buffer(path, &dst_image, dimensions)
    }

    #[tracing::instrument(level = "debug", skip_all)]
    fn to_src_image(img: &image::DynamicImage) -> Result<fr::images::Image<'static>, ScannerError> {
        let width =
            NonZeroU32::new(img.width()).ok_or_else(|| ScannerError::InvalidImageDimensions)?;
        let height =
            NonZeroU32::new(img.height()).ok_or_else(|| ScannerError::InvalidImageDimensions)?;
        let img = fr::images::Image::from_vec_u8(
            width.into(),
            height.into(),
            img.to_rgba8().into_vec(),
            fr::PixelType::U8x4,
        )?;
        Ok(img)
    }

    #[tracing::instrument(level = "debug", skip_all)]
    fn resize_image(
        src_image: &fr::images::Image,
        dimensions: u32,
    ) -> Result<fr::images::Image<'static>, ScannerError> {
        let dst_width =
            NonZeroU32::new(dimensions).ok_or_else(|| ScannerError::InvalidImageDimensions)?;
        let dst_height =
            NonZeroU32::new(dimensions).ok_or_else(|| ScannerError::InvalidImageDimensions)?;
        let mut dst_image =
            fr::images::Image::new(dst_width.into(), dst_height.into(), src_image.pixel_type());
        let mut resizer = fr::Resizer::new();
        resizer.resize(
            src_image,
            &mut dst_image,
            Some(&ResizeOptions {
                algorithm: Convolution(FilterType::Hamming),
                mul_div_alpha: false,
                ..Default::default()
            }),
        )?;
        Ok(dst_image)
    }

    #[tracing::instrument(level = "debug", skip_all)]
    fn save_image_buffer(
        path: &Path,
        dst_image: &fr::images::Image,
        dimensions: u32,
    ) -> Result<(), ScannerError> {
        let dst_width =
            NonZeroU32::new(dimensions).ok_or_else(|| ScannerError::InvalidImageDimensions)?;
        let dst_height =
            NonZeroU32::new(dimensions).ok_or_else(|| ScannerError::InvalidImageDimensions)?;
        image::save_buffer(
            path,
            dst_image.buffer(),
            dst_width.get(),
            dst_height.get(),
            ColorType::Rgba8,
        )?;
        Ok(())
    }
}
