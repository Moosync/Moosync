use std::{
    collections::{HashMap, HashSet},
    fs,
    path::{Path, PathBuf},
    sync::{Arc, RwLock, atomic::AtomicBool},
};

use extensions_proto::moosync::types::{ExtensionDetail, ExtensionManifest};

use crate::{MANIFEST_FILE, context::ReplyHandler, errors::ExtensionError, extension::Extension};

/// Unsigned extension awaiting user approval before being spawned.
#[derive(Debug, Clone)]
pub struct UnsignedExtension {
    pub manifest: ExtensionManifest,
    pub manifest_path: PathBuf,
}

pub(crate) struct ExtensionHandlerInner {
    extensions_path: String,
    pub(crate) extensions_map: RwLock<HashMap<String, Arc<Extension>>>,
    cache_path: PathBuf,
}

#[tracing::instrument(level = "debug", skip_all)]
fn is_manifest_file(path: &Path) -> bool {
    path.is_file() && path.file_name() == Some(MANIFEST_FILE.as_ref())
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
    pub fn spawn_extensions(
        &self,
        reply_handler: Arc<dyn ReplyHandler>,
        keys: &[Vec<u8>],
    ) -> Vec<UnsignedExtension> {
        let manifests = self.find_extension_manifests();
        let existing_names: HashSet<String> = {
            let map = self.extensions_map.read().unwrap();
            map.keys().cloned().collect()
        };
        let mut unsigned_prompts = Vec::new();
        for manifest_path in manifests {
            if let Some(unsigned) =
                self.spawn_manifest_if_new(&manifest_path, &existing_names, &reply_handler, keys)
            {
                unsigned_prompts.push(unsigned);
            }
        }
        unsigned_prompts
    }

    #[tracing::instrument(level = "debug", skip_all)]
    fn spawn_manifest_if_new(
        &self,
        manifest_path: &Path,
        existing_names: &HashSet<String>,
        reply_handler: &Arc<dyn ReplyHandler>,
        keys: &[Vec<u8>],
    ) -> Option<UnsignedExtension> {
        let raw_manifest = match Extension::read_manifest(manifest_path) {
            Ok(m) => m,
            Err(e) => {
                tracing::error!("Error reading manifest {:?}: {:?}", manifest_path, e);
                return None;
            }
        };
        if existing_names.contains(&raw_manifest.package_name) {
            return None;
        }

        match self.spawn_single_extension(manifest_path, reply_handler.clone(), keys, false) {
            Ok(()) => None,
            Err(ExtensionError::UnsignedExtension {
                manifest,
                manifest_path,
            }) => Some(UnsignedExtension {
                manifest: *manifest,
                manifest_path,
            }),
            Err(e) => {
                tracing::error!("Error spawning extension {:?}: {:?}", manifest_path, e);
                None
            }
        }
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub fn spawn_single_extension(
        &self,
        manifest_path: &Path,
        reply_handler: Arc<dyn ReplyHandler>,
        keys: &[Vec<u8>],
        allow_unsigned: bool,
    ) -> Result<(), ExtensionError> {
        let extension = Extension::new_verified(
            manifest_path,
            reply_handler,
            self.cache_path.clone(),
            Arc::new(AtomicBool::new(false)),
            keys,
            allow_unsigned,
        )?;
        self.extensions_map.write().unwrap().insert(
            extension.get_package_name().to_string(),
            Arc::new(extension),
        );
        Ok(())
    }

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
            .ok_or(ExtensionError::NoExtensionFound)
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub fn get_active_extensions(&self) -> Vec<Arc<Extension>> {
        self.extensions_map
            .read()
            .unwrap()
            .values()
            .cloned()
            .collect()
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub fn shutdown(&self) { self.extensions_map.write().unwrap().clear(); }
}
