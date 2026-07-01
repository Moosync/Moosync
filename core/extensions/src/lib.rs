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
    sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard},
};

use ext_runner::ExtensionHandlerInner;
use extensions_proto::moosync::types::{
    ExtensionCommand, ExtensionCommandResponse, ExtensionDetail, ExtensionManifest,
    FetchedExtensionManifest, PackageName,
};
use fs_extra::dir::CopyOptions;
use futures::StreamExt;
use serde_json::Value;
use ui_proto::moosync::types::PreferenceUiData;
use zip_extensions::zip_extract;

pub use crate::{errors::ExtensionError, extension::Extension};

mod context;
pub use context::ReplyHandler;
mod errors;
mod ext_runner;
mod extension;
mod remote;
pub use remote::RemoteExtensions;
pub mod models;

#[cfg(test)]
mod sample_tests;
#[cfg(test)]
mod tests;

#[derive(Debug, Clone)]
pub enum ExtensionInfo {
    Local(ExtensionDetail),
    Remote(FetchedExtensionManifest),
    LocalPath(std::path::PathBuf),
}

pub struct ExtensionHandler {
    pub extensions_dir: PathBuf,
    pub tmp_dir: PathBuf,
    pub cache_dir: PathBuf,
    inner: ExtensionHandlerInner,
    reply_handler: Option<Arc<dyn ReplyHandler>>,
    pub on_extensions_updated:
        types::subscription::SubscriberList<Box<dyn Fn(()) + Send + Sync + 'static>>,
    remote: RemoteExtensions,
}

types::generate_on_event_impl!(
    ExtensionHandler;
    on_extensions_updated, ();
);

#[plugin_macro::generate]
impl ExtensionHandler {
    #[tracing::instrument(level = "debug")]
    pub fn new(extensions_dir: PathBuf, tmp_dir: PathBuf, cache_dir: PathBuf) -> Self {
        Self {
            inner: ExtensionHandlerInner::new(extensions_dir.clone(), cache_dir.clone()),
            extensions_dir: extensions_dir.clone(),
            tmp_dir: tmp_dir.clone(),
            cache_dir,
            reply_handler: None,
            on_extensions_updated: types::subscription::SubscriberList::new(),
            remote: RemoteExtensions::new(extensions_dir, tmp_dir),
        }
    }

    pub fn set_reply_handler(&mut self, reply_handler: Arc<dyn ReplyHandler>) {
        self.reply_handler = Some(reply_handler);
    }

    pub fn trigger_extensions_updated(&self) { self.on_extensions_updated.run_all(|cb| cb(())); }

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

    pub fn get_extension(&self, package_name: &str) -> Result<Arc<Extension>, ExtensionError> {
        Ok(self.inner.get_extension(package_name)?)
    }

    pub fn get_extension_mut(&self, package_name: &str) -> Result<Arc<Extension>, ExtensionError> {
        Ok(self.inner.get_extension(package_name)?)
    }

    #[tracing::instrument(level = "debug", skip(self, info))]
    pub async fn install_extension(&self, info: ExtensionInfo) -> Result<(), ExtensionError> {
        let ext_path = match info {
            ExtensionInfo::Local(_) => {
                return Ok(());
            }
            ExtensionInfo::Remote(manifest) => self.remote.download_extension(manifest).await?,
            ExtensionInfo::LocalPath(path) => path,
        };

        tracing::debug!("ext path {:?}", ext_path);

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
        self.trigger_extensions_updated();

        Ok(())
    }

    #[tracing::instrument(level = "debug", skip(self, package_name))]
    pub fn remove_extension(&mut self, package_name: String) -> Result<(), ExtensionError> {
        let ext_path = self.extensions_dir.join(package_name.clone());
        if ext_path.exists() {
            fs::remove_dir_all(ext_path)?;
            self.send_remove_extension(PackageName { package_name })?;
            self.find_new_extensions()?;
            self.trigger_extensions_updated();
            Ok(())
        } else {
            Err(ExtensionError::NoExtensionFound)
        }
    }

    pub fn toggle_extension(&self, info: ExtensionInfo) -> Result<(), ExtensionError> {
        if let ExtensionInfo::Local(detail) = info {
            let extension = self.get_extension(&detail.package_name)?;
            let new_active = !extension.is_active();
            extension.set_active(new_active)?;
            self.trigger_extensions_updated();
            Ok(())
        } else {
            Err(ExtensionError::NoExtensionFound)
        }
    }

    pub fn get_all_extensions(&self) -> Vec<ExtensionInfo> {
        let installed = self.get_installed_extensions();
        let remote = self.get_cached_remote_manifests();

        let mut ret = vec![];
        for inst in installed {
            ret.push(ExtensionInfo::Local(inst));
        }

        for rem in remote {
            if !ret.iter().any(|item| match item {
                ExtensionInfo::Local(detail) => detail.package_name == rem.package_name,
                _ => false,
            }) {
                ret.push(ExtensionInfo::Remote(rem));
            }
        }

        ret
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
    pub fn get_installed_extensions(&self) -> Vec<ExtensionDetail> {
        self.inner.get_installed_extensions()
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
        self.remote.get_extension_manifest().await
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
