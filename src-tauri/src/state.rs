use std::{
    fs::{self, File},
    io::Write,
    path::PathBuf,
};

use tauri::{App, Error, Manager};

pub struct PreferenceConfig {
    pub config_file: PathBuf,
}

pub fn get_preference_state(app: &mut App) -> Result<PreferenceConfig, Error> {
    let data_dir = app.path().app_config_dir()?;
    let config_file = data_dir.join("config.json");
    println!("{:?}", data_dir);

    if !data_dir.exists() {
        fs::create_dir_all(data_dir)?;
    }

    if !config_file.exists() {
        let mut file = File::create(config_file.clone())?;
        file.write(b"{}")?;
    }

    Ok(PreferenceConfig { config_file })
}
