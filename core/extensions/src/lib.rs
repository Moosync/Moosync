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
    collections::HashSet,
    fs,
    path::{Path, PathBuf},
    sync::{Arc, Mutex, OnceLock},
};

use ext_runner::ExtensionHandlerInner;
pub use extensions_proto::moosync::types::ExtensionLockData;
use extensions_proto::moosync::types::{ExtensionDetail, ExtensionManifest, PackageName};
use fs_extra::dir::CopyOptions;
use prost::Message;
use semver::Version;
use types::subscription::SubscriberList;
use zip_extensions::zip_extract;

pub use crate::{
    errors::ExtensionError,
    extension::{Extension, ExtensionGroupExt},
};

mod context;
pub use context::ReplyHandler;
mod errors;
mod ext_runner;
mod extension;
pub mod models;
mod remote;
pub use remote::{DEFAULT_EXTENSION_REGISTRY, RemoteExtensions};
pub mod sanitize;
pub use sanitize::Sanitize;

#[cfg(test)]
mod ext_runner_test;
#[cfg(test)]
mod extension_test;
#[cfg(test)]
mod lib_test;
#[cfg(test)]
mod lib_test_smoke;
#[cfg(test)]
mod models_test;
#[cfg(test)]
mod remote_test;
#[cfg(test)]
mod remote_test_smoke;
#[cfg(test)]
mod sanitize_test;

#[cfg(feature = "wasm_integration_tests")]
mod sample_tests;
#[cfg(feature = "wasm_integration_tests")]
mod tests;

#[derive(Debug, Clone)]
pub enum ExtensionInfo {
    Local(ExtensionDetail),
    Remote(ExtensionManifest),
    LocalPath(std::path::PathBuf),
}

impl ExtensionInfo {
    #[tracing::instrument(level = "debug", skip_all)]
    pub fn package_name(&self) -> &str {
        match self {
            ExtensionInfo::Local(detail) => &detail.package_name,
            ExtensionInfo::Remote(manifest) => &manifest.name,
            ExtensionInfo::LocalPath(_) => "",
        }
    }
}

impl std::hash::Hash for ExtensionInfo {
    #[tracing::instrument(level = "debug", skip_all)]
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) { self.package_name().hash(state); }
}

impl PartialEq for ExtensionInfo {
    #[tracing::instrument(level = "debug", skip_all)]
    fn eq(&self, other: &Self) -> bool { self.package_name() == other.package_name() }
}

impl Eq for ExtensionInfo {}

#[derive(Default)]
struct ExtensionConfig {
    remote_manifests: Vec<ExtensionManifest>,
    has_updates: bool,
}

pub struct ExtensionHandler {
    pub extensions_dir: PathBuf,
    pub tmp_dir: PathBuf,
    pub cache_dir: PathBuf,
    inner: ExtensionHandlerInner,
    reply_handler: OnceLock<Arc<dyn ReplyHandler>>,
    pub on_extensions_updated: SubscriberList<()>,
    pub on_accounts_updated: SubscriberList<Option<String>>,
    pub on_preferences_updated: SubscriberList<String>,
    remote: RemoteExtensions,
    config: Mutex<ExtensionConfig>,
}

#[plugin_macro::generate]
impl ExtensionHandler {
    #[tracing::instrument(level = "debug", skip_all)]
    pub fn new(extensions_dir: PathBuf, tmp_dir: PathBuf, cache_dir: PathBuf) -> Self {
        Self {
            inner: ExtensionHandlerInner::new(extensions_dir.clone(), cache_dir.clone()),
            extensions_dir: extensions_dir.clone(),
            tmp_dir: tmp_dir.clone(),
            cache_dir: cache_dir.clone(),
            reply_handler: OnceLock::new(),
            on_extensions_updated: SubscriberList::new(),
            on_accounts_updated: SubscriberList::new(),
            on_preferences_updated: SubscriberList::new(),
            remote: RemoteExtensions::new(tmp_dir),
            config: Mutex::new(ExtensionConfig::default()),
        }
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub fn has_updates(&self) -> bool { self.config.lock().unwrap().has_updates }

    #[tracing::instrument(level = "debug", skip_all)]
    pub fn check_for_updates(&self) -> bool {
        let updatables = self.get_updatable_extensions();
        let mut config = self.config.lock().unwrap();
        config.has_updates = !updatables.is_empty();
        config.has_updates
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub fn get_updatable_extensions(&self) -> Vec<ExtensionManifest> {
        let installed = self.get_installed_extensions();
        let config = self.config.lock().unwrap();
        let mut updatables = Vec::new();
        for inst in installed {
            let Some(remote) = config
                .remote_manifests
                .iter()
                .find(|r| r.name == inst.package_name)
            else {
                continue;
            };

            let Ok(remote_ver) = semver::Version::parse(&remote.version) else {
                continue;
            };
            let Ok(inst_ver) = semver::Version::parse(&inst.version) else {
                continue;
            };

            if remote_ver > inst_ver {
                updatables.push(remote.clone());
            }
        }
        updatables
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub async fn update_all_extensions(&self) -> Result<(), ExtensionError> {
        self.config.lock().unwrap().has_updates = false;
        let updatables = self.get_updatable_extensions();
        let mut any_failed = false;
        for remote in updatables {
            if let Err(e) = self.install_extension(ExtensionInfo::Remote(remote)).await {
                tracing::error!("update_all_extensions: Failed to update: {:?}", e);
                any_failed = true;
            }
        }
        self.check_for_updates();
        self.trigger_extensions_updated();
        if any_failed {
            return Err(ExtensionError::UpdateFailed(
                "Failed to update one or more extensions".to_string(),
            ));
        }
        Ok(())
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub fn set_remote_manifests(&self, manifests: Vec<ExtensionManifest>) {
        self.config.lock().unwrap().remote_manifests = manifests;
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub fn set_reply_handler(&self, reply_handler: Arc<dyn ReplyHandler>) {
        let _ = self.reply_handler.set(reply_handler);
        let _ = self.find_new_extensions();
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub fn trigger_extensions_updated(&self) { self.on_extensions_updated.emit(()); }

    #[tracing::instrument(level = "debug", skip_all)]
    pub fn trigger_accounts_updated(&self, account_id: Option<String>) {
        self.on_accounts_updated.emit(account_id);
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub fn trigger_preferences_updated(&self, package_name: String) {
        self.on_preferences_updated.emit(package_name);
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub(crate) fn get_extension_version(
        &self,
        ext_path: PathBuf,
    ) -> Result<String, ExtensionError> {
        let manifest_path = ext_path.join("manifest.pb");
        if manifest_path.exists() {
            let package_manifest = ExtensionManifest::decode(&fs::read(manifest_path)?[..])?;

            return Ok(package_manifest.version);
        }

        Err(ExtensionError::NoExtensionFound)
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub fn get_extension(&self, package_name: &str) -> Result<Arc<Extension>, ExtensionError> {
        self.inner.get_extension(package_name)
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub fn get_extension_mut(&self, package_name: &str) -> Result<Arc<Extension>, ExtensionError> {
        self.inner.get_extension(package_name)
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub async fn install_extension(&self, info: ExtensionInfo) -> Result<(), ExtensionError> {
        let (ext_path, registry_name) = match info {
            ExtensionInfo::Local(_) => {
                return Ok(());
            }
            ExtensionInfo::Remote(manifest) => {
                let reg = manifest
                    .registry
                    .clone()
                    .unwrap_or_else(|| "remote".to_string());
                (self.remote.download_extension(manifest).await?, reg)
            }
            ExtensionInfo::LocalPath(path) => (path, "local".to_string()),
        };

        tracing::debug!("ext path {:?}", ext_path);

        let tmp_dir = self
            .tmp_dir
            .join(format!("moosync_ext_{}", uuid::Uuid::new_v4()));
        let extensions_dir = self.extensions_dir.clone();

        let staged = tokio::task::spawn_blocking(move || {
            stage_extension_package(&ext_path, tmp_dir, &extensions_dir)
        })
        .await
        .map_err(|e| ExtensionError::Io(std::io::Error::other(e)))??;

        if staged.should_remove_old {
            let _ = self.send_remove_extension(PackageName {
                package_name: staged.package_manifest.name.clone(),
            });
        }

        tokio::task::spawn_blocking(move || finalize_extension_install(staged, registry_name))
            .await
            .map_err(|e| ExtensionError::Io(std::io::Error::other(e)))??;

        self.find_new_extensions()?;
        self.trigger_extensions_updated();

        Ok(())
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub fn remove_extension(&self, package_name: String) -> Result<(), ExtensionError> {
        let ext_path = self.extensions_dir.join(&package_name);
        if ext_path.exists() {
            let _ = fs::remove_dir_all(ext_path);
        }
        self.send_remove_extension(PackageName { package_name })?;
        self.find_new_extensions()?;
        self.trigger_extensions_updated();
        Ok(())
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub fn toggle_extension(&self, info: ExtensionInfo) -> Result<(), ExtensionError> {
        if let ExtensionInfo::Local(detail) = info {
            self.remove_extension(detail.package_name)
        } else {
            Err(ExtensionError::NoExtensionFound)
        }
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub async fn get_all_extensions(&self) -> Vec<ExtensionInfo> {
        let installed = self.get_installed_extensions();
        let config = self.config.lock().unwrap();
        let remote = &config.remote_manifests;

        let mut seen_packages = HashSet::with_capacity(installed.len() + remote.len());
        let mut ret = Vec::with_capacity(installed.len() + remote.len());

        for inst in installed {
            seen_packages.insert(inst.package_name.clone());
            ret.push(ExtensionInfo::Local(inst));
        }

        for rem in remote {
            if !seen_packages.contains(&rem.name) {
                seen_packages.insert(rem.name.clone());
                ret.push(ExtensionInfo::Remote(rem.clone()));
            }
        }

        ret
    }

    #[tracing::instrument(level = "debug", skip_all)]
    fn send_remove_extension(&self, package_name: PackageName) -> Result<(), ExtensionError> {
        self.inner.remove_extension(&package_name.package_name);
        Ok(())
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub fn find_new_extensions(&self) -> Result<(), ExtensionError> {
        let reply_handler = self
            .reply_handler
            .get()
            .cloned()
            .expect("Reply handler not set");
        self.inner.spawn_extensions(reply_handler);
        Ok(())
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub fn get_installed_extensions(&self) -> Vec<ExtensionDetail> {
        self.inner.get_installed_extensions()
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub fn get_active_extensions(&self) -> Vec<Arc<Extension>> {
        self.get_installed_extensions()
            .into_iter()
            .filter_map(|d| {
                if d.has_started && d.error.is_none() {
                    self.get_extension(&d.package_name).ok()
                } else {
                    None
                }
            })
            .collect()
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub async fn get_extensions_with_scope(
        &self,
        scope: extensions_proto::moosync::types::ExtensionProviderScope,
    ) -> Vec<Arc<Extension>> {
        let scope_val = scope as i32;
        let extensions_map = self.inner.extensions_map.read().unwrap();
        extensions_map
            .values()
            .filter(|ext| ext.manifest.scopes.contains(&scope_val))
            .cloned()
            .collect()
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub async fn fetch_remote_manifests(
        &self,
        registries: HashSet<String>,
    ) -> Result<(), ExtensionError> {
        let mut registries = registries;
        registries.insert(DEFAULT_EXTENSION_REGISTRY.to_string());
        let manifests = self.remote.get_extension_manifest(&registries).await?;
        self.set_remote_manifests(manifests);
        self.check_for_updates();
        self.trigger_extensions_updated();
        Ok(())
    }
}

impl types::plugin::Plugin for ExtensionHandler {
    #[tracing::instrument(level = "debug", skip_all)]
    fn init(context: &types::plugin::PluginContext) -> Arc<Self> {
        let handler = ExtensionHandler::new(
            context.data_dir.join("extensions"),
            context.tmp_dir.clone(),
            context.cache_dir.clone(),
        );
        Arc::new(handler)
    }
}

struct StagedExtension {
    package_manifest: ExtensionManifest,
    ext_extract_path: PathBuf,
    tmp_dir: PathBuf,
    should_remove_old: bool,
}

#[tracing::instrument(level = "debug", skip_all)]
#[allow(clippy::ptr_arg)]
fn stage_extension_package(
    ext_path: &PathBuf,
    tmp_dir: PathBuf,
    extensions_dir: &Path,
) -> Result<StagedExtension, ExtensionError> {
    zip_extract(ext_path, &tmp_dir).map_err(|e| ExtensionError::Zip(e.to_string()))?;

    let package_manifest = ExtensionManifest::decode(&fs::read(tmp_dir.join("manifest.pb"))?[..])?;

    if !package_manifest.moosync_extension {
        return Err(ExtensionError::NotAnExtension);
    }

    let ext_extract_path = extensions_dir.join(package_manifest.name.clone());
    let manifest_path = ext_extract_path.join("manifest.pb");
    let should_remove_old = if manifest_path.exists() {
        let existing_manifest = ExtensionManifest::decode(&fs::read(manifest_path)?[..])?;
        let old_version = Version::parse(&existing_manifest.version)?;
        let new_version = Version::parse(&package_manifest.version)?;

        if new_version > old_version {
            true
        } else {
            return Err(ExtensionError::DuplicateExtension(package_manifest.name));
        }
    } else {
        false
    };

    Ok(StagedExtension {
        package_manifest,
        ext_extract_path,
        tmp_dir,
        should_remove_old,
    })
}

#[tracing::instrument(level = "debug", skip_all)]
fn finalize_extension_install(
    staged: StagedExtension,
    registry_name: String,
) -> Result<(), ExtensionError> {
    if staged.should_remove_old {
        fs::remove_dir_all(staged.ext_extract_path.clone())?;
    } else {
        let _ = fs::remove_dir_all(staged.ext_extract_path.clone());
    }

    let options = CopyOptions::default().overwrite(true);
    let parent_dir = staged.ext_extract_path.parent().unwrap();
    tracing::debug!(
        "Moving items from {:?} to {:?}",
        staged.tmp_dir.clone(),
        parent_dir
    );
    if !parent_dir.exists() {
        tracing::debug!("Creating dir {:?}", parent_dir);
        fs::create_dir_all(parent_dir)?;
    }
    fs_extra::move_items(std::slice::from_ref(&staged.tmp_dir), parent_dir, &options)?;

    tracing::debug!(
        "Renaming {:?} to {:?}",
        parent_dir.join(staged.tmp_dir.file_name().unwrap()),
        parent_dir.join(staged.package_manifest.name.clone())
    );
    fs::rename(
        parent_dir.join(staged.tmp_dir.file_name().unwrap()),
        parent_dir.join(staged.package_manifest.name.clone()),
    )?;

    // Write extension.lock with registry metadata
    let lock_data = ExtensionLockData {
        registry: registry_name,
    };
    let _ = fs::write(
        staged.ext_extract_path.join("extension.lock"),
        lock_data.encode_to_vec(),
    );

    Ok(())
}
