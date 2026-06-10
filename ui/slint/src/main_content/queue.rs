use std::sync::{Arc, Mutex};

use slint::ComponentHandle;
use state_manager::StateManager;
use types::prelude::SongsExt;

use crate::{AppCallbacks, MainWindow, pages::PageHandler};

pub struct QueuePageHandler<'a> {
    main_window: &'a MainWindow,
    state_manager: &'a StateManager,
    cancel_handles: Arc<Mutex<Vec<types::subscription::CancelHandle>>>,
    hide_timer: Arc<Mutex<slint::Timer>>,
    is_visible: Arc<Mutex<bool>>,
}

impl<'a> QueuePageHandler<'a> {
    pub fn new(main_window: &'a MainWindow, state_manager: &'a StateManager) -> Self {
        Self {
            main_window,
            state_manager,
            cancel_handles: Arc::new(Mutex::new(Vec::new())),
            hide_timer: Arc::new(Mutex::new(slint::Timer::default())),
            is_visible: Arc::new(Mutex::new(false)),
        }
    }

    fn register_ui_callbacks(&self) {
        let state_manager = self.state_manager.clone();

        self.main_window
            .global::<AppCallbacks>()
            .on_play_queue_index({
                let state_manager = state_manager.clone();
                move |idx| {
                    let state_manager = state_manager.clone();
                    tokio::spawn(async move {
                        let mut player_handler = state_manager.get_player_handler_mut().await;
                        player_handler.play_index(idx as usize);
                    });
                }
            });

        self.main_window
            .global::<AppCallbacks>()
            .on_remove_from_queue({
                let state_manager = state_manager.clone();
                move |idx| {
                    let state_manager = state_manager.clone();
                    tokio::spawn(async move {
                        let mut player_handler = state_manager.get_player_handler_mut().await;
                        player_handler.remove_from_queue(idx as usize);
                    });
                }
            });

        self.main_window.global::<AppCallbacks>().on_clear_queue({
            let state_manager = state_manager.clone();
            move || {
                let state_manager = state_manager.clone();
                tokio::spawn(async move {
                    let mut player_handler = state_manager.get_player_handler_mut().await;
                    player_handler.clear_queue();
                });
            }
        });

        self.main_window
            .global::<AppCallbacks>()
            .on_move_queue_item({
                let state_manager = state_manager.clone();
                move |from_idx_str, to_idx| {
                    if let Ok(from_idx) = from_idx_str.parse::<usize>() {
                        let state_manager = state_manager.clone();
                        tokio::spawn(async move {
                            let mut player_handler = state_manager.get_player_handler_mut().await;
                            player_handler.move_queue_item(from_idx, to_idx as usize);
                        });
                    }
                }
            });

        self.main_window
            .global::<AppCallbacks>()
            .on_string_to_transfer(move |text| slint::DataTransfer::from(text));

        self.main_window
            .global::<AppCallbacks>()
            .on_transfer_to_string(move |transfer| transfer.plain_text().unwrap_or_default());
    }

    fn fetch_initial_state(&self) {
        let main_window_weak = self.main_window.as_weak();
        let state_manager = self.state_manager.clone();

        tokio::spawn(async move {
            let player_handler = state_manager.get_player_handler().await;
            let queue = player_handler.get_queue().to_vec();
            let current_song = player_handler.get_current_song().cloned();
            let cache_dir = state_manager.get_cache_dir();

            let (song_id, cover_path_high) = match &current_song {
                Some(s) => (
                    s.get_id().unwrap_or_default(),
                    s.get_cover_high().unwrap_or_default(),
                ),
                None => ("".into(), "".into()),
            };

            let blurred_path = crate::utils::generate_blurred_cover_disk_cache(
                &song_id,
                &cover_path_high,
                &cache_dir,
            );

            let _ = slint::invoke_from_event_loop(move || {
                if let Some(main_window) = main_window_weak.upgrade() {
                    update_ui_queue(&main_window, &queue, cache_dir);
                    update_ui_blurred_cover(&main_window, &blurred_path);
                }
            });
        });
    }

    fn register_player_callbacks(&self) {
        let main_window_weak = self.main_window.as_weak();
        let state_manager = self.state_manager.clone();
        let cancel_handles = self.cancel_handles.clone();

        tokio::spawn(async move {
            let player_handler = state_manager.get_player_handler().await;
            let cache_dir = state_manager.get_cache_dir();
            let mut handles = Vec::new();

            // Song changed listener to update blurred cover background
            let mw_weak_song = main_window_weak.clone();
            let cache_dir_events = cache_dir.clone();
            let ch_song = player_handler.on_song_changed(move |song| {
                let mw_weak = mw_weak_song.clone();
                let song = song.cloned();
                let cache_dir = cache_dir_events.clone();

                tokio::spawn(async move {
                    let (song_id, cover_path_high) = match &song {
                        Some(s) => (
                            s.get_id().unwrap_or_default().to_string(),
                            s.get_cover_high().unwrap_or_default().to_string(),
                        ),
                        None => (String::new(), String::new()),
                    };

                    let blurred_path = crate::utils::generate_blurred_cover_disk_cache(
                        &song_id,
                        &cover_path_high,
                        &cache_dir,
                    );

                    let _ = slint::invoke_from_event_loop(move || {
                        if let Some(main_window) = mw_weak.upgrade() {
                            update_ui_blurred_cover(&main_window, &blurred_path);
                        }
                    });
                });
            });
            handles.push(ch_song);

            let mw_weak_queue = main_window_weak.clone();
            let cache_dir_queue = cache_dir.clone();
            let ch_queue = player_handler.on_queue_updated(move |queue| {
                let queue_cloned = queue.to_vec();
                let mw_weak = mw_weak_queue.clone();
                let cache_dir = cache_dir_queue.clone();
                let _ = slint::invoke_from_event_loop(move || {
                    if let Some(main_window) = mw_weak.upgrade() {
                        update_ui_queue(&main_window, &queue_cloned, cache_dir);
                    }
                });
            });
            handles.push(ch_queue);

            *cancel_handles.lock().unwrap() = handles;
        });
    }
}

fn update_ui_queue(
    main_window: &MainWindow,
    queue: &[songs_proto::moosync::types::Song],
    cache_dir: std::path::PathBuf,
) {
    let queue_models: Vec<crate::SongModel> =
        queue.iter().map(crate::utils::to_song_model).collect();
    main_window.set_queue(slint::ModelRc::new(crate::utils::LazySongVecModel::new(
        queue_models,
        60,
        0,
        cache_dir,
    )));
}

fn update_ui_blurred_cover(main_window: &MainWindow, blurred_path: &Option<std::path::PathBuf>) {
    let blurred_cover = if let Some(path) = blurred_path {
        slint::Image::load_from_path(path).unwrap_or_else(|_| {
            slint::Image::load_from_svg_data(crate::utils::DEFAULT_SONG_SVG).unwrap()
        })
    } else {
        slint::Image::load_from_svg_data(crate::utils::DEFAULT_SONG_SVG).unwrap()
    };
    main_window.set_blurred_cover(blurred_cover);
}

impl<'a> PageHandler for QueuePageHandler<'a> {
    fn initialize(&self) { self.register_ui_callbacks(); }

    fn on_show(&self) {
        *self.is_visible.lock().unwrap() = true;
        self.hide_timer.lock().unwrap().stop();

        self.fetch_initial_state();
        self.register_player_callbacks();
    }

    fn on_hide(&self) {
        *self.is_visible.lock().unwrap() = false;

        let main_window_weak = self.main_window.as_weak();
        let cancel_handles = self.cancel_handles.clone();
        let is_visible = self.is_visible.clone();

        self.hide_timer.lock().unwrap().start(
            slint::TimerMode::SingleShot,
            std::time::Duration::from_millis(250),
            move || {
                if !*is_visible.lock().unwrap() {
                    let mut handles = cancel_handles.lock().unwrap();
                    for handle in handles.drain(..) {
                        handle.cancel();
                    }

                    if let Some(main_window) = main_window_weak.upgrade() {
                        main_window.set_queue(slint::ModelRc::default());
                        main_window.set_blurred_cover(
                            slint::Image::load_from_svg_data(crate::utils::DEFAULT_SONG_SVG)
                                .unwrap(),
                        );
                    }
                }
            },
        );
    }
}
