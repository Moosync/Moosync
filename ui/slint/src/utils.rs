use std::{
    cell::{Cell, RefCell},
    collections::HashSet,
    path::Path,
    rc::Rc,
};

use extensions_proto::moosync::types::{ExtensionDetail, FetchedExtensionManifest};
use slint::{Image, Model, ModelNotify, ModelRc, ModelTracker, SharedString};
use songs_proto::moosync::types::{Album, Artist, Genre, Playlist, Song};
use tracing::trace;
use types::prelude::SongsExt;

use crate::{
    AlbumModel, ArtistModel, ExtensionItem, GenreModel, PlaylistModel, SearchResult, SongModel,
    WINDOW_EVENTS,
};

pub static DEFAULT_SONG_SVG: &[u8] = include_bytes!("icons/song_default.svg");

pub trait LazyModel: Clone {
    fn set_cover(&mut self, image: Image);
    fn get_cover(&self) -> &Image;
    fn get_cover_url(&self) -> &SharedString;
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
fn find_existing_cache_file(
    img_cache_dir: &std::path::Path,
    safe_name: &str,
) -> Option<std::path::PathBuf> {
    let entries = std::fs::read_dir(img_cache_dir).ok()?;
    entries
        .flatten()
        .find(|entry| is_matching_cache_entry(entry, safe_name))
        .map(|entry| entry.path())
}

#[tracing::instrument(level = "debug", skip_all)]
pub async fn cache_image(
    cover_url: &str,
    cache_dir: &std::path::Path,
) -> Option<std::path::PathBuf> {
    if !cover_url.starts_with("http://") && !cover_url.starts_with("https://") {
        let path = std::path::PathBuf::from(cover_url);
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

    let client = reqwest::Client::new();
    let resp = client.get(cover_url).send().await.ok()?;
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
fn load_icon(path: &str) -> Image {
    if path.is_empty() {
        return Image::load_from_svg_data(include_bytes!("icons/empty.svg")).unwrap();
    }
    Image::load_from_path(std::path::Path::new(path))
        .unwrap_or_else(|_| Image::load_from_svg_data(include_bytes!("icons/empty.svg")).unwrap())
}

#[tracing::instrument(level = "debug", skip_all)]
pub fn load_local_icon(path: &str) -> Image { load_icon(path) }

#[tracing::instrument(level = "debug", skip_all)]
pub fn get_extension_icon(detail: Option<&ExtensionDetail>) -> Image {
    detail
        .and_then(|d| d.extension_icon.as_ref())
        .filter(|p| !p.is_empty())
        .map(|p| load_icon(p))
        .unwrap_or_else(|| load_icon(""))
}

#[tracing::instrument(level = "debug", skip_all)]
pub async fn load_image_from_path_or_url(
    path_or_url: &str,
    cache_dir: &std::path::Path,
) -> Option<Image> {
    if path_or_url.is_empty() {
        return None;
    }
    let local_path = cache_image(path_or_url, cache_dir).await?;
    Image::load_from_path(&local_path).ok()
}

pub struct LazySongVecModel<T: LazyModel> {
    array: Rc<RefCell<Vec<T>>>,
    notify: Rc<ModelNotify>,
    max_items: Rc<Cell<usize>>,
    prefetch_count: Rc<Cell<usize>>,
    allocated_rows: Rc<RefCell<HashSet<usize>>>,
    cache_dir: std::path::PathBuf,
}

impl<T: LazyModel + 'static> LazySongVecModel<T> {
    #[tracing::instrument(level = "trace", skip_all)]
    pub fn new(
        array: Vec<T>,
        item_height: usize,
        item_width: usize,
        cache_dir: std::path::PathBuf,
    ) -> Self {
        let notify = Rc::new(ModelNotify::default());
        let max_items = Rc::new(Cell::new(1));
        let max_items_clone = Rc::downgrade(&max_items);
        let prefetch_count = Rc::new(Cell::new(0));
        let prefetch_count_clone = Rc::downgrade(&prefetch_count);

        WINDOW_EVENTS.with(move |window_events| {
            window_events.on_resize(Box::new(move |window| {
                let max_items = match max_items_clone.upgrade() {
                    Some(m) => m,
                    None => return,
                };
                let prefetch_count = match prefetch_count_clone.upgrade() {
                    Some(p) => p,
                    None => return,
                };

                let scale = window.scale_factor();
                let height = (window.size().height as f32 / scale) as usize;
                let width = (window.size().width as f32 / scale) as usize;

                let mut new_max_items = height / item_height;
                let columns = if item_width > 0 {
                    (width / item_width).max(1)
                } else {
                    1
                };

                if item_width > 0 {
                    new_max_items *= columns;
                }

                trace!(
                    "Window resized {}x{}, item size {}x{}, new max items: {}, columns: {}",
                    width, height, item_width, item_height, new_max_items, columns
                );
                max_items.set(new_max_items);
                prefetch_count.set(2 * columns);
            }));
        });

        Self {
            array: Rc::new(RefCell::new(array)),
            notify,
            max_items,
            prefetch_count,
            allocated_rows: Rc::new(RefCell::new(HashSet::new())),
            cache_dir,
        }
    }

    #[tracing::instrument(level = "debug", skip_all)]
    fn update_image_in_row(&self, row: usize, img: Image) {
        let mut array = self.array.borrow_mut();
        let Some(item) = array.get_mut(row) else {
            return;
        };
        item.set_cover(img);
        self.allocated_rows.borrow_mut().insert(row);
        self.notify.row_changed(row);
    }

    #[tracing::instrument(level = "debug", skip_all)]
    fn load_image(&self, row: usize, cover_url: &str) {
        if cover_url.is_empty() {
            return;
        }

        trace!("Fetching image for row {}", row);

        if !cover_url.starts_with("http://") && !cover_url.starts_with("https://") {
            let path = std::path::PathBuf::from(cover_url);
            if let Ok(img) = Image::load_from_path(&path) {
                self.update_image_in_row(row, img);
            }
            return;
        }

        let safe_name = get_safe_name(cover_url);
        let img_cache_dir = self.cache_dir.join("image_cache");
        if let Some(existing_path) = find_existing_cache_file(&img_cache_dir, &safe_name) {
            if let Ok(img) = Image::load_from_path(&existing_path) {
                self.update_image_in_row(row, img);
            }
            return;
        }

        self.allocated_rows.borrow_mut().insert(row);

        let array = self.array.clone();
        let notify = self.notify.clone();
        let allocated_rows = self.allocated_rows.clone();
        let cover_url_str = cover_url.to_string();
        let cache_dir = self.cache_dir.clone();

        slint::spawn_local(async move {
            let Some(img) = load_image_from_path_or_url(&cover_url_str, &cache_dir).await else {
                return;
            };
            let mut array = array.borrow_mut();
            let Some(item) = array.get_mut(row) else {
                return;
            };
            item.set_cover(img);
            tracing::trace!("Loaded image for row {}", row);
            allocated_rows.borrow_mut().insert(row);
            notify.row_changed(row);
        })
        .unwrap();
    }

    #[tracing::instrument(level = "debug", skip_all)]
    fn release_image(&self, row: usize, model: &mut T) {
        trace!("Releasing image for row {}", row);
        if !is_empty_image(&model.get_cover()) {
            model.set_cover(Image::default());
        }
        self.allocated_rows.borrow_mut().remove(&row);
    }

    #[tracing::instrument(level = "debug", skip_all)]
    fn evict_furthest(&self, current_row: usize) {
        let max = self.max_items.get();
        let capacity = (max * 2).max(64);

        loop {
            let to_release = {
                let allocated = self.allocated_rows.borrow();
                if allocated.len() > capacity {
                    let mut furthest_row = None;
                    let mut max_dist = 0;
                    for &r in allocated.iter() {
                        let dist = r.abs_diff(current_row);
                        if dist > max_dist {
                            max_dist = dist;
                            furthest_row = Some(r);
                        }
                    }
                    furthest_row
                } else {
                    None
                }
            };

            if let Some(r) = to_release {
                let mut array = self.array.borrow_mut();
                if let Some(s) = array.get_mut(r) {
                    self.release_image(r, s);
                }
            } else {
                break;
            }
        }
    }
}

impl<T: LazyModel + 'static> Model for LazySongVecModel<T> {
    type Data = T;

    #[tracing::instrument(level = "debug", skip_all)]
    fn row_count(&self) -> usize { self.array.borrow().len() }

    #[tracing::instrument(level = "debug", skip_all)]
    fn row_data(&self, row: usize) -> Option<Self::Data> {
        let (song_model, is_loaded) = {
            let array = self.array.borrow();
            let song_model = array.get(row)?;
            let is_loaded = !is_empty_image(&song_model.get_cover());
            (song_model.clone(), is_loaded)
        };

        if is_loaded {
            return Some(song_model);
        }

        let cover_url = song_model.get_cover_url().to_string();
        if cover_url.is_empty() {
            return Some(song_model);
        }

        self.load_image(row, &cover_url);

        // Prefetch adjacent items (2 rows up and down)
        let prefetch = self.prefetch_count.get();
        tracing::trace!("Prefetching {} items around row {}", prefetch, row);
        if prefetch > 0 {
            let min_idx = row.saturating_sub(prefetch);
            let max_idx = (row + prefetch).min(self.row_count() - 1);
            for i in min_idx..=max_idx {
                if i != row {
                    let prefetch_url = {
                        let array = self.array.borrow();
                        array.get(i).and_then(|item| {
                            is_empty_image(&item.get_cover())
                                .then(|| item.get_cover_url().to_string())
                        })
                    };
                    if let Some(url) = prefetch_url {
                        self.load_image(i, &url);
                    }
                }
            }
        }

        self.evict_furthest(row);

        Some(song_model)
    }

    #[tracing::instrument(level = "debug", skip_all)]
    fn set_row_data(&self, row: usize, data: Self::Data) {
        if row < self.row_count() {
            if is_empty_image(&data.get_cover()) {
                self.allocated_rows.borrow_mut().remove(&row);
            } else {
                self.allocated_rows.borrow_mut().insert(row);
            }
            self.array.borrow_mut()[row] = data;
            self.notify.row_changed(row);
        }
    }

    #[tracing::instrument(level = "debug", skip_all)]
    fn model_tracker(&self) -> &dyn ModelTracker { &*self.notify }

    #[tracing::instrument(level = "debug", skip_all)]
    fn as_any(&self) -> &dyn core::any::Any { self }
}

impl LazyModel for ExtensionItem {
    #[tracing::instrument(level = "debug", skip_all)]
    fn set_cover(&mut self, image: Image) { self.icon = image; }

    #[tracing::instrument(level = "debug", skip_all)]
    fn get_cover(&self) -> &Image { &self.icon }

    #[tracing::instrument(level = "debug", skip_all)]
    fn get_cover_url(&self) -> &SharedString { &self.icon_url }
}

#[tracing::instrument(level = "debug", skip_all)]
fn is_empty_image(image: &Image) -> bool {
    let size = image.size();
    size.width == 0 && size.height == 0
}

impl LazyModel for SongModel {
    #[tracing::instrument(level = "debug", skip_all)]
    fn set_cover(&mut self, image: Image) { self.coverPathLow = image }

    #[tracing::instrument(level = "debug", skip_all)]
    fn get_cover(&self) -> &Image { &self.coverPathLow }

    #[tracing::instrument(level = "debug", skip_all)]
    fn get_cover_url(&self) -> &SharedString { &self.coverPathUrlLow }
}

impl LazyModel for AlbumModel {
    #[tracing::instrument(level = "debug", skip_all)]
    fn set_cover(&mut self, image: Image) { self.coverPath = image; }

    #[tracing::instrument(level = "debug", skip_all)]
    fn get_cover(&self) -> &Image { &self.coverPath }

    #[tracing::instrument(level = "debug", skip_all)]
    fn get_cover_url(&self) -> &SharedString { &self.coverPathUrl }
}

impl LazyModel for PlaylistModel {
    #[tracing::instrument(level = "debug", skip_all)]
    fn set_cover(&mut self, image: Image) { self.coverPath = image; }

    #[tracing::instrument(level = "debug", skip_all)]
    fn get_cover(&self) -> &Image { &self.coverPath }

    #[tracing::instrument(level = "debug", skip_all)]
    fn get_cover_url(&self) -> &SharedString { &self.coverPathUrl }
}

impl LazyModel for ArtistModel {
    #[tracing::instrument(level = "debug", skip_all)]
    fn set_cover(&mut self, image: Image) { self.coverPath = image; }

    #[tracing::instrument(level = "debug", skip_all)]
    fn get_cover(&self) -> &Image { &self.coverPath }

    #[tracing::instrument(level = "debug", skip_all)]
    fn get_cover_url(&self) -> &SharedString { &self.coverPathUrl }
}

impl LazyModel for GenreModel {
    #[tracing::instrument(level = "debug", skip_all)]
    fn set_cover(&mut self, image: Image) { self.coverPath = image; }

    #[tracing::instrument(level = "debug", skip_all)]
    fn get_cover(&self) -> &Image { &self.coverPath }

    #[tracing::instrument(level = "debug", skip_all)]
    fn get_cover_url(&self) -> &SharedString { &self.coverPathUrl }
}

#[tracing::instrument(level = "debug", skip_all)]
pub fn to_song_model(song: &Song, detail: Option<&ExtensionDetail>) -> SongModel {
    let extension = detail.map(|d| d.package_name.clone()).unwrap_or_default();
    let extension_icon = get_extension_icon(detail);

    let raw_duration = song.get_duration_or_default();
    let duration_s = raw_duration.as_secs() as i32;

    let inner = song.song.as_ref();

    let album = song.album.as_ref();

    let artists: Vec<ArtistModel> = song
        .artists
        .iter()
        .map(|a| to_artist_model(a, detail))
        .collect();
    let genres: Vec<GenreModel> = song.genre.iter().map(to_genre_model).collect();

    SongModel {
        // InnerSong fields
        id: inner
            .and_then(|s| s.id.as_deref())
            .unwrap_or_default()
            .into(),
        path: inner
            .and_then(|s| s.path.as_deref())
            .unwrap_or_default()
            .into(),
        size: inner.and_then(|s| s.size).unwrap_or_default() as f32,
        title: inner
            .and_then(|s| s.title.as_deref())
            .unwrap_or_default()
            .into(),
        date: inner
            .and_then(|s| s.date.as_deref())
            .unwrap_or_default()
            .into(),
        year: inner
            .and_then(|s| s.year.as_deref())
            .unwrap_or_default()
            .into(),
        lyrics: inner
            .and_then(|s| s.lyrics.as_deref())
            .unwrap_or_default()
            .into(),
        release_type: inner
            .and_then(|s| s.release_type.as_deref())
            .unwrap_or_default()
            .into(),
        bitrate: inner.and_then(|s| s.bitrate).unwrap_or_default() as f32,
        codec: inner
            .and_then(|s| s.codec.as_deref())
            .unwrap_or_default()
            .into(),
        container: inner
            .and_then(|s| s.container.as_deref())
            .unwrap_or_default()
            .into(),
        duration_s,
        duration_str: song.format_duration().into(),
        sample_rate: inner.and_then(|s| s.sample_rate).unwrap_or_default() as f32,
        hash: inner
            .and_then(|s| s.hash.as_deref())
            .unwrap_or_default()
            .into(),
        r#type: inner.map(|s| s.r#type).unwrap_or_default(),
        url: inner
            .and_then(|s| s.url.as_deref())
            .unwrap_or_default()
            .into(),
        song_cover_path_high: inner
            .and_then(|s| s.song_cover_path_high.as_deref())
            .unwrap_or_default()
            .into(),
        playback_url: inner
            .and_then(|s| s.playback_url.as_deref())
            .unwrap_or_default()
            .into(),
        song_cover_path_low: inner
            .and_then(|s| s.song_cover_path_low.as_deref())
            .unwrap_or_default()
            .into(),
        date_added: inner.and_then(|s| s.date_added).unwrap_or_default() as i32,
        track_no: inner.and_then(|s| s.track_no).unwrap_or_default() as f32,

        // Album fields
        album_id: album
            .and_then(|a| a.album_id.as_deref())
            .unwrap_or_default()
            .into(),
        album_name: album
            .and_then(|a| a.album_name.as_deref())
            .unwrap_or_default()
            .into(),
        album_artist: album
            .and_then(|a| a.album_artist.as_deref())
            .unwrap_or_default()
            .into(),
        album_coverpath_high: album
            .and_then(|a| a.album_coverpath_high.as_deref())
            .unwrap_or_default()
            .into(),
        album_coverpath_low: album
            .and_then(|a| a.album_coverpath_low.as_deref())
            .unwrap_or_default()
            .into(),
        album_song_count: album.map(|a| a.album_song_count).unwrap_or_default() as f32,
        album_year: album
            .and_then(|a| a.year.as_deref())
            .unwrap_or_default()
            .into(),

        // Repeated fields as Slint arrays
        artists: slint::ModelRc::new(slint::VecModel::from(artists)),
        genre: slint::ModelRc::new(slint::VecModel::from(genres)),

        // UI-only display fields
        coverPathHigh: Image::default(),
        coverPathLow: Image::default(),
        coverPathUrlHigh: song
            .get_cover_high()
            .map(|c| c.to_string())
            .unwrap_or_default()
            .into(),
        coverPathUrlLow: song
            .get_cover_low()
            .map(|c| c.to_string())
            .unwrap_or_default()
            .into(),
        extension: extension.into(),
        extension_icon,
    }
}

/// Convert a `SongModel` back to the proto `Song` type.
/// This is used in place of `get_song_from_cache` since `SongModel` now carries
/// all fields needed to reconstruct the full `Song`.
#[tracing::instrument(level = "debug", skip_all)]
pub fn song_model_to_song(model: &SongModel) -> songs_proto::moosync::types::Song {
    use songs_proto::moosync::types::{Album, Artist, Genre, InnerSong, Song};
    use types::prelude::core_to_proto_duration;

    let duration = core_to_proto_duration(std::time::Duration::from_secs(model.duration_s as u64));

    let inner_song = InnerSong {
        id: if model.id.is_empty() {
            None
        } else {
            Some(model.id.to_string())
        },
        path: if model.path.is_empty() {
            None
        } else {
            Some(model.path.to_string())
        },
        size: if model.size == 0.0 {
            None
        } else {
            Some(model.size as f64)
        },
        title: if model.title.is_empty() {
            None
        } else {
            Some(model.title.to_string())
        },
        date: if model.date.is_empty() {
            None
        } else {
            Some(model.date.to_string())
        },
        year: if model.year.is_empty() {
            None
        } else {
            Some(model.year.to_string())
        },
        lyrics: if model.lyrics.is_empty() {
            None
        } else {
            Some(model.lyrics.to_string())
        },
        release_type: if model.release_type.is_empty() {
            None
        } else {
            Some(model.release_type.to_string())
        },
        bitrate: if model.bitrate == 0.0 {
            None
        } else {
            Some(model.bitrate as f64)
        },
        codec: if model.codec.is_empty() {
            None
        } else {
            Some(model.codec.to_string())
        },
        container: if model.container.is_empty() {
            None
        } else {
            Some(model.container.to_string())
        },
        duration: if model.duration_s == 0 {
            None
        } else {
            Some(duration)
        },
        sample_rate: if model.sample_rate == 0.0 {
            None
        } else {
            Some(model.sample_rate as f64)
        },
        hash: if model.hash.is_empty() {
            None
        } else {
            Some(model.hash.to_string())
        },
        r#type: model.r#type,
        url: if model.url.is_empty() {
            None
        } else {
            Some(model.url.to_string())
        },
        song_cover_path_high: if model.song_cover_path_high.is_empty() {
            None
        } else {
            Some(model.song_cover_path_high.to_string())
        },
        playback_url: if model.playback_url.is_empty() {
            None
        } else {
            Some(model.playback_url.to_string())
        },
        song_cover_path_low: if model.song_cover_path_low.is_empty() {
            None
        } else {
            Some(model.song_cover_path_low.to_string())
        },
        date_added: if model.date_added == 0 {
            None
        } else {
            Some(model.date_added as i64)
        },
        track_no: if model.track_no == 0.0 {
            None
        } else {
            Some(model.track_no as f64)
        },
        ..Default::default()
    };

    let album = if model.album_id.is_empty() && model.album_name.is_empty() {
        None
    } else {
        Some(Album {
            album_id: if model.album_id.is_empty() {
                None
            } else {
                Some(model.album_id.to_string())
            },
            album_name: if model.album_name.is_empty() {
                None
            } else {
                Some(model.album_name.to_string())
            },
            album_artist: if model.album_artist.is_empty() {
                None
            } else {
                Some(model.album_artist.to_string())
            },
            album_coverpath_high: if model.album_coverpath_high.is_empty() {
                None
            } else {
                Some(model.album_coverpath_high.to_string())
            },
            album_coverpath_low: if model.album_coverpath_low.is_empty() {
                None
            } else {
                Some(model.album_coverpath_low.to_string())
            },
            album_song_count: model.album_song_count as f64,
            year: if model.album_year.is_empty() {
                None
            } else {
                Some(model.album_year.to_string())
            },
        })
    };

    let artists: Vec<Artist> = (0..model.artists.row_count())
        .filter_map(|i| model.artists.row_data(i))
        .map(|a| Artist {
            artist_id: if a.id.is_empty() {
                None
            } else {
                Some(a.id.to_string())
            },
            artist_mbid: if a.mbid.is_empty() {
                None
            } else {
                Some(a.mbid.to_string())
            },
            artist_name: if a.title.is_empty() {
                None
            } else {
                Some(a.title.to_string())
            },
            artist_coverpath: if a.coverPathUrl.is_empty() {
                None
            } else {
                Some(a.coverPathUrl.to_string())
            },
            artist_song_count: a.songs_count as f64,
            sanitized_artist_name: if a.sanitized_name.is_empty() {
                None
            } else {
                Some(a.sanitized_name.to_string())
            },
        })
        .collect();

    let genres: Vec<Genre> = (0..model.genre.row_count())
        .filter_map(|i| model.genre.row_data(i))
        .map(|g| Genre {
            genre_id: if g.id.is_empty() {
                None
            } else {
                Some(g.id.to_string())
            },
            genre_name: if g.title.is_empty() {
                None
            } else {
                Some(g.title.to_string())
            },
            genre_song_count: g.songs_count as f64,
        })
        .collect();

    Song {
        song: Some(inner_song),
        album,
        artists,
        genre: genres,
    }
}

#[tracing::instrument(level = "debug", skip_all)]
pub fn to_album_model(album: &Album, detail: Option<&ExtensionDetail>) -> AlbumModel {
    let extension = detail.map(|d| d.package_name.clone()).unwrap_or_default();
    let extension_icon = get_extension_icon(detail);
    AlbumModel {
        coverPath: Image::default(),
        coverPathUrl: album.album_coverpath_high().into(),
        id: album.album_id().into(),
        songs_count: album.album_song_count as i32,
        title: album.album_name().into(),
        extension: extension.into(),
        extension_icon,
    }
}

#[tracing::instrument(level = "debug", skip_all)]
pub fn to_artist_model(artist: &Artist, detail: Option<&ExtensionDetail>) -> ArtistModel {
    let extension = detail.map(|d| d.package_name.clone()).unwrap_or_default();
    let extension_icon = get_extension_icon(detail);
    ArtistModel {
        coverPath: Image::default(),
        coverPathUrl: artist.artist_coverpath.clone().unwrap_or_default().into(),
        id: artist.artist_id.clone().unwrap_or_default().into(),
        songs_count: artist.artist_song_count as i32,
        title: artist.artist_name.clone().unwrap_or_default().into(),
        mbid: artist.artist_mbid.clone().unwrap_or_default().into(),
        sanitized_name: artist
            .sanitized_artist_name
            .clone()
            .unwrap_or_default()
            .into(),
        extension: extension.into(),
        extension_icon,
    }
}

#[tracing::instrument(level = "debug", skip_all)]
pub fn to_playlist_model(playlist: &Playlist, detail: Option<&ExtensionDetail>) -> PlaylistModel {
    let extension = detail
        .map(|d| d.package_name.clone())
        .unwrap_or_else(|| playlist.extension.clone().unwrap_or_default());
    let extension_icon = detail
        .map(|d| get_extension_icon(Some(d)))
        .unwrap_or_else(|| {
            playlist
                .icon
                .as_ref()
                .filter(|p| !p.is_empty())
                .map(|p| load_icon(p))
                .unwrap_or_else(|| load_icon(""))
        });

    PlaylistModel {
        coverPath: Image::default(),
        coverPathUrl: playlist
            .playlist_coverpath
            .clone()
            .unwrap_or_default()
            .into(),
        id: playlist.playlist_id.clone().unwrap_or_default().into(),
        songs_count: playlist.playlist_song_count as i32,
        title: playlist.playlist_name.clone().into(),
        extension: extension.into(),
        extension_icon,
    }
}

#[tracing::instrument(level = "debug", skip_all)]
pub fn to_search_result(
    res: songs_proto::moosync::types::SearchResult,
    detail: Option<&ExtensionDetail>,
    icon: Image,
    theme: &crate::Theme,
    cache_dir: &std::path::Path,
) -> SearchResult {
    let extension = detail.map(|d| d.package_name.clone()).unwrap_or_default();
    SearchResult {
        albums: ModelRc::new(LazySongVecModel::new(
            res.albums
                .iter()
                .map(|a| to_album_model(a, detail))
                .collect(),
            theme.get_cardHeight() as usize,
            theme.get_cardWidth() as usize,
            cache_dir.to_path_buf(),
        )),
        artists: ModelRc::new(LazySongVecModel::new(
            res.artists
                .iter()
                .map(|a| to_artist_model(a, detail))
                .collect(),
            theme.get_cardHeight() as usize,
            theme.get_cardWidth() as usize,
            cache_dir.to_path_buf(),
        )),
        genres: ModelRc::new(LazySongVecModel::new(
            res.genres.iter().map(|g| to_genre_model(g)).collect(),
            theme.get_cardHeight() as usize,
            theme.get_cardWidth() as usize,
            cache_dir.to_path_buf(),
        )),
        playlists: ModelRc::new(LazySongVecModel::new(
            res.playlists
                .iter()
                .map(|p| to_playlist_model(p, detail))
                .collect(),
            theme.get_cardHeight() as usize,
            theme.get_cardWidth() as usize,
            cache_dir.to_path_buf(),
        )),
        songs: ModelRc::new(LazySongVecModel::new(
            res.songs.iter().map(|s| to_song_model(s, detail)).collect(),
            theme.get_songListItemHeight() as usize,
            theme.get_songListItemWidth() as usize,
            cache_dir.to_path_buf(),
        )),
        extension: extension.into(),
        extension_icon: icon,
    }
}

#[tracing::instrument(level = "debug", skip_all)]
pub fn to_genre_model(genre: &Genre) -> GenreModel {
    GenreModel {
        coverPath: Image::default(),
        coverPathUrl: "".into(),
        id: genre.genre_id.clone().unwrap_or_default().into(),
        songs_count: genre.genre_song_count as i32,
        title: genre.genre_name.clone().unwrap_or_default().into(),
    }
}

#[tracing::instrument(level = "debug", skip_all)]
pub fn to_extension_item(ext: &ExtensionDetail) -> ExtensionItem {
    ExtensionItem {
        name: ext.name.clone().into(),
        package_name: ext.package_name.clone().into(),
        version: ext.version.clone().into(),
        active: ext.active,
        is_installed: true,
        loading: ext.active && !ext.has_started,
        description: ext.desc.clone().unwrap_or_default().into(),
        icon: slint::Image::default(),
        has_started: ext.has_started,
        icon_url: ext.extension_icon.clone().unwrap_or_default().into(),
    }
}

#[tracing::instrument(level = "debug", skip_all)]
pub fn to_fetched_extension_item(ext: &FetchedExtensionManifest) -> ExtensionItem {
    ExtensionItem {
        name: ext.name.clone().into(),
        package_name: ext.package_name.clone().into(),
        version: ext.version.clone().into(),
        active: false,
        is_installed: false,
        loading: false,
        description: ext.description.clone().unwrap_or_default().into(),
        icon: slint::Image::default(),
        has_started: false,
        icon_url: ext.logo.clone().unwrap_or_default().into(),
    }
}

#[tracing::instrument(level = "debug", skip_all)]
pub fn generate_blurred_cover_disk_cache(
    song_id: &str,
    cover_path_high: &str,
    cache_dir: &Path,
) -> Option<std::path::PathBuf> {
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

#[tracing::instrument(level = "debug", skip_all)]
pub fn parse_color(val: &str) -> Option<slint::Color> {
    let val = val.trim();
    if val.starts_with('#') {
        let hex = &val[1..];
        match hex.len() {
            6 => {
                let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
                let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
                let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
                Some(slint::Color::from_rgb_u8(r, g, b))
            }
            8 => {
                let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
                let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
                let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
                let a = u8::from_str_radix(&hex[6..8], 16).ok()?;
                Some(slint::Color::from_argb_u8(a, r, g, b))
            }
            _ => None,
        }
    } else if val.starts_with("rgb") {
        let start = val.find('(')? + 1;
        let end = val.rfind(')')?;
        let parts: Vec<&str> = val[start..end].split(',').map(|s| s.trim()).collect();
        if parts.len() < 3 {
            return None;
        }
        let r = parts[0].parse::<f32>().ok()? as u8;
        let g = parts[1].parse::<f32>().ok()? as u8;
        let b = parts[2].parse::<f32>().ok()? as u8;
        if parts.len() == 4 {
            let a = parts[3].parse::<f32>().ok()?;
            return Some(slint::Color::from_argb_f32(
                a,
                r as f32 / 255.0,
                g as f32 / 255.0,
                b as f32 / 255.0,
            ));
        }
        Some(slint::Color::from_rgb_u8(r, g, b))
    } else {
        None
    }
}

#[tracing::instrument(level = "debug", skip_all)]
pub fn parse_length(val: &str) -> Option<f32> {
    let val = val.trim();
    if val.ends_with("px") {
        val[..val.len() - 2].parse::<f32>().ok()
    } else {
        val.parse::<f32>().ok()
    }
}
