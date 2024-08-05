use std::{
    sync::{Arc, Mutex},
    thread,
};

use database::database::Database;
use file_scanner::scanner::ScannerHolder;
use macros::generate_command;
use preferences::preferences::PreferenceConfig;
use serde_json::Value;
use tauri::{async_runtime, App, AppHandle, Emitter, Manager, State};
use types::errors::errors::Result;

use crate::scanner::start_scan;

const UI_KEYS: &[&str] = &[
    "prefs.system_settings",
    "prefs.queue_settings",
    "prefs.audio_settings",
    "prefs.gapless_skip",
    "prefs.volume_persist_mode",
];

macro_rules! generate_states {
    ($app:expr, $( $state_type:ty ),*) => {
        {
            // Create a tuple to hold the state variables
            let tuple = ( $( $app.state::<$state_type>().clone() ),* );
            tuple
        }
    };
}

pub fn handle_pref_changes(app: AppHandle) {
    async_runtime::spawn(async move {
        let pref_config: State<PreferenceConfig> = app.state::<PreferenceConfig>().clone();
        let receiver = pref_config.get_receiver();
        for (key, value) in receiver {
            println!("Received key: {} value: {}", key, value);
            if UI_KEYS.contains(&key.as_str()) {
                let app = app.clone();
                println!("Emitting preference-changed event");
                if let Err(e) = app.emit("preference-changed", (key.clone(), value.clone())) {
                    println!("Error emitting preference-changed event{}", e);
                } else {
                    println!("Emitted preference-changed event");
                }
            }

            if key == "prefs.music_paths" || key == "prefs.exclude_music_paths" {
                let app = app.clone();
                thread::spawn(move || {
                    let app = app.clone();
                    let (pref_config, scanner, database) =
                        generate_states!(app, PreferenceConfig, ScannerHolder, Database);
                    if let Err(e) = start_scan(scanner, database, pref_config, None, true) {
                        println!("{}", e);
                    }
                });
            }
        }
    });
}

pub fn get_preference_state(app: &mut App) -> Result<PreferenceConfig> {
    let data_dir = app.path().app_config_dir()?;
    PreferenceConfig::new(data_dir)
}

pub fn initial(app: &mut App) {
    let pref_config: State<PreferenceConfig> = app.state();
    if !pref_config.has_key("thumbnail_path") {
        let path = app.path().app_local_data_dir().unwrap().join("thumbnails");
        let _ = pref_config.save_selective("thumbnail_path".to_string(), path);
    }

    if !pref_config.has_key("artwork_path") {
        let path = app.path().app_local_data_dir().unwrap().join("artwork");
        let _ = pref_config.save_selective("artwork_path".to_string(), path);
    }
}

generate_command!(load_selective, PreferenceConfig, Value, key: String);
generate_command!(save_selective, PreferenceConfig, (), key: String, value: Value);
generate_command!(get_secure, PreferenceConfig, Value, key: String);
generate_command!(set_secure, PreferenceConfig, (), key: String, value: Value);
generate_command!(load_selective_array, PreferenceConfig, Value, key: String);
