use std::{
    collections::HashMap,
    fs::File,
    io::{Read, Write},
};

use json_dotpath::DotPaths;
use preferences::{Preferences, PreferencesMap};
use serde::Serialize;
use serde_json::Value;
use tauri::State;

use crate::state::PreferenceConfig;

#[tauri::command]
pub fn load_selective(config: State<PreferenceConfig>, key: String) -> Value {
    let mut config_file = File::open(&config.config_file).unwrap();
    let mut prefs = String::new();
    config_file.read_to_string(&mut prefs).unwrap();

    let value: Value = serde_json::from_str(&prefs).unwrap();
    let val: Option<Value> = value.dot_get(format!("prefs.{}", key).as_str()).unwrap();
    if val.is_none() {
        println!("No value found for {}", key);
        return Value::Null;
    }

    val.unwrap()
}

#[tauri::command]
pub fn save_selective(config: State<PreferenceConfig>, key: String, value: Value) {
    let mut config_file = File::open(&config.config_file).unwrap();
    let mut prefs = String::new();
    config_file.read_to_string(&mut prefs).unwrap();

    let mut prefs: Value = serde_json::from_str(&prefs).unwrap();
    prefs
        .dot_set(format!("prefs.{}", key).as_str(), value)
        .unwrap();

    let mut config_file = File::create(&config.config_file).unwrap();
    config_file
        .write_all(serde_json::to_string(&prefs).unwrap().as_bytes())
        .unwrap();
}

pub fn initial(state: State<PreferenceConfig>) {
    save_selective(state.clone(), "hotkeys".into(), Value::Array(vec![]));
    save_selective(state.clone(), "isFirstLaunch".into(), Value::Bool(false));
    save_selective(state.clone(), "youtubeAlt".into(), Value::Array(vec![]));
}
