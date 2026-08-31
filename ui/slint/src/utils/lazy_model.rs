use std::{
    cell::{Cell, RefCell},
    collections::HashSet,
    path::PathBuf,
    rc::Rc,
};

use slint::{Image, Model, ModelNotify, ModelTracker, SharedString};
use tracing::trace;

use super::{default_song_cover, load_image_from_path_or_url};
use crate::{
    AlbumModel, ArtistModel, ExtensionItem, GenreModel, PlaylistModel, SongModel, WINDOW_EVENTS,
};

pub trait LazyModel: Clone {
    fn set_cover(&mut self, image: Image);
    fn get_cover_url(&self) -> &SharedString;
}

pub struct LazySongVecModel<T: LazyModel> {
    array: Rc<RefCell<Vec<T>>>,
    notify: Rc<ModelNotify>,
    max_items: Rc<Cell<usize>>,
    prefetch_count: Rc<Cell<usize>>,
    allocated_rows: Rc<RefCell<HashSet<usize>>>,
    cache_dir: PathBuf,
}

impl<T: LazyModel + 'static> LazySongVecModel<T> {
    #[tracing::instrument(level = "trace", skip_all)]
    pub fn new(array: Vec<T>, item_height: usize, item_width: usize, cache_dir: PathBuf) -> Self {
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
            max_items,
            prefetch_count,
            allocated_rows: Rc::new(RefCell::new(HashSet::new())),
            cache_dir,
        }
    }

    #[tracing::instrument(level = "debug", skip_all)]
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
            let Some(img) = load_image_from_path_or_url(&cover_url_str, &cache_dir).await else {
                allocated_rows.borrow_mut().remove(&row);
                return;
            };
            {
                let mut array = array.borrow_mut();
                let Some(item) = array.get_mut(row) else {
                    allocated_rows.borrow_mut().remove(&row);
                    return;
                };
                item.set_cover(img);
            }
            tracing::trace!("Loaded image for row {}", row);
            allocated_rows.borrow_mut().insert(row);
            notify.row_changed(row);
        })
        .unwrap();
    }

    #[tracing::instrument(level = "debug", skip_all)]
    fn release_image(&self, row: usize, model: &mut T) {
        trace!("Releasing image for row {}", row);
        model.set_cover(default_song_cover());
        self.allocated_rows.borrow_mut().remove(&row);
    }

    #[tracing::instrument(level = "debug", skip_all)]
    fn evict_furthest(&self, current_row: usize) {
        let max = self.max_items.get();
        let capacity = (max * 2).max(64);

        loop {
            let to_release = {
                let allocated = self.allocated_rows.borrow();
                if allocated.len() <= capacity {
                    break;
                }
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
            };

            let Some(r) = to_release else {
                break;
            };

            let mut array = self.array.borrow_mut();
            if let Some(s) = array.get_mut(r) {
                self.release_image(r, s);
            } else {
                self.allocated_rows.borrow_mut().remove(&r);
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
            let is_loaded = self.allocated_rows.borrow().contains(&row);
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
                            (!self.allocated_rows.borrow().contains(&i))
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
        if row >= self.row_count() {
            return;
        }

        self.allocated_rows.borrow_mut().insert(row);
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
}

impl LazyModel for SongModel {
    #[tracing::instrument(level = "debug", skip_all)]
    fn set_cover(&mut self, image: Image) { self.coverPathLow = image }

    #[tracing::instrument(level = "debug", skip_all)]
    fn get_cover_url(&self) -> &SharedString { &self.coverPathUrlLow }
}

impl LazyModel for AlbumModel {
    #[tracing::instrument(level = "debug", skip_all)]
    fn set_cover(&mut self, image: Image) { self.coverPath = image; }

    #[tracing::instrument(level = "debug", skip_all)]
    fn get_cover_url(&self) -> &SharedString { &self.coverPathUrl }
}

impl LazyModel for PlaylistModel {
    #[tracing::instrument(level = "debug", skip_all)]
    fn set_cover(&mut self, image: Image) { self.coverPath = image; }

    #[tracing::instrument(level = "debug", skip_all)]
    fn get_cover_url(&self) -> &SharedString { &self.coverPathUrl }
}

impl LazyModel for ArtistModel {
    #[tracing::instrument(level = "debug", skip_all)]
    fn set_cover(&mut self, image: Image) { self.coverPath = image; }

    #[tracing::instrument(level = "debug", skip_all)]
    fn get_cover_url(&self) -> &SharedString { &self.coverPathUrl }
}

impl LazyModel for GenreModel {
    #[tracing::instrument(level = "debug", skip_all)]
    fn set_cover(&mut self, image: Image) { self.coverPath = image; }

    #[tracing::instrument(level = "debug", skip_all)]
    fn get_cover_url(&self) -> &SharedString { &self.coverPathUrl }
}
