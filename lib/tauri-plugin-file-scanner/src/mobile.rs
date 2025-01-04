use std::collections::HashMap;

use serde::de::DeserializeOwned;
use serde_json::Value;
use tauri::{
    plugin::{PluginApi, PluginHandle},
    AppHandle, Runtime,
};

use types::{
    errors::{MoosyncError, Result},
    songs::Song,
};

#[cfg(target_os = "ios")]
tauri::ios_plugin_binding!(init_plugin_file_scanner);

#[cfg(target_os = "android")]
const PLUGIN_IDENTIFIER: &str = "app.moosync.filescanner";

// initializes the Kotlin or Swift plugin classes
pub fn init<R: Runtime, C: DeserializeOwned>(
    _app: &AppHandle<R>,
    api: PluginApi<R, C>,
) -> Result<FileScanner<R>> {
    #[cfg(target_os = "android")]
    let handle = api
        .register_android_plugin(PLUGIN_IDENTIFIER, "FileScannerPlugin")
        .map_err(|e| MoosyncError::String(e.to_string()))?;
    #[cfg(target_os = "ios")]
    let handle = api
        .register_ios_plugin(init_plugin_file_scanner)
        .map_err(|e| MoosyncError::String(e.to_string()))?;
    Ok(FileScanner(handle))
}

/// Access to the file-scanner APIs.
pub struct FileScanner<R: Runtime>(PluginHandle<R>);

impl<R: Runtime> FileScanner<R> {
    pub fn scan_music(&self) -> Result<Vec<Song>> {
        println!("Calling scan music");
        let ret: serde_json::Value = self
            .0
            .run_mobile_plugin("android_scan_music", ())
            .map_err(|e| MoosyncError::String(e.to_string()))?;
        let songs = ret.get("songs");
        if let Some(songs) = songs {
            let songs = songs.as_str();
            if let Some(songs) = songs {
                let songs: Vec<Song> = serde_json::from_str(songs)?;
                return Ok(songs);
            }
        }

        Ok(vec![])
    }
}
