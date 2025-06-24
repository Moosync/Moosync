// Moosync
// Copyright (C) 2024, 2025  Moosync <support@moosync.app>
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <http://www.gnu.org/licenses/>.

use std::{collections::HashMap, fs, sync::mpsc::channel};

use macros::{generate_command, generate_command_async};
use tauri::{App, AppHandle, Emitter, Manager, State};
use themes::themes::ThemeHolder;
use types::{errors::Result, themes::ThemeDetails};

use crate::window::handler::WindowHandler;

#[tracing::instrument(level = "debug", skip(app))]
pub fn get_theme_handler_state(app: &mut App) -> ThemeHolder {
    let path = app.path().app_local_data_dir().unwrap().join("themes");
    if !path.exists() {
        fs::create_dir_all(path.clone()).unwrap();
    }

    let tmp_dir = app.path().temp_dir().unwrap();

    let (tx, rx) = channel();

    let app_handle = app.app_handle().clone();
    tauri::async_runtime::spawn(async move {
        while let Ok(event) = rx.recv() {
            if let Err(e) = app_handle.emit("theme-updated", event) {
                tracing::error!("Failed to emit theme_updated event: {:?}", e);
            }
        }
    });

    ThemeHolder::new(path, tmp_dir, tx)
}

#[tracing::instrument(level = "debug", skip(app, theme_handler, window_handler))]
#[tauri::command(async)]
#[tauri_invoke_proc::parse_tauri_command]
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
generate_command!(get_css, ThemeHolder, String, id: String);
generate_command_async!(get_themes_manifest, ThemeHolder, HashMap<String, ThemeDetails>,);
generate_command_async!(download_theme, ThemeHolder, (), url: String);
generate_command!(get_system_theme_preference, ThemeHolder, String,);
