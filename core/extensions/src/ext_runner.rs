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
    fs,
    path::PathBuf,
    sync::{Arc, Mutex},
};

use extensions_proto::moosync::types::ExtensionDetail;

use crate::{context::ReplyHandler, errors::ExtensionError, extension::Extension};

pub(crate) struct ExtensionHandlerInner {
    extensions_path: String,
    pub(crate) extensions_map: Mutex<HashMap<String, Arc<Extension>>>,
    cache_path: PathBuf,
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
        let mut package_json_paths = Vec::new();

        if let Ok(entries) = fs::read_dir(self.extensions_path.clone()) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    // Check only the first level subdirectories
                    if let Ok(sub_entries) = fs::read_dir(&path) {
                        for sub_entry in sub_entries.flatten() {
                            let sub_path = sub_entry.path();
                            if sub_path.is_file()
                                && sub_path.file_name() == Some("package.json".as_ref())
                            {
                                package_json_paths.push(sub_path);
                            }
                        }
                    }
                } else if path.is_file() && path.file_name() == Some("package.json".as_ref()) {
                    package_json_paths.push(path);
                }
            }
        }
        package_json_paths
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub fn spawn_extensions(&self, reply_handler: Arc<dyn ReplyHandler>) {
        let manifests = self.find_extension_manifests();
        for manifest_path in manifests {
            match Extension::new(
                &manifest_path,
                reply_handler.clone(),
                self.cache_path.clone(),
                Arc::new(std::sync::atomic::AtomicBool::new(false)),
            ) {
                Ok(extension) => {
                    let mut extensions_map = self.extensions_map.lock().unwrap();
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
        let extensions_map = self.extensions_map.lock().unwrap();
        extensions_map
            .values()
            .map(|e| e.get_extension_detail())
            .collect::<Vec<ExtensionDetail>>()
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub fn remove_extension(&self, package_name: &str) {
        let mut extensions_map = self.extensions_map.lock().unwrap();
        extensions_map.remove(package_name);
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub fn get_extension(&self, package_name: &str) -> Result<Arc<Extension>, ExtensionError> {
        let extensions_map = self.extensions_map.lock().unwrap();
        extensions_map
            .get(package_name)
            .cloned()
            .ok_or_else(|| ExtensionError::NoExtensionFound)
    }
}
