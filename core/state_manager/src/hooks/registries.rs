use std::{
    collections::{BTreeSet, HashSet},
    error::Error,
    io,
    sync::{Arc, Mutex},
};

use async_trait::async_trait;
use extensions::{DEFAULT_EXTENSION_REGISTRY, ExtensionError, RegistryKeyStorage};
use preferences::{
    PreferenceConfig,
    error::PreferencesError,
    keys::{EXTENSION_REGISTRIES, PreferenceItemExt, THEME_REGISTRIES},
};
use tracing::Instrument;

use super::Hook;
use crate::StateManager;

const REGISTRY_PUBKEY_PREFIX: &str = "__registry_pubkey__.";
const REGISTRY_PUBKEY_INDEX_KEY: &str = "__registry_pubkey_index__";

/// Stores trusted extension registry public keys in encrypted
/// `PreferenceConfig` secure storage.
pub struct PrefsRegistryKeyStorage(pub Arc<PreferenceConfig>);

impl PrefsRegistryKeyStorage {
    #[tracing::instrument(level = "debug", skip_all)]
    fn storage_key(registry_url: &str) -> String {
        format!(
            "{REGISTRY_PUBKEY_PREFIX}{}",
            hex::encode(registry_url.as_bytes())
        )
    }

    #[tracing::instrument(level = "debug", skip_all)]
    fn read_optional_secure(&self, key: &str) -> Result<Option<String>, ExtensionError> {
        match self.0.get_secure::<String>(key.to_string()) {
            Ok(value) => Ok(Some(value)),
            Err(PreferencesError::KeyNotFound(_)) => Ok(None),
            Err(err) => {
                tracing::error!("Failed to read secure key {}: {:?}", key, err);
                Err(ExtensionError::Io(io::Error::other(err.to_string())))
            }
        }
    }

    #[tracing::instrument(level = "debug", skip_all)]
    fn write_secure(&self, key: &str, value: Option<String>) -> Result<(), ExtensionError> {
        self.0.set_secure(key.to_string(), value).map_err(|err| {
            tracing::error!("Failed to write secure key {}: {:?}", key, err);
            ExtensionError::Io(io::Error::other(err.to_string()))
        })
    }

    #[tracing::instrument(level = "debug", skip_all)]
    fn read_key_index(&self) -> Result<BTreeSet<String>, ExtensionError> {
        Ok(self
            .read_optional_secure(REGISTRY_PUBKEY_INDEX_KEY)?
            .unwrap_or_default()
            .lines()
            .filter(|line| !line.is_empty())
            .map(str::to_string)
            .collect())
    }

    #[tracing::instrument(level = "debug", skip_all)]
    fn write_key_index(&self, index: &BTreeSet<String>) -> Result<(), ExtensionError> {
        let updated = if index.is_empty() {
            None
        } else {
            Some(index.iter().cloned().collect::<Vec<_>>().join("\n"))
        };
        self.write_secure(REGISTRY_PUBKEY_INDEX_KEY, updated)
    }
}

impl RegistryKeyStorage for PrefsRegistryKeyStorage {
    #[tracing::instrument(level = "debug", skip_all)]
    fn get_registry_key(&self, registry_url: &str) -> Result<Option<Vec<u8>>, ExtensionError> {
        let storage_key = Self::storage_key(registry_url);
        Ok(self
            .read_optional_secure(&storage_key)?
            .map(String::into_bytes))
    }

    #[tracing::instrument(level = "debug", skip_all)]
    fn set_registry_key(
        &self,
        registry_url: &str,
        public_key: &[u8],
    ) -> Result<(), ExtensionError> {
        let armored = String::from_utf8(public_key.to_vec())
            .map_err(|err| ExtensionError::InvalidSignature(err.to_string()))?;
        let storage_key = Self::storage_key(registry_url);
        self.write_secure(&storage_key, Some(armored))?;

        let mut index = self.read_key_index()?;
        if index.insert(storage_key) {
            self.write_key_index(&index)?;
        }
        Ok(())
    }

    #[tracing::instrument(level = "debug", skip_all)]
    fn remove_registry_key(&self, registry_url: &str) -> Result<(), ExtensionError> {
        let storage_key = Self::storage_key(registry_url);
        self.write_secure(&storage_key, None)?;

        let mut index = self.read_key_index()?;
        if index.remove(&storage_key) {
            self.write_key_index(&index)?;
        }
        Ok(())
    }

    #[tracing::instrument(level = "debug", skip_all)]
    fn list_registry_keys(&self) -> Result<Vec<Vec<u8>>, ExtensionError> {
        let index = self.read_key_index()?;
        let mut keys = Vec::new();
        for storage_key in index {
            if let Some(armored) = self.read_optional_secure(&storage_key)? {
                keys.push(armored.into_bytes());
            }
        }
        Ok(keys)
    }
}

pub struct ExtensionRegistriesHook;

impl Default for ExtensionRegistriesHook {
    fn default() -> Self { Self::new() }
}

impl ExtensionRegistriesHook {
    #[tracing::instrument(level = "debug", skip_all)]
    pub fn new() -> Self { Self }
}

#[async_trait]
impl Hook for ExtensionRegistriesHook {
    #[tracing::instrument(level = "debug", skip_all)]
    async fn on_startup(
        &self,
        state_manager: &StateManager,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        let preferences = state_manager.get_preference_config();
        state_manager
            .get_extension_handler()
            .set_registry_key_storage(Arc::new(PrefsRegistryKeyStorage(preferences.inner.clone())));

        let initial_registries: HashSet<String> = preferences
            .load(&EXTENSION_REGISTRIES)
            .value::<Vec<String>>()
            .unwrap_or_default()
            .into_iter()
            .collect();
        let tracked_registries = Arc::new(Mutex::new(initial_registries));

        preferences.on_preference_changed.listen_filtered_immediate(
            {
                let state_manager = state_manager.clone();
                let tracked_registries = tracked_registries.clone();
                move |_key| {
                    let state_manager = state_manager.clone();
                    let tracked_registries = tracked_registries.clone();
                    tokio::spawn(
                        async move {
                            let preferences = state_manager.get_preference_config();
                            let registries = preferences
                                .load(&EXTENSION_REGISTRIES)
                                .value::<Vec<String>>()
                                .unwrap_or_default();
                            let new_set: HashSet<String> = registries.iter().cloned().collect();
                            let removed: Vec<String> = {
                                let mut prev = tracked_registries.lock().unwrap();
                                let diff = prev.difference(&new_set).cloned().collect();
                                *prev = new_set.clone();
                                diff
                            };
                            let extensions = state_manager.get_extension_handler();
                            for url in removed {
                                if url != DEFAULT_EXTENSION_REGISTRY
                                    && let Err(err) = extensions.remove_registry_key(&url)
                                {
                                    tracing::error!(
                                        "Failed to remove key for deleted registry {}: {:?}",
                                        url,
                                        err
                                    );
                                }
                            }
                            if let Err(e) = extensions.fetch_remote_manifests(new_set).await {
                                tracing::error!("Failed to fetch remote manifests: {:?}", e);
                            } else if extensions.check_for_updates()
                                && let Err(e) = extensions.update_all_extensions().await
                            {
                                tracing::error!("Failed to auto-update extensions: {:?}", e);
                            }
                        }
                        .in_current_span(),
                    );
                }
            },
            EXTENSION_REGISTRIES.id.clone(),
        );

        Ok(())
    }
}

pub struct ThemeRegistriesHook;

impl Default for ThemeRegistriesHook {
    fn default() -> Self { Self::new() }
}

impl ThemeRegistriesHook {
    #[tracing::instrument(level = "debug", skip_all)]
    pub fn new() -> Self { Self }
}

#[async_trait]
impl Hook for ThemeRegistriesHook {
    #[tracing::instrument(level = "debug", skip_all)]
    async fn on_startup(
        &self,
        state_manager: &StateManager,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        let preferences = state_manager.get_preference_config();

        preferences.on_preference_changed.listen_filtered_immediate(
            {
                let state_manager = state_manager.clone();
                move |_key| {
                    let state_manager = state_manager.clone();
                    tokio::spawn(
                        async move {
                            let preferences = state_manager.get_preference_config();
                            let registries = preferences
                                .load(&THEME_REGISTRIES)
                                .value::<Vec<String>>()
                                .unwrap_or_default();
                            let themes = state_manager.get_theme_holder();
                            if let Err(e) = themes
                                .fetch_remote_manifests(registries.into_iter().collect())
                                .await
                            {
                                tracing::error!("Failed to fetch remote theme manifests: {:?}", e);
                            }
                        }
                        .in_current_span(),
                    );
                }
            },
            THEME_REGISTRIES.id.clone(),
        );

        Ok(())
    }
}
