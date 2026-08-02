// Manually find the generated files since bazel doesn't set vars for slint
// slint::include_modules!();
include!(concat!(env!("OUT_DIR"), "/app.rs"));

use std::{path::Path, time::Duration};

use extensions_proto;
use player;
use slint::{Image, ModelRc, VecModel};
use songs_proto;
use state_manager::StateManager;
use tracing::{debug, trace};
use tracing_subscriber::{EnvFilter, fmt, layer::SubscriberExt};
use types::prelude::format_duration;

use crate::pages::{AppPage, PageHandler};

pub mod error;
mod main_content;
mod pages;
mod settings;
mod utils;
mod window_info;

pub use window_info::{WINDOW_EVENTS, WindowEvents};

#[cfg(target_os = "android")]
static ANDROID_APP: std::sync::OnceLock<slint::android::AndroidApp> = std::sync::OnceLock::new();

#[cfg(target_os = "android")]
#[unsafe(no_mangle)]
#[tracing::instrument(level = "debug", skip_all)]
fn android_main(app: slint::android::AndroidApp) {
    slint::android::init(app.clone()).unwrap();
    ANDROID_APP.set(app).expect("failed to set ANDROID_APP");
    let rt = tokio::runtime::Runtime::new().unwrap();
    let _guard = rt.enter();
    run();
}

#[tracing::instrument(level = "debug", skip_all)]
fn get_all_pages(
    main_window: &'static MainWindow,
    state_manager: &'static StateManager,
) -> std::collections::HashMap<AppPage, Box<dyn PageHandler + 'static>> {
    let mut map: std::collections::HashMap<AppPage, Box<dyn PageHandler + 'static>> =
        std::collections::HashMap::new();

    map.insert(
        AppPage::AllSongs,
        Box::new(main_content::all_songs::AllSongsPageHandler::new(
            main_window,
            state_manager,
        )),
    );
    map.insert(
        AppPage::Albums,
        Box::new(main_content::albums::AlbumsPageHandler::new(
            main_window,
            state_manager,
        )),
    );
    map.insert(
        AppPage::Artists,
        Box::new(main_content::artists::ArtistsPageHandler::new(
            main_window,
            state_manager,
        )),
    );
    map.insert(
        AppPage::Playlists,
        Box::new(main_content::playlists::PlaylistsPageHandler::new(
            main_window,
            state_manager,
        )),
    );
    map.insert(
        AppPage::Genres,
        Box::new(main_content::genres::GenresPageHandler::new(
            main_window,
            state_manager,
        )),
    );
    map.insert(
        AppPage::Explore,
        Box::new(main_content::explore::ExplorePageHandler::new(
            main_window,
            state_manager,
        )),
    );
    map.insert(
        AppPage::Search,
        Box::new(main_content::search::SearchPageHandler::new(
            main_window,
            state_manager,
        )),
    );
    map.insert(
        AppPage::PlaylistContent,
        Box::new(
            main_content::playlist_content::PlaylistContentPageHandler::new(
                main_window,
                state_manager,
            ),
        ),
    );
    map.insert(
        AppPage::AlbumContent,
        Box::new(main_content::album_content::AlbumContentPageHandler::new(
            main_window,
            state_manager,
        )),
    );
    map.insert(
        AppPage::ArtistContent,
        Box::new(main_content::artist_content::ArtistContentPageHandler::new(
            main_window,
            state_manager,
        )),
    );
    map.insert(
        AppPage::GenreContent,
        Box::new(main_content::genre_content::GenreContentPageHandler::new(
            main_window,
            state_manager,
        )),
    );
    map.insert(
        AppPage::Queue,
        Box::new(main_content::queue::QueuePageHandler::new(
            main_window,
            state_manager,
        )),
    );

    map.insert(
        AppPage::Paths,
        Box::new(settings::paths::PathsPageHandler::new(
            main_window,
            state_manager,
        )),
    );
    map.insert(
        AppPage::System,
        Box::new(settings::system::SystemPageHandler::new(
            main_window,
            state_manager,
        )),
    );
    map.insert(
        AppPage::Extensions,
        Box::new(settings::extensions::ExtensionsPageHandler::new(
            main_window,
            state_manager,
        )),
    );
    map.insert(
        AppPage::Themes,
        Box::new(settings::themes::ThemesPageHandler::new(
            main_window,
            state_manager,
        )),
    );

    map
}

#[tracing::instrument(level = "debug", skip_all)]
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
    main_window.show().unwrap();
    state_manager.delayed_setup().await;
    slint::run_event_loop().unwrap();
    state_manager.shutdown().await;
}

#[tracing::instrument(level = "debug", skip_all)]
fn setup_resize(main_window: &MainWindow) {
    let main_window_weak = main_window.as_weak();
    main_window.global::<AppCallbacks>().on_resize(move || {
        if let Some(main_window) = main_window_weak.upgrade() {
            WINDOW_EVENTS.with(|we| we.trigger_resize(main_window.window()));
        }
    });
}

#[tracing::instrument(level = "debug", skip_all)]
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

    main_window
        .global::<CoverHelper>()
        .on_fetch_cover_low(move |song_model| {
            trace!("Fetching low-res cover for song {}", song_model.title);
            if !song_model.coverPathUrlLow.is_empty() {
                if let Ok(image) =
                    Image::load_from_path(Path::new(song_model.coverPathUrlLow.as_str()))
                {
                    return image;
                }
            }
            Image::load_from_svg_data(utils::DEFAULT_SONG_SVG).unwrap()
        });
}

#[tracing::instrument(level = "debug", skip_all)]
fn setup_song_cbs(main_window: &MainWindow, state_manager: &'static StateManager) {
    main_window
        .global::<AppCallbacks>()
        .on_play_song(move |song_model| {
            let song = utils::song_model_to_song(&song_model);
            tokio::spawn(async move {
                let mut queue = state_manager.get_player_handler_mut().await;
                queue.play_now(song);
            });
        });

    main_window
        .global::<AppCallbacks>()
        .on_add_song_to_queue(move |song_model| {
            let song = utils::song_model_to_song(&song_model);
            tokio::spawn(async move {
                let mut queue = state_manager.get_player_handler_mut().await;
                queue.add_to_queue(song);
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

#[tracing::instrument(level = "debug", skip_all)]
fn setup_player_events(main_window: &'static MainWindow, state_manager: &'static StateManager) {
    // Clear default values on load
    main_window.set_current_song(utils::to_song_model(
        &songs_proto::moosync::types::Song::default(),
        None,
    ));
    main_window
        .global::<QueuePageProps>()
        .set_queue(ModelRc::new(VecModel::default()));

    let main_window_weak = main_window.as_weak();
    tokio::spawn(async move {
        let player_handler = state_manager.get_player_handler().await;

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
            let mw_weak = mw_weak_song.clone();
            let song = song.cloned();
            let _ = slint::invoke_from_event_loop(move || {
                if let Some(main_window) = mw_weak.upgrade() {
                    let song_model = match &song {
                        Some(s) => utils::to_song_model(s, None),
                        None => utils::to_song_model(
                            &songs_proto::moosync::types::Song::default(),
                            None,
                        ),
                    };
                    main_window.set_current_song(song_model);
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

struct PageLifecycleManager {
    pages: std::collections::HashMap<AppPage, Box<dyn PageHandler + 'static>>,
    visible_states: std::collections::HashMap<AppPage, bool>,
    active_main_page: AppPage,
    active_settings_page: AppPage,
    settings_open: bool,
    queue_open: bool,
}

impl PageLifecycleManager {
    #[tracing::instrument(level = "debug", skip_all)]
    fn new(
        pages: std::collections::HashMap<AppPage, Box<dyn PageHandler + 'static>>,
        initial_main_page: AppPage,
    ) -> Self {
        let mut visible_states = std::collections::HashMap::new();
        for &page_type in pages.keys() {
            visible_states.insert(page_type, false);
        }

        Self {
            pages,
            visible_states,
            active_main_page: initial_main_page,
            active_settings_page: AppPage::Extensions, // Default tab in Settings is Extensions
            settings_open: false,
            queue_open: false,
        }
    }

    #[tracing::instrument(level = "debug", skip_all)]
    fn update_visibility(&mut self) {
        for (&page_type, handler) in &self.pages {
            let was_visible = *self.visible_states.get(&page_type).unwrap_or(&false);
            let is_visible = match page_type {
                AppPage::Queue => self.queue_open,

                AppPage::Paths => {
                    self.settings_open
                        && !self.queue_open
                        && (was_visible || self.active_settings_page == AppPage::Paths)
                }
                AppPage::System => {
                    self.settings_open
                        && !self.queue_open
                        && (was_visible || self.active_settings_page == AppPage::System)
                }
                AppPage::Extensions => {
                    self.settings_open
                        && !self.queue_open
                        && (was_visible || self.active_settings_page == AppPage::Extensions)
                }
                AppPage::Themes => {
                    self.settings_open
                        && !self.queue_open
                        && (was_visible || self.active_settings_page == AppPage::Themes)
                }

                AppPage::AllSongs => {
                    !self.queue_open && (self.active_main_page == AppPage::AllSongs)
                }
                AppPage::Albums => !self.queue_open && (self.active_main_page == AppPage::Albums),
                AppPage::Artists => !self.queue_open && (self.active_main_page == AppPage::Artists),
                AppPage::Playlists => {
                    !self.queue_open && (self.active_main_page == AppPage::Playlists)
                }
                AppPage::Genres => !self.queue_open && (self.active_main_page == AppPage::Genres),
                AppPage::Explore => !self.queue_open && (self.active_main_page == AppPage::Explore),
                AppPage::Search => !self.queue_open && (self.active_main_page == AppPage::Search),
                AppPage::PlaylistContent => {
                    !self.queue_open && (self.active_main_page == AppPage::PlaylistContent)
                }
                AppPage::AlbumContent => {
                    !self.queue_open && (self.active_main_page == AppPage::AlbumContent)
                }
                AppPage::ArtistContent => {
                    !self.queue_open && (self.active_main_page == AppPage::ArtistContent)
                }
                AppPage::GenreContent => {
                    !self.queue_open && (self.active_main_page == AppPage::GenreContent)
                }
            };

            if is_visible != was_visible {
                self.visible_states.insert(page_type, is_visible);
                if is_visible {
                    handler.on_show();
                }
                if !is_visible {
                    handler.on_hide();
                }
            }
        }
    }
}

#[tracing::instrument(level = "debug", skip_all)]
fn setup_page_navigation(
    main_window: &MainWindow,
    pages: std::collections::HashMap<AppPage, Box<dyn PageHandler + 'static>>,
) {
    for page in pages.values() {
        page.initialize();
    }

    let initial_main_page = AppPage::from(main_window.get_active_page());

    // Create the manager inside Rc<RefCell<...>> to allow sharing in main thread
    // callbacks
    let manager = std::rc::Rc::new(std::cell::RefCell::new(PageLifecycleManager::new(
        pages,
        initial_main_page,
    )));

    // Trigger initial on_show
    manager.borrow_mut().update_visibility();

    // 1. Listen to active page change
    let manager_main = manager.clone();
    main_window
        .global::<AppCallbacks>()
        .on_active_page_changed(move |new_page| {
            let mut mgr = manager_main.borrow_mut();
            mgr.active_main_page = AppPage::from(new_page);
            mgr.update_visibility();
        });

    // 2. Listen to settings page change
    let manager_settings = manager.clone();
    main_window
        .global::<AppCallbacks>()
        .on_settings_active_page_changed(move |new_page| {
            let mut mgr = manager_settings.borrow_mut();
            mgr.active_settings_page = AppPage::from(new_page);
            mgr.update_visibility();
        });

    // 3. Listen to settings toggle
    let manager_settings_toggle = manager.clone();
    main_window
        .global::<AppCallbacks>()
        .on_settings_toggled(move |open| {
            let mut mgr = manager_settings_toggle.borrow_mut();
            mgr.settings_open = open;
            mgr.update_visibility();
        });

    // 4. Listen to queue toggle
    let manager_queue_toggle = manager.clone();
    main_window
        .global::<AppCallbacks>()
        .on_queue_toggled(move |open| {
            let mut mgr = manager_queue_toggle.borrow_mut();
            mgr.queue_open = open;
            mgr.update_visibility();
        });
}

#[tracing::instrument(level = "debug", skip_all)]
fn setup_ui(main_window: &'static MainWindow, state_manager: &'static StateManager) {
    setup_resize(main_window);
    setup_cover_helper(main_window);
    let pages = get_all_pages(main_window, state_manager);
    setup_page_navigation(main_window, pages);
    setup_song_cbs(main_window, state_manager);
    setup_player_events(main_window, state_manager);
    settings::setup_settings(main_window, state_manager);
}

#[tracing::instrument(level = "debug", skip_all)]
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
#[tracing::instrument(level = "debug", skip_all)]
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

        // Load Class via Activity's ClassLoader to avoid ClassNotFoundException on
        // native threads
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
