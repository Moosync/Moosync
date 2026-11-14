use std::{collections::HashSet, path::PathBuf};

use futures::StreamExt;
use prost::Message;
use themes_proto::moosync::types::{ThemeManifest, ThemeRegistryManifest};
use tokio::{fs::File, io::AsyncWriteExt};
use uuid::Uuid;

use crate::error::ThemesError;

pub struct RemoteThemes {
    tmp_dir: PathBuf,
    client: reqwest::Client,
}

impl RemoteThemes {
    #[tracing::instrument(level = "debug", skip_all)]
    pub fn new(tmp_dir: PathBuf) -> Self {
        Self {
            tmp_dir,
            client: reqwest::Client::new(),
        }
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub async fn get_theme_manifests(
        &self,
        registries: &HashSet<String>,
    ) -> Result<Vec<ThemeManifest>, ThemesError> {
        tracing::info!("Getting theme manifests from registries: {:?}", registries);
        let mut ret = Vec::new();

        for registry_url in registries {
            match self.fetch_registry(&self.client, registry_url).await {
                Ok(manifests) => {
                    ret.extend(manifests);
                }
                Err(err) => {
                    tracing::error!(
                        "Failed to fetch theme manifest from registry {}: {:?}",
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
    ) -> Result<Vec<ThemeManifest>, ThemesError> {
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
        let parsed = ThemeRegistryManifest::decode(bytes)?;

        let registry_name = parsed.name;

        let base_url = if let Some(idx) = registry_url.rfind('/') {
            &registry_url[..=idx]
        } else {
            registry_url
        };

        let mut list = Vec::new();
        for mut item in parsed.themes {
            if item.id.is_empty() || item.name.is_empty() {
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
    pub async fn download_theme_file(
        &self,
        theme_manifest: &ThemeManifest,
    ) -> Result<PathBuf, ThemesError> {
        let parsed_url = theme_manifest.url.as_ref().ok_or_else(|| {
            ThemesError::InvalidResponse("Missing theme download url".to_string())
        })?;
        let file_path = self
            .tmp_dir
            .join(format!("{}-{}.mstx", theme_manifest.id, Uuid::new_v4()));

        tracing::info!("Downloading theme from {} to {:?}", parsed_url, file_path);

        let mut stream = self
            .client
            .get(parsed_url)
            .send()
            .await?
            .error_for_status()?
            .bytes_stream();
        let mut file = File::create(file_path.clone()).await?;

        while let Some(chunk_result) = stream.next().await {
            let chunk = chunk_result?;
            file.write_all(&chunk).await?;
        }
        file.flush().await?;

        tracing::info!("Saved theme file to {:?}", file_path);

        Ok(file_path)
    }
}
