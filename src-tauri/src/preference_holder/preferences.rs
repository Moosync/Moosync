use std::fs::File;

use preferences::{Preferences, PreferencesMap};
use serde::Serialize;
use serde_json::Value;
use tauri::State;

use crate::state::PreferenceConfig;

#[tauri::command]
pub fn load_selective(config: State<PreferenceConfig>, key: String) -> Value {
    let mut config_file = File::open(&config.config_file).unwrap();
    let preferences = PreferencesMap::load_from(&mut config_file).unwrap();
    let val: Option<&String> = preferences.get(key.as_str());
    if val.is_none() {
        println!("No value found for {}", key);
        return Value::Null;
    }

    let parsed: Value = serde_json::from_str(val.unwrap()).unwrap();
    parsed
}

#[tauri::command]
pub fn save_selective(config: State<PreferenceConfig>, key: String, value: Value) {
    let mut config_file = File::open(&config.config_file).unwrap();
    let mut prefs: PreferencesMap<String> = PreferencesMap::load_from(&mut config_file).unwrap();
    prefs.insert(
        key,
        value
            .serialize(serde_json::value::Serializer)
            .unwrap()
            .to_string(),
    );

    let mut config_file = File::create(&config.config_file).unwrap();
    prefs.save_to(&mut config_file).unwrap();
}

pub fn initial(state: State<PreferenceConfig>) {
    save_selective(state.clone(), "hotkeys".into(), Value::Array(vec![]));
    save_selective(state.clone(), "isFirstLaunch".into(), Value::Bool(false));
    save_selective(state.clone(), "youtubeAlt".into(), Value::Array(vec![]));
}
