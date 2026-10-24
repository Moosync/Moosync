use std::{
    cell::{Cell, RefCell},
    collections::VecDeque,
    path::PathBuf,
    rc::Rc,
};

use slint::{ComponentHandle, Image, Model, ModelNotify, ModelRc, ModelTracker, SharedString};
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
    ModelRc::new(LazySongVecModel::new(
        songs,
        theme.get_songListItemHeight() as usize,
        theme.get_songListItemWidth() as usize,
        cache_dir,
        false,
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
    ModelRc::new(LazySongVecModel::new(
        items,
        theme.get_cardHeight() as usize,
        theme.get_cardWidth() as usize,
        cache_dir,
        true,
    ))
}

pub trait LazyModel: Clone {
    fn set_cover(&mut self, image: Image);
    fn get_cover_url(&self) -> &SharedString;
    fn get_image(&self) -> &Image;
    fn has_image_set(&self) -> bool { !self.get_image().size().is_empty() }
}

pub struct LazySongVecModel<T: LazyModel> {
    array: Rc<RefCell<Vec<T>>>,
    notify: Rc<ModelNotify>,
    max_items: Rc<Cell<usize>>,
    prefetch_count: Rc<Cell<usize>>,
    allocated_rows: RefCell<VecDeque<usize>>,
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

                let columns = width.checked_div(item_width).map_or(1, |c| c.max(1));
                let mut new_max_items = height.checked_div(item_height).unwrap_or(0);
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
            prefetch_count,
            max_items,
            allocated_rows: RefCell::new(VecDeque::new()),
            cache_dir,
            high_res,
        }
    }

    #[tracing::instrument(level = "debug", skip_all)]
    fn trim_allocated(&self, row: usize) {
        let mut array = self.array.borrow_mut();
        let mut allocated = self.allocated_rows.borrow_mut();
        allocated.push_back(row);

        while allocated.len() >= self.max_items.get() + (self.prefetch_count.get() * 2) {
            if let Some(oldest_row) = allocated.pop_front() {
                if let Some(model) = array.get_mut(oldest_row) {
                    model.set_cover(Image::default());
                }
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
        let song_model = {
            let array = self.array.borrow();
            array.get(row)?.clone()
        };

        if song_model.has_image_set() {
            return Some(song_model);
        }

        let cover_url = song_model.get_cover_url().to_string();
        let array = self.array.clone();
        let cache_dir = self.cache_dir.clone();
        let notify = self.notify.clone();
        let high_res = self.high_res;

        let _ = slint::spawn_local(
            async move {
                trace!("Fetching image for row {}; cover_url: {}", row, cover_url);
                let pixel_buffer = tokio::spawn(async move {
                    resolve_image_buffer(&cover_url, &cache_dir, high_res).await
                })
                .await
                .unwrap_or_default();
                trace!(
                    "Image fetched for row {}: {:?}",
                    row,
                    pixel_buffer.is_some()
                );
                if let Some(buffer) = pixel_buffer {
                    let image = Image::from_rgba8(buffer);
                    trace!("Image loaded for row {}: (size: {:?})", row, image.size());

                    {
                        array.borrow_mut()[row].set_cover(image);
                    }

                    notify.row_changed(row);
                    trace!("exiting");
                }
            }
            .in_current_span(),
        );

        self.trim_allocated(row);

        Some(song_model)
    }

    #[tracing::instrument(level = "debug", skip_all)]
    fn set_row_data(&self, row: usize, data: Self::Data) {
        if row >= self.row_count() {
            return;
        }

        self.array.borrow_mut()[row] = data;
        self.notify.row_changed(row);
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
    fn get_cover_url(&self) -> &SharedString { &self.icon_url }

    #[tracing::instrument(level = "debug", skip_all)]
    fn get_image(&self) -> &Image { &self.icon }
}

impl LazyModel for SongModel {
    #[tracing::instrument(level = "debug", skip_all)]
    fn set_cover(&mut self, image: Image) { self.coverPathLow = image }

    #[tracing::instrument(level = "debug", skip_all)]
    fn get_cover_url(&self) -> &SharedString { &self.coverPathUrlLow }

    #[tracing::instrument(level = "debug", skip_all)]
    fn get_image(&self) -> &Image { &self.coverPathLow }
}

impl LazyModel for AlbumModel {
    #[tracing::instrument(level = "debug", skip_all)]
    fn set_cover(&mut self, image: Image) { self.coverPath = image; }

    #[tracing::instrument(level = "debug", skip_all)]
    fn get_cover_url(&self) -> &SharedString { &self.coverPathUrl }

    #[tracing::instrument(level = "debug", skip_all)]
    fn get_image(&self) -> &Image { &self.coverPath }
}

impl LazyModel for PlaylistModel {
    #[tracing::instrument(level = "debug", skip_all)]
    fn set_cover(&mut self, image: Image) { self.coverPath = image; }

    #[tracing::instrument(level = "debug", skip_all)]
    fn get_cover_url(&self) -> &SharedString { &self.coverPathUrl }

    #[tracing::instrument(level = "debug", skip_all)]
    fn get_image(&self) -> &Image { &self.coverPath }
}

impl LazyModel for ArtistModel {
    #[tracing::instrument(level = "debug", skip_all)]
    fn set_cover(&mut self, image: Image) { self.coverPath = image; }

    #[tracing::instrument(level = "debug", skip_all)]
    fn get_cover_url(&self) -> &SharedString { &self.coverPathUrl }

    #[tracing::instrument(level = "debug", skip_all)]
    fn get_image(&self) -> &Image { &self.coverPath }
}

impl LazyModel for GenreModel {
    #[tracing::instrument(level = "debug", skip_all)]
    fn set_cover(&mut self, image: Image) { self.coverPath = image; }

    #[tracing::instrument(level = "debug", skip_all)]
    fn get_cover_url(&self) -> &SharedString { &self.coverPathUrl }

    #[tracing::instrument(level = "debug", skip_all)]
    fn get_image(&self) -> &Image { &self.coverPath }
}
