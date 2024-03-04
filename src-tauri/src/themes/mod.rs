use std::{collections::HashMap, fs};

use macros::generate_command;
use serde_json::Value;
use tauri::{App, Manager, State};
use themes::themes::{ThemeDetails, ThemeHolder};

pub fn get_theme_handler_state(app: &mut App) -> ThemeHolder {
    let path = app.path().app_data_dir().unwrap().join("themes");
    if let Some(parent) = path.parent() {
        if !parent.exists() {
            fs::create_dir_all(parent).unwrap();
        }
    }

    let tmp_dir = app.path().temp_dir().unwrap();

    ThemeHolder::new(path, tmp_dir)
}

generate_command!(save_theme, ThemeHolder, (), theme: Value);
generate_command!(remove_theme, ThemeHolder, (), id: String);
generate_command!(load_theme, ThemeHolder, ThemeDetails, id: String);
generate_command!(load_all_themes, ThemeHolder, HashMap<String, ThemeDetails>,);
generate_command!(import_theme, ThemeHolder, (), theme_path: String);
generate_command!(transform_css, ThemeHolder, String, css_path: String, root: Option<String>);
