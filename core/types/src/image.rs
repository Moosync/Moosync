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

use std::{
    fs::create_dir_all,
    path::{Path, PathBuf},
    sync::LazyLock,
};

use image::{DynamicImage, ImageFormat, guess_format, load_from_memory};
use tracing::{debug, error};

pub const LOW_RES_THUMBNAIL_SIZE: u32 = 80;
pub const HIGH_RES_THUMBNAIL_SIZE: u32 = 800;

static HTTP_CLIENT: LazyLock<reqwest::Client> = LazyLock::new(reqwest::Client::new);

#[tracing::instrument(level = "debug", skip_all)]
pub fn get_safe_name(cover_url: &str) -> String {
    cover_url
        .chars()
        .map(|c| if c.is_alphanumeric() { c } else { '_' })
        .collect()
}

#[tracing::instrument(level = "debug", skip_all)]
pub fn get_extension_from_bytes(bytes: &[u8]) -> &'static str {
    match guess_format(bytes) {
        Ok(ImageFormat::Jpeg) => "jpg",
        Ok(ImageFormat::WebP) => "webp",
        Ok(ImageFormat::Gif) => "gif",
        Ok(ImageFormat::Bmp) => "bmp",
        Ok(ImageFormat::Avif) => "avif",
        Ok(ImageFormat::Ico) => "ico",
        Ok(ImageFormat::Tiff) => "tiff",
        _ => "png",
    }
}

#[tracing::instrument(level = "debug", skip_all)]
pub fn resize_image(image: DynamicImage, max_size: u32) -> DynamicImage {
    if image.width() > max_size || image.height() > max_size {
        image.thumbnail(max_size, max_size)
    } else {
        image
    }
}

#[tracing::instrument(level = "debug", skip_all)]
pub fn find_cached_path(cover_url: &str, cache_dir: &Path, low_res: bool) -> Option<PathBuf> {
    if cover_url.is_empty() {
        return None;
    }
    if !cover_url.starts_with("http://") && !cover_url.starts_with("https://") {
        let path = PathBuf::from(cover_url);
        return path.exists().then_some(path);
    }

    let safe_name = get_safe_name(cover_url);
    let img_cache_dir = cache_dir.join("image_cache");
    const EXTENSIONS: &[&str] = &["png", "jpg", "jpeg", "webp", "gif", "avif", "ico", "bmp"];

    if low_res {
        for ext in EXTENSIONS {
            let path = img_cache_dir.join(format!("{safe_name}-low.{ext}"));
            if path.exists() {
                return Some(path);
            }
        }
    }

    for ext in EXTENSIONS {
        let path = img_cache_dir.join(format!("{safe_name}.{ext}"));
        if path.exists() {
            return Some(path);
        }
    }

    None
}

#[tracing::instrument(level = "debug", skip_all)]
pub fn find_cached_or_local_path(cover_url: &str, cache_dir: &Path) -> Option<PathBuf> {
    find_cached_path(cover_url, cache_dir, false)
}

#[tracing::instrument(level = "debug", skip_all)]
pub fn find_cached_low_path(cover_url: &str, cache_dir: &Path) -> Option<PathBuf> {
    find_cached_path(cover_url, cache_dir, true)
}

#[tracing::instrument(level = "debug", skip_all)]
pub async fn download_single_image(url: &str, img_cache_dir: &Path) -> Option<(PathBuf, PathBuf)> {
    debug!("Attempting to download cover from {url}");
    let response = match HTTP_CLIENT.get(url).send().await {
        Ok(resp) if resp.status().is_success() => resp,
        Ok(resp) => {
            error!(
                "Image download failed with status {} for {url}",
                resp.status()
            );
            return None;
        }
        Err(e) => {
            error!("HTTP request error downloading {url}: {e:?}");
            return None;
        }
    };

    let bytes = match response.bytes().await {
        Ok(b) => b,
        Err(e) => {
            error!("Failed to read response bytes from {url}: {e:?}");
            return None;
        }
    };

    let decoded_image = match load_from_memory(&bytes) {
        Ok(img) => img,
        Err(e) => {
            error!("Failed to decode image from {url}: {e:?}");
            return None;
        }
    };

    let ext = get_extension_from_bytes(&bytes);
    let safe_name = get_safe_name(url);

    let high_image = resize_image(decoded_image.clone(), HIGH_RES_THUMBNAIL_SIZE);
    let low_image = resize_image(decoded_image, LOW_RES_THUMBNAIL_SIZE);

    let high_path = img_cache_dir.join(format!("{safe_name}.{ext}"));
    let low_path = img_cache_dir.join(format!("{safe_name}-low.{ext}"));

    if let Err(e) = high_image.save(&high_path) {
        error!("Failed to save high resolution image to {high_path:?}: {e:?}");
        return None;
    }

    if let Err(e) = low_image.save(&low_path) {
        error!("Failed to save low resolution image to {low_path:?}: {e:?}");
        return None;
    }

    debug!("Saved cover images for {url} to {high_path:?} and {low_path:?}");
    Some((high_path, low_path))
}

#[tracing::instrument(level = "debug", skip_all)]
pub async fn download_and_save_image(
    cover_urls: &[String],
    img_cache_dir: &Path,
) -> Option<(PathBuf, PathBuf)> {
    if let Err(e) = create_dir_all(img_cache_dir) {
        error!("Failed to create image cache directory {img_cache_dir:?}: {e:?}");
        return None;
    }

    for url in cover_urls {
        if !url.is_empty()
            && let Some(paths) = download_single_image(url, img_cache_dir).await
        {
            return Some(paths);
        }
    }

    None
}
