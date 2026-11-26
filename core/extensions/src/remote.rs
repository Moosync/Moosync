use std::{
    collections::{HashMap, HashSet},
    path::PathBuf,
    sync::{Arc, Mutex, OnceLock},
};

use extensions_proto::moosync::types::{ExtensionManifest, ExtensionRegistryManifest};
use futures::StreamExt;
use prost::Message;
use sha2::{Digest, Sha256};
use tokio::{fs::File, io::AsyncWriteExt};

use crate::{
    errors::ExtensionError,
    signing::{KeyStoreOutcome, RegistryKeyStorage, store_registry_key_checked},
};

pub const DEFAULT_EXTENSION_REGISTRY: &str =
    "https://raw.githubusercontent.com/Moosync/moosync-exts/refs/heads/v2/manifest.pb";

pub struct RemoteExtensions {
    tmp_dir: PathBuf,
    client: reqwest::Client,
    key_storage: Arc<OnceLock<Arc<dyn RegistryKeyStorage>>>,
    registry_urls: Mutex<HashMap<String, String>>,
    key_changed_registries: Mutex<HashSet<String>>,
}

impl RemoteExtensions {
    #[tracing::instrument(level = "debug", skip_all)]
    pub fn new(tmp_dir: PathBuf) -> Self {
        Self {
            tmp_dir,
            client: reqwest::Client::new(),
            key_storage: Arc::new(OnceLock::new()),
            registry_urls: Mutex::new(HashMap::new()),
            key_changed_registries: Mutex::new(HashSet::new()),
        }
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub fn set_key_storage(&self, storage: Arc<dyn RegistryKeyStorage>) {
        let _ = self.key_storage.set(storage);
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub fn list_registry_keys(&self) -> Result<Vec<Vec<u8>>, ExtensionError> {
        match self.key_storage.get() {
            Some(storage) => storage.list_registry_keys(),
            None => Ok(Vec::new()),
        }
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub fn remove_registry_key(&self, registry_url: &str) -> Result<(), ExtensionError> {
        self.key_changed_registries
            .lock()
            .unwrap()
            .remove(registry_url);
        if let Some(storage) = self.key_storage.get() {
            storage.remove_registry_key(registry_url)?;
        }
        Ok(())
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub fn get_registry_key(&self, registry_name: &str) -> Result<Option<Vec<u8>>, ExtensionError> {
        let url = self
            .registry_urls
            .lock()
            .unwrap()
            .get(registry_name)
            .cloned()
            .unwrap_or_else(|| registry_name.to_string());
        if self.key_changed_registries.lock().unwrap().contains(&url) {
            return Err(ExtensionError::InvalidSignature(format!(
                "Registry public key changed for {url}; remove and re-add the registry to trust the new key"
            )));
        }
        let Some(storage) = self.key_storage.get() else {
            return Ok(None);
        };
        storage.get_registry_key(&url)
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub async fn get_extension_manifest(
        &self,
        registries: &HashSet<String>,
    ) -> Result<Vec<ExtensionManifest>, ExtensionError> {
        tracing::info!(
            "Getting extension manifests from registries: {:?}",
            registries
        );
        let mut ret = Vec::new();

        for registry_url in registries {
            match self.fetch_registry(&self.client, registry_url).await {
                Ok(manifests) => {
                    ret.extend(manifests);
                }
                Err(err) => {
                    tracing::error!(
                        "Failed to fetch extension manifest from registry {}: {:?}",
                        registry_url,
                        err
                    );
                }
            }
        }

        Ok(ret)
    }

    #[tracing::instrument(level = "debug", skip_all)]
    async fn fetch_registry(
        &self,
        client: &reqwest::Client,
        registry_url: &str,
    ) -> Result<Vec<ExtensionManifest>, ExtensionError> {
        let res = client
            .get(registry_url)
            .header(
                "User-Agent",
                "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/58.0.3029.110 Safari/537.3",
            )
            .header("Accept", "application/x-protobuf, application/octet-stream, */*")
            .send()
            .await?
            .error_for_status()?;

        let bytes = res.bytes().await?;
        let parsed = ExtensionRegistryManifest::decode(bytes)?;

        let registry_name = parsed.name;
        self.registry_urls
            .lock()
            .unwrap()
            .insert(registry_name.clone(), registry_url.to_string());

        if let Some(public_key) = parsed.public_key
            && !public_key.is_empty()
            && let Some(storage) = self.key_storage.get()
        {
            match store_registry_key_checked(storage.as_ref(), registry_url, &public_key) {
                Ok(KeyStoreOutcome::Changed {
                    old_fingerprint,
                    new_fingerprint,
                }) => {
                    tracing::error!(
                        "Registry {} ({}) public key changed ({} -> {}); refusing extensions until registry is removed and re-added",
                        registry_name,
                        registry_url,
                        old_fingerprint,
                        new_fingerprint
                    );
                    self.key_changed_registries
                        .lock()
                        .unwrap()
                        .insert(registry_url.to_string());
                }
                Ok(_) => {
                    self.key_changed_registries
                        .lock()
                        .unwrap()
                        .remove(registry_url);
                }
                Err(err) => {
                    tracing::error!(
                        "Failed to store OpenPGP public key for registry {}: {:?}",
                        registry_url,
                        err
                    );
                }
            }
        }

        let base_url = if let Some(idx) = registry_url.rfind('/') {
            &registry_url[..=idx]
        } else {
            registry_url
        };

        let mut list = Vec::new();
        for mut item in parsed.extensions {
            if item.version.is_empty() || item.package_name.is_empty() {
                continue;
            }

            if let Some(url) = item.url.as_ref()
                && !url.starts_with("http://")
                && !url.starts_with("https://")
            {
                item.url = Some(format!("{}{}", base_url, url.trim_start_matches('/')));
            }

            if item.registry.is_none() || item.registry.as_deref() == Some("") {
                item.registry = Some(registry_name.clone());
            }

            list.push(item);
        }

        Ok(list)
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub async fn download_extension(
        &self,
        ext_manifest: ExtensionManifest,
    ) -> Result<PathBuf, ExtensionError> {
        let parsed_url = ext_manifest
            .url
            .as_ref()
            .ok_or(ExtensionError::InvalidResponse)?;
        let file_path = self.tmp_dir.join(format!(
            "{}-{}.msox",
            ext_manifest.package_name,
            uuid::Uuid::new_v4()
        ));

        tracing::info!("parsed url {}. Saving at {:?}", parsed_url, file_path);

        let mut stream = self
            .client
            .get(parsed_url)
            .send()
            .await?
            .error_for_status()?
            .bytes_stream();
        let mut file = File::create(file_path.clone()).await?;
        let mut hasher = Sha256::new();

        while let Some(chunk_result) = stream.next().await {
            let chunk = chunk_result?;
            hasher.update(&chunk);
            file.write_all(&chunk).await?;
        }
        file.flush().await?;

        let actual_sha256 = hex::encode(hasher.finalize());
        if let Some(expected_sha256) = ext_manifest.sha256.as_ref()
            && !expected_sha256.is_empty()
            && actual_sha256 != expected_sha256.to_lowercase()
        {
            let _ = tokio::fs::remove_file(&file_path).await;
            return Err(ExtensionError::HashMismatch {
                expected: expected_sha256.clone(),
                actual: actual_sha256,
            });
        }

        tracing::info!("Wrote file");

        Ok(file_path)
    }
}
