use slint::ComponentHandle;
use state_manager::StateManager;
use types::prelude::SongsExt;

use crate::{AppCallbacks, MainWindow, pages::PageHandler};

pub struct QueuePageHandler<'a> {
    main_window: &'a MainWindow,
    state_manager: &'a StateManager,
}

impl<'a> QueuePageHandler<'a> {
    pub fn new(main_window: &'a MainWindow, state_manager: &'a StateManager) -> Self {
        Self {
            main_window,
            state_manager,
        }
    }
}

impl<'a> PageHandler for QueuePageHandler<'a> {
    fn initialize(&self) {
        let state_manager = self.state_manager.clone();

        let main_window_weak = self.main_window.as_weak();
        let state_manager_events = self.state_manager.clone();
        tokio::spawn(async move {
            let mut player_handler = state_manager_events.get_player_handler_mut().await;

            // 1. Song changed listener to update blurred cover background
            let cache_dir = state_manager_events.get_cache_dir();
            let mw_weak_song = main_window_weak.clone();
            player_handler.on_song_changed(move |song| {
                let mw_weak = mw_weak_song.clone();
                let song = song.cloned();
                let cache_dir = cache_dir.clone();

                tokio::spawn(async move {
                    let (song_id, cover_path_high) = match &song {
                        Some(s) => (
                            s.get_id().unwrap_or_default(),
                            s.get_cover_high().unwrap_or_default(),
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
                            let blurred_cover = if let Some(path) = &blurred_path {
                                slint::Image::load_from_path(path).unwrap_or_else(|_| {
                                    slint::Image::load_from_svg_data(crate::utils::DEFAULT_SONG_SVG)
                                        .unwrap()
                                })
                            } else {
                                slint::Image::load_from_svg_data(crate::utils::DEFAULT_SONG_SVG)
                                    .unwrap()
                            };

                            main_window.set_blurred_cover(blurred_cover);
                        }
                    });
                });
            });

            // 2. Queue updated listener to update queue list
            let mw_weak_queue = main_window_weak.clone();
            player_handler.on_queue_updated(move |queue| {
                let queue_cloned = queue.to_vec();
                let mw_weak = mw_weak_queue.clone();
                let _ = slint::invoke_from_event_loop(move || {
                    if let Some(main_window) = mw_weak.upgrade() {
                        let queue_models: Vec<crate::SongModel> = queue_cloned
                            .iter()
                            .map(crate::utils::to_song_model)
                            .collect();
                        main_window
                            .set_queue(slint::ModelRc::new(slint::VecModel::from(queue_models)));
                    }
                });
            });
        });

        self.main_window
            .global::<AppCallbacks>()
            .on_play_queue_index(move |idx| {
                let state_manager = state_manager.clone();
                tokio::spawn(async move {
                    let mut player_handler = state_manager.get_player_handler_mut().await;
                    player_handler.play_index(idx as usize);
                });
            });

        let state_manager = self.state_manager.clone();
        self.main_window
            .global::<AppCallbacks>()
            .on_remove_from_queue(move |idx| {
                let state_manager = state_manager.clone();
                tokio::spawn(async move {
                    let mut player_handler = state_manager.get_player_handler_mut().await;
                    player_handler.remove_from_queue(idx as usize);
                });
            });

        let state_manager = self.state_manager.clone();
        self.main_window
            .global::<AppCallbacks>()
            .on_clear_queue(move || {
                let state_manager = state_manager.clone();
                tokio::spawn(async move {
                    let mut player_handler = state_manager.get_player_handler_mut().await;
                    player_handler.clear_queue();
                });
            });

        let state_manager = self.state_manager.clone();
        self.main_window
            .global::<AppCallbacks>()
            .on_move_queue_item(move |from_idx_str, to_idx| {
                if let Ok(from_idx) = from_idx_str.parse::<usize>() {
                    let state_manager = state_manager.clone();
                    tokio::spawn(async move {
                        let mut player_handler = state_manager.get_player_handler_mut().await;
                        player_handler.move_queue_item(from_idx, to_idx as usize);
                    });
                }
            });

        self.main_window
            .global::<AppCallbacks>()
            .on_string_to_transfer(move |text| slint::DataTransfer::from(text));

        self.main_window
            .global::<AppCallbacks>()
            .on_transfer_to_string(move |transfer| transfer.fetch_plaintext().unwrap_or_default());
    }

    fn on_show(&self) {}
    fn on_hide(&self) {}
}
