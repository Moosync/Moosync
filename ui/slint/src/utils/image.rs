use std::{
    fs::create_dir_all,
    path::{Path, PathBuf},
    sync::LazyLock,
};

use cache::{CacheCategory, CacheManager};
use extensions_proto::moosync::types::ExtensionDetail;
use image::{DynamicImage, ImageFormat, load_from_memory, open};
use slint::{Image, Rgba8Pixel, SharedPixelBuffer};
use tokio::{fs, task::spawn_blocking};
use tracing::Instrument;
pub use types::image::{
    HIGH_RES_THUMBNAIL_SIZE, LOW_RES_THUMBNAIL_SIZE, find_cached_low_path,
    find_cached_or_local_path, find_cached_path, get_safe_name, resize_image,
    save_local_image_as_webp,
};

static HTTP_CLIENT: LazyLock<reqwest::Client> = LazyLock::new(reqwest::Client::new);

#[tracing::instrument(level = "debug", skip_all)]
pub fn dynamic_image_to_pixel_buffer(
    dynamic_image: &DynamicImage,
) -> SharedPixelBuffer<Rgba8Pixel> {
    let rgba_image = dynamic_image.to_rgba8();
    let width = rgba_image.width();
    let height = rgba_image.height();
    let mut pixel_buffer = SharedPixelBuffer::<Rgba8Pixel>::new(width, height);
    pixel_buffer
        .make_mut_bytes()
        .copy_from_slice(rgba_image.as_raw());
    pixel_buffer
}

#[tracing::instrument(level = "debug", skip_all)]
pub fn load_image(path: impl AsRef<Path>) -> Image {
    let p = path.as_ref();
    if p.as_os_str().is_empty() {
        return Image::default();
    }
    Image::load_from_path(p).unwrap_or_default()
}

#[tracing::instrument(level = "debug", skip_all)]
pub fn find_cached_or_local_image(cover_url: &str, cache_dir: &Path) -> Option<Image> {
    find_cached_or_local_path(cover_url, cache_dir).and_then(|p| Image::load_from_path(&p).ok())
}

#[tracing::instrument(level = "debug", skip_all)]
async fn download_image_bytes(cover_url: &str) -> Option<Vec<u8>> {
    let response = match HTTP_CLIENT.get(cover_url).send().await {
        Ok(resp) if resp.status() == reqwest::StatusCode::OK => resp,
        Ok(resp) => {
            tracing::error!("Image request failed with status: {}", resp.status());
            return None;
        }
        Err(error) => {
            tracing::error!("Failed to fetch image {cover_url}: {error:?}");
            return None;
        }
    };
    match response.bytes().await {
        Ok(bytes) => Some(bytes.to_vec()),
        Err(error) => {
            tracing::error!("Failed to read response bytes from {cover_url}: {error:?}");
            None
        }
    }
}

#[tracing::instrument(level = "debug", skip_all)]
pub async fn download_image_as_pixel_buffer(url: &str) -> Option<SharedPixelBuffer<Rgba8Pixel>> {
    let bytes = download_image_bytes(url).await?;
    match load_from_memory(&bytes) {
        Ok(img) => Some(dynamic_image_to_pixel_buffer(&img)),
        Err(error) => {
            tracing::error!("Failed to decode image from memory for {url}: {error:?}");
            None
        }
    }
}

#[tracing::instrument(level = "debug", skip_all)]
async fn fetch_and_decode(cover_url: &str) -> Option<(DynamicImage, DynamicImage)> {
    let bytes = download_image_bytes(cover_url).await?;
    let dynamic_image = match load_from_memory(&bytes) {
        Ok(img) => img,
        Err(error) => {
            tracing::error!("Failed to decode image from memory for {cover_url}: {error:?}");
            return None;
        }
    };

    let low_image = resize_image(dynamic_image.clone(), LOW_RES_THUMBNAIL_SIZE);
    let high_image = resize_image(dynamic_image, HIGH_RES_THUMBNAIL_SIZE);
    Some((low_image, high_image))
}

#[tracing::instrument(level = "debug", skip_all)]
async fn save_images_to_disk(
    img_cache_dir: PathBuf,
    safe_name: String,
    low_image: DynamicImage,
    high_image: DynamicImage,
) -> (PathBuf, PathBuf) {
    if !img_cache_dir.exists() {
        let _ = fs::create_dir_all(&img_cache_dir).await;
    }

    let low_path = img_cache_dir.join(format!("{safe_name}-low.webp"));
    let high_path = img_cache_dir.join(format!("{safe_name}.webp"));
    let low_save = low_path.clone();
    let high_save = high_path.clone();

    let save_result = spawn_blocking(move || {
        let cache_manager = CacheManager::new(img_cache_dir);
        let low_res =
            cache_manager.write_with(CacheCategory::ImageCache, "app", &low_save, |target| {
                low_image.save_with_format(target, ImageFormat::WebP)
            });
        let high_res =
            cache_manager.write_with(CacheCategory::ImageCache, "app", &high_save, |target| {
                high_image.save_with_format(target, ImageFormat::WebP)
            });
        (low_res, high_res)
    })
    .await;

    if let Ok((Err(ref low_error), _)) = save_result {
        tracing::error!("Failed to save low resolution image: {low_error:?}");
    }
    if let Ok((_, Err(ref high_error))) = save_result {
        tracing::error!("Failed to save high resolution image: {high_error:?}");
    }

    (high_path, low_path)
}

#[tracing::instrument(level = "debug", skip_all)]
async fn load_from_file_unresized(path: PathBuf) -> Option<SharedPixelBuffer<Rgba8Pixel>> {
    let result = spawn_blocking(move || {
        let dynamic_image = match open(&path) {
            Ok(img) => img,
            Err(error) => {
                tracing::error!("Failed to open image from path {path:?}: {error:?}");
                return None;
            }
        };

        Some(dynamic_image_to_pixel_buffer(&dynamic_image))
    })
    .await;

    match result {
        Ok(buffer_option) => buffer_option,
        Err(error) => {
            tracing::error!("Failed to run blocking task to load image: {error:?}");
            None
        }
    }
}

#[tracing::instrument(level = "debug", skip_all)]
pub async fn resolve_image_buffer(
    cover_url: &str,
    cache_dir: &Path,
    high_res: bool,
) -> Option<SharedPixelBuffer<Rgba8Pixel>> {
    if cover_url.is_empty() {
        return None;
    }

    if let Some(path) = find_cached_path(cover_url, cache_dir, !high_res) {
        return load_from_file_unresized(path).await;
    }

    let safe_name = get_safe_name(cover_url);
    let img_cache_dir = cache_dir.join("image_cache");

    let path = Path::new(cover_url);
    if path.exists() {
        let path_buf = path.to_path_buf();
        let (low_image, high_image) = spawn_blocking(move || {
            let dynamic = match open(&path_buf) {
                Ok(img) => img,
                Err(error) => {
                    tracing::error!("Failed to open image from path {path_buf:?}: {error:?}");
                    return None;
                }
            };
            let low = resize_image(dynamic.clone(), LOW_RES_THUMBNAIL_SIZE);
            let high = resize_image(dynamic, HIGH_RES_THUMBNAIL_SIZE);
            Some((low, high))
        })
        .await
        .ok()??;

        let low_cache = low_image.clone();
        let high_cache = high_image.clone();
        let cache_dir_clone = img_cache_dir.clone();
        let safe_name_clone = safe_name.clone();

        tokio::spawn(
            async move {
                save_images_to_disk(cache_dir_clone, safe_name_clone, low_cache, high_cache).await;
            }
            .in_current_span(),
        );

        let target = if high_res { high_image } else { low_image };
        return Some(dynamic_image_to_pixel_buffer(&target));
    }

    if !cover_url.starts_with("http://") && !cover_url.starts_with("https://") {
        return None;
    }

    let (low_image, high_image) = fetch_and_decode(cover_url).await?;
    let low_cache = low_image.clone();
    let high_cache = high_image.clone();
    let cache_dir_clone = img_cache_dir.clone();
    let safe_name_clone = safe_name.clone();

    tokio::spawn(
        async move {
            save_images_to_disk(cache_dir_clone, safe_name_clone, low_cache, high_cache).await;
        }
        .in_current_span(),
    );

    let target = if high_res { high_image } else { low_image };
    Some(dynamic_image_to_pixel_buffer(&target))
}

#[tracing::instrument(level = "debug", skip_all)]
pub async fn resolve_image(cover_url: &str, cache_dir: &Path) -> Option<PathBuf> {
    if cover_url.is_empty() {
        return None;
    }

    if let Some(path) = find_cached_path(cover_url, cache_dir, false) {
        return Some(path);
    }

    let safe_name = get_safe_name(cover_url);
    let img_cache_dir = cache_dir.join("image_cache");

    let path = Path::new(cover_url);
    if path.exists() {
        let path_buf = path.to_path_buf();
        let (low_image, high_image) = spawn_blocking(move || {
            let dynamic = match open(&path_buf) {
                Ok(img) => img,
                Err(error) => {
                    tracing::error!("Failed to open image from path {path_buf:?}: {error:?}");
                    return None;
                }
            };
            let low = resize_image(dynamic.clone(), LOW_RES_THUMBNAIL_SIZE);
            let high = resize_image(dynamic, HIGH_RES_THUMBNAIL_SIZE);
            Some((low, high))
        })
        .await
        .ok()??;

        let (high_path, _) =
            save_images_to_disk(img_cache_dir, safe_name, low_image, high_image).await;
        return Some(high_path);
    }

    if !cover_url.starts_with("http://") && !cover_url.starts_with("https://") {
        return None;
    }

    let (low_image, high_image) = fetch_and_decode(cover_url).await?;
    let (high_path, _) = save_images_to_disk(img_cache_dir, safe_name, low_image, high_image).await;
    Some(high_path)
}

#[tracing::instrument(level = "debug", skip_all)]
pub fn resolve_extension_icon(
    detail: Option<&ExtensionDetail>,
    fallback_icon: Option<&str>,
) -> Image {
    if let Some(detail) = detail {
        if let Some(icon_path) = detail.extension_icon.as_deref() {
            let img = load_image(icon_path);
            if img.size().width > 0 {
                return img;
            }
        }
    }
    fallback_icon.map(load_image).unwrap_or_default()
}

#[tracing::instrument(level = "debug", skip_all)]
pub fn generate_blurred_cover_disk_cache(
    song_id: &str,
    cover_path_high: &str,
    cache_dir: &Path,
) -> Option<PathBuf> {
    if song_id.is_empty() || cover_path_high.is_empty() {
        return None;
    }

    let img_cache_dir = cache_dir.join("image_cache");
    if !img_cache_dir.exists() {
        let _ = create_dir_all(&img_cache_dir);
    }
    let blurred_path = img_cache_dir.join(format!("blurred_{song_id}.webp"));

    if blurred_path.exists() {
        return Some(blurred_path);
    }

    let path = Path::new(cover_path_high);
    if path.exists() {
        if let Ok(img) = open(path) {
            let cache_manager = CacheManager::new(cache_dir.to_path_buf());
            if cache_manager
                .write_with(
                    CacheCategory::BlurredCover,
                    "app",
                    &blurred_path,
                    |target| {
                        img.fast_blur(5.0)
                            .save_with_format(target, ImageFormat::WebP)
                    },
                )
                .is_ok()
            {
                return Some(blurred_path);
            }
        }
    }

    None
}
