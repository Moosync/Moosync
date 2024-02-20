use std::{collections::HashMap, fs, path::PathBuf};

use serde::{Deserialize, Serialize};
use serde_json::Value;
use types::errors::errors::{MoosyncError, Result};

#[derive(Debug, Serialize, Deserialize, Clone)]

pub struct ThemeItem {
    primary: String,
    secondary: String,
    tertiary: String,
    textPrimary: String,
    textSecondary: String,
    textInverse: String,
    accent: String,
    divider: String,
    customCSS: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ThemeDetails {
    id: String,
    name: String,
    author: Option<String>,
    theme: ThemeItem,
}

pub struct ThemeHolder {
    pub theme_dir: PathBuf,
}

impl ThemeHolder {
    pub fn new(theme_dir: PathBuf) -> Self {
        Self { theme_dir }
    }

    fn validate_theme(&self, theme: Value) -> Result<ThemeDetails> {
        if !theme.is_object() {
            return Err(MoosyncError::String("Theme is not an object".to_string()));
        }

        Ok(serde_json::from_value(theme)?)
    }

    pub fn save_theme(&self, theme: Value) -> Result<()> {
        let parsed = self.validate_theme(theme)?;
        let theme_path = self.theme_dir.join(parsed.id.clone());

        if !theme_path.exists() {
            fs::create_dir_all(&theme_path)?;
            let theme_config = theme_path.join("config.json");
            fs::write(theme_config, serde_json::to_string(&parsed)?)?;
        }

        Ok(())
    }

    pub fn remove_theme(&self, id: String) -> Result<()> {
        let theme_path = self.theme_dir.join(id.clone());
        if theme_path.exists() {
            fs::remove_dir_all(&theme_path)?;
        }

        Ok(())
    }

    pub fn load_theme(&self, id: String) -> Result<ThemeDetails> {
        let theme_config = self.theme_dir.join(id.clone()).join("config.json");
        if theme_config.exists() {
            let data = fs::read_to_string(theme_config)?;
            return Ok(serde_json::from_str(&data)?);
        }

        Err(MoosyncError::String("Theme not found".to_string()))
    }

    pub fn load_all_themes(&self) -> Result<HashMap<String, ThemeDetails>> {
        let theme_dir = self.theme_dir.clone();
        let entries = fs::read_dir(theme_dir)?;
        let mut ret = HashMap::new();
        for theme_dir in entries.flatten() {
            if theme_dir.path().is_dir() {
                let id = theme_dir.file_name().to_str().unwrap().to_string();
                let theme = self.load_theme(id.clone())?;
                ret.insert(id, theme);
            }
        }

        Ok(ret)
    }
}
