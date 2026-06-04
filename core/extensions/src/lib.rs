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
    sync::Arc,
};

use crate::errors::ExtensionError;
use ext_runner::ExtensionHandlerInner;
use extensions_proto::moosync::types::{
    ExtensionCommand, ExtensionCommandResponse, ExtensionDetail, ExtensionManifest,
    FetchedExtensionManifest, PackageName,
};
use fs_extra::dir::CopyOptions;
use futures::{StreamExt, lock::Mutex};
use serde_json::Value;
use ui_proto::moosync::types::PreferenceUiData;
use zip_extensions::zip_extract;

mod context;
pub use context::ReplyHandler;
mod errors;
mod ext_runner;
pub mod models;

#[cfg(test)]
mod sample_tests;
#[cfg(test)]
mod tests;

pub struct ExtensionHandler {
    pub extensions_dir: PathBuf,
    pub tmp_dir: PathBuf,
    pub cache_dir: PathBuf,
    inner: Arc<ExtensionHandlerInner>,
    reply_handler: Option<Arc<dyn ReplyHandler>>,
}

#[plugin_macro::generate]
impl ExtensionHandler {
    #[tracing::instrument(level = "debug")]
    pub fn new(extensions_dir: PathBuf, tmp_dir: PathBuf, cache_dir: PathBuf) -> Self {
        Self {
            inner: Arc::new(ExtensionHandlerInner::new(
                extensions_dir.clone(),
                cache_dir.clone(),
            )),
            extensions_dir,
            tmp_dir,
            cache_dir,
            reply_handler: None,
        }
    }

    pub fn set_reply_handler(&mut self, reply_handler: Arc<dyn ReplyHandler>) {
        self.reply_handler = Some(reply_handler);
    }

    pub fn trigger_extensions_updated(&self) {
        if let Some(ref reply_handler) = self.reply_handler {
            let _ = reply_handler.extensions_updated("");
        }
    }

    #[tracing::instrument(level = "debug", skip(self, ext_path))]
    fn get_extension_version(&self, ext_path: PathBuf) -> Result<String, ExtensionError> {
        let manifest_path = ext_path.join("package.json");
        if manifest_path.exists() {
            let package_manifest: ExtensionManifest =
                serde_json::from_slice(&fs::read(manifest_path)?)?;

            return Ok(package_manifest.version);
        }

        Err(ExtensionError::NoExtensionFound)
    }

    #[tracing::instrument(level = "debug", skip(self, version))]
    fn get_ext_version(&self, version: String) -> Result<u64, ExtensionError> {
        Ok(u64::from_str(
            &version.split('.').collect::<Vec<&str>>().join(""),
        )?)
    }

    #[tracing::instrument(level = "debug", skip(self, ext_path))]
    pub fn install_extension(&self, ext_path: String) -> Result<(), ExtensionError> {
        tracing::debug!("ext path {}", ext_path);
        let ext_path = PathBuf::from_str(&ext_path).unwrap();

        let tmp_dir = self
            .tmp_dir
            .join(format!("moosync_ext_{}", uuid::Uuid::new_v4()));

        zip_extract(&ext_path, &tmp_dir).map_err(|e| ExtensionError::ZipError(Box::new(e)))?;

        let package_manifest: ExtensionManifest =
            serde_json::from_slice(&fs::read(tmp_dir.join("package.json"))?)?;

        if !package_manifest.moosync_extension {
            return Err(ExtensionError::NotAnExtension);
        }

        let ext_extract_path = self.extensions_dir.join(package_manifest.name.clone());

        match self.get_extension_version(ext_extract_path.clone()) {
            Ok(version) => {
                let old_version = self.get_ext_version(version)?;
                let new_version = self.get_ext_version(package_manifest.version)?;

                if new_version > old_version {
                    fs::remove_dir_all(ext_extract_path.clone())?;
                } else {
                    return Err(ExtensionError::DuplicateExtension(package_manifest.name));
                }
            }
            Err(_) => {
                let _ = fs::remove_dir_all(ext_extract_path.clone());
            }
        }

        let options = CopyOptions::default().overwrite(true);
        let parent_dir = ext_extract_path.parent().unwrap();
        tracing::debug!(
            "Moving items from {:?} to {:?}",
            tmp_dir.clone(),
            parent_dir
        );
        if !parent_dir.exists() {
            tracing::debug!("Creating dir {:?}", parent_dir);
            fs::create_dir_all(parent_dir)?;
        }
        fs_extra::move_items(std::slice::from_ref(&tmp_dir), parent_dir, &options)?;

        tracing::debug!(
            "Renaming {:?} to {:?}",
            parent_dir.join(tmp_dir.file_name().unwrap()),
            parent_dir.join(package_manifest.name.clone())
        );
        fs::rename(
            parent_dir.join(tmp_dir.file_name().unwrap()),
            parent_dir.join(package_manifest.name),
        )?;

        self.find_new_extensions()?;

        Ok(())
    }

    #[tracing::instrument(level = "debug", skip(self, package_name))]
    pub fn remove_extension(&self, package_name: String) -> Result<(), ExtensionError> {
        let ext_path = self.extensions_dir.join(package_name.clone());
        if ext_path.exists() {
            fs::remove_dir_all(ext_path)?;
            self.send_remove_extension(PackageName { package_name })?;
            self.find_new_extensions()?;
            Ok(())
        } else {
            Err(ExtensionError::NoExtensionFound)
        }
    }

    #[tracing::instrument(level = "debug", skip(self, fetched_ext))]
    pub async fn download_extension(
        &self,
        fetched_ext: FetchedExtensionManifest,
    ) -> Result<(), ExtensionError> {
        let parsed_url = fetched_ext.url;
        let file_path = self.tmp_dir.join(format!(
            "{}-{}.msox",
            fetched_ext.package_name,
            uuid::Uuid::new_v4()
        ));

        tracing::info!("parsed url {}. Saving at {:?}", parsed_url, file_path);

        let mut stream = reqwest::get(parsed_url).await?.bytes_stream();
        let mut file = File::create(file_path.clone())?;

        while let Some(chunk_result) = stream.next().await {
            let chunk = chunk_result?;
            file.write_all(&chunk)?;
        }

        tracing::info!("Wrote file");

        self.install_extension(file_path.to_string_lossy().to_string())?;

        Ok(())
    }

    fn send_remove_extension(&self, package_name: PackageName) -> Result<(), ExtensionError> {
        self.inner.remove_extension(&package_name.package_name);
        Ok(())
    }

    pub fn find_new_extensions(&self) -> Result<(), ExtensionError> {
        let reply_handler = self.reply_handler.clone().expect("Reply handler not set");
        self.inner.spawn_extensions(reply_handler);
        Ok(())
    }

    #[tracing::instrument(level = "debug", skip(self))]
    pub fn get_installed_extensions(&self) -> Result<Vec<ExtensionDetail>, ExtensionError> {
        Ok(self.inner.get_installed_extensions())
    }

    pub fn get_extension_icon(&self, package_name: String) -> Result<String, ExtensionError> {
        self.inner
            .get_extension_icon(&package_name)
            .ok_or_else(|| ExtensionError::NoExtensionIconFound(package_name))
    }

    pub fn register_ui_preferences(
        &self,
        package_name: String,
        prefs: Vec<PreferenceUiData>,
    ) -> Result<(), ExtensionError> {
        self.inner.register_ui_preferences(package_name, prefs)
    }

    pub fn unregister_ui_preferences(
        &self,
        package_name: String,
        pref_keys: Vec<String>,
    ) -> Result<(), ExtensionError> {
        self.inner
            .unregister_ui_preferences(package_name, pref_keys)
    }

    pub async fn send_extension_command(
        &self,
        command: ExtensionCommand,
    ) -> Result<Option<ExtensionCommandResponse>, ExtensionError> {
        tracing::trace!("Sending extension command {:?}", command);

        self.inner.handle_extension_command(command).await
    }

    pub fn set_extension_active(
        &self,
        package_name: String,
        active: bool,
    ) -> Result<(), ExtensionError> {
        let reply_handler = self.reply_handler.clone().expect("Reply handler not set");
        self.inner
            .set_extension_active(&package_name, active, reply_handler)?;
        Ok(())
    }

    pub fn get_cached_remote_manifests(&self) -> Vec<FetchedExtensionManifest> {
        let path = self.extensions_dir.join("remote_manifest_cache.json");
        if path.exists() {
            if let Ok(contents) = fs::read(path) {
                if let Ok(manifests) = serde_json::from_slice(&contents) {
                    return manifests;
                }
            }
        }
        vec![]
    }

    #[tracing::instrument(level = "debug", skip(self))]
    pub async fn get_extension_manifest(
        &self,
    ) -> Result<Vec<FetchedExtensionManifest>, ExtensionError> {
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
        #[serde(rename_all = "camelCase")]
        struct ExtensionManifestItem {
            display_name: String,
            version: String,
            icon: Option<String>,
            _permissions: HashMap<String, Value>,
        }

        tracing::info!("Getting extension manifest");
        let client = reqwest::Client::new();
        let res = client.get(
            "https://api.github.com/repos/Moosync/moosync-exts/releases/latest",
        )
        .header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/58.0.3029.110 Safari/537.3")
        .header("Accept", "application/json")
        .send()
        .await?;
        let releases_resp = res.json::<GithubReleasesResp>().await?;

        let mut ret = vec![];
        for item in releases_resp.assets.clone() {
            if item.name == "manifest.json" {
                let res = client.get(&item.browser_download_url).header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/58.0.3029.110 Safari/537.3")
                        .header("Accept", "application/json")
                        .send().await?;

                let bytes = res.bytes().await?;
                let manifests: HashMap<String, ExtensionManifestItem> =
                    serde_json::from_slice(&bytes)?;
                for (package_name, manifest) in manifests {
                    let asset = releases_resp.assets.iter().find(|asset| {
                        asset.name.starts_with(package_name.as_str())
                            && asset.name.ends_with(".msox")
                    });
                    if let Some(asset) = asset {
                        let logo_url = manifest.icon.map(|icon| format!("https://raw.githubusercontent.com/Moosync/moosync-exts/refs/heads/v2/{}", icon));
                        ret.push(FetchedExtensionManifest {
                            name: manifest.display_name,
                            package_name,
                            logo: logo_url,
                            description: None,
                            url: asset.browser_download_url.clone(),
                            version: manifest.version,
                        })
                    }
                }
                break;
            }
        }

        let path = self.extensions_dir.join("remote_manifest_cache.json");
        if let Ok(contents) = serde_json::to_vec(&ret) {
            let _ = fs::write(path, contents);
        }

        Ok(ret)
    }
}

impl types::plugin::Plugin for ExtensionHandler {
    fn init(
        context: &types::plugin::PluginContext,
    ) -> types::plugin::Arc<types::plugin::RwLock<Self>> {
        let handler = ExtensionHandler::new(
            context.data_dir.join("extensions"),
            context.tmp_dir.clone(),
            context.cache_dir.clone(),
        );
        types::plugin::Arc::new(types::plugin::RwLock::new(handler))
    }
}
