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
    fs::{self, File},
    io::Write,
    path::PathBuf,
};

use extensions_proto::moosync::types::FetchedExtensionManifest;
use futures::StreamExt;

use crate::errors::ExtensionError;

pub const DEFAULT_EXTENSION_REGISTRY: &str =
    "https://raw.githubusercontent.com/Moosync/moosync-exts/refs/heads/v2/manifest.json";

#[derive(serde::Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ExtensionManifestItem {
    pub display_name: Option<String>,
    pub name: Option<String>,
    pub version: Option<String>,
    pub icon: Option<String>,
    pub logo: Option<String>,
    pub description: Option<String>,
    pub desc: Option<String>,
    pub url: Option<String>,
    pub download_url: Option<String>,
}

#[derive(serde::Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct RegistryManifestObject {
    name: Option<String>,
    display_name: Option<String>,
    #[serde(default)]
    extensions: HashMap<String, ExtensionManifestItem>,
}

pub struct RemoteExtensions {
    _extensions_dir: PathBuf,
    tmp_dir: PathBuf,
    cache_dir: PathBuf,
}

impl RemoteExtensions {
    #[tracing::instrument(level = "debug", skip_all)]
    pub fn new(extensions_dir: PathBuf, tmp_dir: PathBuf, cache_dir: PathBuf) -> Self {
        Self {
            _extensions_dir: extensions_dir,
            tmp_dir,
            cache_dir,
        }
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub async fn get_extension_manifest(
        &self,
        registries: &HashSet<String>,
    ) -> Result<Vec<FetchedExtensionManifest>, ExtensionError> {
        tracing::info!(
            "Getting extension manifests from registries: {:?}",
            registries
        );
        let client = reqwest::Client::new();
        let mut seen_packages = HashSet::new();
        let mut ret = Vec::new();

        for registry_url in registries {
            match self.fetch_registry(&client, registry_url).await {
                Ok(manifests) => {
                    for manifest in manifests {
                        if !seen_packages.contains(&manifest.package_name) {
                            seen_packages.insert(manifest.package_name.clone());
                            ret.push(manifest);
                        }
                    }
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

        let path = self.cache_dir.join("remote_manifest_cache.json");
        if let Ok(contents) = serde_json::to_vec(&ret) {
            let _ = fs::write(path, contents);
        }

        Ok(ret)
    }

    #[tracing::instrument(level = "debug", skip_all)]
    async fn fetch_registry(
        &self,
        client: &reqwest::Client,
        registry_url: &str,
    ) -> Result<Vec<FetchedExtensionManifest>, ExtensionError> {
        let res = client
            .get(registry_url)
            .header(
                "User-Agent",
                "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/58.0.3029.110 Safari/537.3",
            )
            .header("Accept", "application/json")
            .send()
            .await?
            .error_for_status()?;

        let bytes = res.bytes().await?;
        let parsed: RegistryManifestObject = serde_json::from_slice(&bytes)?;

        let registry_name = parsed
            .display_name
            .or(parsed.name)
            .filter(|n| !n.is_empty())
            .ok_or(ExtensionError::NotAnExtension)?;

        let base_url = if let Some(idx) = registry_url.rfind('/') {
            &registry_url[..=idx]
        } else {
            registry_url
        };

        let mut list = Vec::new();
        for (pkg_name, item) in parsed.extensions {
            let Some(version) = item.version.filter(|v| !v.is_empty()) else {
                continue;
            };

            let display_name = item
                .display_name
                .or(item.name)
                .unwrap_or_else(|| pkg_name.clone());

            let icon = item.icon.or(item.logo).map(|i| {
                if i.starts_with("http://") || i.starts_with("https://") {
                    return i;
                }
                format!("{}{}", base_url, i.trim_start_matches('/'))
            });

            let raw_url = item
                .url
                .or(item.download_url)
                .unwrap_or_else(|| format!("{}.msox", pkg_name));

            let download_url = if raw_url.starts_with("http://") || raw_url.starts_with("https://")
            {
                raw_url
            } else {
                format!("{}{}", base_url, raw_url.trim_start_matches('/'))
            };

            list.push(FetchedExtensionManifest {
                name: display_name,
                package_name: pkg_name,
                logo: icon,
                description: item.description.or(item.desc),
                url: download_url,
                version,
                registry: Some(registry_name.clone()),
            });
        }

        Ok(list)
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub async fn download_extension(
        &self,
        fetched_ext: FetchedExtensionManifest,
    ) -> Result<PathBuf, ExtensionError> {
        let parsed_url = fetched_ext.url;
        let file_path = self.tmp_dir.join(format!(
            "{}-{}.msox",
            fetched_ext.package_name,
            uuid::Uuid::new_v4()
        ));

        tracing::info!("parsed url {}. Saving at {:?}", parsed_url, file_path);

        let mut stream = reqwest::get(parsed_url)
            .await?
            .error_for_status()?
            .bytes_stream();
        let mut file = File::create(file_path.clone())?;

        while let Some(chunk_result) = stream.next().await {
            let chunk = chunk_result?;
            file.write_all(&chunk)?;
        }

        tracing::info!("Wrote file");

        Ok(file_path)
    }
}
