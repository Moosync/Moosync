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
    collections::{HashMap, HashSet},
    fs,
    path::{Path, PathBuf},
    sync::{Arc, RwLock},
};

use extensions_proto::moosync::types::ExtensionDetail;

use crate::{context::ReplyHandler, errors::ExtensionError, extension::Extension};

pub(crate) struct ExtensionHandlerInner {
    extensions_path: String,
    pub(crate) extensions_map: RwLock<HashMap<String, Arc<Extension>>>,
    cache_path: PathBuf,
}

#[tracing::instrument(level = "debug", skip_all)]
fn is_manifest_file(path: &Path) -> bool {
    path.is_file() && path.file_name() == Some("manifest.pb".as_ref())
}

#[tracing::instrument(level = "debug", skip_all)]
fn find_manifests_in_dir(dir_path: &Path, manifest_paths: &mut Vec<PathBuf>) {
    let Ok(sub_entries) = fs::read_dir(dir_path) else {
        return;
    };

    for sub_entry in sub_entries.flatten() {
        let sub_path = sub_entry.path();
        if is_manifest_file(&sub_path) {
            manifest_paths.push(sub_path);
        }
    }
}

impl ExtensionHandlerInner {
    #[tracing::instrument(level = "debug", skip_all)]
    pub fn new(extensions_path: PathBuf, cache_path: PathBuf) -> Self {
        Self {
            extensions_path: extensions_path.to_string_lossy().to_string(),
            extensions_map: Default::default(),
            cache_path,
        }
    }

    #[tracing::instrument(level = "debug", skip_all)]
    fn find_extension_manifests(&self) -> Vec<PathBuf> {
        let mut manifest_paths = Vec::new();

        let Ok(entries) = fs::read_dir(&self.extensions_path) else {
            return manifest_paths;
        };

        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                find_manifests_in_dir(&path, &mut manifest_paths);
            } else if is_manifest_file(&path) {
                manifest_paths.push(path);
            }
        }
        manifest_paths
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub fn spawn_extensions(&self, reply_handler: Arc<dyn ReplyHandler>) {
        let manifests = self.find_extension_manifests();
        let existing_names: HashSet<String> = {
            let map = self.extensions_map.read().unwrap();
            map.keys().cloned().collect()
        };

        for manifest_path in manifests {
            let manifest = match Extension::read_manifest(&manifest_path) {
                Ok(m) => m,
                Err(e) => {
                    tracing::error!("Error reading manifest {:?}: {:?}", manifest_path, e);
                    continue;
                }
            };
            if existing_names.contains(&manifest.name) {
                continue;
            }
            match Extension::new(
                &manifest_path,
                reply_handler.clone(),
                self.cache_path.clone(),
                Arc::new(std::sync::atomic::AtomicBool::new(false)),
            ) {
                Ok(extension) => {
                    let mut extensions_map = self.extensions_map.write().unwrap();
                    extensions_map.insert(
                        extension.get_package_name().to_string(),
                        Arc::new(extension),
                    );
                }
                Err(e) => tracing::error!("Error spawning extension {:?}: {:?}", manifest_path, e),
            }
        }
    }

    /*
    #[cfg(test)]
    #[tracing::instrument(level = "debug", skip_all)]
    pub fn spawn_single_extension(
        &self,
        manifest_path: &std::path::Path,
        reply_handler: Arc<dyn ReplyHandler>,
    ) -> Result<(), ExtensionError> {
        let manifest = Extension::read_manifest(manifest_path)?;
        let package_name = manifest.name.clone();
        let has_started = Arc::new(std::sync::atomic::AtomicBool::new(false));
        let extension = Extension::new(
            manifest_path,
            reply_handler,
            active,
            self.cache_path.clone(),
            has_started,
        )?;
        let mut extension_map = self.extensions_map.lock().unwrap();
        extension_map.insert(package_name, extension);
        Ok(())
    }
    */

    #[tracing::instrument(level = "debug", skip_all)]
    pub fn get_installed_extensions(&self) -> Vec<ExtensionDetail> {
        let extensions_map = self.extensions_map.read().unwrap();
        extensions_map
            .values()
            .map(|e| e.get_extension_detail())
            .collect::<Vec<ExtensionDetail>>()
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub fn remove_extension(&self, package_name: &str) {
        let mut extensions_map = self.extensions_map.write().unwrap();
        extensions_map.remove(package_name);
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub fn get_extension(&self, package_name: &str) -> Result<Arc<Extension>, ExtensionError> {
        let extensions_map = self.extensions_map.read().unwrap();
        extensions_map
            .get(package_name)
            .cloned()
            .ok_or_else(|| ExtensionError::NoExtensionFound)
    }
}
