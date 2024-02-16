use database::database::Database;
use file_scanner::scanner::ScannerHolder;
use preferences::preferences::PreferenceConfig;
use tauri::State;
use types::errors::errors::Result;

pub fn get_scanner_state() -> ScannerHolder {
    ScannerHolder::new()
}

#[tauri::command(async)]
pub fn start_scan(
    scanner: State<ScannerHolder>,
    database: State<Database>,
    preferences: State<PreferenceConfig>,
    mut paths: Option<Vec<String>>,
    force: bool,
) -> Result<()> {
    if paths.is_none() {
        paths = Some(preferences.get_scan_paths()?);
    }

    let thumbnail_dir = preferences
        .load_selective("thumbnailPath".to_string())?
        .as_str()
        .unwrap()
        .to_string();

    let artist_split = preferences
        .load_selective("scan_splitter".to_string())?
        .as_str()
        .unwrap_or(";")
        .to_string();

    for path in paths.unwrap() {
        scanner.start_scan(
            database.inner(),
            path,
            thumbnail_dir.clone(),
            artist_split.clone(),
            force,
        )?;
    }

    Ok(())
}
