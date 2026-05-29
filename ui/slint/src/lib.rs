// Manually find the generated files since bazel doesn't set vars for slint
// slint::include_modules!();
include!(concat!(env!("OUT_DIR"), "/app.rs"));

use std::path::Path;

use slint::Image;
use state_manager::StateManager;
use tracing::{debug, trace};
use tracing_subscriber::{EnvFilter, fmt, layer::SubscriberExt};

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
    main_window.on_resize(move || {
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

    main_window.on_active_page_changed(move |new_page| {
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
                        if let Some(handler) = get_page_handler(&pages_clone_inner, pending_page) {
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
