// Manually find the generated files since bazel doesn't set vars for slint
// slint::include_modules!();
include!(concat!(env!("OUT_DIR"), "/app.rs"));

use std::{path::Path, time::Duration};

use extensions_proto::moosync::types::player_event::Event as PlayerEvent;
use player::RepeatMode;
use slint::{Image, Model, ModelRc, VecModel};
use songs_proto::moosync::types::Song;
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

#[cfg(test)]
mod integration_test;
#[cfg(test)]
mod lib_test;
#[cfg(test)]
mod pages_test;
#[cfg(test)]
pub mod test_utils;
#[cfg(test)]
mod window_info_test;

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
    rt.block_on(run());
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
            utils::default_song_cover()
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
            utils::default_song_cover()
        });
}

#[tracing::instrument(level = "debug", skip_all)]
fn setup_song_list_helper(main_window: &MainWindow, state_manager: &'static StateManager) {
    let main_window_weak = main_window.as_weak();
    main_window
        .global::<AppCallbacks>()
        .on_filter_and_sort_songs(move |songs, query, criterion, ascending| {
            let Some(main_window) = main_window_weak.upgrade() else {
                return songs;
            };
            let theme = main_window.global::<Theme>();
            let cache_dir = state_manager.get_cache_dir();
            utils::filter_and_sort_songs(
                songs,
                &query,
                criterion,
                ascending,
                theme.get_songListItemHeight() as usize,
                theme.get_songListItemWidth() as usize,
                cache_dir,
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
                return display_songs;
            }
            let songs: Vec<SongModel> = (0..selected_indices.row_count())
                .filter_map(|i| selected_indices.row_data(i))
                .filter(|&idx| idx >= 0)
                .filter_map(|idx| display_songs.row_data(idx as usize))
                .collect();
            ModelRc::new(VecModel::from(songs))
        },
    );
}

#[tracing::instrument(level = "debug", skip_all)]
fn setup_song_cbs(main_window: &MainWindow, state_manager: &'static StateManager) {
    main_window
        .global::<AppCallbacks>()
        .on_play_song(move |song_model| {
            let song = Song::from(song_model);
            tokio::spawn(async move {
                let mut queue = state_manager.get_player_handler_mut().await;
                queue.play_now(vec![song]);
            });
        });

    main_window
        .global::<AppCallbacks>()
        .on_add_song_to_queue(move |song_model| {
            let song = Song::from(song_model);
            tokio::spawn(async move {
                let mut queue = state_manager.get_player_handler_mut().await;
                queue.add_to_queue(vec![song]);
            });
        });

    main_window
        .global::<AppCallbacks>()
        .on_song_detail_action(move |action, song_models| {
            let songs = (0..song_models.row_count())
                .filter_map(|i| song_models.row_data(i))
                .map(Song::from)
                .collect::<Vec<_>>();
            tokio::spawn(async move {
                let mut queue = state_manager.get_player_handler_mut().await;
                match action {
                    SongDetailAction::Play => {
                        queue.play_now(songs);
                    }
                    SongDetailAction::AddToQueue => {
                        queue.add_to_queue(songs);
                    }
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
                    RepeatMode::None => RepeatMode::Once,
                    RepeatMode::Once => RepeatMode::Infinite,
                    RepeatMode::Infinite => RepeatMode::None,
                };
                player_handler.repeat(next_mode);
            });
        });
}

#[tracing::instrument(level = "debug", skip_all)]
fn setup_player_events(main_window: &'static MainWindow, state_manager: &'static StateManager) {
    // Clear default values on load
    main_window.set_current_song(SongModel::from(Song::default()));
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
                    let song_model = match song {
                        Some(s) => SongModel::from(s),
                        None => SongModel::from(Song::default()),
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
                            PlayerEvent::Play(_) => {
                                main_window.set_playing(true);
                            }
                            PlayerEvent::Pause(_) => {
                                main_window.set_playing(false);
                            }
                            PlayerEvent::TimeUpdate(pos) => {
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
    visible_states: std::collections::HashMap<AppPage, bool>,
    active_main_page: AppPage,
    active_settings_page: AppPage,
    settings_open: bool,
    queue_open: bool,
}

impl PageLifecycleManager {
    #[tracing::instrument(level = "debug", skip_all)]
    fn new(page_types: &[AppPage], initial_main_page: AppPage) -> Self {
        let mut visible_states = std::collections::HashMap::new();
        for &page_type in page_types {
            visible_states.insert(page_type, false);
        }

        Self {
            visible_states,
            active_main_page: initial_main_page,
            active_settings_page: AppPage::Extensions, // Default tab in Settings is Extensions
            settings_open: false,
            queue_open: false,
        }
    }

    #[tracing::instrument(level = "debug", skip_all)]
    fn compute_visibility_changes(&mut self, page_types: &[AppPage]) -> Vec<(AppPage, bool)> {
        let mut actions = Vec::new();
        for &page_type in page_types {
            let was_visible = *self.visible_states.get(&page_type).unwrap_or(&false);
            let is_visible = match page_type {
                AppPage::Queue => self.queue_open,

                AppPage::Paths => {
                    self.settings_open
                        && (was_visible || self.active_settings_page == AppPage::Paths)
                }
                AppPage::System => {
                    self.settings_open
                        && (was_visible || self.active_settings_page == AppPage::System)
                }
                AppPage::Extensions => {
                    self.settings_open
                        && (was_visible || self.active_settings_page == AppPage::Extensions)
                }
                AppPage::Themes => {
                    self.settings_open
                        && (was_visible || self.active_settings_page == AppPage::Themes)
                }

                AppPage::AllSongs => self.active_main_page == AppPage::AllSongs,
                AppPage::Albums => self.active_main_page == AppPage::Albums,
                AppPage::Artists => self.active_main_page == AppPage::Artists,
                AppPage::Playlists => self.active_main_page == AppPage::Playlists,
                AppPage::Genres => self.active_main_page == AppPage::Genres,
                AppPage::Explore => self.active_main_page == AppPage::Explore,
                AppPage::Search => self.active_main_page == AppPage::Search,
                AppPage::PlaylistContent => self.active_main_page == AppPage::PlaylistContent,
                AppPage::AlbumContent => self.active_main_page == AppPage::AlbumContent,
                AppPage::ArtistContent => self.active_main_page == AppPage::ArtistContent,
                AppPage::GenreContent => self.active_main_page == AppPage::GenreContent,
            };

            if is_visible != was_visible {
                self.visible_states.insert(page_type, is_visible);
                actions.push((page_type, is_visible));
            }
        }
        actions
    }
}

#[tracing::instrument(level = "debug", skip_all)]
fn update_manager_visibility(
    manager: &std::rc::Rc<std::cell::RefCell<PageLifecycleManager>>,
    pages: &std::rc::Rc<std::collections::HashMap<AppPage, Box<dyn PageHandler + 'static>>>,
) {
    let page_types: Vec<AppPage> = pages.keys().copied().collect();
    let actions = manager.borrow_mut().compute_visibility_changes(&page_types);

    for (page_type, is_visible) in actions {
        let Some(handler) = pages.get(&page_type) else {
            continue;
        };
        if is_visible {
            handler.on_show();
        }
        if !is_visible {
            handler.on_hide();
        }
    }
}

#[tracing::instrument(level = "debug", skip_all)]
fn setup_page_navigation(
    main_window: &MainWindow,
    pages: std::rc::Rc<std::collections::HashMap<AppPage, Box<dyn PageHandler + 'static>>>,
) {
    for page in pages.values() {
        page.initialize();
    }

    let initial_main_page = AppPage::from(main_window.get_active_page());
    let page_types: Vec<AppPage> = pages.keys().copied().collect();

    let manager = std::rc::Rc::new(std::cell::RefCell::new(PageLifecycleManager::new(
        &page_types,
        initial_main_page,
    )));

    // Trigger initial on_show
    update_manager_visibility(&manager, &pages);

    // 1. Listen to active page change
    let manager_main = manager.clone();
    let pages_main = pages.clone();
    main_window
        .global::<AppCallbacks>()
        .on_active_page_changed(move |new_page| {
            manager_main.borrow_mut().active_main_page = AppPage::from(new_page);
            update_manager_visibility(&manager_main, &pages_main);
        });

    // 2. Listen to settings page change
    let manager_settings = manager.clone();
    let pages_settings = pages.clone();
    main_window
        .global::<AppCallbacks>()
        .on_settings_active_page_changed(move |new_page| {
            manager_settings.borrow_mut().active_settings_page = AppPage::from(new_page);
            update_manager_visibility(&manager_settings, &pages_settings);
        });

    // 3. Listen to settings toggle
    let manager_settings_toggle = manager.clone();
    let pages_settings_toggle = pages.clone();
    main_window
        .global::<AppCallbacks>()
        .on_settings_toggled(move |open| {
            manager_settings_toggle.borrow_mut().settings_open = open;
            update_manager_visibility(&manager_settings_toggle, &pages_settings_toggle);
        });

    // 4. Listen to queue toggle
    let manager_queue_toggle = manager.clone();
    let pages_queue_toggle = pages.clone();
    main_window
        .global::<AppCallbacks>()
        .on_queue_toggled(move |open| {
            manager_queue_toggle.borrow_mut().queue_open = open;
            update_manager_visibility(&manager_queue_toggle, &pages_queue_toggle);
        });
}

#[tracing::instrument(level = "debug", skip_all)]
fn setup_ui(main_window: &'static MainWindow, state_manager: &'static StateManager) {
    setup_resize(main_window);
    setup_cover_helper(main_window);
    setup_song_list_helper(main_window, state_manager);
    let pages = std::rc::Rc::new(get_all_pages(main_window, state_manager));
    setup_page_navigation(main_window, pages.clone());
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
