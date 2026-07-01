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
    collections::HashMap,
    fs::{self, File},
    io::Write,
    path::PathBuf,
    str::FromStr,
};

use fs_extra::dir::CopyOptions;
use futures::StreamExt;
use themes_proto::moosync::types::ThemeDetails;
use types::subscription::SubscriberList;
use uuid::Uuid;

use crate::error::ThemesError;

pub type OnThemeChangedCallback = Box<dyn Fn(&ThemeDetails) -> () + Send + Sync + 'static>;

pub struct ThemeHolder {
    pub theme_dir: PathBuf,
    pub tmp_dir: PathBuf,
    pub on_theme_changed: SubscriberList<OnThemeChangedCallback>,
}

#[plugin_macro::generate]
impl ThemeHolder {
    #[tracing::instrument(level = "debug", skip_all)]
    pub fn new(theme_dir: PathBuf, tmp_dir: PathBuf) -> Self {
        Self {
            theme_dir,
            tmp_dir,
            on_theme_changed: SubscriberList::new(),
        }
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub fn save_theme(&self, theme: ThemeDetails) -> Result<(), ThemesError> {
        let mut theme = theme;
        if theme.id.is_empty() {
            theme.id = Uuid::new_v4().to_string();
        }

        let theme_path = self.theme_dir.join(theme.id.clone());

        if !theme_path.exists() {
            fs::create_dir_all(&theme_path).map_err(ThemesError::Io)?;
        }
        let theme_config = theme_path.join("config.json");
        fs::write(theme_config, serde_json::to_string(&theme)?).map_err(ThemesError::Io)?;

        self.on_theme_changed.run_all(|sub| {
            sub(&theme);
        });
        Ok(())
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub fn remove_theme(&self, id: String) -> Result<(), ThemesError> {
        let theme_path = self.theme_dir.join(id.clone());
        if theme_path.exists() {
            fs::remove_dir_all(&theme_path).map_err(ThemesError::Io)?;
        }

        Ok(())
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub fn load_theme(&self, id: String) -> Result<ThemeDetails, ThemesError> {
        if id == "default" {
            return Ok(ThemeDetails::default());
        }
        let root_dir = self.theme_dir.join(id.clone());
        let theme_config = root_dir.join("config.json");
        if theme_config.exists() {
            let data = fs::read_to_string(theme_config).map_err(ThemesError::Io)?;
            return Ok(serde_json::from_str(&data)?);
        }

        Err(ThemesError::ThemeNotFound)
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub fn load_all_themes(&self) -> Result<HashMap<String, ThemeDetails>, ThemesError> {
        let theme_dir = self.theme_dir.clone();
        if !theme_dir.exists() {
            fs::create_dir_all(&theme_dir).map_err(ThemesError::Io)?;
        }
        let entries = fs::read_dir(theme_dir).map_err(ThemesError::Io)?;
        let mut ret = HashMap::new();
        ret.insert("default".into(), ThemeDetails::default());
        for theme_dir in entries.flatten() {
            if theme_dir.path().is_dir() {
                let id = theme_dir.file_name().to_str().unwrap().to_string();
                match self.load_theme(id.clone()) {
                    Ok(theme) => {
                        ret.insert(id, theme);
                    }
                    Err(e) => tracing::error!("Failed to load theme {}: {:?}", id, e),
                }
            }
        }

        Ok(ret)
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub fn import_theme(&self, theme_path: String) -> Result<(), ThemesError> {
        let extract_dir = self
            .tmp_dir
            .join(format!("moosync_theme_{}", uuid::Uuid::new_v4()));

        let theme_path = PathBuf::from_str(&theme_path).unwrap();
        zip_extensions::zip_extract(&theme_path, &extract_dir.clone())
            .map_err(|e| ThemesError::Zip(format!("{:?}", e)))?;

        for item in extract_dir.read_dir().map_err(ThemesError::Io)?.flatten() {
            let item = item.path();
            if item.is_file() && item.file_name().unwrap().to_string_lossy() == "config.json" {
                let config = fs::read(item).map_err(ThemesError::Io)?;
                let parsed: ThemeDetails = serde_json::from_slice(config.as_slice())?;
                let final_theme_path = self.theme_dir.join(&parsed.id);
                let options = CopyOptions::default().overwrite(true);

                fs::create_dir_all(final_theme_path.clone()).map_err(ThemesError::Io)?;

                let mut item_list = vec![];
                for items in extract_dir.read_dir().map_err(ThemesError::Io)? {
                    item_list.push(items.unwrap().path());
                }
                tracing::info!("Moving from {:?} to {:?}", item_list, final_theme_path);
                fs_extra::move_items(item_list.as_slice(), final_theme_path, &options)
                    .map_err(ThemesError::FsExtra)?;

                return Ok(());
            }
        }
        Err(ThemesError::ParseThemeFailed)
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub fn export_theme(&self, id: String, export_path: PathBuf) -> Result<(), ThemesError> {
        let mut export_path = export_path.clone();
        export_path.set_extension("mstx");

        let theme = self.load_theme(id.clone())?;
        let theme_dir = self.tmp_dir.join(format!("theme-unpacked-{}", id));
        if !theme_dir.exists() {
            fs::create_dir_all(theme_dir.clone()).map_err(ThemesError::Io)?;
        }

        let config_path = theme_dir.clone().join("config.json");
        fs::write(config_path.clone(), serde_json::to_string_pretty(&theme)?)
            .map_err(ThemesError::Io)?;

        zip_extensions::zip_create_from_directory(&export_path, &theme_dir)
            .map_err(|e| ThemesError::Zip(format!("{:?}", e)))?;

        fs::remove_file(config_path).map_err(ThemesError::Io)?;
        fs::remove_dir(theme_dir).map_err(ThemesError::Io)?;

        Ok(())
    }

    // TODO: Validate URL somehow
    #[tracing::instrument(level = "debug", skip_all)]
    pub async fn download_theme(&self, url: String) -> Result<(), ThemesError> {
        let file_path = self.tmp_dir.join(format!("{}.mstx", uuid::Uuid::new_v4()));

        let mut stream = reqwest::get(url)
            .await
            .map_err(ThemesError::Network)?
            .bytes_stream();
        let mut file = File::create(file_path.clone()).map_err(ThemesError::Io)?;

        while let Some(chunk_result) = stream.next().await {
            let chunk = chunk_result.map_err(ThemesError::Network)?;
            file.write_all(&chunk).map_err(ThemesError::Io)?;
        }

        tracing::info!("Wrote file");

        self.import_theme(file_path.to_string_lossy().to_string())?;

        Ok(())
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub async fn get_themes_manifest(&self) -> Result<HashMap<String, ThemeDetails>, ThemesError> {
        #[derive(serde::Deserialize, Debug, Clone)]
        struct GithubReleaseAsset {
            browser_download_url: String,
            name: String,
        }

        #[derive(serde::Deserialize, Debug)]
        struct GithubReleasesResp {
            assets: Vec<GithubReleaseAsset>,
        }

        #[derive(serde::Deserialize, Debug)]
        struct ThemeItemHelper {
            data: ThemeDetails,
        }

        let client = reqwest::Client::new();
        let res = client.get(
            "https://api.github.com/repos/Moosync/themes/releases/latest",
        )        .header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/58.0.3029.110 Safari/537.3")
        .header("Accept", "application/json")
        .send()
        .await
        .map_err(ThemesError::Network)?;

        let releases_resp = res
            .json::<GithubReleasesResp>()
            .await
            .map_err(ThemesError::Network)?;

        let mut ret = HashMap::new();
        for item in releases_resp.assets.clone() {
            if item.name == "manifest.json" {
                let res = client.get(&item.browser_download_url).header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/58.0.3029.110 Safari/537.3")
                        .header("Accept", "application/json")
                        .send().await
                        .map_err(ThemesError::Network)?;

                let bytes = res.bytes().await.map_err(ThemesError::Network)?;
                let manifests: HashMap<String, ThemeItemHelper> = serde_json::from_slice(&bytes)?;
                for (theme_id, manifest) in manifests {
                    let asset = releases_resp.assets.iter().find(|asset| {
                        asset.name.starts_with(theme_id.as_str()) && asset.name.ends_with(".mstx")
                    });
                    if let Some(asset) = asset {
                        ret.insert(asset.browser_download_url.clone(), manifest.data);
                    }
                }
                break;
            }
        }

        Ok(ret)
    }
}

types::generate_on_event_impl!(
    ThemeHolder;
    on_theme_changed, on_theme_changed_immediate, &ThemeDetails;
);

impl types::plugin::Plugin for ThemeHolder {
    #[tracing::instrument(level = "debug", skip_all)]
    fn init(
        context: &types::plugin::PluginContext,
    ) -> types::plugin::Arc<types::plugin::RwLock<Self>> {
        types::plugin::Arc::new(types::plugin::RwLock::new(ThemeHolder::new(
            context.data_dir.join("themes"),
            context.tmp_dir.clone(),
        )))
    }
}
