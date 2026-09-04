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

use std::{collections::HashSet, fs::File, io::Write, path::PathBuf};

use extensions_proto::moosync::types::{ExtensionRegistryManifest, FetchedExtensionManifest};
use futures::StreamExt;
use prost::Message;

use crate::errors::ExtensionError;

pub const DEFAULT_EXTENSION_REGISTRY: &str =
    "https://raw.githubusercontent.com/Moosync/moosync-exts/refs/heads/v2/manifest.pb";

pub struct RemoteExtensions {
    tmp_dir: PathBuf,
    client: reqwest::Client,
}

impl RemoteExtensions {
    #[tracing::instrument(level = "debug", skip_all)]
    pub fn new(tmp_dir: PathBuf) -> Self {
        Self {
            tmp_dir,
            client: reqwest::Client::new(),
        }
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub async fn get_extension_manifest(
        &self,
        registries: &HashSet<String>,
    ) -> Result<HashSet<FetchedExtensionManifest>, ExtensionError> {
        tracing::info!(
            "Getting extension manifests from registries: {:?}",
            registries
        );
        let mut ret = HashSet::new();

        for registry_url in registries {
            match self.fetch_registry(&self.client, registry_url).await {
                Ok(manifests) => {
                    for manifest in manifests {
                        ret.insert(manifest);
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
            .header("Accept", "application/x-protobuf, application/octet-stream, */*")
            .send()
            .await?
            .error_for_status()?;

        let bytes = res.bytes().await?;
        let parsed = ExtensionRegistryManifest::decode(bytes)?;

        let registry_name = parsed.name;

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

            if item.name.is_empty() {
                item.name = item.package_name.clone();
            }

            if let Some(logo) = item.logo.as_mut()
                && !logo.starts_with("http://")
                && !logo.starts_with("https://")
            {
                *logo = format!("{}{}", base_url, logo.trim_start_matches('/'));
            }

            if !item.url.starts_with("http://") && !item.url.starts_with("https://") {
                item.url = format!("{}{}", base_url, item.url.trim_start_matches('/'));
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
        fetched_ext: FetchedExtensionManifest,
    ) -> Result<PathBuf, ExtensionError> {
        let parsed_url = fetched_ext.url;
        let file_path = self.tmp_dir.join(format!(
            "{}-{}.msox",
            fetched_ext.package_name,
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
        let mut file = File::create(file_path.clone())?;

        while let Some(chunk_result) = stream.next().await {
            let chunk = chunk_result?;
            file.write_all(&chunk)?;
        }

        tracing::info!("Wrote file");

        Ok(file_path)
    }
}
