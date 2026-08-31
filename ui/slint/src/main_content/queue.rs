use std::{
    cell::RefCell,
    path::PathBuf,
    sync::{Arc, Mutex},
};

use slint::{ComponentHandle, Image, ModelRc, Timer};
use songs_proto::moosync::types::Song;
use state_manager::StateManager;
use types::{prelude::SongsExt, subscription::CancelHandle};

use crate::{
    AppCallbacks, ContextMenuCallbacks, MainWindow, QueuePageProps, SongModel, Theme,
    pages::PageHandler,
    utils::{
        LazySongVecModel, build_queue_context_menu_items, default_song_cover,
        dispatch_song_context_action, save_queue,
    },
};

pub struct QueuePageHandler<'a> {
    main_window: &'a MainWindow,
    state_manager: &'a StateManager,
    cancel_handles: Arc<Mutex<Vec<CancelHandle>>>,
    hide_timer: RefCell<Timer>,
    is_visible: Arc<Mutex<bool>>,
}

impl<'a> QueuePageHandler<'a> {
    #[tracing::instrument(level = "debug", skip_all)]
    pub fn new(main_window: &'a MainWindow, state_manager: &'a StateManager) -> Self {
        Self {
            main_window,
            state_manager,
            cancel_handles: Arc::new(Mutex::new(Vec::new())),
            hide_timer: RefCell::new(slint::Timer::default()),
            is_visible: Arc::new(Mutex::new(false)),
        }
    }

    #[tracing::instrument(level = "debug", skip_all)]
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
            .on_save_queue_as_playlist({
                let state_manager = state_manager.clone();
                move |name, desc| {
                    let state_manager = state_manager.clone();
                    let name_str = name.to_string();
                    let desc_str = desc.to_string();
                    tokio::spawn(async move {
                        save_queue(&state_manager, name_str, desc_str).await;
                    });
                }
            });

        self.main_window
            .global::<AppCallbacks>()
            .on_string_to_transfer(slint::DataTransfer::from);

        self.main_window
            .global::<AppCallbacks>()
            .on_transfer_to_string(move |transfer| transfer.plain_text().unwrap_or_default());
    }

    #[tracing::instrument(level = "debug", skip_all)]
    fn fetch_initial_state(state_manager: StateManager, main_window_weak: slint::Weak<MainWindow>) {
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
                    Self::update_ui_queue(&main_window, &state_manager, queue);
                    Self::update_ui_blurred_cover(&main_window, &blurred_path);
                }
            });
        });
    }

    #[tracing::instrument(level = "debug", skip_all)]
    fn register_player_callbacks(
        state_manager: StateManager,
        main_window_weak: slint::Weak<MainWindow>,
        cancel_handles: Arc<Mutex<Vec<types::subscription::CancelHandle>>>,
    ) {
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
                            Self::update_ui_blurred_cover(&main_window, &blurred_path);
                        }
                    });
                });
            });
            handles.push(ch_song);

            let mw_weak_queue = main_window_weak.clone();
            let state_manager_queue = state_manager.clone();
            let ch_queue = player_handler.on_queue_updated(move |queue| {
                let queue_cloned = queue.to_vec();
                let mw_weak = mw_weak_queue.clone();
                let state_manager = state_manager_queue.clone();
                let _ = slint::invoke_from_event_loop(move || {
                    if let Some(main_window) = mw_weak.upgrade() {
                        Self::update_ui_queue(&main_window, &state_manager, queue_cloned);
                    }
                });
            });
            handles.push(ch_queue);

            *cancel_handles.lock().unwrap() = handles;
        });
    }

    #[tracing::instrument(level = "debug", skip_all)]
    fn update_ui_queue(main_window: &MainWindow, state_manager: &StateManager, queue: Vec<Song>) {
        let queue_models: Vec<SongModel> = queue.into_iter().map(Into::into).collect();
        let theme = main_window.global::<Theme>();
        let cache_dir = state_manager.get_cache_dir();
        main_window
            .global::<QueuePageProps>()
            .set_queue(ModelRc::new(LazySongVecModel::new(
                queue_models,
                theme.get_songListItemHeight() as usize,
                theme.get_songListItemWidth() as usize,
                cache_dir,
            )));
    }

    #[tracing::instrument(level = "debug", skip_all)]
    fn update_ui_blurred_cover(main_window: &MainWindow, blurred_path: &Option<PathBuf>) {
        let blurred_cover = blurred_path
            .as_deref()
            .and_then(|path| Image::load_from_path(path).ok())
            .unwrap_or_else(default_song_cover);
        main_window
            .global::<QueuePageProps>()
            .set_blurred_cover(blurred_cover);
    }

    #[tracing::instrument(level = "debug", skip_all)]
    fn register_context_menu_callbacks(&self) {
        let main_window_weak = self.main_window.as_weak();
        let state_manager_clone = self.state_manager.clone();

        self.main_window
            .global::<ContextMenuCallbacks>()
            .on_get_queue_menu_items(move |song_models, idx| {
                let Some(main_window) = main_window_weak.upgrade() else {
                    return ModelRc::default();
                };

                build_queue_context_menu_items(
                    &main_window,
                    &state_manager_clone,
                    &song_models,
                    idx,
                )
            });

        let state_manager = self.state_manager.clone();
        let main_window_weak = self.main_window.as_weak();
        self.main_window
            .global::<ContextMenuCallbacks>()
            .on_queue_action(move |song_models, idx, action_id| {
                if action_id == "play_now" {
                    let state_manager = state_manager.clone();
                    let queue_idx = idx as usize;
                    tokio::spawn(async move {
                        let mut player = state_manager.get_player_handler_mut().await;
                        player.play_index(queue_idx);
                    });
                    return;
                }

                if action_id == "remove_from_queue" {
                    let state_manager = state_manager.clone();
                    let queue_idx = idx as usize;
                    tokio::spawn(async move {
                        let mut player = state_manager.get_player_handler_mut().await;
                        player.remove_from_queue(queue_idx);
                    });
                    return;
                }

                dispatch_song_context_action(
                    &main_window_weak,
                    &state_manager,
                    &song_models,
                    action_id.as_str(),
                );
            });
    }
}

impl<'a> PageHandler for QueuePageHandler<'a> {
    #[tracing::instrument(level = "debug", skip_all)]
    fn initialize(&self) {
        self.register_ui_callbacks();
        self.register_context_menu_callbacks();
    }

    #[tracing::instrument(level = "debug", skip_all)]
    fn on_show(&self) {
        *self.is_visible.lock().unwrap() = true;
        self.hide_timer.borrow().stop();

        Self::fetch_initial_state(self.state_manager.clone(), self.main_window.as_weak());
        Self::register_player_callbacks(
            self.state_manager.clone(),
            self.main_window.as_weak(),
            self.cancel_handles.clone(),
        );
    }

    #[tracing::instrument(level = "debug", skip_all)]
    fn on_hide(&self) {
        *self.is_visible.lock().unwrap() = false;

        let main_window_weak = self.main_window.as_weak();
        let cancel_handles = self.cancel_handles.clone();
        let is_visible = self.is_visible.clone();

        self.hide_timer.borrow().start(
            slint::TimerMode::SingleShot,
            std::time::Duration::from_millis(250),
            move || {
                if !*is_visible.lock().unwrap() {
                    let mut handles = cancel_handles.lock().unwrap();
                    for handle in handles.drain(..) {
                        handle.cancel();
                    }

                    if let Some(main_window) = main_window_weak.upgrade() {
                        main_window
                            .global::<QueuePageProps>()
                            .set_queue(slint::ModelRc::default());
                        main_window
                            .global::<QueuePageProps>()
                            .set_blurred_cover(crate::utils::default_song_cover());
                    }
                }
            },
        );
    }
}
