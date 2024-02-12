// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use logger::logger::{log_debug, log_error, log_info, log_warn};
use preference_holder::preferences::{initial, load_selective, save_selective};
use serde::Serialize;
use state::get_preference_state;
use tauri::{Manager, WebviewWindowBuilder};

use crate::{
    db::database::{get_db_state, get_entity_by_options, get_songs_by_options, insert_songs},
    types::songs::{GetSongOptions, QueryableSong},
};

mod db;
mod logger;
mod preference_holder;
mod state;
mod types;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            save_selective,
            load_selective,
            // Logger
            log_error,
            log_debug,
            log_info,
            log_warn,
            // DB
            get_songs_by_options,
            get_entity_by_options
        ])
        .setup(|app| {
            let db = get_db_state(app);
            app.manage(db);

            let config = get_preference_state(app)?;
            app.manage(config);

            initial(app.state());

            println!("Creating new window");
            WebviewWindowBuilder::new(app, "main", tauri::WebviewUrl::App("/mainWindow".into()))
                .build()?;
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
