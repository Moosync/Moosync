use std::{
    path::{Path, PathBuf},
    sync::LazyLock,
};

use extensions_proto::moosync::types::ExtensionDetail;
use slint::Image;

static HTTP_CLIENT: LazyLock<reqwest::Client> = LazyLock::new(reqwest::Client::new);

pub static DEFAULT_SONG_SVG: &[u8] = include_bytes!("../icons/song_default.svg");
pub static DEFAULT_ENTITY_SVG: &[u8] = include_bytes!("../icons/entity_default.svg");
pub static DEFAULT_EMPTY_SVG: &[u8] = include_bytes!("../icons/empty.svg");
pub static DEFAULT_FOLDER_SVG: &[u8] = include_bytes!("../icons/folder.svg");

#[tracing::instrument(level = "debug", skip_all)]
pub fn default_song_cover() -> Image {
    Image::load_from_svg_data(DEFAULT_SONG_SVG).expect("default song SVG should be valid")
}

#[tracing::instrument(level = "debug", skip_all)]
pub fn default_entity_cover() -> Image {
    Image::load_from_svg_data(DEFAULT_ENTITY_SVG).expect("default entity SVG should be valid")
}

#[tracing::instrument(level = "debug", skip_all)]
pub fn default_empty_icon() -> Image {
    Image::load_from_svg_data(DEFAULT_EMPTY_SVG).expect("default empty SVG should be valid")
}

#[tracing::instrument(level = "debug", skip_all)]
pub fn default_folder_icon() -> Image {
    Image::load_from_svg_data(DEFAULT_FOLDER_SVG).expect("default folder SVG should be valid")
}

#[tracing::instrument(level = "debug", skip_all)]
fn detect_image_extension(bytes: &[u8], url: &str) -> &'static str {
    if url.contains(".svg") || bytes.starts_with(b"<svg") || bytes.starts_with(b"<?xml") {
        return "svg";
    }

    let Ok(fmt) = image::guess_format(bytes) else {
        return "png";
    };

    match fmt {
        image::ImageFormat::Jpeg => "jpg",
        image::ImageFormat::Png => "png",
        image::ImageFormat::Gif => "gif",
        image::ImageFormat::WebP => "webp",
        image::ImageFormat::Bmp => "bmp",
        image::ImageFormat::Ico => "ico",
        image::ImageFormat::Tiff => "tiff",
        image::ImageFormat::Qoi => "qoi",
        _ => "png",
    }
}

#[tracing::instrument(level = "debug", skip_all)]
pub fn get_safe_name(cover_url: &str) -> String {
    cover_url
        .chars()
        .map(|c| if c.is_alphanumeric() { c } else { '_' })
        .collect()
}

#[tracing::instrument(level = "debug", skip_all)]
fn is_matching_cache_entry(entry: &std::fs::DirEntry, safe_name: &str) -> bool {
    let file_name = entry.file_name();
    let name_str = file_name.to_string_lossy();
    name_str.starts_with(&format!("{safe_name}.")) || name_str == safe_name
}

#[tracing::instrument(level = "debug", skip_all)]
fn find_existing_cache_file(img_cache_dir: &Path, safe_name: &str) -> Option<PathBuf> {
    let entries = std::fs::read_dir(img_cache_dir).ok()?;
    entries
        .flatten()
        .find(|entry| is_matching_cache_entry(entry, safe_name))
        .map(|entry| entry.path())
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

    if let Some(existing_path) = find_existing_cache_file(&img_cache_dir, &safe_name) {
        return Some(existing_path);
    }

    if !img_cache_dir.exists() {
        let _ = std::fs::create_dir_all(&img_cache_dir);
    }

    let resp = HTTP_CLIENT.get(cover_url).send().await.ok()?;
    if resp.status() != reqwest::StatusCode::OK {
        return None;
    }
    let bytes = resp.bytes().await.ok()?;
    let ext = detect_image_extension(&bytes, cover_url);
    let cached_path = img_cache_dir.join(format!("{safe_name}.{ext}"));
    std::fs::write(&cached_path, bytes).ok()?;
    Some(cached_path)
}

#[tracing::instrument(level = "debug", skip_all)]
pub fn load_icon(path: &str) -> Image {
    if path.is_empty() {
        return default_empty_icon();
    }
    Image::load_from_path(Path::new(path)).unwrap_or_else(|_| default_empty_icon())
}

#[tracing::instrument(level = "debug", skip_all)]
pub fn get_extension_icon(detail: Option<&ExtensionDetail>) -> Image {
    detail
        .and_then(|d| d.extension_icon.as_ref())
        .filter(|p| !p.is_empty())
        .map(|p| load_icon(p))
        .unwrap_or_else(|| load_icon(""))
}

#[tracing::instrument(level = "debug", skip_all)]
pub async fn load_image_from_path_or_url(path_or_url: &str, cache_dir: &Path) -> Option<Image> {
    if path_or_url.is_empty() {
        return None;
    }
    let local_path = cache_image(path_or_url, cache_dir).await?;
    Image::load_from_path(&local_path).ok()
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
