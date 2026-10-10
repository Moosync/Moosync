use std::{
    path::{Path, PathBuf},
    sync::LazyLock,
};

use extensions_proto::moosync::types::ExtensionDetail;
use image::ImageFormat;
use slint::Image;
use tokio::fs;

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
    match image::guess_format(bytes) {
        Ok(ImageFormat::Png) => "png",
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
async fn find_cached_image(img_cache_dir: &Path, safe_name: &str) -> Option<PathBuf> {
    const EXTENSIONS: &[&str] = &["png", "jpg", "jpeg", "webp", "gif", "avif", "ico", "bmp"];
    for ext in EXTENSIONS {
        let path = img_cache_dir.join(format!("{}.{}", safe_name, ext));
        if path.exists() {
            return Some(path);
        }
    }

    let unextended_path = img_cache_dir.join(safe_name);
    if unextended_path.exists() {
        if let Ok(bytes) = fs::read(&unextended_path).await {
            let ext = get_extension_from_bytes(&bytes);
            let target_path = img_cache_dir.join(format!("{}.{}", safe_name, ext));
            if fs::rename(&unextended_path, &target_path).await.is_ok() {
                return Some(target_path);
            }
        }
    }

    None
}

#[tracing::instrument(level = "debug", skip_all)]
pub async fn cache_image(cover_url: &str, cache_dir: &Path) -> Option<PathBuf> {
    if !cover_url.starts_with("http://") && !cover_url.starts_with("https://") {
        let path = PathBuf::from(cover_url);
        if path.exists() {
            return Some(path);
        }
        return None;
    }

    let safe_name = get_safe_name(cover_url);
    let img_cache_dir = cache_dir.join("image_cache");

    if let Some(cached_path) = find_cached_image(&img_cache_dir, &safe_name).await {
        return Some(cached_path);
    }

    if !img_cache_dir.exists() {
        let _ = fs::create_dir_all(&img_cache_dir).await;
    }

    let resp = HTTP_CLIENT.get(cover_url).send().await.ok()?;
    if resp.status() != reqwest::StatusCode::OK {
        return None;
    }
    let bytes = resp.bytes().await.ok()?;
    let ext = get_extension_from_bytes(&bytes);
    let cached_path = img_cache_dir.join(format!("{}.{}", safe_name, ext));
    fs::write(&cached_path, bytes).await.ok()?;
    Some(cached_path)
}

#[tracing::instrument(level = "debug", skip_all)]
pub fn load_icon(path: &str) -> Image {
    if path.is_empty() {
        return Image::default();
    }
    Image::load_from_path(Path::new(path)).unwrap_or_default()
}

#[tracing::instrument(level = "debug", skip_all)]
pub fn get_extension_icon(detail: Option<&ExtensionDetail>) -> Image {
    detail
        .and_then(|d| d.extension_icon.as_ref())
        .filter(|p| !p.is_empty())
        .map(|p| load_icon(p))
        .unwrap_or_default()
}

#[tracing::instrument(level = "debug", skip_all)]
pub fn generate_blurred_cover_disk_cache(
    song_id: &str,
    cover_path_high: &str,
    cache_dir: &Path,
) -> Option<PathBuf> {
    if song_id.is_empty() {
        return None;
    }

    let img_cache_dir = cache_dir.join("image_cache");
    if !img_cache_dir.exists() {
        let _ = std::fs::create_dir_all(&img_cache_dir);
    }
    let blurred_path = img_cache_dir.join(format!("blurred_{}.png", song_id));

    if blurred_path.exists() {
        return Some(blurred_path);
    }

    let path = Path::new(cover_path_high);
    if !cover_path_high.is_empty() && path.exists() && blur_and_save(path, &blurred_path).is_some()
    {
        return Some(blurred_path);
    }

    None
}

#[tracing::instrument(level = "debug", skip_all)]
fn blur_and_save(path: &Path, blurred_path: &Path) -> Option<()> {
    let img = image::open(path).ok()?;
    let blurred = img.fast_blur(5.0);
    let _ = blurred.save(blurred_path);
    Some(())
}
