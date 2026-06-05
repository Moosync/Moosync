use std::{
    cell::{Cell, RefCell},
    collections::HashSet,
    path::Path,
    rc::Rc,
};

use extensions_proto::moosync::types::{ExtensionDetail, FetchedExtensionManifest};
use slint::{Image, Model, ModelNotify, ModelTracker, SharedString};
use songs_proto::moosync::types::{Album, Artist, Genre, Playlist, Song};
use tracing::trace;
use types::prelude::SongsExt;

use crate::{
    AlbumModel, ArtistModel, ExtensionItem, GenreModel, PlaylistModel, SongModel, WINDOW_EVENTS,
};

pub static DEFAULT_SONG_SVG: &[u8] = include_bytes!("icons/song_default.svg");

pub trait LazyModel: Clone {
    fn set_cover(&mut self, image: Image);
    fn get_cover(&self) -> &Image;
    fn get_cover_url(&self) -> &SharedString;
}

async fn download_and_cache_image(
    cover_url: &str,
    cache_dir: &std::path::Path,
) -> Option<std::path::PathBuf> {
    if cover_url.starts_with("http://") || cover_url.starts_with("https://") {
        // Remote URL. Check if already in cache.
        let safe_name: String = cover_url
            .chars()
            .map(|c| if c.is_alphanumeric() { c } else { '_' })
            .collect();
        let img_cache_dir = cache_dir.join("image_cache");
        if !img_cache_dir.exists() {
            let _ = std::fs::create_dir_all(&img_cache_dir);
        }
        let ext = if cover_url.contains(".svg") {
            "svg"
        } else {
            "png"
        };
        let cached_path = img_cache_dir.join(format!("{}.{}", safe_name, ext));

        if cached_path.exists() {
            Some(cached_path)
        } else {
            let client = reqwest::Client::new();
            if let Ok(resp) = client.get(cover_url).send().await {
                if let Ok(bytes) = resp.bytes().await {
                    if std::fs::write(&cached_path, bytes).is_ok() {
                        Some(cached_path)
                    } else {
                        None
                    }
                } else {
                    None
                }
            } else {
                None
            }
        }
    } else {
        // Local file path
        Some(std::path::PathBuf::from(cover_url))
    }
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
    #[tracing::instrument(level = "trace", skip(array))]
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

    fn load_image(&self, row: usize, cover_url: &str) {
        if cover_url.is_empty() {
            return;
        }

        trace!("Fetching image for row {}", row);

        self.allocated_rows.borrow_mut().insert(row);

        let array = self.array.clone();
        let notify = self.notify.clone();
        let allocated_rows = self.allocated_rows.clone();
        let cover_url_str = cover_url.to_string();
        let cache_dir = self.cache_dir.clone();

        slint::spawn_local(async move {
            let local_path = download_and_cache_image(&cover_url_str, &cache_dir).await;

            if let Some(path) = local_path {
                if let Ok(img) = Image::load_from_path(&path) {
                    let mut changed = false;
                    {
                        let mut array = array.borrow_mut();
                        if let Some(item) = array.get_mut(row) {
                            item.set_cover(img);
                            tracing::trace!("Loaded image for row {} from {}", row, path.display());
                            allocated_rows.borrow_mut().insert(row);
                            changed = true;
                        }
                    }
                    if changed {
                        notify.row_changed(row);
                    }
                }
            }
        })
        .unwrap();
    }

    fn release_image(&self, row: usize, model: &mut T) {
        trace!("Releasing image for row {}", row);
        if !is_empty_image(&model.get_cover()) {
            model.set_cover(Image::default());
        }
        self.allocated_rows.borrow_mut().remove(&row);
    }

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

    fn row_count(&self) -> usize { self.array.borrow().len() }

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
        tracing::info!("Prefetching {} items around row {}", prefetch, row);
        if prefetch > 0 {
            let min_idx = row.saturating_sub(prefetch);
            let max_idx = (row + prefetch).min(self.row_count() - 1);
            for i in min_idx..=max_idx {
                if i != row {
                    let prefetch_url = {
                        let array = self.array.borrow();
                        if let Some(item) = array.get(i) {
                            let url = item.get_cover_url().to_string();
                            if is_empty_image(&item.get_cover()) {
                                Some(url)
                            } else {
                                None
                            }
                        } else {
                            None
                        }
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

    fn model_tracker(&self) -> &dyn ModelTracker { &*self.notify }

    fn as_any(&self) -> &dyn core::any::Any { self }
}

impl LazyModel for ExtensionItem {
    fn set_cover(&mut self, image: Image) { self.icon = image; }

    fn get_cover(&self) -> &Image { &self.icon }

    fn get_cover_url(&self) -> &SharedString { &self.icon_url }
}

fn is_empty_image(image: &Image) -> bool {
    let size = image.size();
    size.width == 0 && size.height == 0
}

impl LazyModel for SongModel {
    fn set_cover(&mut self, image: Image) { self.coverPathLow = image }

    fn get_cover(&self) -> &Image { &self.coverPathLow }

    fn get_cover_url(&self) -> &SharedString { &self.coverPathUrlLow }
}

impl LazyModel for AlbumModel {
    fn set_cover(&mut self, image: Image) { self.coverPath = image; }

    fn get_cover(&self) -> &Image { &self.coverPath }

    fn get_cover_url(&self) -> &SharedString { &self.coverPathUrl }
}

impl LazyModel for PlaylistModel {
    fn set_cover(&mut self, image: Image) { self.coverPath = image; }

    fn get_cover(&self) -> &Image { &self.coverPath }

    fn get_cover_url(&self) -> &SharedString { &self.coverPathUrl }
}

impl LazyModel for ArtistModel {
    fn set_cover(&mut self, image: Image) { self.coverPath = image; }

    fn get_cover(&self) -> &Image { &self.coverPath }

    fn get_cover_url(&self) -> &SharedString { &self.coverPathUrl }
}

impl LazyModel for GenreModel {
    fn set_cover(&mut self, image: Image) { self.coverPath = image; }

    fn get_cover(&self) -> &Image { &self.coverPath }

    fn get_cover_url(&self) -> &SharedString { &self.coverPathUrl }
}

pub fn to_song_model(song: &Song) -> SongModel {
    let extension_icon = if let Some(icon_path) = song.song.as_ref().and_then(|s| s.icon.clone()) {
        if let Ok(image) = Image::load_from_path(Path::new(&icon_path)) {
            image
        } else {
            Image::load_from_svg_data(include_bytes!("icons/empty.svg")).unwrap()
        }
    } else {
        Image::load_from_svg_data(include_bytes!("icons/empty.svg")).unwrap()
    };

    let raw_duration = song.get_duration_or_default();
    let duration_s = raw_duration.as_secs() as i32;

    SongModel {
        id: song.get_id().unwrap_or_default().into(),
        title: song.get_title().unwrap_or_default().into(),
        artist_name: song.get_artist_string().unwrap_or_default().into(),
        album_name: song.get_album_string().unwrap_or_default().into(),
        duration: song.format_duration().into(),
        duration_s,
        coverPathHigh: Image::default(),
        coverPathLow: Image::default(),
        extensionIcon: extension_icon,
        coverPathUrlHigh: song.get_cover_high().unwrap_or_default().into(),
        coverPathUrlLow: song.get_cover_low().unwrap_or_default().into(),
    }
}

pub fn to_album_model(album: &Album) -> AlbumModel {
    AlbumModel {
        coverPath: Image::default(),
        coverPathUrl: album.album_coverpath_high().into(),
        id: album.album_id().into(),
        songs_count: album.album_song_count as i32,
        title: album.album_name().into(),
    }
}

pub fn to_artist_model(artist: &Artist) -> ArtistModel {
    ArtistModel {
        coverPath: Image::default(),
        coverPathUrl: artist.artist_coverpath.clone().unwrap_or_default().into(),
        id: artist.artist_id.clone().unwrap_or_default().into(),
        songs_count: artist.artist_song_count as i32,
        title: artist.artist_name.clone().unwrap_or_default().into(),
    }
}

pub fn to_playlist_model(playlist: &Playlist) -> PlaylistModel {
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
    }
}

pub fn to_genre_model(genre: &Genre) -> GenreModel {
    GenreModel {
        coverPath: Image::default(),
        coverPathUrl: "".into(),
        id: genre.genre_id.clone().unwrap_or_default().into(),
        songs_count: genre.genre_song_count as i32,
        title: genre.genre_name.clone().unwrap_or_default().into(),
    }
}

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
