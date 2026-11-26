use std::{
    collections::HashSet,
    env, fs,
    path::{Path, PathBuf},
    sync::{Arc, LazyLock, Mutex, OnceLock},
    time::Duration,
};

use ext_runner::ExtensionHandlerInner;
pub use extensions_proto::moosync::types::ExtensionLockData;
use extensions_proto::moosync::types::{
    AppExitRequest, ExtensionDetail, ExtensionManifest, PackageName,
};
use regex::{Captures, Regex};
use types::{prelude::ScopeListExt, subscription::SubscriberList};

pub use crate::{
    errors::ExtensionError,
    extension::{Extension, ExtensionGroupExt},
};

pub mod cache;
#[cfg(test)]
mod cache_test;
mod context;
pub use context::ReplyHandler;
mod errors;
pub mod event_sink;
#[cfg(test)]
mod event_sink_test;
mod ext_runner;
pub mod extension;
pub mod http;
#[cfg(test)]
mod http_test;
pub mod models;
pub mod process;
#[cfg(test)]
mod process_test;
mod remote;
pub mod socket;
#[cfg(test)]
mod socket_test;
pub mod websocket;
#[cfg(test)]
mod websocket_test;
pub use ext_runner::UnsignedExtension;
pub use remote::{DEFAULT_EXTENSION_REGISTRY, RemoteExtensions};
pub mod sanitize;
pub mod signing;
pub use signing::{
    InspectedPackage, MANIFEST_SIGNATURE_FILE, PackageVerification, RegistryKeyStorage,
    SignatureStatus, read_signing_key_fingerprint, validate_package_name, verify_package_contents,
};

pub const MANIFEST_FILE: &str = "manifest.pb";
pub const EXTENSION_LOCK_FILE: &str = "extension.lock";

static ENV_VAR_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\{([A-Z_][A-Z0-9_]*)\}").unwrap());

/// Resolves `{CACHE_DIR}` and `{ENV_VAR}` placeholders in extension paths or
/// binary names.
#[tracing::instrument(level = "debug", skip_all)]
pub fn resolve_env_vars(input: &str, ext_cache_dir: &Path) -> String {
    ENV_VAR_REGEX
        .replace_all(input, |caps: &Captures| {
            let var_name = &caps[1];
            if var_name == "CACHE_DIR" {
                return ext_cache_dir.to_string_lossy().to_string();
            }
            env::var(var_name).unwrap_or_default()
        })
        .to_string()
}
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
#[cfg(test)]
mod signing_test;

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
            ExtensionInfo::Remote(manifest) => &manifest.package_name,
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
    pub on_unsigned_extension: SubscriberList<UnsignedExtension>,
    remote: RemoteExtensions,
    config: Mutex<ExtensionConfig>,
}

#[plugin_macro::generate]
impl ExtensionHandler {
    #[tracing::instrument(level = "debug", skip_all)]
    pub fn new(extensions_dir: PathBuf, tmp_dir: PathBuf, cache_dir: PathBuf) -> Self {
        let remote = RemoteExtensions::new(tmp_dir.clone());
        Self {
            inner: ExtensionHandlerInner::new(extensions_dir.clone(), cache_dir.clone()),
            extensions_dir,
            tmp_dir,
            cache_dir,
            reply_handler: OnceLock::new(),
            on_extensions_updated: SubscriberList::new(),
            on_accounts_updated: SubscriberList::new(),
            on_preferences_updated: SubscriberList::new(),
            on_unsigned_extension: SubscriberList::new(),
            remote,
            config: Mutex::new(ExtensionConfig::default()),
        }
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub fn set_registry_key_storage(&self, storage: Arc<dyn RegistryKeyStorage>) {
        self.remote.set_key_storage(storage);
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
                .find(|r| r.package_name == inst.package_name)
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
            let package_name = remote.package_name.clone();
            if let Err(e) = self.install_extension(ExtensionInfo::Remote(remote)).await {
                tracing::error!(
                    "update_all_extensions: Failed to update {}: {:?}",
                    package_name,
                    e
                );
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
    pub fn get_extension(&self, package_name: &str) -> Result<Arc<Extension>, ExtensionError> {
        self.inner.get_extension(package_name)
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub async fn install_extension(&self, info: ExtensionInfo) -> Result<(), ExtensionError> {
        let (ext_path, registry_name, should_cleanup_ext_path, expected_remote, registry_key) =
            match info {
                ExtensionInfo::Local(_) => {
                    return Ok(());
                }
                ExtensionInfo::Remote(manifest) => {
                    let reg = manifest
                        .registry
                        .clone()
                        .unwrap_or_else(|| "remote".to_string());
                    let key = self.remote.get_registry_key(&reg)?;
                    let expected = (manifest.package_name.clone(), manifest.version.clone());
                    (
                        self.remote.download_extension(manifest).await?,
                        reg,
                        true,
                        Some(expected),
                        key,
                    )
                }
                ExtensionInfo::LocalPath(path) => (path, "local".to_string(), false, None, None),
            };

        let is_remote = expected_remote.is_some();
        tracing::debug!("ext path {:?}", ext_path);

        let tmp_dir = self
            .tmp_dir
            .join(format!("moosync_ext_{}", uuid::Uuid::new_v4()));
        let extensions_dir = self.extensions_dir.clone();
        let stage_path = ext_path.clone();
        let all_keys = self.remote.list_registry_keys()?;
        let keys = if let Some(key) = registry_key {
            vec![key]
        } else {
            all_keys.clone()
        };

        let staged = tokio::task::spawn_blocking(move || {
            Extension::stage_package(
                &stage_path,
                tmp_dir,
                &extensions_dir,
                is_remote,
                &keys,
                &all_keys,
            )
        })
        .await
        .map_err(|e| ExtensionError::Io(std::io::Error::other(e)))??;

        if should_cleanup_ext_path && ext_path.exists() {
            let _ = tokio::fs::remove_file(&ext_path).await;
        }

        if let Some((expected_pkg, expected_ver)) = expected_remote
            && (staged.package_manifest.package_name != expected_pkg
                || staged.package_manifest.version != expected_ver)
        {
            let _ = tokio::fs::remove_dir_all(&staged.tmp_dir).await;
            return Err(ExtensionError::InvalidSignature(format!(
                "Package manifest mismatch: expected {}@{}, got {}@{}",
                expected_pkg,
                expected_ver,
                staged.package_manifest.package_name,
                staged.package_manifest.version
            )));
        }

        if staged.should_remove_old {
            let _ = self.send_remove_extension(PackageName {
                package_name: staged.package_manifest.package_name.clone(),
            });
        }

        let installed_manifest_path =
            tokio::task::spawn_blocking(move || Extension::finalize_install(staged, registry_name))
                .await
                .map_err(|e| ExtensionError::Io(std::io::Error::other(e)))??;

        if is_remote {
            self.find_new_extensions()?;
        } else {
            self.load_extension(&installed_manifest_path, true)?;
        }
        self.trigger_extensions_updated();

        Ok(())
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub fn remove_extension(&self, package_name: String) -> Result<(), ExtensionError> {
        let ext_path = self
            .get_extension(&package_name)
            .map(|ext| ext.extension_path().to_path_buf())
            .unwrap_or_else(|_| self.extensions_dir.join(&package_name));
        if ext_path != self.extensions_dir
            && ext_path.exists()
            && let Err(err) = fs::remove_dir_all(&ext_path)
        {
            tracing::error!(
                "Failed to remove extension directory {:?}: {:?}",
                ext_path,
                err
            );
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
            if !seen_packages.contains(&rem.package_name) {
                seen_packages.insert(rem.package_name.clone());
                ret.push(ExtensionInfo::Remote(rem.clone()));
            }
        }

        ret
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub async fn notify_app_exit(&self) {
        let extensions = self.inner.get_active_extensions();
        for ext in extensions {
            match tokio::time::timeout(Duration::from_secs(2), ext.app_exit(AppExitRequest {}))
                .await
            {
                Ok(Ok(_)) => {}
                Ok(Err(e)) => {
                    tracing::debug!(
                        "Failed to send app_exit to {}: {:?}",
                        ext.get_package_name(),
                        e
                    );
                }
                Err(_) => {
                    tracing::debug!(
                        "App exit notification timed out for {}",
                        ext.get_package_name()
                    );
                }
            }
        }
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub fn shutdown(&self) { self.inner.shutdown(); }

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
        let keys = self.remote.list_registry_keys()?;
        for unsigned in self.inner.spawn_extensions(reply_handler, &keys) {
            self.on_unsigned_extension.emit(unsigned);
        }
        Ok(())
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub fn load_extension(
        &self,
        manifest_path: &Path,
        allow_unsigned: bool,
    ) -> Result<(), ExtensionError> {
        let reply_handler = self
            .reply_handler
            .get()
            .cloned()
            .expect("Reply handler not set");
        let keys = self.remote.list_registry_keys()?;
        self.inner
            .spawn_single_extension(manifest_path, reply_handler, &keys, allow_unsigned)?;
        self.trigger_extensions_updated();
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
        let extensions_map = self.inner.extensions_map.read().unwrap();
        extensions_map
            .values()
            .filter(|ext| ext.manifest.contains_scope(scope))
            .cloned()
            .collect()
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub fn inspect_extension_package(
        &self,
        ext_path: &Path,
    ) -> Result<InspectedPackage, ExtensionError> {
        let keys = self.remote.list_registry_keys()?;
        Extension::inspect_package(ext_path, &self.tmp_dir, &keys)
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub fn remove_registry_key(&self, registry_url: &str) -> Result<(), ExtensionError> {
        self.remote.remove_registry_key(registry_url)
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
