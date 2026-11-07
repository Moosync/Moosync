use std::{
    cell::{Cell, RefCell},
    collections::{HashSet, VecDeque},
    path::PathBuf,
    rc::Rc,
};

use slint::{
    ComponentHandle, Image, Model, ModelNotify, ModelRc, ModelTracker, SharedString, Window,
};
use state_manager::StateManager;
use tracing::{Instrument, trace};

use super::resolve_image_buffer;
use crate::{
    AlbumModel, ArtistModel, ExtensionItem, GenreModel, MainWindow, PlaylistModel, SongModel,
    Theme, WINDOW_EVENTS,
};

#[tracing::instrument(level = "debug", skip_all)]
pub fn make_lazy_song_model(
    main_window: &MainWindow,
    state_manager: &StateManager,
    songs: Vec<SongModel>,
) -> ModelRc<SongModel> {
    let theme = main_window.global::<Theme>();
    let cache_dir = state_manager.get_cache_dir();
    ModelRc::new(LazySongVecModel::new_with_window(
        songs,
        theme.get_songListItemHeight() as usize,
        theme.get_songListItemWidth() as usize,
        cache_dir,
        false,
        Some(main_window.window()),
    ))
}

#[tracing::instrument(level = "debug", skip_all)]
pub fn make_lazy_card_model<T: LazyModel + 'static>(
    main_window: &MainWindow,
    state_manager: &StateManager,
    items: Vec<T>,
) -> ModelRc<T> {
    let theme = main_window.global::<Theme>();
    let cache_dir = state_manager.get_cache_dir();
    ModelRc::new(LazySongVecModel::new_with_window(
        items,
        theme.get_cardHeight() as usize,
        theme.get_cardWidth() as usize,
        cache_dir,
        true,
        Some(main_window.window()),
    ))
}

#[tracing::instrument(level = "debug", skip_all)]
pub fn calculate_window_limits(
    width: usize,
    height: usize,
    item_width: usize,
    item_height: usize,
) -> (usize, usize) {
    let columns = if item_width > 0 {
        width.checked_div(item_width).map_or(1, |c| c.max(1))
    } else {
        1
    };
    let rows = if item_height > 0 {
        height.checked_div(item_height).unwrap_or(0) + 2
    } else {
        1
    };
    let max_items = (rows * columns).max(20);
    let prefetch = (2 * columns).max(4);
    (max_items, prefetch)
}

pub trait LazyModel: Clone {
    fn get_id(&self) -> &SharedString;
    fn set_cover(&mut self, image: Image);
    fn get_cover_url(&self) -> &SharedString;
    fn get_image(&self) -> &Image;
    fn has_image_set(&self) -> bool { !self.get_image().size().is_empty() }
    fn update_from(&mut self, updated: Self);
}

#[tracing::instrument(level = "debug", skip_all)]
pub fn update_lazy_model_item<T: LazyModel + 'static>(model: &ModelRc<T>, updated: T) -> bool {
    let target_id = updated.get_id();
    if target_id.is_empty() {
        return false;
    }
    for row in 0..model.row_count() {
        if let Some(item) = model.row_data(row) {
            if item.get_id() == target_id {
                model.set_row_data(row, updated);
                return true;
            }
        }
    }
    false
}

pub struct LazySongVecModel<T: LazyModel> {
    array: Rc<RefCell<Vec<T>>>,
    notify: Rc<ModelNotify>,
    max_items: Rc<Cell<usize>>,
    prefetch_count: Rc<Cell<usize>>,
    allocated_rows: Rc<RefCell<VecDeque<usize>>>,
    in_flight: Rc<RefCell<HashSet<usize>>>,
    failed_rows: Rc<RefCell<HashSet<usize>>>,
    cache_dir: PathBuf,
    high_res: bool,
}

impl<T: LazyModel + 'static> LazySongVecModel<T> {
    #[tracing::instrument(level = "trace", skip_all)]
    pub fn new(
        array: Vec<T>,
        item_height: usize,
        item_width: usize,
        cache_dir: PathBuf,
        high_res: bool,
    ) -> Self {
        Self::new_with_window(array, item_height, item_width, cache_dir, high_res, None)
    }

    #[tracing::instrument(level = "trace", skip_all)]
    pub fn new_with_window(
        array: Vec<T>,
        item_height: usize,
        item_width: usize,
        cache_dir: PathBuf,
        high_res: bool,
        window: Option<&Window>,
    ) -> Self {
        let (initial_width, initial_height) = if let Some(w) = window {
            let scale = w.scale_factor();
            let width = (w.size().width as f32 / scale) as usize;
            let height = (w.size().height as f32 / scale) as usize;
            WINDOW_EVENTS.with(|we| we.set_last_size(width, height));
            (width, height)
        } else {
            WINDOW_EVENTS
                .with(|we| we.get_last_size())
                .unwrap_or((800, 600))
        };

        let (initial_max, initial_prefetch) =
            calculate_window_limits(initial_width, initial_height, item_width, item_height);

        let notify = Rc::new(ModelNotify::default());
        let max_items = Rc::new(Cell::new(initial_max));
        let max_items_clone = Rc::downgrade(&max_items);
        let prefetch_count = Rc::new(Cell::new(initial_prefetch));
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

                let (new_max_items, new_prefetch) =
                    calculate_window_limits(width, height, item_width, item_height);

                trace!(
                    "Window resized {}x{}, item size {}x{}, new max items: {}, prefetch: {}",
                    width, height, item_width, item_height, new_max_items, new_prefetch
                );
                max_items.set(new_max_items);
                prefetch_count.set(new_prefetch);
            }));
        });

        Self {
            array: Rc::new(RefCell::new(array)),
            notify,
            prefetch_count,
            max_items,
            allocated_rows: Rc::new(RefCell::new(VecDeque::new())),
            in_flight: Rc::new(RefCell::new(HashSet::new())),
            failed_rows: Rc::new(RefCell::new(HashSet::new())),
            cache_dir,
            high_res,
        }
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub fn update_item(&self, updated: T) -> bool {
        let target_id = updated.get_id();
        if target_id.is_empty() {
            return false;
        }
        let found_row = {
            let array = self.array.borrow();
            array.iter().position(|item| item.get_id() == target_id)
        };
        let Some(row) = found_row else {
            return false;
        };
        self.set_row_data(row, updated);
        true
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub fn max_len(&self) -> usize { self.max_items.get() + self.prefetch_count.get() * 2 }
}

impl<T: LazyModel + 'static> Model for LazySongVecModel<T> {
    type Data = T;

    #[tracing::instrument(level = "debug", skip_all)]
    fn row_count(&self) -> usize { self.array.borrow().len() }

    #[tracing::instrument(level = "debug", skip_all)]
    fn row_data(&self, row: usize) -> Option<Self::Data> {
        let song_model = {
            let array = self.array.borrow();
            array.get(row)?.clone()
        };

        if song_model.has_image_set()
            || self.in_flight.borrow().contains(&row)
            || self.failed_rows.borrow().contains(&row)
        {
            return Some(song_model);
        }

        let cover_url = song_model.get_cover_url().to_string();
        if cover_url.is_empty() {
            self.failed_rows.borrow_mut().insert(row);
            return Some(song_model);
        }

        self.in_flight.borrow_mut().insert(row);

        let array = self.array.clone();
        let cache_dir = self.cache_dir.clone();
        let notify = self.notify.clone();
        let high_res = self.high_res;
        let in_flight = self.in_flight.clone();
        let failed_rows = self.failed_rows.clone();
        let allocated_rows = self.allocated_rows.clone();
        let max_items = self.max_items.clone();
        let prefetch_count = self.prefetch_count.clone();

        let _ = slint::spawn_local(
            async move {
                trace!("Fetching image for row {}; cover_url: {}", row, cover_url);
                let pixel_buffer = tokio::spawn(async move {
                    resolve_image_buffer(&cover_url, &cache_dir, high_res).await
                })
                .await
                .unwrap_or_default();

                in_flight.borrow_mut().remove(&row);

                if let Some(buffer) = pixel_buffer {
                    let image = Image::from_rgba8(buffer);
                    trace!("Image loaded for row {}: (size: {:?})", row, image.size());

                    let mut arr = array.borrow_mut();
                    if let Some(item) = arr.get_mut(row) {
                        item.set_cover(image);
                    }

                    let mut rows = allocated_rows.borrow_mut();
                    rows.retain(|&r| r != row);
                    rows.push_back(row);

                    let max_len = max_items.get() + prefetch_count.get() * 2;
                    while rows.len() > max_len {
                        if let Some(oldest) = rows.pop_front() {
                            trace!(
                                "LazySongVecModel: Evicting image for row {} (max_len: {}, remaining allocated: {})",
                                oldest, max_len, rows.len()
                            );
                            if let Some(model) = arr.get_mut(oldest) {
                                model.set_cover(Image::default());
                            }
                        }
                    }
                    drop(rows);
                    drop(arr);

                    notify.row_changed(row);
                    trace!("exiting");
                } else {
                    failed_rows.borrow_mut().insert(row);
                }
            }
            .in_current_span(),
        );

        Some(song_model)
    }

    #[tracing::instrument(level = "debug", skip_all)]
    fn set_row_data(&self, row: usize, data: Self::Data) {
        if row >= self.row_count() {
            return;
        }

        self.failed_rows.borrow_mut().remove(&row);
        let mut array = self.array.borrow_mut();
        if let Some(item) = array.get_mut(row) {
            item.update_from(data);
            drop(array);
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
    fn get_id(&self) -> &SharedString { &self.package_name }

    #[tracing::instrument(level = "debug", skip_all)]
    fn set_cover(&mut self, image: Image) { self.icon = image; }

    #[tracing::instrument(level = "debug", skip_all)]
    fn get_cover_url(&self) -> &SharedString { &self.icon_url }

    #[tracing::instrument(level = "debug", skip_all)]
    fn get_image(&self) -> &Image { &self.icon }

    #[tracing::instrument(level = "debug", skip_all)]
    fn update_from(&mut self, updated: Self) {
        if !updated.name.is_empty() {
            self.name = updated.name;
        }
        if !updated.icon_url.is_empty() && self.icon_url != updated.icon_url {
            self.icon_url = updated.icon_url;
            self.icon = Image::default();
        }
    }
}

impl LazyModel for SongModel {
    #[tracing::instrument(level = "debug", skip_all)]
    fn get_id(&self) -> &SharedString { &self.id }

    #[tracing::instrument(level = "debug", skip_all)]
    fn set_cover(&mut self, image: Image) { self.coverPathLow = image }

    #[tracing::instrument(level = "debug", skip_all)]
    fn get_cover_url(&self) -> &SharedString { &self.coverPathUrlLow }

    #[tracing::instrument(level = "debug", skip_all)]
    fn get_image(&self) -> &Image { &self.coverPathLow }

    #[tracing::instrument(level = "debug", skip_all)]
    fn update_from(&mut self, updated: Self) {
        if !updated.title.is_empty() {
            self.title = updated.title;
        }
        if updated.duration_s > 0 {
            self.duration_s = updated.duration_s;
        }
        if !updated.album_name.is_empty() {
            self.album_name = updated.album_name;
        }
        if !updated.album_id.is_empty() {
            self.album_id = updated.album_id;
        }
        if !updated.album_artist.is_empty() {
            self.album_artist = updated.album_artist;
        }
        if !updated.playback_url.is_empty() {
            self.playback_url = updated.playback_url;
        }
        if !updated.coverPathUrlLow.is_empty() && self.coverPathUrlLow != updated.coverPathUrlLow {
            self.coverPathUrlLow = updated.coverPathUrlLow;
            self.coverPathLow = Image::default();
        }
        if !updated.coverPathUrlHigh.is_empty() && self.coverPathUrlHigh != updated.coverPathUrlHigh
        {
            self.coverPathUrlHigh = updated.coverPathUrlHigh;
            self.coverPathHigh = Image::default();
        }
        if updated.artists.row_count() > 0 {
            self.artists = updated.artists;
        }
        if updated.genre.row_count() > 0 {
            self.genre = updated.genre;
        }
    }
}

impl LazyModel for AlbumModel {
    #[tracing::instrument(level = "debug", skip_all)]
    fn get_id(&self) -> &SharedString { &self.id }

    #[tracing::instrument(level = "debug", skip_all)]
    fn set_cover(&mut self, image: Image) { self.coverPath = image; }

    #[tracing::instrument(level = "debug", skip_all)]
    fn get_cover_url(&self) -> &SharedString { &self.coverPathUrl }

    #[tracing::instrument(level = "debug", skip_all)]
    fn get_image(&self) -> &Image { &self.coverPath }

    #[tracing::instrument(level = "debug", skip_all)]
    fn update_from(&mut self, updated: Self) {
        if !updated.title.is_empty() {
            self.title = updated.title;
        }
        if updated.songs_count > 0 {
            self.songs_count = updated.songs_count;
        }
        if !updated.coverPathUrl.is_empty() && self.coverPathUrl != updated.coverPathUrl {
            self.coverPathUrl = updated.coverPathUrl;
            self.coverPath = Image::default();
        }
    }
}

impl LazyModel for PlaylistModel {
    #[tracing::instrument(level = "debug", skip_all)]
    fn get_id(&self) -> &SharedString { &self.id }

    #[tracing::instrument(level = "debug", skip_all)]
    fn set_cover(&mut self, image: Image) { self.coverPath = image; }

    #[tracing::instrument(level = "debug", skip_all)]
    fn get_cover_url(&self) -> &SharedString { &self.coverPathUrl }

    #[tracing::instrument(level = "debug", skip_all)]
    fn get_image(&self) -> &Image { &self.coverPath }

    #[tracing::instrument(level = "debug", skip_all)]
    fn update_from(&mut self, updated: Self) {
        if !updated.title.is_empty() {
            self.title = updated.title;
        }
        if updated.songs_count > 0 {
            self.songs_count = updated.songs_count;
        }
        if !updated.coverPathUrl.is_empty() && self.coverPathUrl != updated.coverPathUrl {
            self.coverPathUrl = updated.coverPathUrl;
            self.coverPath = Image::default();
        }
    }
}

impl LazyModel for ArtistModel {
    #[tracing::instrument(level = "debug", skip_all)]
    fn get_id(&self) -> &SharedString { &self.id }

    #[tracing::instrument(level = "debug", skip_all)]
    fn set_cover(&mut self, image: Image) { self.coverPath = image; }

    #[tracing::instrument(level = "debug", skip_all)]
    fn get_cover_url(&self) -> &SharedString { &self.coverPathUrl }

    #[tracing::instrument(level = "debug", skip_all)]
    fn get_image(&self) -> &Image { &self.coverPath }

    #[tracing::instrument(level = "debug", skip_all)]
    fn update_from(&mut self, updated: Self) {
        if !updated.title.is_empty() {
            self.title = updated.title;
        }
        if updated.songs_count > 0 {
            self.songs_count = updated.songs_count;
        }
        if !updated.coverPathUrl.is_empty() && self.coverPathUrl != updated.coverPathUrl {
            self.coverPathUrl = updated.coverPathUrl;
            self.coverPath = Image::default();
        }
    }
}

impl LazyModel for GenreModel {
    #[tracing::instrument(level = "debug", skip_all)]
    fn get_id(&self) -> &SharedString { &self.id }

    #[tracing::instrument(level = "debug", skip_all)]
    fn set_cover(&mut self, image: Image) { self.coverPath = image; }

    #[tracing::instrument(level = "debug", skip_all)]
    fn get_cover_url(&self) -> &SharedString { &self.coverPathUrl }

    #[tracing::instrument(level = "debug", skip_all)]
    fn get_image(&self) -> &Image { &self.coverPath }

    #[tracing::instrument(level = "debug", skip_all)]
    fn update_from(&mut self, updated: Self) {
        if !updated.title.is_empty() {
            self.title = updated.title;
        }
        if updated.songs_count > 0 {
            self.songs_count = updated.songs_count;
        }
        if !updated.coverPathUrl.is_empty() && self.coverPathUrl != updated.coverPathUrl {
            self.coverPathUrl = updated.coverPathUrl;
            self.coverPath = Image::default();
        }
    }
}
