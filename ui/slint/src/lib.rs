// Manually find the generated files since bazel doesn't set vars for slint
// slint::include_modules!();
include!(concat!(env!("OUT_DIR"), "/app.rs"));

#[cfg(target_os = "android")]
use std::sync::OnceLock;
use std::{path::PathBuf, thread::available_parallelism, time::Duration};

use extensions_proto::moosync::types::{
    PlayerEvent as PlayerEventMessage, player_event::Event as PlayerEvent,
};
use player_proto::moosync::types::RepeatMode;
#[cfg(not(target_os = "android"))]
use preferences::keys::{MINIMIZE_TO_TRAY, PreferenceItemExt};
use slint::{Image, Model, ModelRc, VecModel};
use songs_proto::moosync::types::Song;
use state_manager::StateManager;
use tokio::runtime::Builder;
#[cfg(target_os = "android")]
use tokio::runtime::Runtime;
use tracing::{Instrument, debug, trace};
use types::{debounce::DebounceHandle, prelude::format_duration};

#[cfg(target_os = "android")]
use crate::utils::android::get_android_context;
use crate::{
    pages::{NavigationManager, PageLifecycleManager},
    utils::{find_cached_or_local_image, load_image, resolve_image, update_lazy_model_item},
};

pub mod accounts;
#[cfg(test)]
mod accounts_test;
#[cfg(target_os = "android")]
mod android_bridge;
pub mod error;
mod main_content;
mod pages;
pub mod settings;
#[cfg(not(target_os = "android"))]
pub mod test_utils;
pub mod utils;
mod window_info;

#[cfg(test)]
mod lib_test;
#[cfg(test)]
mod pages_test;
#[cfg(test)]
mod window_info_test;

pub use window_info::{WINDOW_EVENTS, WindowEvents};

#[cfg(not(target_os = "android"))]
#[tracing::instrument(level = "debug", skip_all)]
pub fn run() {
    let threads = available_parallelism()
        .map(|parallelism| parallelism.get().clamp(2, 4))
        .unwrap_or(4);

    let runtime = Builder::new_multi_thread()
        .worker_threads(threads)
        .max_blocking_threads(threads * 2)
        .enable_all()
        .build()
        .expect("Failed to build Tokio runtime");

    let _guard = runtime.enter();

    setup_tracing();
    debug!("Starting Moosync...");

    let main_window = MainWindow::new().unwrap();
    let state_manager = StateManager::new().expect("StateManager::new failed");

    setup_ui(&main_window, &state_manager);
    let _tray = setup_tray(&main_window, &state_manager);

    runtime.block_on(
        async {
            state_manager.setup().await;
        }
        .in_current_span(),
    );

    settings::apply_saved_language(&state_manager);

    main_window.show().unwrap();

    runtime.block_on(
        async {
            state_manager.delayed_setup().await;
        }
        .in_current_span(),
    );

    setup_signal_handlers();

    slint::run_event_loop().unwrap();

    runtime.block_on(
        async {
            state_manager.shutdown().await;
        }
        .in_current_span(),
    );
}

#[tracing::instrument(level = "debug", skip_all)]
fn setup_signal_handlers() {
    tokio::spawn(
        async move {
            #[cfg(unix)]
            {
                use tokio::signal::unix::{SignalKind, signal};
                match signal(SignalKind::terminate()) {
                    Ok(mut sigterm) => {
                        tokio::select! {
                            res = tokio::signal::ctrl_c() => {
                                if let Err(e) = res {
                                    tracing::error!("Failed to listen for ctrl_c: {:?}", e);
                                    return;
                                }
                            }
                            _ = sigterm.recv() => {}
                        }
                    }
                    Err(e) => {
                        tracing::error!("Failed to register SIGTERM handler: {:?}", e);
                        if let Err(err) = tokio::signal::ctrl_c().await {
                            tracing::error!("Failed to listen for ctrl_c: {:?}", err);
                            return;
                        }
                    }
                }
            }

            #[cfg(not(unix))]
            {
                if let Err(e) = tokio::signal::ctrl_c().await {
                    tracing::error!("Failed to listen for ctrl_c: {:?}", e);
                    return;
                }
            }

            tracing::info!("Received termination signal, shutting down...");
            if let Err(e) = slint::quit_event_loop() {
                tracing::error!("Failed to quit Slint event loop: {:?}", e);
            }
        }
        .in_current_span(),
    );
}

#[cfg(target_os = "android")]
static TOKIO_RUNTIME: OnceLock<Runtime> = OnceLock::new();

#[cfg(target_os = "android")]
static STATE_MANAGER: OnceLock<StateManager> = OnceLock::new();

#[cfg(target_os = "android")]
#[tracing::instrument(level = "debug", skip_all)]
pub fn run() {
    let is_first_run = TOKIO_RUNTIME.get().is_none();
    let runtime = TOKIO_RUNTIME.get_or_init(|| {
        let threads = available_parallelism()
            .map(|parallelism| parallelism.get().clamp(2, 4))
            .unwrap_or(4);

        Builder::new_multi_thread()
            .worker_threads(threads)
            .max_blocking_threads(threads * 2)
            .enable_all()
            .build()
            .expect("Failed to build Tokio runtime")
    });

    let _guard = runtime.enter();

    if is_first_run {
        setup_tracing();
        debug!("Starting Moosync Android...");
    }

    let android_context = get_android_context();

    let state_manager = STATE_MANAGER
        .get_or_init(|| StateManager::new(android_context).expect("StateManager::new failed"));

    let main_window = MainWindow::new().unwrap();

    setup_ui(&main_window, state_manager);

    if is_first_run {
        runtime.block_on(
            async {
                state_manager.setup().await;
            }
            .in_current_span(),
        );

        settings::apply_saved_language(state_manager);
    }

    main_window.show().unwrap();

    if is_first_run {
        runtime.block_on(
            async {
                state_manager.delayed_setup().await;
            }
            .in_current_span(),
        );
    }

    slint::run_event_loop().unwrap();
}

#[tracing::instrument(level = "debug", skip_all)]
fn setup_resize(main_window: &MainWindow) {
    let window = main_window.window();
    let scale = window.scale_factor();
    let width = (window.size().width as f32 / scale) as usize;
    let height = (window.size().height as f32 / scale) as usize;
    WINDOW_EVENTS.with(|we| we.set_last_size(width, height));

    let main_window_weak = main_window.as_weak();
    main_window.global::<AppCallbacks>().on_resize(move || {
        if let Some(main_window) = main_window_weak.upgrade() {
            WINDOW_EVENTS.with(|we| we.trigger_resize(main_window.window()));
        }
    });
}

#[cfg(not(target_os = "android"))]
#[tracing::instrument(level = "debug", skip_all)]
fn setup_tray(main_window: &MainWindow, state_manager: &StateManager) -> Option<AppTray> {
    let tray = match AppTray::new() {
        Ok(t) => t,
        Err(e) => {
            tracing::error!("Failed to initialize system tray: {:?}", e);
            return None;
        }
    };

    let mw_weak = main_window.as_weak();
    tray.on_show_window(move || {
        if let Some(mw) = mw_weak.upgrade() {
            let _ = mw.show();
        }
    });

    tray.on_quit(move || {
        let _ = slint::quit_event_loop();
    });

    let mw_weak = main_window.as_weak();
    let state_manager = state_manager.clone();
    main_window.window().on_close_requested(move || {
        let minimize_to_tray = state_manager
            .get_preference_config()
            .load(&MINIMIZE_TO_TRAY)
            .value::<bool>()
            .unwrap_or(false);

        if minimize_to_tray {
            if let Some(mw) = mw_weak.upgrade() {
                let _ = mw.hide();
            }
            slint::CloseRequestResponse::HideWindow
        } else {
            let _ = slint::quit_event_loop();
            slint::CloseRequestResponse::HideWindow
        }
    });

    Some(tray)
}

#[tracing::instrument(level = "debug", skip_all)]
fn handle_cover_fetch<F>(
    mw_weak: slint::Weak<MainWindow>,
    cache_dir: PathBuf,
    song_id: String,
    cover_url: String,
    update_cover: F,
) -> Image
where
    F: Fn(&MainWindow, Image) + Send + 'static,
{
    if let Some(image) = find_cached_or_local_image(cover_url.as_str(), &cache_dir) {
        return image;
    }
    if !cover_url.is_empty() {
        tokio::spawn(
            async move {
                let Some(img) = resolve_image(cover_url.as_str(), &cache_dir).await else {
                    return;
                };

                let _ = slint::invoke_from_event_loop(move || {
                    let Some(main_window) = mw_weak.upgrade() else {
                        return;
                    };
                    let player_props = main_window.global::<PlayerProps>();
                    let current = player_props.get_current_song();
                    if current.id == song_id {
                        update_cover(&main_window, load_image(&img));
                    }
                });
            }
            .in_current_span(),
        );
    }
    Image::default()
}

#[tracing::instrument(level = "debug", skip_all)]
fn setup_cover_helper(main_window: &MainWindow, state_manager: &StateManager) {
    let mw_high = main_window.as_weak();
    let cache_dir_high = state_manager.get_cache_dir();
    main_window
        .global::<CoverHelper>()
        .on_fetch_cover_high(move |song_model| {
            trace!("Fetching high-res cover for song {}", song_model.title);
            let song_id = song_model.id.to_string();
            let cover_url = song_model.coverPathUrlHigh.to_string();
            handle_cover_fetch(
                mw_high.clone(),
                cache_dir_high.clone(),
                song_id,
                cover_url,
                |main_window, image| {
                    let player_props = main_window.global::<PlayerProps>();
                    let mut current = player_props.get_current_song();
                    current.coverPathHigh = image;
                    player_props.set_current_song(current);
                },
            )
        });

    let mw_low = main_window.as_weak();
    let cache_dir_low = state_manager.get_cache_dir();
    main_window
        .global::<CoverHelper>()
        .on_fetch_cover_low(move |song_model| {
            trace!("Fetching low-res cover for song {}", song_model.title);
            let song_id = song_model.id.to_string();
            let cover_url = song_model.coverPathUrlLow.to_string();
            handle_cover_fetch(
                mw_low.clone(),
                cache_dir_low.clone(),
                song_id,
                cover_url,
                |main_window, image| {
                    let player_props = main_window.global::<PlayerProps>();
                    let mut current = player_props.get_current_song();
                    current.coverPathLow = image;
                    player_props.set_current_song(current);
                },
            )
        });
}

#[tracing::instrument(level = "debug", skip_all)]
fn setup_song_list_helper(main_window: &MainWindow, state_manager: &StateManager) {
    let main_window_weak = main_window.as_weak();
    let cache_dir = state_manager.get_cache_dir();
    main_window
        .global::<AppCallbacks>()
        .on_filter_and_sort_songs(move |songs, query, criterion, ascending| {
            let Some(main_window) = main_window_weak.upgrade() else {
                return songs;
            };
            let theme = main_window.global::<Theme>();
            utils::filter_and_sort_songs(
                songs,
                &query,
                criterion,
                ascending,
                theme.get_songListItemHeight() as usize,
                theme.get_songListItemWidth() as usize,
                cache_dir.clone(),
            )
        });

    main_window.global::<UtilCallbacks>().on_update_selection(
        move |current, clicked, anchor, is_ctrl, is_shift, is_right, total| {
            let current_vec: Vec<i32> = (0..current.row_count())
                .filter_map(|i| current.row_data(i))
                .collect();
            let res = utils::update_selection(
                &current_vec,
                clicked,
                anchor,
                is_ctrl,
                is_shift,
                is_right,
                total as usize,
            );
            ModelRc::new(VecModel::from(res))
        },
    );

    main_window
        .global::<UtilCallbacks>()
        .on_is_index_selected(move |selected, index| {
            (0..selected.row_count()).any(|i| selected.row_data(i) == Some(index))
        });

    main_window.global::<UtilCallbacks>().on_get_selected_songs(
        move |display_songs, selected_indices| {
            if selected_indices.row_count() == 0 {
                return ModelRc::default();
            }
            let songs: Vec<SongModel> = (0..selected_indices.row_count())
                .filter_map(|i| selected_indices.row_data(i))
                .filter(|&idx| idx >= 0)
                .filter_map(|idx| display_songs.row_data(idx as usize))
                .collect();
            ModelRc::new(VecModel::from(songs))
        },
    );

    main_window
        .global::<UtilCallbacks>()
        .on_select_all(move |total| {
            let res = utils::select_all(total as usize);
            ModelRc::new(VecModel::from(res))
        });

    main_window
        .global::<UtilCallbacks>()
        .on_validate_input(move |value, rule| utils::validate_input(value.as_str(), rule.as_str()));

    main_window
        .global::<UtilCallbacks>()
        .on_has_active_extension_providers(move |providers| {
            (0..providers.row_count())
                .filter_map(|i| providers.row_data(i))
                .any(|p| p.enabled)
        });

    main_window
        .global::<UtilCallbacks>()
        .on_options_to_context_items(move |options| {
            let items: Vec<ContextMenuItem> = (0..options.row_count())
                .filter_map(|i| options.row_data(i))
                .map(|opt| ContextMenuItem {
                    action_id: if opt.id.is_empty() {
                        opt.label.clone()
                    } else {
                        opt.id.clone()
                    },
                    title: if opt.label.is_empty() {
                        opt.id
                    } else {
                        opt.label
                    },
                    ..Default::default()
                })
                .collect();
            ModelRc::new(VecModel::from(items))
        });
}

#[tracing::instrument(level = "debug", skip_all)]
fn setup_song_cbs(main_window: &MainWindow, state_manager: &StateManager) {
    let sm = state_manager.clone();
    main_window
        .global::<AppCallbacks>()
        .on_play_song(move |song_model| {
            let song = Song::from(song_model);
            let sm = sm.clone();
            tokio::spawn(
                async move {
                    let queue = sm.get_player_handler();
                    queue.play_now(vec![song]).await;
                }
                .instrument(tracing::debug_span!("slint_cb_on_play_song")),
            );
        });

    let sm = state_manager.clone();
    main_window
        .global::<AppCallbacks>()
        .on_add_song_to_queue(move |song_model| {
            let song = Song::from(song_model);
            let sm = sm.clone();
            tokio::spawn(
                async move {
                    let queue = sm.get_player_handler();
                    queue.add_to_queue(vec![song]).await;
                }
                .instrument(tracing::debug_span!("slint_cb_on_add_song_to_queue")),
            );
        });

    let sm = state_manager.clone();
    main_window
        .global::<AppCallbacks>()
        .on_song_detail_action(move |action, song_models| {
            let songs = (0..song_models.row_count())
                .filter_map(|i| song_models.row_data(i))
                .map(Song::from)
                .collect::<Vec<_>>();
            let sm = sm.clone();
            tokio::spawn(
                async move {
                    let queue = sm.get_player_handler();
                    match action {
                        SongDetailAction::Play => {
                            queue.play_now(songs).await;
                        }
                        SongDetailAction::AddToQueue => {
                            queue.add_to_queue(songs).await;
                        }
                    }
                }
                .instrument(tracing::debug_span!("slint_cb_on_song_detail_action")),
            );
        });

    let main_window_weak = main_window.as_weak();
    let sm = state_manager.clone();
    main_window
        .global::<BottomBarCallbacks>()
        .on_play_pause_clicked(move || {
            let Some(main_window) = main_window_weak.upgrade() else {
                return;
            };
            let currently_playing = main_window.global::<PlayerProps>().get_playing();
            let sm = sm.clone();
            tokio::spawn(
                async move {
                    let player_handler = sm.get_player_handler();
                    if currently_playing {
                        let _ = player_handler.pause().await;
                    } else {
                        let _ = player_handler.play().await;
                    }
                }
                .instrument(tracing::debug_span!("slint_cb_on_play_pause_clicked")),
            );
        });

    let sm = state_manager.clone();
    main_window
        .global::<BottomBarCallbacks>()
        .on_toggle_repeat(move || {
            let sm = sm.clone();
            tokio::spawn(
                async move {
                    let player_handler = sm.get_player_handler();
                    let next_mode = match player_handler.get_repeat_mode() {
                        RepeatMode::RepeatNone => RepeatMode::RepeatOnce,
                        RepeatMode::RepeatOnce => RepeatMode::RepeatInfinite,
                        RepeatMode::RepeatInfinite => RepeatMode::RepeatNone,
                    };
                    player_handler.repeat(next_mode);
                }
                .instrument(tracing::debug_span!("slint_cb_on_toggle_repeat")),
            );
        });
}

#[tracing::instrument(level = "debug", skip_all)]
fn resolve_current_song_covers(
    main_window: &MainWindow,
    state_manager: &StateManager,
    song_model: SongModel,
) {
    let song_id = song_model.id.clone();
    if song_id.is_empty() {
        return;
    }
    let mw_weak = main_window.as_weak();
    let cache_dir = state_manager.get_cache_dir();
    let low_url = song_model.coverPathUrlLow.clone();
    let high_url = song_model.coverPathUrlHigh.clone();

    tokio::spawn(
        async move {
            let low_img = resolve_image(low_url.as_str(), &cache_dir).await;
            let high_img = resolve_image(high_url.as_str(), &cache_dir).await;
            if low_img.is_none() && high_img.is_none() {
                return;
            }

            let _ = slint::invoke_from_event_loop(move || {
                let low_img = low_img.as_ref().map(load_image).unwrap_or_default();
                let high_img = high_img.as_ref().map(load_image).unwrap_or_default();

                let Some(main_window) = mw_weak.upgrade() else {
                    return;
                };
                let player_props = main_window.global::<PlayerProps>();
                let mut current = player_props.get_current_song();
                if current.id != song_id {
                    return;
                }

                current.coverPathLow = low_img;
                current.coverPathHigh = high_img;

                player_props.set_current_song(current);
            });
        }
        .in_current_span(),
    );
}

#[tracing::instrument(level = "debug", skip_all)]
pub(crate) fn update_player_event_props(player_props: &PlayerProps, event: &PlayerEvent) {
    match event {
        PlayerEvent::Loading(_) => {
            player_props.set_loading(true);
        }
        PlayerEvent::Play(_) => {
            player_props.set_playing(true);
            player_props.set_loading(false);
        }
        PlayerEvent::Pause(_) => {
            player_props.set_playing(false);
            player_props.set_loading(false);
        }
        PlayerEvent::Ended(_) | PlayerEvent::Error(_) => {
            player_props.set_loading(false);
        }
        PlayerEvent::TimeUpdate(pos) => {
            let ms = (pos.seconds * 1000 + (pos.nanos / 1_000_000) as i64) as i32;
            player_props.set_current_duration_ms(ms);
            player_props.set_current_pos_str(format_duration(pos.seconds).into());
        }
    }
}

#[tracing::instrument(level = "debug", skip_all)]
fn song_to_model_with_ext(state_manager: &StateManager, song: Song) -> SongModel {
    let ext_detail = song
        .song
        .as_ref()
        .and_then(|s| s.extension.as_deref())
        .and_then(|pkg| {
            state_manager
                .get_extension_handler()
                .get_extension(pkg)
                .ok()
                .map(|ext| ext.get_extension_detail())
        });
    SongModel::from((song, ext_detail.as_ref()))
}

#[tracing::instrument(level = "debug", skip_all)]
fn apply_current_song(
    main_window_weak: slint::Weak<MainWindow>,
    state_manager: StateManager,
    song: Option<Song>,
) {
    let active_idx = if song.is_some() {
        state_manager.get_player_handler().get_current_idx() as i32
    } else {
        -1
    };
    let _ = slint::invoke_from_event_loop(move || {
        let Some(main_window) = main_window_weak.upgrade() else {
            return;
        };
        let song_model = song
            .map(|s| song_to_model_with_ext(&state_manager, s))
            .unwrap_or_else(|| SongModel::from(Song::default()));
        main_window
            .global::<PlayerProps>()
            .set_current_song(song_model.clone());
        main_window
            .global::<QueuePageProps>()
            .set_active_idx(active_idx);
        resolve_current_song_covers(&main_window, &state_manager, song_model);
    });
}

#[tracing::instrument(level = "debug", skip_all)]
fn apply_player_event(main_window_weak: slint::Weak<MainWindow>, event: PlayerEventMessage) {
    let _ = slint::invoke_from_event_loop(move || {
        let Some(main_window) = main_window_weak.upgrade() else {
            return;
        };
        if let Some(ev) = &event.event {
            let player_props = main_window.global::<PlayerProps>();
            update_player_event_props(&player_props, ev);
        }
    });
}

#[tracing::instrument(level = "debug", skip_all)]
fn setup_player_listener(main_window: &MainWindow, state_manager: &StateManager) {
    let main_window_weak = main_window.as_weak();
    let state_manager = state_manager.clone();
    tokio::spawn(
        async move {
            let player_handler = state_manager.get_player_handler();

            let repeat_mode = player_handler.get_repeat_mode();
            let current_song = player_handler.get_current_song();
            let player_state = player_handler.get_player_state();
            let is_playing =
                player_state == extensions_proto::moosync::types::PlayerState::Playing as i32;
            let current_pos = player_handler.get_current_pos().unwrap_or_default();
            let queue = player_handler.get_queue();

            let mw_weak_init = main_window_weak.clone();
            let _ = slint::invoke_from_event_loop(move || {
                if let Some(main_window) = mw_weak_init.upgrade() {
                    let player_props = main_window.global::<PlayerProps>();
                    player_props.set_repeat_mode(repeat_mode as i32);
                    player_props.set_playing(is_playing);
                    player_props.set_current_duration_ms(current_pos.as_millis() as i32);
                    player_props
                        .set_current_pos_str(format_duration(current_pos.as_secs() as i64).into());

                    let queue_props = main_window.global::<QueuePageProps>();
                    let queue_models: Vec<SongModel> =
                        queue.into_iter().map(SongModel::from).collect();
                    queue_props.set_queue(ModelRc::new(VecModel::from(queue_models)));
                }
            });
            apply_current_song(
                main_window_weak.clone(),
                state_manager.clone(),
                current_song,
            );

            let mw_weak_song = main_window_weak.clone();
            let sm_song = state_manager.clone();
            player_handler.on_song_changed.listen(move |song| {
                apply_current_song(mw_weak_song.clone(), sm_song.clone(), song);
            });

            let mw_weak_repeat = main_window_weak.clone();
            player_handler.on_repeat_changed.listen(move |mode| {
                let mw_weak = mw_weak_repeat.clone();
                let _ = slint::invoke_from_event_loop(move || {
                    if let Some(main_window) = mw_weak.upgrade() {
                        main_window
                            .global::<PlayerProps>()
                            .set_repeat_mode(mode as i32);
                    }
                });
            });

            let mw_weak_event = main_window_weak.clone();
            player_handler.on_player_event.listen(move |event| {
                apply_player_event(mw_weak_event.clone(), event);
            });
        }
        .in_current_span(),
    );
}

#[tracing::instrument(level = "debug", skip_all)]
fn setup_player_bottom_bar_callbacks(main_window: &MainWindow, state_manager: &StateManager) {
    let sm = state_manager.clone();
    main_window
        .global::<BottomBarCallbacks>()
        .on_next_song(move || {
            let sm = sm.clone();
            tokio::spawn(
                async move {
                    let player_handler = sm.get_player_handler();
                    player_handler.next().await;
                }
                .instrument(tracing::debug_span!("slint_cb_on_next_song")),
            );
        });

    let sm = state_manager.clone();
    main_window
        .global::<BottomBarCallbacks>()
        .on_prev_song(move || {
            let sm = sm.clone();
            tokio::spawn(
                async move {
                    let player_handler = sm.get_player_handler();
                    player_handler.prev().await;
                }
                .instrument(tracing::debug_span!("slint_cb_on_prev_song")),
            );
        });

    let sm = state_manager.clone();
    main_window
        .global::<BottomBarCallbacks>()
        .on_set_volume(move |volume| {
            let sm = sm.clone();
            tokio::spawn(
                async move {
                    let player_handler = sm.get_player_handler();
                    player_handler.set_volume(volume as u8).await;
                }
                .instrument(tracing::debug_span!("slint_cb_on_set_volume")),
            );
        });

    let sm = state_manager.clone();
    main_window
        .global::<BottomBarCallbacks>()
        .on_shuffle(move || {
            let sm = sm.clone();
            tokio::spawn(
                async move {
                    let player_handler = sm.get_player_handler();
                    player_handler.shuffle();
                }
                .instrument(tracing::debug_span!("slint_cb_on_shuffle")),
            );
        });

    let seek_debounce = DebounceHandle::new();
    let sm = state_manager.clone();
    main_window
        .global::<BottomBarCallbacks>()
        .on_seek(move |pos_ms| {
            let safe_ms = pos_ms.max(0) as u64;
            let target_duration = Duration::from_millis(safe_ms);
            let player_handler = sm.get_player_handler();

            seek_debounce.debounce(
                Duration::from_millis(150),
                async move {
                    player_handler.seek(target_duration).await;
                }
                .in_current_span(),
            );
        });
}

#[tracing::instrument(level = "debug", skip_all)]
pub(crate) fn apply_updated_song_to_ui(
    main_window: &MainWindow,
    state_manager: &StateManager,
    song_model: SongModel,
) {
    update_lazy_model_item(
        &main_window.global::<AllSongsPageProps>().get_songs(),
        song_model.clone(),
    );
    update_lazy_model_item(
        &main_window.global::<AlbumContentPageProps>().get_songs(),
        song_model.clone(),
    );
    update_lazy_model_item(
        &main_window.global::<ArtistContentPageProps>().get_songs(),
        song_model.clone(),
    );
    update_lazy_model_item(
        &main_window.global::<PlaylistContentPageProps>().get_songs(),
        song_model.clone(),
    );
    update_lazy_model_item(
        &main_window.global::<GenreContentPageProps>().get_songs(),
        song_model.clone(),
    );
    update_lazy_model_item(
        &main_window.global::<QueuePageProps>().get_queue(),
        song_model.clone(),
    );
    for result in main_window
        .global::<SearchPageProps>()
        .get_provider_results()
        .iter()
    {
        update_lazy_model_item(&result.songs, song_model.clone());
    }

    let current = main_window.global::<PlayerProps>().get_current_song();
    if !song_model.id.is_empty() && current.id == song_model.id {
        main_window
            .global::<PlayerProps>()
            .set_current_song(song_model.clone());
        resolve_current_song_covers(main_window, state_manager, song_model);
    }
}

#[tracing::instrument(level = "debug", skip_all)]
fn setup_entity_update_listeners(main_window: &MainWindow, state_manager: &StateManager) {
    let database = state_manager.get_database();

    let mw_song = main_window.as_weak();
    let sm_song = state_manager.clone();
    database.on_song_updated.listen(move |updated_song| {
        sm_song
            .get_player_handler()
            .update_song_in_queue(&updated_song);
        let mw_weak = mw_song.clone();
        let sm_ui = sm_song.clone();
        let _ = slint::invoke_from_event_loop(move || {
            let Some(main_window) = mw_weak.upgrade() else {
                return;
            };
            let song_model = song_to_model_with_ext(&sm_ui, updated_song);
            apply_updated_song_to_ui(&main_window, &sm_ui, song_model);
        });
    });

    let mw_album = main_window.as_weak();
    let sm_album = state_manager.clone();
    database.on_album_updated.listen(move |updated_album| {
        let mw_weak = mw_album.clone();
        let sm_ui = sm_album.clone();
        let _ = slint::invoke_from_event_loop(move || {
            let Some(main_window) = mw_weak.upgrade() else {
                return;
            };
            let mut album_model = AlbumModel::from(updated_album);
            update_lazy_model_item(
                &main_window.global::<AlbumsPageProps>().get_albums(),
                album_model.clone(),
            );
            for result in main_window
                .global::<SearchPageProps>()
                .get_provider_results()
                .iter()
            {
                update_lazy_model_item(&result.albums, album_model.clone());
            }
            if !album_model.id.is_empty()
                && main_window
                    .global::<AlbumsPageProps>()
                    .get_selected_album()
                    .id
                    == album_model.id
            {
                album_model.coverPath = find_cached_or_local_image(
                    album_model.coverPathUrl.as_str(),
                    &sm_ui.get_cache_dir(),
                )
                .unwrap_or_default();
                main_window
                    .global::<AlbumsPageProps>()
                    .set_selected_album(album_model);
            }
        });
    });

    let mw_artist = main_window.as_weak();
    let sm_artist = state_manager.clone();
    database.on_artist_updated.listen(move |updated_artist| {
        let mw_weak = mw_artist.clone();
        let sm_ui = sm_artist.clone();
        let _ = slint::invoke_from_event_loop(move || {
            let Some(main_window) = mw_weak.upgrade() else {
                return;
            };
            let mut artist_model = ArtistModel::from(updated_artist);
            update_lazy_model_item(
                &main_window.global::<ArtistsPageProps>().get_artists(),
                artist_model.clone(),
            );
            for result in main_window
                .global::<SearchPageProps>()
                .get_provider_results()
                .iter()
            {
                update_lazy_model_item(&result.artists, artist_model.clone());
            }
            if !artist_model.id.is_empty()
                && main_window
                    .global::<ArtistsPageProps>()
                    .get_selected_artist()
                    .id
                    == artist_model.id
            {
                artist_model.coverPath = find_cached_or_local_image(
                    artist_model.coverPathUrl.as_str(),
                    &sm_ui.get_cache_dir(),
                )
                .unwrap_or_default();
                main_window
                    .global::<ArtistsPageProps>()
                    .set_selected_artist(artist_model);
            }
        });
    });

    let mw_playlist = main_window.as_weak();
    let sm_playlist = state_manager.clone();
    database
        .on_playlist_updated
        .listen(move |updated_playlist| {
            let mw_weak = mw_playlist.clone();
            let sm_ui = sm_playlist.clone();
            let _ = slint::invoke_from_event_loop(move || {
                let Some(main_window) = mw_weak.upgrade() else {
                    return;
                };
                let mut playlist_model = PlaylistModel::from(updated_playlist);
                update_lazy_model_item(
                    &main_window.global::<PlaylistsPageProps>().get_playlists(),
                    playlist_model.clone(),
                );
                for result in main_window
                    .global::<SearchPageProps>()
                    .get_provider_results()
                    .iter()
                {
                    update_lazy_model_item(&result.playlists, playlist_model.clone());
                }
                if !playlist_model.id.is_empty()
                    && main_window
                        .global::<PlaylistsPageProps>()
                        .get_selected_playlist()
                        .id
                        == playlist_model.id
                {
                    playlist_model.coverPath = find_cached_or_local_image(
                        playlist_model.coverPathUrl.as_str(),
                        &sm_ui.get_cache_dir(),
                    )
                    .unwrap_or_default();
                    main_window
                        .global::<PlaylistsPageProps>()
                        .set_selected_playlist(playlist_model);
                }
            });
        });
}

#[tracing::instrument(level = "debug", skip_all)]
fn setup_player_events(main_window: &MainWindow, state_manager: &StateManager) {
    // Clear default values on load
    main_window
        .global::<PlayerProps>()
        .set_current_song(SongModel::from(Song::default()));
    main_window
        .global::<QueuePageProps>()
        .set_queue(ModelRc::new(VecModel::default()));

    setup_player_listener(main_window, state_manager);
    setup_player_bottom_bar_callbacks(main_window, state_manager);
    setup_entity_update_listeners(main_window, state_manager);
}

#[tracing::instrument(level = "debug", skip_all)]
fn setup_ui(main_window: &MainWindow, state_manager: &StateManager) {
    setup_resize(main_window);
    setup_cover_helper(main_window, state_manager);
    setup_song_list_helper(main_window, state_manager);

    let main_pages = PageLifecycleManager::get_main_pages(main_window, state_manager);
    let settings_pages = PageLifecycleManager::get_settings_pages(main_window, state_manager);
    let queue_page = PageLifecycleManager::get_queue_page(main_window, state_manager);

    let initial_main_page = main_window.global::<AppProps>().get_active_page();
    let initial_settings_page = SettingsPages::Extensions;

    let lifecycle = std::rc::Rc::new(std::cell::RefCell::new(PageLifecycleManager::new(
        main_pages,
        settings_pages,
        queue_page,
    )));
    lifecycle.borrow().initialize_all();

    let nav = std::rc::Rc::new(std::cell::RefCell::new(NavigationManager::new(
        initial_main_page,
        initial_settings_page,
    )));

    nav.borrow().sync_main_ui(main_window);
    lifecycle.borrow_mut().update_all_visibility(&nav.borrow());

    NavigationManager::setup_page_navigation(main_window, lifecycle, nav);

    setup_song_cbs(main_window, state_manager);
    setup_player_events(main_window, state_manager);
    settings::setup_settings(main_window, state_manager);
    accounts::AccountsHandler::setup(main_window, state_manager);
    accounts::AccountsHandler::setup_uri_scheme(main_window, state_manager);
}

#[tracing::instrument(level = "debug", skip_all)]
fn setup_tracing() { logging::LogController::init_default_tracing(); }
