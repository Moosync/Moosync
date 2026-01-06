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
    InstalledExtensions, RunnerCommand, RunnerCommandResponse, runner_command,
    runner_command_response,
};
use extism::Plugin;
use ui_proto::moosync::types::PreferenceUiData;

use crate::{
    context::{Extism, ExtismContext, ReplyHandler},
    errors::ExtensionError,
};

#[derive(Debug, Clone)]
struct Extension {
    plugin: Arc<Mutex<Plugin>>,
    package_name: String,
    name: String,
    icon: String,
    author: Option<String>,
    version: String,
    preferences: HashMap<String, PreferenceUiData>,
    active: bool,
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
            has_started: true,
            preferences: val.preferences.clone().into_values().collect(),
            extension_icon: Some(val.icon.clone()),
            active: val.active,
        }
    }
}

#[derive(Debug)]
pub(crate) struct ExtensionHandlerInner {
    extensions_path: String,
    extensions_map: Mutex<HashMap<String, Extension>>,
    extism_context: Box<dyn Extism>,
}

impl ExtensionHandlerInner {
    #[tracing::instrument(level = "debug", skip(reply_handler))]
    pub fn new(extensions_path: PathBuf, cache_path: PathBuf, reply_handler: ReplyHandler) -> Self {
        Self::new_with_context(
            extensions_path,
            Box::new(ExtismContext::new(cache_path, reply_handler)),
        )
    }

    pub fn new_with_context(extensions_path: PathBuf, extism_context: Box<dyn Extism>) -> Self {
        Self {
            extensions_path: extensions_path.to_string_lossy().to_string(),
            extensions_map: Default::default(),
            extism_context,
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
    fn find_extensions(&self) -> Vec<ExtensionManifest> {
        let manifests = self.find_extension_manifests();
        let mut parsed_manifests = vec![];

        let extensions_map = self.extensions_map.lock().unwrap();
        for manifest_path in manifests {
            if let Ok(contents) = fs::read(manifest_path.clone()) {
                match serde_json::from_slice::<ExtensionManifest>(&contents) {
                    Ok(mut manifest) => {
                        let extension_entry_path = manifest_path
                            .parent()
                            .unwrap()
                            .join(manifest.extension_entry);
                        manifest.extension_entry =
                            extension_entry_path.to_string_lossy().to_string();
                        if !extensions_map.contains_key(&manifest.name)
                            && extension_entry_path.extension().unwrap() == "wasm"
                            && extension_entry_path.exists()
                        {
                            manifest.icon = manifest_path
                                .parent()
                                .unwrap()
                                .join(manifest.icon)
                                .to_string_lossy()
                                .to_string();
                            parsed_manifests.push(manifest);
                        }
                    }
                    Err(e) => tracing::error!("Error parsing manifest: {:?}", e),
                }
            }
        }

        parsed_manifests
    }

    fn get_extension(manifest: ExtensionManifest, plugin: Arc<Mutex<Plugin>>) -> Extension {
        Extension {
            plugin,
            name: manifest.display_name,
            package_name: manifest.name,
            icon: manifest.icon,
            author: manifest.author,
            version: manifest.version,
            preferences: Default::default(),
            active: true,
        }
    }

    #[tracing::instrument(level = "debug", skip(self))]
    fn spawn_extensions(&mut self) {
        let manifests = self.find_extensions();
        let mut extension_map = self.extensions_map.lock().unwrap();
        for manifest in manifests {
            let package_name = manifest.name.clone();
            let plugin = self.extism_context.spawn_extension(&manifest);
            let extension = Self::get_extension(manifest, plugin);
            extension_map.insert(package_name, extension);
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

    #[tracing::instrument(level = "debug", skip(self))]
    fn remove_extension(&mut self, package_name: &String) {
        let mut extensions_map = self.extensions_map.lock().unwrap();
        extensions_map.remove(package_name);
    }

    #[tracing::instrument(level = "debug", skip(self))]
    pub async fn handle_extension_command(
        &mut self,
        command: ExtensionCommand,
    ) -> Result<Option<ExtensionCommandResponse>, ExtensionError> {
        tracing::debug!("Executing command {:?}", command);

        let package_name = command.package_name.clone();
        let plugins = self.get_extensions(package_name);
        let plugin_len = plugins.len();

        for extension in plugins {
            let resp = self.extism_context.execute_command(
                extension.package_name,
                extension.plugin,
                command.clone(),
            );

            if plugin_len == 1 {
                return Ok(Some(resp.await?));
            }

            // if let Err(e) = resp {
            //     tracing::error!("Extension responded with error {:?}", e);
            // }
        }

        Ok(None)
    }

    #[tracing::instrument(level = "debug", skip(self))]
    pub fn handle_runner_command(
        &mut self,
        command: RunnerCommand,
    ) -> Result<RunnerCommandResponse, ExtensionError> {
        tracing::info!("Got runner command {:?}", command);
        let ret = match command.command.unwrap() {
            runner_command::Command::GetInstalledExtensions(_) => {
                let extensions_map = self.extensions_map.lock().unwrap();
                let extensions = extensions_map
                    .values()
                    .map(|e| e.into())
                    .collect::<Vec<ExtensionDetail>>();
                tracing::debug!("Extension map: {:?}", extensions);
                RunnerCommandResponse {
                    response: Some(runner_command_response::Response::GetInstalledExtensions(
                        InstalledExtensions { extensions },
                    )),
                }
            }
            runner_command::Command::FindNewExtensions(_) => {
                self.spawn_extensions();
                RunnerCommandResponse {
                    response: Some(runner_command_response::Response::FindNewExtensions(
                        Default::default(),
                    )),
                }
            }
            runner_command::Command::GetExtensionIcon(p) => RunnerCommandResponse {
                response: Some(runner_command_response::Response::GetExtensionIcon(
                    self.get_extensions(p.package_name)
                        .first()
                        .map(|e| e.icon.clone())
                        .unwrap_or_default(),
                )),
            },
            runner_command::Command::RemoveExtension(p) => {
                self.remove_extension(&p.package_name);
                RunnerCommandResponse {
                    response: Some(runner_command_response::Response::RemoveExtension(
                        Default::default(),
                    )),
                }
            }
            runner_command::Command::GetDisplayName(p) => RunnerCommandResponse {
                response: Some(runner_command_response::Response::GetDisplayName(
                    self.get_extensions(p.package_name)
                        .first()
                        .map(|e| e.name.clone())
                        .unwrap_or_default(),
                )),
            },
        };

        tracing::debug!("Got runner command response {:?}", ret);
        Ok(ret)
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
