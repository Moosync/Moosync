use std::env;

use macros::{generate_command, generate_command_async};
use open;
use preferences::preferences::PreferenceConfig;
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager, State, WebviewWindow, WebviewWindowBuilder, Window};
use types::errors::errors::Result;

#[derive(Debug, Serialize, Deserialize)]
pub struct DialogFilter {
    name: String,
    extensions: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FileResponse {
    name: String,
    path: String,
    size: usize,
}

pub struct WindowHandler {}

impl WindowHandler {
    pub fn new() -> WindowHandler {
        WindowHandler {}
    }

    pub fn is_maximized(&self, window: Window) -> Result<bool> {
        Ok(window.is_maximized()?)
    }

    pub fn has_frame(&self) -> Result<bool> {
        Ok(cfg!(unix) || cfg!(macos))
    }

    pub fn close_window(&self, window: Window) -> Result<()> {
        window.close()?;
        Ok(())
    }

    pub fn get_platform(&self) -> Result<String> {
        Ok(env::consts::OS.to_string())
    }

    pub fn maximize_window(&self, window: Window) -> Result<()> {
        window.maximize()?;
        Ok(())
    }

    pub fn minimize_window(&self, window: Window) -> Result<()> {
        window.minimize()?;
        Ok(())
    }

    pub fn update_zoom(&self, app: AppHandle, preference: State<PreferenceConfig>) -> Result<()> {
        let scale_factor = preference.load_selective("zoomFactor".into())?.as_f64();
        if let Some(scale_factor) = scale_factor {
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

                    #[cfg(target_os = "macos")]
                    unsafe {
                        let () = msg_send![webview.inner(), setPageZoom: scale_factor];
                    }
                })?;
            }
        }
        Ok(())
    }

    pub fn open_external(&self, url: String) -> Result<()> {
        open::that(url)?;
        Ok(())
    }

    pub fn open_window(&self, app: AppHandle, is_main_window: bool) -> Result<()> {
        if !is_main_window {
            WebviewWindowBuilder::new(
                &app,
                "settings",
                tauri::WebviewUrl::App("/preferenceWindow".into()),
            )
            .build()?;
        }

        Ok(())
    }

    pub fn enable_fullscreen(&self, window: Window) -> Result<()> {
        window.set_fullscreen(true)?;
        Ok(())
    }

    pub fn disable_fullscreen(&self, window: Window) -> Result<()> {
        window.set_fullscreen(false)?;
        Ok(())
    }

    pub fn toggle_fullscreen(&self, window: Window) -> Result<()> {
        let is_fullscreen = window.is_fullscreen()?;
        window.set_fullscreen(!is_fullscreen)?;
        Ok(())
    }

    pub fn toggle_dev_tools(&self, window: WebviewWindow) -> Result<()> {
        let is_devtools_open = window.is_devtools_open();
        if !is_devtools_open {
            window.open_devtools();
        } else {
            window.close_devtools();
        }

        Ok(())
    }

    pub fn restart_app(&self, app: AppHandle) -> Result<()> {
        app.restart();
        Ok(())
    }

    pub async fn open_file_browser(
        &self,
        directory: bool,
        multiple: bool,
        filters: Vec<DialogFilter>,
    ) -> Result<Vec<FileResponse>> {
        let mut dialog = rfd::AsyncFileDialog::new();
        for filter in filters {
            dialog = dialog.add_filter(filter.name, filter.extensions.as_slice());
        }

        let files = if directory {
            if multiple {
                dialog.pick_folders().await
            } else {
                dialog.pick_folder().await.map(|v| vec![v])
            }
        } else if multiple {
            dialog.pick_files().await
        } else {
            dialog.pick_file().await.map(|v| vec![v])
        };

        let mut ret = vec![];
        if let Some(files) = files {
            for file in files {
                ret.push(FileResponse {
                    name: file.file_name(),
                    path: file.path().to_string_lossy().to_string(),
                    size: 0,
                })
            }
        }

        Ok(ret)
    }
}

pub fn get_window_state() -> WindowHandler {
    WindowHandler::new()
}

generate_command!(is_maximized, WindowHandler, bool, window: Window);
generate_command!(has_frame, WindowHandler, bool,);
generate_command!(close_window, WindowHandler, (), window: Window);
generate_command!(get_platform, WindowHandler, String,);
generate_command!(maximize_window, WindowHandler, (), window: Window);
generate_command!(minimize_window, WindowHandler, (), window: Window);
generate_command!(update_zoom, WindowHandler, (), app: AppHandle, preference: State<PreferenceConfig>);
generate_command!(open_external, WindowHandler, (), url: String);
generate_command!(open_window, WindowHandler, (), app: AppHandle, is_main_window: bool);
generate_command!(enable_fullscreen, WindowHandler, (), window: Window);
generate_command!(disable_fullscreen, WindowHandler, (), window: Window);
generate_command!(toggle_fullscreen, WindowHandler, (), window: Window);
generate_command!(toggle_dev_tools, WindowHandler, (), window: WebviewWindow);
generate_command!(restart_app, WindowHandler, (), app: AppHandle);
generate_command_async!(open_file_browser, WindowHandler, Vec<FileResponse>, directory: bool, multiple: bool, filters: Vec<DialogFilter>);
