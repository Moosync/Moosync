// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use logger::logger::{log_debug, log_error, log_info, log_warn};
use preference_holder::preferences::{
    get_secure, initial, load_selective, save_selective, set_secure,
};

use tauri::{Manager, State, WebviewWindowBuilder};

use crate::{
    db::{
        cache::get_cache_state,
        database::{get_db_state, get_entity_by_options, get_songs_by_options},
    },
    oauth::handler::{get_oauth_state, OAuthHandler},
    preference_holder::preferences::get_preference_state,
    window::handler::{
        close_window, get_platform, get_window_state, has_frame, is_maximized, maximize_window,
        minimize_window, open_external, update_zoom,
    },
    youtube::youtube::{get_video_url, get_youtube_scraper_state, search_yt},
};

use crate::oauth::handler::{register_oauth_path, unregister_oauth_path};

mod db;
mod errors;
mod logger;
mod macros;
mod oauth;
mod preference_holder;
mod types;
mod window;
mod youtube;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_deep_link::init())
        .plugin(tauri_plugin_single_instance::init(|app, argv, cwd| {
            println!("{}, {argv:?}, {cwd}", app.package_info().name);

            if let Some(url) = argv.get(1) {
                let state: State<OAuthHandler> = app.state();
                state.handle_oauth(app, url.to_string()).unwrap();
            }
        }))
        .invoke_handler(tauri::generate_handler![
            // Preferences
            save_selective,
            load_selective,
            get_secure,
            set_secure,
            // Logger
            log_error,
            log_debug,
            log_info,
            log_warn,
            // DB
            get_songs_by_options,
            get_entity_by_options,
            // OAuth
            register_oauth_path,
            unregister_oauth_path,
            // Window
            is_maximized,
            has_frame,
            close_window,
            get_platform,
            maximize_window,
            minimize_window,
            update_zoom,
            open_external,
            // Youtube
            search_yt,
            get_video_url,
        ])
        .setup(|app| {
            let db = get_db_state(app);
            app.manage(db);

            let cache = get_cache_state(app);
            app.manage(cache);

            let config = get_preference_state(app)?;
            app.manage(config);

            let oauth = get_oauth_state()?;
            app.manage(oauth);

            let window_state = get_window_state();
            app.manage(window_state);

            let yt_state = get_youtube_scraper_state();
            app.manage(yt_state);

            initial(app.state());

            app.listen("deep-link://new-url", |url| {
                println!("got url {:?}", url);
            });

            println!("Creating new window");
            WebviewWindowBuilder::new(app, "main", tauri::WebviewUrl::App("/mainWindow".into()))
                .build()?;
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
