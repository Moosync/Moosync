use std::{collections::HashMap, fmt::Debug};

use lazy_static::lazy_static;
use preferences::{AppInfo, Preferences, PreferencesMap};
use serde::{Deserialize, Serialize};
use serde_json::Value;

const APP_INFO: AppInfo = AppInfo {
    name: "Moosync",
    author: "Sahil Gupte <sahilsachingupte@gmail.com>",
};

lazy_static! {
    static ref PREFERENCE_MAP: PreferencesMap<String> = PreferencesMap::new();
}

#[tauri::command]
pub fn load_selective(key: String) -> Value {
    println!("Loading {}", key);
    let preferences = PreferencesMap::load(&APP_INFO, "config").unwrap();
    let val: Option<&String> = preferences.get(key.as_str());
    if val.is_none() {
        println!("No value found for {}", key);
        return Value::Null;
    }

    let parsed: Value = serde_json::from_str(val.unwrap()).unwrap();
    parsed
}

#[tauri::command]
pub fn save_selective(key: String, value: Value) {
    println!("Saving {} {:?}", key, value);
    let mut prefs: PreferencesMap<String> = PreferencesMap::load(&APP_INFO, "config").unwrap();
    prefs.insert(
        key,
        value
            .serialize(serde_json::value::Serializer)
            .unwrap()
            .to_string(),
    );
    prefs.save(&APP_INFO, "config").unwrap();
}

pub fn initial() {
    save_selective("hotkeys".to_string(), Value::Array(vec![]));
    save_selective("isFirstLaunch".into(), Value::Bool(false));
    save_selective("youtubeAlt".into(), Value::Array(vec![]));
}
