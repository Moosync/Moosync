use std::{
    fs::create_dir_all,
    path::{Path, PathBuf},
    sync::LazyLock,
};

use extensions_proto::moosync::types::ExtensionDetail;
use image::{DynamicImage, ImageFormat, guess_format, load_from_memory, open};
use slint::{Image, Rgba8Pixel, SharedPixelBuffer};
use tokio::{fs, task::spawn_blocking};
use tracing::Instrument;

pub const LOW_RES_THUMBNAIL_SIZE: u32 = 80;
pub const HIGH_RES_THUMBNAIL_SIZE: u32 = 800;

static HTTP_CLIENT: LazyLock<reqwest::Client> = LazyLock::new(reqwest::Client::new);

#[tracing::instrument(level = "debug", skip_all)]
fn get_safe_name(cover_url: &str) -> String {
    cover_url
        .chars()
        .map(|c| if c.is_alphanumeric() { c } else { '_' })
        .collect()
}

#[tracing::instrument(level = "debug", skip_all)]
fn get_extension_from_bytes(bytes: &[u8]) -> &'static str {
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
fn resize_image(image: DynamicImage, max_size: u32) -> DynamicImage {
    if image.width() > max_size || image.height() > max_size {
        image.thumbnail(max_size, max_size)
    } else {
        image
    }
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
pub fn find_cached_or_local_image(cover_url: &str, cache_dir: &Path) -> Option<Image> {
    find_cached_or_local_path(cover_url, cache_dir).and_then(|p| Image::load_from_path(&p).ok())
}

#[tracing::instrument(level = "debug", skip_all)]
async fn migrate_unextended_file(img_cache_dir: &Path, safe_name: &str) -> Option<PathBuf> {
    let unextended_path = img_cache_dir.join(safe_name);
    if unextended_path.exists() {
        if let Ok(bytes) = fs::read(&unextended_path).await {
            let ext = get_extension_from_bytes(&bytes);
            let target_path = img_cache_dir.join(format!("{safe_name}.{ext}"));
            if fs::rename(&unextended_path, &target_path).await.is_ok() {
                return Some(target_path);
            }
        }
    }
    None
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

    let low_path = img_cache_dir.join(format!("{safe_name}-low.png"));
    let high_path = img_cache_dir.join(format!("{safe_name}.png"));
    let low_save = low_path.clone();
    let high_save = high_path.clone();

    let save_result = spawn_blocking(move || {
        let low_res = low_image.save(&low_save);
        let high_res = high_image.save(&high_save);
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
async fn load_and_resize_from_file(
    path: PathBuf,
    target_size: u32,
) -> Option<SharedPixelBuffer<Rgba8Pixel>> {
    let result = spawn_blocking(move || {
        let dynamic_image = match open(&path) {
            Ok(img) => img,
            Err(error) => {
                tracing::error!("Failed to open image from path {path:?}: {error:?}");
                return None;
            }
        };

        let target_image = resize_image(dynamic_image, target_size);
        Some(dynamic_image_to_pixel_buffer(&target_image))
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

    let target_size = if high_res {
        HIGH_RES_THUMBNAIL_SIZE
    } else {
        LOW_RES_THUMBNAIL_SIZE
    };

    if let Some(path) = find_cached_path(cover_url, cache_dir, !high_res) {
        return load_and_resize_from_file(path, target_size).await;
    }

    if !cover_url.starts_with("http://") && !cover_url.starts_with("https://") {
        return None;
    }

    let safe_name = get_safe_name(cover_url);
    let img_cache_dir = cache_dir.join("image_cache");

    if let Some(migrated_path) = migrate_unextended_file(&img_cache_dir, &safe_name).await {
        return load_and_resize_from_file(migrated_path, target_size).await;
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

    if let Some(path) = find_cached_or_local_path(cover_url, cache_dir) {
        return Some(path);
    }

    if !cover_url.starts_with("http://") && !cover_url.starts_with("https://") {
        return None;
    }

    let safe_name = get_safe_name(cover_url);
    let img_cache_dir = cache_dir.join("image_cache");

    if let Some(migrated_path) = migrate_unextended_file(&img_cache_dir, &safe_name).await {
        return Some(migrated_path);
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
    let blurred_path = img_cache_dir.join(format!("blurred_{song_id}.png"));

    if blurred_path.exists() {
        return Some(blurred_path);
    }

    let path = Path::new(cover_path_high);
    if path.exists() {
        if let Ok(img) = open(path) {
            if img.fast_blur(5.0).save(&blurred_path).is_ok() {
                return Some(blurred_path);
            }
        }
    }

    None
}
