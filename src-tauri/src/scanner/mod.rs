use std::{
    sync::{atomic::AtomicBool, Arc, Mutex},
    thread::{self},
    time::Duration,
};

use database::database::Database;
use file_scanner::ScannerHolder;
use preferences::preferences::PreferenceConfig;
use tauri::{AppHandle, Manager, State};
use types::errors::Result;

#[tracing::instrument(level = "trace", skip())]
pub fn get_scanner_state() -> ScannerHolder {
    ScannerHolder::new()
}

#[tracing::instrument(level = "trace", skip(preferences))]
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

        thread::spawn(move || loop {
            thread::sleep(Duration::from_secs(scan_duration_s));

            tracing::info!("Running scan task - {}s", scan_duration_s);
            if cancellation_token_inner.load(std::sync::atomic::Ordering::Acquire) {
                tracing::info!("Scan task cancelled - {}s", scan_duration_s);
                break;
            }

            let scanner = app.state();
            let database = app.state();
            let preferences = app.state();

            let _ = start_scan(scanner, database, preferences, None, false);
        });

        let mut cancellation_token_lock = self.cancellation_token.lock().unwrap();
        *cancellation_token_lock = Some(cancellation_token);
    }
}

#[tracing::instrument(level = "trace", skip(scanner, database, preferences, paths, force))]
#[tauri_invoke_proc::parse_tauri_command]
#[tauri::command(async)]
pub fn start_scan(
    scanner: State<ScannerHolder>,
    database: State<Database>,
    preferences: State<PreferenceConfig>,
    mut paths: Option<Vec<String>>,
    force: bool,
) -> Result<()> {
    if paths.is_none() {
        paths = Some(get_scan_paths(&preferences)?);
    }

    let thumbnail_dir: String = preferences.load_selective("thumbnail_path".to_string())?;

    let artist_split: String = preferences
        .load_selective("artist_splitter".to_string())
        .unwrap_or(";".to_string());

    let scan_threads: f64 = preferences
        .load_selective("scan_threads".to_string())
        .unwrap_or(-1f64);

    for path in paths.unwrap() {
        tracing::info!("Scanning path: {}", path);
        scanner.start_scan(
            database.inner(),
            path,
            thumbnail_dir.clone(),
            artist_split.clone(),
            scan_threads,
            force,
        )?;
    }

    Ok(())
}
