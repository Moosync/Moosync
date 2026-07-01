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
    fs::{self, File},
    io::Write,
    path::PathBuf,
};

use extensions_proto::moosync::types::FetchedExtensionManifest;
use futures::StreamExt;
use serde_json::Value;

use crate::errors::ExtensionError;

pub struct RemoteExtensions {
    extensions_dir: PathBuf,
    tmp_dir: PathBuf,
}

impl RemoteExtensions {
    #[tracing::instrument(level = "debug", skip_all)]
    pub fn new(extensions_dir: PathBuf, tmp_dir: PathBuf) -> Self {
        Self {
            extensions_dir,
            tmp_dir,
        }
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub async fn get_extension_manifest(
        &self,
    ) -> Result<Vec<FetchedExtensionManifest>, ExtensionError> {
        #[derive(serde::Deserialize, Debug, Clone)]
        struct GithubReleaseAsset {
            browser_download_url: String,
            name: String,
        }

        #[derive(serde::Deserialize, Debug)]
        struct GithubReleasesResp {
            assets: Vec<GithubReleaseAsset>,
        }

        #[derive(serde::Deserialize, Debug)]
        #[serde(rename_all = "camelCase")]
        struct ExtensionManifestItem {
            display_name: String,
            version: String,
            icon: Option<String>,
            _permissions: HashMap<String, Value>,
        }

        tracing::info!("Getting extension manifest");
        let client = reqwest::Client::new();
        let res = client.get(
            "https://api.github.com/repos/Moosync/moosync-exts/releases/latest",
        )
        .header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/58.0.3029.110 Safari/537.3")
        .header("Accept", "application/json")
        .send()
        .await?;
        let releases_resp = res.json::<GithubReleasesResp>().await?;

        let mut ret = vec![];
        for item in releases_resp.assets.clone() {
            if item.name == "manifest.json" {
                let res = client.get(&item.browser_download_url).header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/58.0.3029.110 Safari/537.3")
                        .header("Accept", "application/json")
                        .send().await?;

                let bytes = res.bytes().await?;
                let manifests: HashMap<String, ExtensionManifestItem> =
                    serde_json::from_slice(&bytes)?;
                for (package_name, manifest) in manifests {
                    let asset = releases_resp.assets.iter().find(|asset| {
                        asset.name.starts_with(package_name.as_str())
                            && asset.name.ends_with(".msox")
                    });
                    if let Some(asset) = asset {
                        let logo_url = manifest.icon.map(|icon| format!("https://raw.githubusercontent.com/Moosync/moosync-exts/refs/heads/v2/{}", icon));
                        ret.push(FetchedExtensionManifest {
                            name: manifest.display_name,
                            package_name,
                            logo: logo_url,
                            description: None,
                            url: asset.browser_download_url.clone(),
                            version: manifest.version,
                        })
                    }
                }
                break;
            }
        }

        let path = self.extensions_dir.join("remote_manifest_cache.json");
        if let Ok(contents) = serde_json::to_vec(&ret) {
            let _ = fs::write(path, contents);
        }

        Ok(ret)
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

        let mut stream = reqwest::get(parsed_url).await?.bytes_stream();
        let mut file = File::create(file_path.clone())?;

        while let Some(chunk_result) = stream.next().await {
            let chunk = chunk_result?;
            file.write_all(&chunk)?;
        }

        tracing::info!("Wrote file");

        Ok(file_path)
    }
}
