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
    sync::{Mutex, mpsc::Sender},
};

use fs_extra::dir::CopyOptions;
use futures::StreamExt;
use notify::{Event, Watcher};
use regex::Regex;
use types::errors::error_helpers;
use types::{
    errors::{MoosyncError, Result},
    themes::ThemeDetails,
};
use uuid::Uuid;

pub struct ThemeHolder {
    pub theme_dir: PathBuf,
    pub tmp_dir: PathBuf,
    watchers: Mutex<HashMap<PathBuf, Box<dyn Watcher + Send>>>,
    change_tx: Sender<String>,
}

impl ThemeHolder {
    #[tracing::instrument(level = "debug", skip(theme_dir, tmp_dir))]
    pub fn new(theme_dir: PathBuf, tmp_dir: PathBuf, change_tx: Sender<String>) -> Self {
        Self {
            theme_dir,
            tmp_dir,
            watchers: Default::default(),
            change_tx,
        }
    }

    fn watch_theme_changes(&self, imports: Vec<PathBuf>) -> Result<()> {
        tracing::info!("Got css immports {:?}", imports);

        let mut watchers = self.watchers.lock().unwrap();
        watchers.clear();

        let root_path = imports.first().unwrap().clone();

        for import in imports {
            let tx = self.change_tx.clone();
            let root_path = root_path.clone();
            let root = self.theme_dir.clone();
            if let Ok(mut watcher) =
                notify::recommended_watcher(move |ev: notify::Result<Event>| {
                    if let Ok(ev) = ev {
                        if ev.kind.is_modify() {
                            match transform_css(
                                root_path.clone().to_string_lossy().to_string(),
                                Some(root.clone()),
                            ) {
                                Ok((transformed, _)) => {
                                    if let Err(e) = tx.send(transformed) {
                                        tracing::error!(
                                            "Failed to notify of file changes: {:?}",
                                            e
                                        );
                                    }
                                }
                                Err(e) => tracing::error!("Failed to transform css: {:?}", e),
                            }
                        }
                    }
                })
            {
                tracing::debug!("Watching path {:?} for changes", import);
                if let Err(e) = watcher.watch(import.as_path(), notify::RecursiveMode::NonRecursive)
                {
                    tracing::error!("Failed to watch path {:?}: {:?}", import, e);
                }
                watchers.insert(import, Box::new(watcher));
            }
        }

        Ok(())
    }

    #[tracing::instrument(level = "debug", skip(self, theme))]
    pub fn save_theme(&self, mut theme: ThemeDetails) -> Result<()> {
        if theme.id.is_empty() {
            theme.id = Uuid::new_v4().to_string();
        }

        let theme_path = self.theme_dir.join(theme.id.clone());

        if !theme_path.exists() {
            fs::create_dir_all(&theme_path).map_err(error_helpers::to_file_system_error)?;
        }
        let theme_config = theme_path.join("config.json");
        fs::write(theme_config, serde_json::to_string(&theme)?)
            .map_err(error_helpers::to_file_system_error)?;

        Ok(())
    }

    #[tracing::instrument(level = "debug", skip(self, id))]
    pub fn remove_theme(&self, id: String) -> Result<()> {
        let theme_path = self.theme_dir.join(id.clone());
        if theme_path.exists() {
            fs::remove_dir_all(&theme_path).map_err(error_helpers::to_file_system_error)?;
        }

        Ok(())
    }

    pub fn get_css(&self, id: String) -> Result<String> {
        let root_dir = self.theme_dir.join(id.clone());
        let ret = self.load_theme(id)?;
        if let Some(custom_css) = &ret.theme.custom_css {
            let (transformed, imports) = transform_css(custom_css.clone(), Some(root_dir))?;
            if let Err(e) = self.watch_theme_changes(imports) {
                tracing::error!("Failed to watch css changes: {:?}", e);
            }
            return Ok(transformed);
        }
        Ok(String::new())
    }

    #[tracing::instrument(level = "debug", skip(self, id))]
    pub fn load_theme(&self, id: String) -> Result<ThemeDetails> {
        {
            let mut watchers = self.watchers.lock().unwrap();
            watchers.clear();
        }
        if id == "default" {
            return Ok(ThemeDetails::default());
        }
        let root_dir = self.theme_dir.join(id.clone());
        let theme_config = root_dir.join("config.json");
        if theme_config.exists() {
            let data =
                fs::read_to_string(theme_config).map_err(error_helpers::to_file_system_error)?;
            return Ok(serde_json::from_str(&data)?);
        }

        Err(MoosyncError::String("Theme not found".to_string()))
    }

    #[tracing::instrument(level = "debug", skip(self))]
    pub fn load_all_themes(&self) -> Result<HashMap<String, ThemeDetails>> {
        let theme_dir = self.theme_dir.clone();
        let entries = fs::read_dir(theme_dir).map_err(error_helpers::to_file_system_error)?;
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

    #[tracing::instrument(level = "debug", skip(self, theme_path))]
    pub fn import_theme(&self, theme_path: String) -> Result<()> {
        let extract_dir = self
            .tmp_dir
            .join(format!("moosync_theme_{}", uuid::Uuid::new_v4()));

        let theme_path = PathBuf::from_str(&theme_path).unwrap();
        zip_extensions::zip_extract(&theme_path, &extract_dir.clone())
            .map_err(error_helpers::to_file_system_error)?;

        for item in extract_dir
            .read_dir()
            .map_err(error_helpers::to_file_system_error)?
        {
            if item.is_ok() {
                let item = item.unwrap().path();
                if item.is_file() && item.file_name().unwrap().to_string_lossy() == "config.json" {
                    let config = fs::read(item).map_err(error_helpers::to_file_system_error)?;
                    let parsed: ThemeDetails = serde_json::from_slice(config.as_slice())?;
                    let final_theme_path = self.theme_dir.join(parsed.id);
                    let options = CopyOptions::default().overwrite(true);

                    fs::create_dir_all(final_theme_path.clone())
                        .map_err(error_helpers::to_file_system_error)?;

                    let mut item_list = vec![];
                    for items in extract_dir
                        .read_dir()
                        .map_err(error_helpers::to_file_system_error)?
                    {
                        item_list.push(items.unwrap().path());
                    }
                    tracing::info!("Moving from {:?} to {:?}", item_list, final_theme_path);
                    fs_extra::move_items(item_list.as_slice(), final_theme_path, &options)
                        .map_err(error_helpers::to_file_system_error)?;

                    return Ok(());
                }
            }
        }
        Err(MoosyncError::String("Failed to parse theme".to_string()))
    }

    #[tracing::instrument(level = "debug", skip(self, id, export_path))]
    pub fn export_theme(&self, id: String, export_path: PathBuf) -> Result<()> {
        let mut export_path = export_path.clone();
        export_path.set_extension("mstx");

        let mut theme = self.load_theme(id.clone())?;
        let theme_dir = self.tmp_dir.join(format!("theme-unpacked-{}", id));
        if !theme_dir.exists() {
            fs::create_dir_all(theme_dir.clone()).map_err(error_helpers::to_file_system_error)?;
        }

        let config_path = theme_dir.clone().join("config.json");

        if let Some(custom_css) = theme.theme.custom_css {
            let (transformed, _) = transform_css(custom_css, None)?;
            let custom_css_path = theme_dir.join("custom.css");
            fs::write(custom_css_path.clone(), transformed)
                .map_err(error_helpers::to_file_system_error)?;
            theme.theme.custom_css = Some("custom.css".into());
        }

        fs::write(config_path.clone(), serde_json::to_string_pretty(&theme)?)
            .map_err(error_helpers::to_file_system_error)?;

        zip_extensions::zip_create_from_directory(&export_path, &theme_dir)
            .map_err(error_helpers::to_file_system_error)?;

        if let Some(custom_css_path) = theme.theme.custom_css {
            fs::remove_file(custom_css_path).map_err(error_helpers::to_file_system_error)?;
        }
        fs::remove_file(config_path).map_err(error_helpers::to_file_system_error)?;
        fs::remove_dir(theme_dir).map_err(error_helpers::to_file_system_error)?;

        Ok(())
    }

    // TODO: Validate URL somehow
    pub async fn download_theme(&self, url: String) -> Result<()> {
        let file_path = self.tmp_dir.join(format!("{}.mstx", uuid::Uuid::new_v4()));

        let mut stream = reqwest::get(url)
            .await
            .map_err(error_helpers::to_network_error)?
            .bytes_stream();
        let mut file =
            File::create(file_path.clone()).map_err(error_helpers::to_file_system_error)?;

        while let Some(chunk_result) = stream.next().await {
            let chunk = chunk_result.map_err(error_helpers::to_network_error)?;
            file.write_all(&chunk)
                .map_err(error_helpers::to_file_system_error)?;
        }

        tracing::info!("Wrote file");

        self.import_theme(file_path.to_string_lossy().to_string())?;

        Ok(())
    }

    pub async fn get_themes_manifest(&self) -> Result<HashMap<String, ThemeDetails>> {
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
        struct ThemeItem {
            data: ThemeDetails,
        }

        let client = reqwest::Client::new();
        let res = client.get(
            "https://api.github.com/repos/Moosync/themes/releases/latest",
        )        .header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/58.0.3029.110 Safari/537.3")
        .header("Accept", "application/json")
        .send()
        .await
        .map_err(error_helpers::to_network_error)?;

        let releases_resp = res
            .json::<GithubReleasesResp>()
            .await
            .map_err(error_helpers::to_network_error)?;

        let mut ret = HashMap::new();
        for item in releases_resp.assets.clone() {
            if item.name == "manifest.json" {
                let res = client.get(&item.browser_download_url).header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/58.0.3029.110 Safari/537.3")
                        .header("Accept", "application/json")
                        .send().await
                        .map_err(error_helpers::to_network_error)?;

                let bytes = res.bytes().await.map_err(error_helpers::to_network_error)?;
                let manifests: HashMap<String, ThemeItem> = serde_json::from_slice(&bytes)?;
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

pub fn transform_css(css_path: String, root: Option<PathBuf>) -> Result<(String, Vec<PathBuf>)> {
    let parsed_path = if let Some(root) = root {
        root.join(css_path)
    } else {
        PathBuf::from(css_path)
    };

    if !parsed_path.exists() {
        return Err(MoosyncError::String(format!(
            "CSS path does not exist: {:?}",
            parsed_path
        )));
    }

    let mut imports = vec![parsed_path.clone()];
    let mut css =
        fs::read_to_string(parsed_path.clone()).map_err(error_helpers::to_file_system_error)?;
    let import_regex = Regex::new(r"@import\s(.*)").unwrap();
    let cloned_css = css.clone();
    let matches = import_regex.captures_iter(cloned_css.as_str());
    for mat in matches {
        let path = mat.get(1);
        if let Some(path) = path {
            let path = path
                .as_str()
                .replace('"', "")
                .strip_suffix(";")
                .unwrap_or_default()
                .to_string();
            let (transformed_css, parsed_imports) =
                transform_css(path, parsed_path.parent().map(|v| v.to_path_buf()))?;

            imports.extend(parsed_imports.into_iter());
            css = css.replace(mat.get(0).unwrap().as_str(), transformed_css.as_str());
        }
    }

    let theme_dir = parsed_path.parent().unwrap();
    css = css.replace("%themeDir%", theme_dir.to_str().unwrap());

    Ok((css, imports))
}
