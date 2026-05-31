// Manually find the generated files since bazel doesn't set vars for slint
// slint::include_modules!();
include!(concat!(env!("OUT_DIR"), "/app.rs"));

use std::{path::Path, time::Duration};

use extensions_proto;
use player;
use slint::{Image, ModelRc, VecModel};
use state_manager::StateManager;
use tracing::{debug, trace};
use tracing_subscriber::{EnvFilter, fmt, layer::SubscriberExt};
use types::prelude::format_duration;

use crate::pages::PageHandler;

mod main_content;
mod pages;
mod utils;
mod window_info;

pub use window_info::{WINDOW_EVENTS, WindowEvents};

#[cfg(target_os = "android")]
static ANDROID_APP: std::sync::OnceLock<slint::android::AndroidApp> = std::sync::OnceLock::new();

#[cfg(target_os = "android")]
#[unsafe(no_mangle)]
fn android_main(app: slint::android::AndroidApp) {
    slint::android::init(app.clone()).unwrap();
    ANDROID_APP.set(app).expect("failed to set ANDROID_APP");
    let rt = tokio::runtime::Runtime::new().unwrap();
    let _guard = rt.enter();
    run();
}

fn get_all_pages<'a>(
    main_window: &'a MainWindow,
    state_manager: &'a StateManager,
) -> Vec<Box<dyn PageHandler + 'a>> {
    vec![
        Box::new(main_content::all_songs::AllSongsPageHandler::new(
            main_window,
            state_manager,
        )),
        Box::new(main_content::albums::AlbumsPageHandler::new(
            main_window,
            state_manager,
        )),
        Box::new(main_content::artists::ArtistsPageHandler::new(
            main_window,
            state_manager,
        )),
        Box::new(main_content::playlists::PlaylistsPageHandler::new(
            main_window,
            state_manager,
        )),
        Box::new(main_content::genres::GenresPageHandler::new(
            main_window,
            state_manager,
        )),
        Box::new(main_content::explore::ExplorePageHandler::new(
            main_window,
            state_manager,
        )),
    ]
}

pub async fn run() {
    setup_tracing();
    debug!("Starting Moosync...");

    #[cfg(target_os = "android")]
    let android_context = setup_android();

    let main_window = Box::leak(Box::new(MainWindow::new().unwrap()));
    let state_manager = Box::leak(Box::new(
        StateManager::new(
            #[cfg(target_os = "android")]
            android_context,
        )
        .expect("StateManager::new failed"),
    ));

    setup_ui(main_window, state_manager);

    state_manager.setup().await;
    main_window.run().unwrap();
}

fn setup_resize(main_window: &MainWindow) {
    let main_window_weak = main_window.as_weak();
    main_window.global::<AppCallbacks>().on_resize(move || {
        if let Some(main_window) = main_window_weak.upgrade() {
            WINDOW_EVENTS.with(|we| we.trigger_resize(main_window.window()));
        }
    });
}

fn setup_cover_helper(main_window: &MainWindow) {
    main_window
        .global::<CoverHelper>()
        .on_fetch_cover_high(move |song_model| {
            trace!("Fetching high-res cover for song {}", song_model.title);
            if !song_model.coverPathUrlHigh.is_empty() {
                if let Ok(image) =
                    Image::load_from_path(Path::new(song_model.coverPathUrlHigh.as_str()))
                {
                    return image;
                }
            }
            Image::load_from_svg_data(utils::DEFAULT_SONG_SVG).unwrap()
        });
}

fn get_page_handler<'b>(
    pages: &'b [Box<dyn PageHandler + '_>],
    page: Pages,
) -> Option<&'b dyn PageHandler> {
    let idx = match page {
        Pages::AllSongs => 0,
        Pages::Albums => 1,
        Pages::Artists => 2,
        Pages::Playlists => 3,
        Pages::Genres => 4,
        Pages::Explore => 5,
    };
    pages.get(idx).map(|p| p.as_ref())
}

fn setup_song_cbs(main_window: &MainWindow, state_manager: &'static StateManager) {
    main_window
        .global::<AppCallbacks>()
        .on_play_song(move |song_model| {
            tokio::spawn(async move {
                let song = state_manager
                    .get_song_from_cache(song_model.id.into())
                    .await;
                if let Some(song) = song {
                    let mut queue = state_manager.get_player_handler_mut().await;
                    queue.play_now(song);
                }
            });
        });

    main_window
        .global::<AppCallbacks>()
        .on_add_song_to_queue(move |song_model| {
            tokio::spawn(async move {
                let song = state_manager
                    .get_song_from_cache(song_model.id.into())
                    .await;
                if let Some(song) = song {
                    let mut queue = state_manager.get_player_handler_mut().await;
                    queue.add_to_queue(song);
                }
            });
        });

    let main_window_weak = main_window.as_weak();
    main_window
        .global::<BottomBarCallbacks>()
        .on_play_pause_clicked(move || {
            let main_window_weak = main_window_weak.clone();
            tokio::spawn(async move {
                let mut player_handler = state_manager.get_player_handler_mut().await;
                main_window_weak.upgrade_in_event_loop(move |main_window| {
                    let currently_playing = main_window.get_playing();
                    if currently_playing {
                        let _ = player_handler.pause();
                    } else {
                        let _ = player_handler.play();
                    }
                })
            });
        });

    main_window
        .global::<BottomBarCallbacks>()
        .on_toggle_repeat(move || {
            tokio::spawn(async move {
                let mut player_handler = state_manager.get_player_handler_mut().await;
                let next_mode = match player_handler.get_repeat_mode() {
                    player::RepeatMode::None => player::RepeatMode::Once,
                    player::RepeatMode::Once => player::RepeatMode::Infinite,
                    player::RepeatMode::Infinite => player::RepeatMode::None,
                };
                player_handler.repeat(next_mode);
            });
        });
}

fn setup_player_events(main_window: &'static MainWindow, state_manager: &'static StateManager) {
    // Clear default values on load
    main_window.set_current_song(utils::to_song_model(None));
    main_window.set_queue(ModelRc::new(VecModel::default()));

    let main_window_weak = main_window.as_weak();
    tokio::spawn(async move {
        let mut player_handler = state_manager.get_player_handler_mut().await;

        // Set initial repeat mode
        let repeat_mode = player_handler.get_repeat_mode();
        let mw_weak_init = main_window_weak.clone();
        let _ = slint::invoke_from_event_loop(move || {
            if let Some(main_window) = mw_weak_init.upgrade() {
                main_window.set_repeat_mode(repeat_mode as i32);
            }
        });

        let mw_weak_song = main_window_weak.clone();
        player_handler.on_song_changed(move |song| {
            let song_cloned = song.cloned();
            let mw_weak = mw_weak_song.clone();
            let _ = slint::invoke_from_event_loop(move || {
                if let Some(main_window) = mw_weak.upgrade() {
                    let song_model = utils::to_song_model(song_cloned.as_ref());
                    main_window.set_current_song(song_model);
                }
            });
        });

        let mw_weak_queue = main_window_weak.clone();
        player_handler.on_queue_updated(move |queue| {
            let queue_cloned = queue.to_vec();
            let mw_weak = mw_weak_queue.clone();
            let _ = slint::invoke_from_event_loop(move || {
                if let Some(main_window) = mw_weak.upgrade() {
                    let queue_models: Vec<SongModel> = queue_cloned
                        .iter()
                        .map(|s| utils::to_song_model(Some(s)))
                        .collect();
                    main_window.set_queue(ModelRc::new(VecModel::from(queue_models)));
                }
            });
        });

        let mw_weak_repeat = main_window_weak.clone();
        player_handler.on_repeat_changed(move |mode| {
            let mw_weak = mw_weak_repeat.clone();
            let _ = slint::invoke_from_event_loop(move || {
                if let Some(main_window) = mw_weak.upgrade() {
                    main_window.set_repeat_mode(mode as i32);
                }
            });
        });

        let mw_weak_event = main_window_weak.clone();
        player_handler.on_player_event(move |event| {
            let event_cloned = event.clone();
            let mw_weak = mw_weak_event.clone();
            let _ = slint::invoke_from_event_loop(move || {
                if let Some(main_window) = mw_weak.upgrade() {
                    if let Some(ev) = &event_cloned.event {
                        match ev {
                            extensions_proto::moosync::types::player_event::Event::Play(_) => {
                                main_window.set_playing(true);
                            }
                            extensions_proto::moosync::types::player_event::Event::Pause(_) => {
                                main_window.set_playing(false);
                            }
                            extensions_proto::moosync::types::player_event::Event::TimeUpdate(
                                pos,
                            ) => {
                                main_window.set_current_duration(pos.seconds as i32);
                                main_window
                                    .set_current_pos_str(format_duration(pos.seconds).into());
                            }
                            _ => {}
                        }
                    }
                }
            });
        });
    });

    main_window
        .global::<BottomBarCallbacks>()
        .on_next_song(move || {
            tokio::spawn(async move {
                let state_manager_clone = state_manager.clone();
                let mut player_handler = state_manager_clone.get_player_handler_mut().await;
                player_handler.next();
            });
        });

    main_window
        .global::<BottomBarCallbacks>()
        .on_prev_song(move || {
            tokio::spawn(async move {
                let state_manager_clone = state_manager.clone();
                let mut player_handler = state_manager_clone.get_player_handler_mut().await;
                player_handler.prev();
            });
        });

    main_window
        .global::<BottomBarCallbacks>()
        .on_set_volume(move |volume| {
            tokio::spawn(async move {
                let state_manager_clone = state_manager.clone();
                let player_handler = state_manager_clone.get_player_handler().await;
                player_handler.set_volume(volume as u8);
            });
        });

    main_window
        .global::<BottomBarCallbacks>()
        .on_shuffle(move || {
            tokio::spawn(async move {
                let state_manager_clone = state_manager.clone();
                let mut player_handler = state_manager_clone.get_player_handler_mut().await;
                player_handler.shuffle();
            });
        });

    main_window
        .global::<BottomBarCallbacks>()
        .on_seek(move |pos| {
            tokio::spawn(async move {
                let safe_secs = pos.max(0) as u64;
                let target_duration = Duration::from_secs(safe_secs);

                let state_manager_clone = state_manager.clone();
                let player_handler = state_manager_clone.get_player_handler().await;

                player_handler.seek(target_duration);
            });
        });
}

fn setup_page_navigation(main_window: &MainWindow, pages: Vec<Box<dyn PageHandler + 'static>>) {
    for page in &pages {
        page.initialize();
    }

    let current_page = std::cell::Cell::new(main_window.get_active_page());
    if let Some(handler) = get_page_handler(&pages, current_page.get()) {
        handler.on_show();
    }

    let duration_ms = main_window
        .global::<Constants>()
        .get_page_transition_duration();
    let transition_duration = std::time::Duration::from_millis(duration_ms as u64);

    let pending_hide = std::rc::Rc::new(std::cell::RefCell::new(None::<(Pages, slint::Timer)>));

    let pages_ref = std::rc::Rc::new(pages);
    let pages_clone = pages_ref.clone();
    let pending_hide_clone = pending_hide.clone();

    main_window
        .global::<AppCallbacks>()
        .on_active_page_changed(move |new_page| {
            let prev_page = current_page.replace(new_page);

            if let Some((pending_page, timer)) = pending_hide_clone.borrow_mut().take() {
                timer.stop();
                if let Some(pending_handler) = get_page_handler(&pages_clone, pending_page) {
                    pending_handler.on_hide();
                }
            }

            if let Some(new_handler) = get_page_handler(&pages_clone, new_page) {
                new_handler.on_show();
            }

            if get_page_handler(&pages_clone, prev_page).is_some() {
                let timer = slint::Timer::default();
                let pages_clone_inner = pages_clone.clone();
                let pending_hide_inner = pending_hide_clone.clone();
                timer.start(
                    slint::TimerMode::SingleShot,
                    transition_duration,
                    move || {
                        if let Some((pending_page, _)) = pending_hide_inner.borrow_mut().take() {
                            if let Some(handler) =
                                get_page_handler(&pages_clone_inner, pending_page)
                            {
                                handler.on_hide();
                            }
                        }
                    },
                );
                *pending_hide_clone.borrow_mut() = Some((prev_page, timer));
            }
        });
}

fn setup_ui(main_window: &'static MainWindow, state_manager: &'static StateManager) {
    setup_resize(main_window);
    setup_cover_helper(main_window);
    let pages = get_all_pages(main_window, state_manager);
    setup_page_navigation(main_window, pages);
    setup_song_cbs(main_window, state_manager);
    setup_player_events(main_window, state_manager);
}

fn setup_tracing() {
    let env_filter = EnvFilter::try_from_env("MOOSYNC_LOG").unwrap_or(EnvFilter::new("info"));

    let layer = fmt::layer().pretty().with_target(true).with_ansi(true);

    #[cfg(not(target_os = "android"))]
    let subscriber = {
        use std::fs;
        use tracing_appender::rolling::{RollingFileAppender, Rotation};

        let log_path = platform_dirs::AppDirs::new(Some("moosync"), false)
            .unwrap()
            .data_dir
            .join("logs");
        if !log_path.exists() {
            fs::create_dir_all(log_path.clone()).unwrap();
        }
        let file_appender = RollingFileAppender::new(Rotation::DAILY, &log_path, "moosync");
        let log_layer = fmt::layer()
            .pretty()
            .with_ansi(false)
            .with_target(true)
            .with_writer(file_appender);
        tracing_subscriber::registry()
            .with(env_filter)
            .with(layer)
            .with(log_layer)
    };

    #[cfg(target_os = "android")]
    let subscriber = tracing_subscriber::registry().with(env_filter).with(layer);

    tracing::subscriber::set_global_default(subscriber).unwrap();
}

#[cfg(target_os = "android")]
fn setup_android() -> types::android::AndroidJNIContext {
    let app = ANDROID_APP.get().expect("ANDROID_APP not initialized");

    // Safety: vm_as_ptr() returns the raw *mut JavaVM for this process.
    let vm =
        unsafe { jni::JavaVM::from_raw(app.vm_as_ptr().cast()) }.expect("failed to get JavaVM");

    let (activity, service_class) = {
        let mut env = vm.attach_current_thread().expect("JNI attach");

        let act_ptr = app.activity_as_ptr() as jni::sys::jobject;
        let act_obj = unsafe { jni::objects::JObject::from_raw(act_ptr) };
        let activity_ref = env.new_global_ref(act_obj).expect("new_global_ref");

        // Load Class via Activity's ClassLoader to avoid ClassNotFoundException on native threads
        let class_obj = env
            .call_method(&activity_ref, "getClass", "()Ljava/lang/Class;", &[])
            .expect("getClass failed")
            .l()
            .expect("getClass returned null/non-object");

        let class_loader = env
            .call_method(
                &class_obj,
                "getClassLoader",
                "()Ljava/lang/ClassLoader;",
                &[],
            )
            .expect("getClassLoader failed")
            .l()
            .expect("getClassLoader returned null/non-object");

        let class_name_jstr = env
            .new_string("app.moosync.android.services.MoosyncService")
            .expect("new_string failed");

        let cls_obj = env
            .call_method(
                &class_loader,
                "loadClass",
                "(Ljava/lang/String;)Ljava/lang/Class;",
                &[jni::objects::JValue::Object(&class_name_jstr)],
            )
            .expect("loadClass MoosyncService failed")
            .l()
            .expect("loadClass returned null/non-class");

        let service_class_ref = env.new_global_ref(cls_obj).expect("new_global_ref");

        (activity_ref, service_class_ref)
    };

    types::android::AndroidJNIContext {
        jvm: std::sync::Arc::new(vm),
        activity,
        service_class,
    }
}
