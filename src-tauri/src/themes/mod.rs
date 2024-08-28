use std::{collections::HashMap, fs};

use macros::generate_command;
use tauri::{App, AppHandle, Manager, State};
use themes::themes::ThemeHolder;
use types::{errors::Result, themes::ThemeDetails};

use crate::window::handler::WindowHandler;

#[tracing::instrument(level = "trace", skip(app))]
pub fn get_theme_handler_state(app: &mut App) -> ThemeHolder {
    let path = app.path().app_local_data_dir().unwrap().join("themes");
    if !path.exists() {
        fs::create_dir_all(path.clone()).unwrap();
    }

    let tmp_dir = app.path().temp_dir().unwrap();

    ThemeHolder::new(path, tmp_dir)
}

#[tracing::instrument(level = "trace", skip(app, theme_handler, window_handler))]
#[tauri::command(async)]
pub fn export_theme(
    app: AppHandle,
    theme_handler: State<ThemeHolder>,
    window_handler: State<WindowHandler>,
    id: String,
) -> Result<()> {
    let selected_file = window_handler.open_save_file(app)?;
    theme_handler.export_theme(id, selected_file)?;
    Ok(())
}

generate_command!(save_theme, ThemeHolder, (), theme: ThemeDetails);
generate_command!(remove_theme, ThemeHolder, (), id: String);
generate_command!(load_theme, ThemeHolder, ThemeDetails, id: String);
generate_command!(load_all_themes, ThemeHolder, HashMap<String, ThemeDetails>,);
generate_command!(import_theme, ThemeHolder, (), path: String);
generate_command!(transform_css, ThemeHolder, String, css_path: String, root: Option<String>);
