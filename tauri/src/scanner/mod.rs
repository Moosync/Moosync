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

use std::{
    sync::{Arc, Mutex, atomic::AtomicBool, mpsc::channel},
    thread::{self},
    time::Duration,
};

use database::database::Database;
use file_scanner::ScannerHolder;
use preferences::preferences::PreferenceConfig;
use songs_proto::moosync::types::Song;
use tauri::{AppHandle, Manager, State};
use types::{errors::Result, prelude::SongsExt};

#[tracing::instrument(level = "debug", skip())]
pub fn get_scanner_state() -> ScannerHolder {
    ScannerHolder::new()
}

#[tracing::instrument(level = "debug", skip(preferences))]
fn get_scan_paths(preferences: &State<PreferenceConfig>) -> Result<Vec<String>> {
    let tmp: Vec<String> = preferences.load_selective("music_paths".to_string())?;

    // TODO: Filter using exclude paths
    Ok(tmp)
}

#[derive(Default)]
pub struct ScanTask {
    cancellation_token: Mutex<Option<Arc<AtomicBool>>>,
}

impl ScanTask {
    pub fn spawn_scan_task(&self, app: AppHandle, scan_duration_s: u64) {
        {
            let mut cancellation_token = self.cancellation_token.lock().unwrap();
            if let Some(cancellation_token) = cancellation_token.as_mut() {
                cancellation_token.store(true, std::sync::atomic::Ordering::Release);
            }
        }

        let cancellation_token = Arc::new(AtomicBool::new(false));
        let cancellation_token_inner = Arc::clone(&cancellation_token);

        thread::spawn(move || {
            loop {
                thread::sleep(Duration::from_secs(scan_duration_s));

                tracing::info!("Running scan task - {}s", scan_duration_s);
                if cancellation_token_inner.load(std::sync::atomic::Ordering::Acquire) {
                    tracing::info!("Scan task cancelled - {}s", scan_duration_s);
                    break;
                }

                let app = app.clone();
                let res = start_scan(app, None);
                if let Err(e) = res {
                    tracing::error!("Scan failed: {:?}", e);
                }
            }
        });

        let mut cancellation_token_lock = self.cancellation_token.lock().unwrap();
        *cancellation_token_lock = Some(cancellation_token);
    }
}

#[tracing::instrument(level = "debug", skip(app, paths))]
#[tauri_invoke_proc::parse_tauri_command]
#[tauri::command(async)]
pub fn start_scan(app: AppHandle, paths: Option<Vec<String>>) -> Result<()> {
    start_scan_inner(app, paths)
}

#[cfg(desktop)]
pub fn start_scan_inner(app: AppHandle, mut paths: Option<Vec<String>>) -> Result<()> {
    let preferences = app.state::<PreferenceConfig>();
    if paths.is_none() {
        paths = Some(get_scan_paths(&preferences)?);
    }

    let thumbnail_dir: String = preferences.load_selective("thumbnail_path".to_string())?;
    tracing::debug!("Got thumbnail dir {:?}", thumbnail_dir);

    let artist_split: String = preferences
        .load_selective("artist_splitter".to_string())
        .unwrap_or(";".to_string());

    let scan_threads: f64 = preferences
        .load_selective("scan_threads".to_string())
        .unwrap_or(-1f64);

    for path in paths.unwrap() {
        tracing::info!("Scanning path: {}", path);

        let (playlist_tx, playlist_rx) = channel();
        let (song_tx, song_rx) = channel::<(Option<String>, Vec<Song>)>();

        let app_clone = app.clone();
        thread::spawn(move || {
            let app = app_clone;
            let database = app.state::<Database>();
            for item in playlist_rx {
                for playlist in item {
                    let _ = database.create_playlist(playlist);
                }
            }

            for (playlist_id, songs) in song_rx {
                let res = database.insert_songs(songs);
                if let Ok(res) = res
                    && let Some(playlist_id) = playlist_id.as_ref()
                {
                    for song in res {
                        if let Some(song_id) = song.get_id() {
                            let _ = database.add_to_playlist_bridge(playlist_id.clone(), song_id);
                        }
                    }
                }
            }
        });

        let scanner = app.state::<ScannerHolder>();
        scanner.start_scan(
            path,
            thumbnail_dir.clone(),
            artist_split.clone(),
            scan_threads,
            song_tx,
            playlist_tx,
        )?;
    }

    Ok(())
}

#[cfg(mobile)]
pub fn start_scan_inner(app: AppHandle, mut paths: Option<Vec<String>>) -> Result<()> {
    use tauri_plugin_file_scanner::FileScannerExt;

    tracing::debug!("calling file scanner");
    let file_scanner = app.file_scanner();
    let res: Vec<Song> = file_scanner.scan_music()?;

    tracing::debug!("Got scanned songs {:?}", res);

    let database = app.state::<Database>();
    database.insert_songs(res)?;

    Ok(())
}
