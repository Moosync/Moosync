use std::{
    cell::{Cell, RefCell},
    collections::HashSet,
    path::Path,
    rc::Rc,
};

use slint::{Image, Model, ModelNotify, ModelTracker, SharedString};
use tracing::trace;

use crate::{AlbumModel, ArtistModel, GenreModel, PlaylistModel, SongModel, WINDOW_EVENTS};

pub static DEFAULT_SONG_SVG: &[u8] = include_bytes!("icons/song_default.svg");

pub trait LazyModel: Clone {
    fn set_cover(&mut self, image: Image);
    fn get_cover(&self) -> &Image;
    fn get_cover_url(&self) -> &SharedString;
}

pub struct LazySongVecModel<T: LazyModel> {
    array: RefCell<Vec<T>>,
    notify: Rc<ModelNotify>,
    max_items: Rc<Cell<usize>>,
    prefetch_count: Rc<Cell<usize>>,
    allocated_rows: RefCell<HashSet<usize>>,
}

impl<T: LazyModel + 'static> LazySongVecModel<T> {
    #[tracing::instrument(level = "trace", skip(array))]
    pub fn new(array: Vec<T>, item_height: usize, item_width: usize) -> Self {
        let notify = Rc::new(ModelNotify::default());
        let max_items = Rc::new(Cell::new(1));
        let max_items_clone = max_items.clone();
        let prefetch_count = Rc::new(Cell::new(0));
        let prefetch_count_clone = prefetch_count.clone();

        WINDOW_EVENTS.with(move |window_events| {
            window_events.on_resize(Box::new(move |window| {
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
                max_items_clone.set(new_max_items);
                prefetch_count_clone.set(2 * columns);
            }));
        });

        Self {
            array: RefCell::new(array),
            notify,
            max_items,
            prefetch_count,
            allocated_rows: RefCell::new(HashSet::new()),
        }
    }


    fn load_image(&self, row: usize, model: &mut T) {
        if !is_empty_image(&model.get_cover()) {
            return;
        }

        if model.get_cover_url().is_empty() {
            return;
        }

        trace!("Fetching image for row {}", row);
        if !model.get_cover_url().is_empty() {
            let image = Image::load_from_path(Path::new(&model.get_cover_url()))
                .unwrap_or(Image::load_from_svg_data(DEFAULT_SONG_SVG).unwrap());
            model.set_cover(image);
            self.allocated_rows.borrow_mut().insert(row);
            return;
        }

        let image = Image::load_from_svg_data(DEFAULT_SONG_SVG).unwrap();
        model.set_cover(image);
        self.allocated_rows.borrow_mut().insert(row);
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

    fn row_count(&self) -> usize {
        self.array.borrow().len()
    }

    fn row_data(&self, row: usize) -> Option<Self::Data> {
        let song_model = {
            let mut array = self.array.borrow_mut();
            
            // First load the requested item
            let song_model = array.get_mut(row)?;
            self.load_image(row, song_model);
            let cloned = song_model.clone();
            
            // Prefetch adjacent items (2 rows up and down)
            let prefetch = self.prefetch_count.get();
            if prefetch > 0 {
                let min_idx = row.saturating_sub(prefetch);
                let max_idx = (row + prefetch).min(array.len() - 1);
                for i in min_idx..=max_idx {
                    if i != row {
                        if let Some(item) = array.get_mut(i) {
                            self.load_image(i, item);
                        }
                    }
                }
            }
            
            cloned
        };

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

    fn model_tracker(&self) -> &dyn ModelTracker {
        &*self.notify
    }

    fn as_any(&self) -> &dyn core::any::Any {
        self
    }
}

fn is_empty_image(image: &Image) -> bool {
    let size = image.size();
    size.width == 0 && size.height == 0
}

impl LazyModel for SongModel {
    fn set_cover(&mut self, image: Image) {
        self.coverPathLow = image
    }

    fn get_cover(&self) -> &Image {
        &self.coverPathLow
    }

    fn get_cover_url(&self) -> &SharedString {
        &self.coverPathUrlLow
    }
}

impl LazyModel for AlbumModel {
    fn set_cover(&mut self, image: Image) {
        self.coverPath = image;
    }

    fn get_cover(&self) -> &Image {
        &self.coverPath
    }

    fn get_cover_url(&self) -> &SharedString {
        &self.coverPathUrl
    }
}

impl LazyModel for PlaylistModel {
    fn set_cover(&mut self, image: Image) {
        self.coverPath = image;
    }

    fn get_cover(&self) -> &Image {
        &self.coverPath
    }

    fn get_cover_url(&self) -> &SharedString {
        &self.coverPathUrl
    }
}

impl LazyModel for ArtistModel {
    fn set_cover(&mut self, image: Image) {
        self.coverPath = image;
    }

    fn get_cover(&self) -> &Image {
        &self.coverPath
    }

    fn get_cover_url(&self) -> &SharedString {
        &self.coverPathUrl
    }
}

impl LazyModel for GenreModel {
    fn set_cover(&mut self, image: Image) {
        self.coverPath = image;
    }

    fn get_cover(&self) -> &Image {
        &self.coverPath
    }

    fn get_cover_url(&self) -> &SharedString {
        &self.coverPathUrl
    }
}
