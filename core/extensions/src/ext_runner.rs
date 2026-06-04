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

#[cfg(any(target_os = "android", target_os = "ios"))]
use std::path::Path;
use std::{
    collections::HashMap,
    fs,
    path::PathBuf,
    sync::{Arc, Mutex},
};

use extensions_proto::moosync::types::{
    ExtensionCommand, ExtensionCommandResponse, ExtensionDetail, ExtensionManifest,
};
use ui_proto::moosync::types::PreferenceUiData;

use crate::{
    context::{ExtensionContext, ExtismContext, ReplyHandler},
    errors::ExtensionError,
};

#[derive(Debug, Clone)]
pub(crate) struct Extension {
    pub(crate) context: Option<Arc<dyn ExtensionContext>>,
    pub(crate) manifest: ExtensionManifest,
    package_name: String,
    name: String,
    icon: String,
    author: Option<String>,
    version: String,
    preferences: HashMap<String, PreferenceUiData>,
    pub(crate) active: bool,
    has_started: Arc<std::sync::atomic::AtomicBool>,
}

impl From<&Extension> for ExtensionDetail {
    #[tracing::instrument(level = "debug", skip(val))]
    fn from(val: &Extension) -> Self {
        ExtensionDetail {
            name: val.name.clone(),
            package_name: val.package_name.clone(),
            desc: None,
            author: val.author.clone(),
            version: val.version.clone(),
            has_started: val.has_started.load(std::sync::atomic::Ordering::SeqCst),
            preferences: val.preferences.clone().into_values().collect(),
            extension_icon: Some(val.icon.clone()),
            active: val.active,
        }
    }
}

impl Extension {
    pub fn read_manifest(manifest_path: &std::path::Path) -> Result<ExtensionManifest, ExtensionError> {
        let contents = std::fs::read(manifest_path)?;
        let mut manifest = serde_json::from_slice::<ExtensionManifest>(&contents)?;

        let parent = manifest_path.parent().unwrap();
        let extension_entry_path = parent.join(&manifest.extension_entry);
        manifest.extension_entry = extension_entry_path.to_string_lossy().to_string();

        let icon_path = parent.join(&manifest.icon);
        manifest.icon = icon_path.to_string_lossy().to_string();

        Ok(manifest)
    }

    pub fn new(
        manifest_path: &std::path::Path,
        reply_handler: Arc<dyn ReplyHandler>,
        active: bool,
        cache_path: PathBuf,
        has_started: Arc<std::sync::atomic::AtomicBool>,
    ) -> Result<Self, ExtensionError> {
        let manifest = Self::read_manifest(manifest_path)?;

        let context = if active {
            Some(Arc::new(ExtismContext::new(
                &manifest,
                has_started.clone(),
                cache_path,
                reply_handler,
            )) as Arc<dyn ExtensionContext>)
        } else {
            None
        };
        Ok(Self {
            context,
            name: manifest.display_name.clone(),
            package_name: manifest.name.clone(),
            icon: manifest.icon.clone(),
            author: manifest.author.clone(),
            version: manifest.version.clone(),
            preferences: Default::default(),
            active,
            has_started,
            manifest,
        })
    }
}

#[derive(Debug)]
pub(crate) struct ExtensionHandlerInner {
    extensions_path: String,
    pub(crate) extensions_map: Mutex<HashMap<String, Extension>>,
    cache_path: PathBuf,
}

impl ExtensionHandlerInner {
    #[tracing::instrument(level = "debug")]
    pub fn new(
        extensions_path: PathBuf,
        cache_path: PathBuf,
    ) -> Self {
        Self {
            extensions_path: extensions_path.to_string_lossy().to_string(),
            extensions_map: Default::default(),
            cache_path,
        }
    }

    #[tracing::instrument(level = "debug", skip(self))]
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

    #[tracing::instrument(level = "debug", skip(self))]
    fn find_extensions(&self) -> Vec<(PathBuf, ExtensionManifest)> {
        let manifests = self.find_extension_manifests();
        let mut parsed_manifests = vec![];

        let extensions_map = self.extensions_map.lock().unwrap();
        for manifest_path in manifests {
            match Extension::read_manifest(&manifest_path) {
                Ok(manifest) => {
                    let extension_entry_path = PathBuf::from(&manifest.extension_entry);
                    if !extensions_map.contains_key(&manifest.name)
                        && extension_entry_path.extension().and_then(|ext| ext.to_str()) == Some("wasm")
                        && extension_entry_path.exists()
                    {
                        parsed_manifests.push((manifest_path, manifest));
                    }
                }
                Err(e) => tracing::error!("Error parsing manifest: {:?}", e),
            }
        }

        parsed_manifests
    }

    fn is_extension_disabled(&self, package_name: &str) -> bool {
        PathBuf::from(&self.extensions_path)
            .join(package_name)
            .join(".disabled")
            .exists()
    }

    fn set_extension_disabled_file(
        &self,
        package_name: &str,
        disabled: bool,
    ) -> Result<(), ExtensionError> {
        let dir = PathBuf::from(&self.extensions_path).join(package_name);
        if !dir.exists() {
            return Err(ExtensionError::NoExtensionFound);
        }
        let disabled_file = dir.join(".disabled");
        if disabled {
            fs::write(disabled_file, "")?;
        } else if disabled_file.exists() {
            fs::remove_file(disabled_file)?;
        }
        Ok(())
    }

    #[tracing::instrument(level = "debug", skip(self, reply_handler))]
    pub fn spawn_extensions(&self, reply_handler: Arc<dyn ReplyHandler>) {
        let manifests = self.find_extensions();
        let mut extension_map = self.extensions_map.lock().unwrap();
        for (manifest_path, manifest) in manifests {
            let package_name = manifest.name.clone();
            let has_started = Arc::new(std::sync::atomic::AtomicBool::new(false));
            let active = !self.is_extension_disabled(&package_name);
            match Extension::new(
                &manifest_path,
                reply_handler.clone(),
                active,
                self.cache_path.clone(),
                has_started,
            ) {
                Ok(extension) => {
                    extension_map.insert(package_name, extension);
                }
                Err(e) => tracing::error!("Error spawning extension {}: {:?}", package_name, e),
            }
        }
    }

    pub fn set_extension_active(
        &self,
        package_name: &str,
        active: bool,
        reply_handler: Arc<dyn ReplyHandler>,
    ) -> Result<(), ExtensionError> {
        let mut extensions_map = self.extensions_map.lock().unwrap();
        if let Some(ext) = extensions_map.get_mut(package_name) {
            ext.active = active;
            self.set_extension_disabled_file(package_name, !active)?;
            if active {
                ext.has_started.store(false, std::sync::atomic::Ordering::SeqCst);
                let context = Arc::new(ExtismContext::new(
                    &ext.manifest,
                    ext.has_started.clone(),
                    self.cache_path.clone(),
                    reply_handler,
                ));
                ext.context = Some(context);
            } else {
                ext.context = None;
            }
            Ok(())
        } else {
            Err(ExtensionError::NoExtensionFound)
        }
    }

    #[tracing::instrument(level = "debug", skip(self))]
    fn get_extensions(&self, package_name: String) -> Vec<Extension> {
        let mut plugins = vec![];
        let extensions_map = self.extensions_map.lock().unwrap();
        if package_name.is_empty() {
            plugins.extend(extensions_map.values().cloned());
        } else {
            let plugin = extensions_map.get(&package_name).cloned();
            if let Some(plugin) = plugin {
                plugins.push(plugin);
            }
        }
        plugins
    }

    pub fn get_installed_extensions(&self) -> Vec<ExtensionDetail> {
        let mut extensions_map = self.extensions_map.lock().unwrap();
        for (package_name, extension) in extensions_map.iter_mut() {
            if self.is_extension_disabled(package_name) {
                extension.active = false;
            }
        }
        extensions_map
            .values()
            .map(|e| e.into())
            .collect::<Vec<ExtensionDetail>>()
    }

    pub fn get_extension_icon(&self, package_name: &str) -> Option<String> {
        self.get_extensions(package_name.to_string())
            .first()
            .map(|e| e.icon.clone())
    }

    pub fn remove_extension(&self, package_name: &str) {
        let mut extensions_map = self.extensions_map.lock().unwrap();
        extensions_map.remove(package_name);
    }

    pub fn get_display_name(&self, package_name: &str) -> Option<String> {
        self.get_extensions(package_name.to_string())
            .first()
            .map(|e| e.name.clone())
    }

    #[tracing::instrument(level = "debug", skip(self))]
    pub async fn handle_extension_command(
        &self,
        command: ExtensionCommand,
    ) -> Result<Option<ExtensionCommandResponse>, ExtensionError> {
        tracing::debug!("Executing command {:?}", command);

        let package_name = command.package_name.clone();
        let plugins = self.get_extensions(package_name);
        let plugin_len = plugins.len();

        for extension in plugins {
            if !extension.active {
                continue;
            }
            if let Some(ref context) = extension.context {
                let resp = context.execute_command(command.clone());

                if plugin_len == 1 {
                    return Ok(Some(resp.await?));
                }
            }

            // if let Err(e) = resp {
            //     tracing::error!("Extension responded with error {:?}", e);
            // }
        }

        Ok(None)
    }

    pub fn register_ui_preferences(
        &self,
        package_name: String,
        prefs: Vec<PreferenceUiData>,
    ) -> Result<(), ExtensionError> {
        let mut extensions = self.extensions_map.lock().unwrap();
        if let Some(ext) = extensions.get_mut(&package_name) {
            for pref in prefs {
                ext.preferences.insert(pref.key.clone(), pref);
            }

            return Ok(());
        }

        Err(ExtensionError::NoExtensionFound)
    }

    pub fn unregister_ui_preferences(
        &self,
        package_name: String,
        pref_keys: Vec<String>,
    ) -> Result<(), ExtensionError> {
        let mut extensions = self.extensions_map.lock().unwrap();
        if let Some(ext) = extensions.get_mut(&package_name) {
            for pref in pref_keys {
                ext.preferences.remove(&pref);
            }

            return Ok(());
        }

        Err(ExtensionError::NoExtensionFound)
    }
}
