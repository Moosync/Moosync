use std::{
    fs::{create_dir_all, read},
    path::{Path, PathBuf},
    sync::LazyLock,
};

use image::{DynamicImage, ImageFormat, guess_format, load_from_memory, open};
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
pub fn save_local_image_as_webp(source_path: &Path, target_dir: &Path) -> Option<PathBuf> {
    if !source_path.exists() {
        return None;
    }
    if let Err(e) = create_dir_all(target_dir) {
        error!("Failed to create target image directory {target_dir:?}: {e:?}");
        return None;
    }
    let bytes = match read(source_path) {
        Ok(b) => b,
        Err(e) => {
            error!("Failed to read local image {source_path:?}: {e:?}");
            return None;
        }
    };
    let decoded = match open(source_path).or_else(|_| load_from_memory(&bytes)) {
        Ok(img) => img,
        Err(e) => {
            error!("Failed to decode local image {source_path:?}: {e:?}");
            return None;
        }
    };
    let safe_name = get_safe_name(&source_path.to_string_lossy());
    let resized = resize_image(decoded, HIGH_RES_THUMBNAIL_SIZE);
    let out_path = target_dir.join(format!("{safe_name}.webp"));
    if let Err(e) = resized.save_with_format(&out_path, ImageFormat::WebP) {
        error!("Failed to save local image as WebP to {out_path:?}: {e:?}");
        return None;
    }
    Some(out_path)
}

#[tracing::instrument(level = "debug", skip_all)]
pub fn find_cached_path(cover_url: &str, cache_dir: &Path, low_res: bool) -> Option<PathBuf> {
    if cover_url.is_empty() {
        return None;
    }

    let img_cache_dir = cache_dir.join("image_cache");
    let safe_name = get_safe_name(cover_url);

    if low_res {
        let low_path = img_cache_dir.join(format!("{safe_name}-low.webp"));
        if low_path.exists() {
            return Some(low_path);
        }
    }

    let high_path = img_cache_dir.join(format!("{safe_name}.webp"));
    if high_path.exists() {
        return Some(high_path);
    }

    let path = Path::new(cover_url);
    if path.starts_with(cache_dir) && path.exists() {
        return Some(path.to_path_buf());
    }

    None
}

#[tracing::instrument(level = "debug", skip_all)]
pub fn find_cached_or_local_path(cover_url: &str, cache_dir: &Path) -> Option<PathBuf> {
    if let Some(cached) = find_cached_path(cover_url, cache_dir, false) {
        return Some(cached);
    }
    if !cover_url.starts_with("http://") && !cover_url.starts_with("https://") {
        let path = PathBuf::from(cover_url);
        if path.exists() {
            return Some(path);
        }
    }
    None
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

    let safe_name = get_safe_name(url);

    let high_image = resize_image(decoded_image.clone(), HIGH_RES_THUMBNAIL_SIZE);
    let low_image = resize_image(decoded_image, LOW_RES_THUMBNAIL_SIZE);

    let high_path = img_cache_dir.join(format!("{safe_name}.webp"));
    let low_path = img_cache_dir.join(format!("{safe_name}-low.webp"));

    if let Err(e) = high_image.save_with_format(&high_path, ImageFormat::WebP) {
        error!("Failed to save high resolution image to {high_path:?}: {e:?}");
        return None;
    }

    if let Err(e) = low_image.save_with_format(&low_path, ImageFormat::WebP) {
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
