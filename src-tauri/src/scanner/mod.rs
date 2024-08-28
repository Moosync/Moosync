use database::database::Database;
use file_scanner::scanner::ScannerHolder;
use preferences::preferences::PreferenceConfig;
use tauri::State;
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

#[tracing::instrument(level = "trace", skip(scanner, database, preferences, paths, force))]
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
