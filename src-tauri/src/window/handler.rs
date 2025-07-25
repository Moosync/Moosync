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

use std::env;
use std::path::PathBuf;

use macros::{generate_command, generate_command_async};
use preferences::preferences::PreferenceConfig;
use serde_json::Value;
use tauri::{App, AppHandle, Emitter, Manager, State, WebviewWindow, WebviewWindowBuilder, Window};
use tauri_plugin_dialog::{DialogExt, FilePath};
use tauri_plugin_opener::OpenerExt;
use types::errors::{MoosyncError, Result};
use types::preferences::CheckboxPreference;
use types::window::{DialogFilter, FileResponse};
use types::errors::error_helpers;

#[derive(Debug)]
pub struct WindowHandler {}

impl WindowHandler {
    #[tracing::instrument(level = "debug", skip())]
    pub fn new() -> WindowHandler {
        WindowHandler {}
    }

    #[tracing::instrument(level = "debug", skip(self, window))]
    pub fn is_maximized(&self, window: Window) -> Result<bool> {
        window.is_maximized()
            .map_err(error_helpers::to_plugin_error)
    }

    #[tracing::instrument(level = "debug", skip(self))]
    pub fn has_frame(&self) -> Result<bool> {
        Ok(cfg!(unix) || cfg!(target_os = "macos"))
    }

    #[tracing::instrument(level = "debug", skip(self, window))]
    pub fn close_window(&self, window: Window) -> Result<()> {
        #[cfg(not(any(target_os = "android", target_os = "ios")))]
        window.close()
            .map_err(error_helpers::to_plugin_error)?;
        Ok(())
    }

    #[tracing::instrument(level = "debug", skip(self))]
    pub fn get_platform(&self) -> Result<String> {
        Ok(env::consts::OS.to_string())
    }

    #[tracing::instrument(level = "debug", skip(self, window))]
    pub fn maximize_window(&self, window: Window) -> Result<()> {
        #[cfg(not(any(target_os = "android", target_os = "ios")))]
        window.maximize()
            .map_err(error_helpers::to_plugin_error)?;
        Ok(())
    }

    #[tracing::instrument(level = "debug", skip(self, window))]
    pub fn minimize_window(&self, window: Window) -> Result<()> {
        #[cfg(not(any(target_os = "android", target_os = "ios")))]
        window.minimize()
            .map_err(error_helpers::to_plugin_error)?;
        Ok(())
    }

    #[tracing::instrument(level = "debug", skip(self, app, preference))]
    pub fn update_zoom(&self, app: AppHandle, preference: State<PreferenceConfig>) -> Result<()> {
        let scale_factor: f64 = preference.load_selective("zoomFactor".into())?;
        let windows = app.webview_windows();
        for window in windows.values() {
            window.with_webview(move |webview| {
                #[cfg(target_os = "linux")]
                {
                    // see https://docs.rs/webkit2gtk/0.18.2/webkit2gtk/struct.WebView.html
                    // and https://docs.rs/webkit2gtk/0.18.2/webkit2gtk/trait.WebViewExt.html
                    use webkit2gtk::WebViewExt;
                    webview.inner().set_zoom_level(scale_factor);
                }

                #[cfg(windows)]
                unsafe {
                    // see https://docs.rs/webview2-com/0.19.1/webview2_com/Microsoft/Web/WebView2/Win32/struct.ICoreWebView2Controller.html
                    webview.controller().SetZoomFactor(scale_factor).unwrap();
                }

                // #[cfg(target_os = "macos")]
                // unsafe {
                //     use objc::{sel, sel_impl};
                //     let () = objc::msg_send![webview.inner(), setPageZoom: scale_factor];
                // }
            })
            .map_err(error_helpers::to_plugin_error)?;
        }

        Ok(())
    }

    #[tracing::instrument(level = "debug", skip(self, url))]
    pub fn open_external(&self, app: AppHandle, url: String) -> Result<()> {
        app.opener()
            .open_url(url, None::<&str>)
            .map_err(|err| MoosyncError::String(err.to_string()))?;
        Ok(())
    }

    #[tracing::instrument(level = "debug", skip(self, app, is_main_window))]
    pub fn open_window(&self, app: AppHandle, is_main_window: bool) -> Result<()> {
        if !is_main_window {
            WebviewWindowBuilder::new(
                &app,
                "settings",
                tauri::WebviewUrl::App("/preferenceWindow".into()),
            )
            .build()
            .map_err(error_helpers::to_plugin_error)?;
        }

        Ok(())
    }

    #[tracing::instrument(level = "debug", skip(self, window))]
    pub fn enable_fullscreen(&self, window: Window) -> Result<()> {
        #[cfg(not(any(target_os = "android", target_os = "ios")))]
        window.set_fullscreen(true)
            .map_err(error_helpers::to_plugin_error)?;
        Ok(())
    }

    #[tracing::instrument(level = "debug", skip(self, window))]
    pub fn disable_fullscreen(&self, window: Window) -> Result<()> {
        #[cfg(not(any(target_os = "android", target_os = "ios")))]
        window.set_fullscreen(false)
            .map_err(error_helpers::to_plugin_error)?;
        Ok(())
    }

    #[tracing::instrument(level = "debug", skip(self, window))]
    pub fn toggle_fullscreen(&self, window: Window) -> Result<()> {
        #[cfg(not(any(target_os = "android", target_os = "ios")))]
        {
            let is_fullscreen = window.is_fullscreen()
                .map_err(error_helpers::to_plugin_error)?;
            window.set_fullscreen(!is_fullscreen)
                .map_err(error_helpers::to_plugin_error)?;
        }
        Ok(())
    }

    #[tracing::instrument(level = "debug", skip(self, window))]
    pub fn toggle_dev_tools(&self, window: WebviewWindow) -> Result<()> {
        let is_devtools_open = window.is_devtools_open();
        if !is_devtools_open {
            window.open_devtools();
        } else {
            window.close_devtools();
        }

        Ok(())
    }

    #[tracing::instrument(level = "debug", skip(self, app))]
    pub fn restart_app(&self, app: AppHandle) -> Result<()> {
        app.restart();
    }

    #[tracing::instrument(level = "debug", skip(self, app, directory, multiple, filters))]
    pub async fn open_file_browser(
        &self,
        app: AppHandle,
        directory: bool,
        multiple: bool,
        filters: Vec<DialogFilter>,
    ) -> Result<Vec<FileResponse>> {
        #[cfg(any(target_os = "android", target_os = "ios"))]
        {
            Ok(vec![])
        }

        #[cfg(not(any(target_os = "android", target_os = "ios")))]
        {
            let mut dialog = app.dialog().file();
            for filter in filters {
                dialog = dialog.add_filter(
                    filter.name,
                    filter
                        .extensions
                        .iter()
                        .map(|e| e.as_str())
                        .collect::<Vec<&str>>()
                        .as_slice(),
                );
            }

            let files = if directory {
                if multiple {
                    dialog.blocking_pick_folders().map(|v| {
                        v.iter()
                            .filter_map(|f| {
                                if let FilePath::Path(path) = f {
                                    Some(path.clone())
                                } else {
                                    None
                                }
                            })
                            .collect()
                    })
                } else {
                    let file_path = dialog.blocking_pick_folder();
                    if let Some(FilePath::Path(path)) = file_path {
                        Some(vec![path])
                    } else {
                        Some(vec![])
                    }
                }
            } else if multiple {
                dialog.blocking_pick_files().map(|v| {
                    v.iter()
                        .filter_map(|f| {
                            if let FilePath::Path(path) = f {
                                Some(path.clone())
                            } else {
                                None
                            }
                        })
                        .collect()
                })
            } else {
                let file_path = dialog.blocking_pick_file();
                if let Some(FilePath::Path(path)) = file_path {
                    Some(vec![path])
                } else {
                    Some(vec![])
                }
            };

            let mut ret = vec![];
            if let Some(files) = files {
                for file in files {
                    ret.push(FileResponse {
                        name: file.file_name().unwrap().to_string_lossy().to_string(),
                        path: file.to_string_lossy().to_string(),
                        size: 0,
                    })
                }
            }

            Ok(ret)
        }
    }

    #[tracing::instrument(level = "debug", skip(self))]
    pub fn open_save_file(&self, app: AppHandle) -> Result<PathBuf> {
        #[cfg(any(target_os = "android", target_os = "ios"))]
        {
            let res = app.dialog().file().blocking_save_file();
            if let Some(FilePath::Path(path)) = res {
                return Ok(path.clone());
            }
        }
        Err("No file selected".into())
    }
}

#[tracing::instrument(level = "debug", skip())]
pub fn get_window_state() -> WindowHandler {
    WindowHandler::new()
}

#[tracing::instrument(level = "debug", skip(app))]
pub fn handle_window_close(app: &AppHandle) -> Result<bool> {
    let preferences: State<PreferenceConfig> = app.state();
    let preferences: CheckboxPreference =
        preferences.load_selective_array("system_settings.minimize_to_tray".into())?;
    if preferences.enabled {
        return Ok(false);
    }

    Ok(true)
}

#[tracing::instrument(level = "debug", skip(app))]
pub fn build_tray_menu(app: &App) -> Result<()> {
    #[cfg(not(any(target_os = "android", target_os = "ios")))]
    {
        use tauri::menu::MenuBuilder;
        use tauri::tray::{MouseButton, MouseButtonState, TrayIconEvent};
        let menu = MenuBuilder::new(app)
            .icon(
                "show",
                "Show App",
                app.default_window_icon().cloned().unwrap(),
            )
            .icon("play", "Play", app.default_window_icon().cloned().unwrap())
            .icon(
                "pause",
                "Pause",
                app.default_window_icon().cloned().unwrap(),
            )
            .icon("next", "Next", app.default_window_icon().cloned().unwrap())
            .icon("prev", "Prev", app.default_window_icon().cloned().unwrap())
            .icon("quit", "Quit", app.default_window_icon().cloned().unwrap())
            .build()
            .map_err(error_helpers::to_plugin_error)?;

        tauri::tray::TrayIconBuilder::new()
            .icon(app.default_window_icon().unwrap().clone())
            .menu(&menu)
            .on_menu_event(move |app, event| match event.id().as_ref() {
                "show" => {
                    let _ = app.get_webview_window("main").unwrap().show();
                }
                "play" => {
                    let _ = app.emit("media_button_press", (0, Value::Null));
                }
                "pause" => {
                    let _ = app.emit("media_button_press", (1, Value::Null));
                }
                "next" => {
                    let _ = app.emit("media_button_press", (6, Value::Null));
                }
                "prev" => {
                    let _ = app.emit("media_button_press", (7, Value::Null));
                }
                "quit" => {
                    app.exit(0);
                }
                _ => (),
            })
            .on_tray_icon_event(|tray, event| {
                if let TrayIconEvent::Click {
                    button: MouseButton::Left,
                    button_state: MouseButtonState::Up,
                    ..
                } = event
                {
                    let app = tray.app_handle();
                    if let Some(webview_window) = app.get_webview_window("main") {
                        let _ = webview_window.show();
                        let _ = webview_window.set_focus();
                    }
                }
            })
            .build(app)
            .map_err(error_helpers::to_plugin_error)?;
    }
    Ok(())
}

generate_command!(is_maximized, WindowHandler, bool, window: Window);
generate_command!(has_frame, WindowHandler, bool,);
generate_command!(close_window, WindowHandler, (), window: Window);
generate_command!(get_platform, WindowHandler, String,);
generate_command!(maximize_window, WindowHandler, (), window: Window);
generate_command!(minimize_window, WindowHandler, (), window: Window);
generate_command!(update_zoom, WindowHandler, (), app: AppHandle, preference: State<PreferenceConfig>);
generate_command!(open_external, WindowHandler, (), app: AppHandle, url: String);
generate_command!(open_window, WindowHandler, (), app: AppHandle, is_main_window: bool);
generate_command!(enable_fullscreen, WindowHandler, (), window: Window);
generate_command!(disable_fullscreen, WindowHandler, (), window: Window);
generate_command!(toggle_fullscreen, WindowHandler, (), window: Window);
generate_command!(toggle_dev_tools, WindowHandler, (), window: WebviewWindow);
generate_command!(restart_app, WindowHandler, (), app: AppHandle);
generate_command_async!(open_file_browser, WindowHandler, Vec<FileResponse>, app: AppHandle, directory: bool, multiple: bool, filters: Vec<DialogFilter>);
