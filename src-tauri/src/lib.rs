// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::{
    io::{Read, Seek, SeekFrom, Write},
    sync::{Arc, Mutex},
};

use http::{header::*, response::Builder as ResponseBuilder, status::StatusCode};
use http_range::HttpRange;
use logger::logger::{log_debug, log_error, log_info, log_warn};
use preference_holder::preferences::{
    get_secure, initial, load_selective, save_selective, set_secure,
};
use serde::Serialize;
use tauri::{
    http::{self, header::CONTENT_TYPE},
    Manager, State, WebviewWindowBuilder,
};
use tauri_plugin_deep_link::DeepLinkExt;

use crate::{
    db::database::{get_db_state, get_entity_by_options, get_songs_by_options, insert_songs},
    oauth::handler::{get_oauth_state, OAuthHandler},
    preference_holder::preferences::get_preference_state,
    types::songs::{GetSongOptions, QueryableSong},
    window::handler::{
        close_window, get_platform, get_window_state, has_frame, is_maximized, maximize_window,
        minimize_window, open_external, update_zoom,
    },
};

use crate::oauth::handler::{register_oauth_path, unregister_oauth_path};

use percent_encoding::percent_decode;

mod db;
mod logger;
mod macros;
mod oauth;
mod preference_holder;
mod types;
mod window;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // let boundary_id = Arc::new(Mutex::new(0));

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
            open_external
        ])
        .setup(|app| {
            let db = get_db_state(app);
            app.manage(db);

            let config = get_preference_state(app)?;
            app.manage(config);

            let oauth = get_oauth_state()?;
            app.manage(oauth);

            let window_state = get_window_state();
            app.manage(window_state);

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
