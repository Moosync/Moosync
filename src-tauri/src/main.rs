// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use preference_holder::preferences::{initial, load_selective, save_selective};
use tauri::{Manager, WindowBuilder};

mod logger;
mod preference_holder;

fn main() {
    initial();
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![save_selective, load_selective])
        .setup(|app| {
            WindowBuilder::new(app, "main", tauri::WindowUrl::App("/mainWindow".into())).build()?;
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
