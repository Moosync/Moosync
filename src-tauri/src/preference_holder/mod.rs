use macros::generate_command;
use preferences::preferences::PreferenceConfig;
use serde_json::Value;
use tauri::{App, Manager, State};
use types::errors::errors::Result;

pub fn get_preference_state(app: &mut App) -> Result<PreferenceConfig> {
    let data_dir = app.path().app_config_dir()?;
    PreferenceConfig::new(data_dir)
}

pub fn initial(state: State<PreferenceConfig>) {
    state
        .save_selective("hotkeys".into(), Value::Array(vec![]))
        .unwrap();
    state
        .save_selective("isFirstLaunch".into(), Value::Bool(false))
        .unwrap();
    state
        .save_selective("youtubeAlt".into(), Value::Array(vec![]))
        .unwrap();
}

generate_command!(load_selective, PreferenceConfig, Value, key: String);
generate_command!(save_selective, PreferenceConfig, (), key: String, value: Value);
generate_command!(get_secure, PreferenceConfig, Value, key: String);
generate_command!(set_secure, PreferenceConfig, (), key: String, value: Value);
generate_command!(load_selective_array, PreferenceConfig, Value, key: String);
