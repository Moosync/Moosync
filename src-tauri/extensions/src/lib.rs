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
    str::FromStr,
    sync::Arc,
    thread,
};

use ext_runner::{ExtCommandReceiver, ExtensionHandlerInner};
use fs_extra::dir::CopyOptions;
use futures::{executor::block_on, StreamExt};
use futures::{lock::Mutex, SinkExt};
use serde_json::Value;
use tokio::{
    select,
    sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender},
};
use types::{
    errors::{MoosyncError, Result},
    extensions::{
        ExtensionCommand, ExtensionCommandResponse, ExtensionManifest, GenericExtensionHostRequest,
        MainCommand, MainCommandResponse, RunnerCommand, RunnerCommandResp,
    },
    ui::extensions::{
        AccountLoginArgs, ExtensionAccountDetail, ExtensionDetail, ExtensionExtraEventArgs,
        ExtensionProviderScope, FetchedExtensionManifest, PackageNameArgs,
    },
};
use zip_extensions::zip_extract;

mod ext_runner;

type UiRequestSender = UnboundedSender<GenericExtensionHostRequest<MainCommand>>;
type UiRequestReceiver = UnboundedReceiver<GenericExtensionHostRequest<MainCommand>>;

type UiReplySender = UnboundedSender<GenericExtensionHostRequest<MainCommandResponse>>;
type UiReplyReceiver = UnboundedReceiver<GenericExtensionHostRequest<MainCommandResponse>>;

pub struct ExtensionHandler {
    pub extensions_dir: PathBuf,
    pub tmp_dir: PathBuf,
    inner: Arc<Mutex<ExtensionHandlerInner>>,
}

impl ExtensionHandler {
    #[tracing::instrument(level = "trace", skip(extensions_dir, tmp_dir))]
    pub fn new(
        extensions_dir: PathBuf,
        tmp_dir: PathBuf,
        cache_dir: PathBuf,
    ) -> (Self, UiRequestReceiver, UiReplySender) {
        let (ext_command_tx, ext_command_rx) = unbounded_channel();
        let (ui_request_tx, ui_request_rx) = unbounded_channel();
        let (ui_reply_tx, ui_reply_rx) = unbounded_channel();

        let ret = Self {
            inner: Arc::new(Mutex::new(ExtensionHandlerInner::new(
                &extensions_dir,
                &cache_dir,
                ext_command_tx,
            ))),
            extensions_dir,
            tmp_dir,
        };

        ret.listen_ext_reply_and_command(ext_command_rx, ui_request_tx, ui_reply_rx);

        (ret, ui_request_rx, ui_reply_tx)
    }

    fn listen_ext_reply_and_command(
        &self,
        mut ext_command_rx: ExtCommandReceiver,
        ui_request_tx: UiRequestSender,
        mut ui_reply_rx: UiReplyReceiver,
    ) {
        let inner = self.inner.clone();
        thread::spawn(move || {
            block_on(async move {
                loop {
                    tracing::trace!("handling ext commands");
                    select! {
                        resp = ext_command_rx.recv() => {
                            if let Some(resp) = resp {
                                tracing::trace!("Got ext command {:?}", resp);
                                ui_request_tx.send(resp).unwrap();
                            }
                        }
                        resp = ui_reply_rx.recv() => {
                            if let Some(resp) = resp {
                                tracing::trace!("Got ui reply {:?} {:?}", resp, inner);
                                let inner = inner.lock().await;
                                inner.handle_main_command_reply(&resp).unwrap();
                            }
                        }
                    }
                }
            });
        });
    }

    #[tracing::instrument(level = "trace", skip(self, ext_path))]
    fn get_extension_version(&self, ext_path: PathBuf) -> Result<String> {
        let manifest_path = ext_path.join("package.json");
        if manifest_path.exists() {
            let package_manifest: ExtensionManifest =
                serde_json::from_slice(&fs::read(manifest_path)?)?;

            return Ok(package_manifest.version);
        }

        Err(MoosyncError::String("No extension found".into()))
    }

    #[tracing::instrument(level = "trace", skip(self, version))]
    fn get_ext_version(&self, version: String) -> Result<u64> {
        Ok(u64::from_str(
            &version.split('.').collect::<Vec<&str>>().join(""),
        )?)
    }

    #[tracing::instrument(level = "trace", skip(self, ext_path))]
    pub async fn install_extension(&self, ext_path: String) -> Result<()> {
        tracing::debug!("ext path {}", ext_path);
        let ext_path =
            PathBuf::from_str(&ext_path).map_err(|e| MoosyncError::String(e.to_string()))?;

        let tmp_dir = self
            .tmp_dir
            .join(format!("moosync_ext_{}", uuid::Uuid::new_v4()));

        zip_extract(&ext_path, &tmp_dir)?;

        let package_manifest: ExtensionManifest =
            serde_json::from_slice(&fs::read(tmp_dir.join("package.json"))?)?;

        if !package_manifest.moosync_extension {
            return Err(MoosyncError::String(
                "Extension is not a moosync extension".to_string(),
            ));
        }

        let ext_extract_path = self.extensions_dir.join(package_manifest.name.clone());

        match self.get_extension_version(ext_extract_path.clone()) {
            Ok(version) => {
                let old_version = self.get_ext_version(version)?;
                let new_version = self.get_ext_version(package_manifest.version)?;

                if new_version > old_version {
                    fs::remove_dir_all(ext_extract_path.clone())?;
                } else {
                    return Err(MoosyncError::String(format!(
                        "Duplicate extension {}. Can not install",
                        package_manifest.name
                    )));
                }
            }
            Err(_) => {
                let _ = fs::remove_dir_all(ext_extract_path.clone());
            }
        }

        let options = CopyOptions::default().overwrite(true);
        let parent_dir = ext_extract_path.parent().unwrap();
        tracing::debug!(
            "Moving items from {:?} to {:?}",
            tmp_dir.clone(),
            parent_dir
        );
        if !parent_dir.exists() {
            tracing::debug!("Creating dir {:?}", parent_dir);
            fs::create_dir_all(parent_dir)?;
        }
        fs_extra::move_items(&[tmp_dir.clone()], parent_dir, &options)?;

        tracing::debug!(
            "Renaming {:?} to {:?}",
            parent_dir.join(tmp_dir.file_name().unwrap()),
            parent_dir.join(package_manifest.name.clone())
        );
        fs::rename(
            parent_dir.join(tmp_dir.file_name().unwrap()),
            parent_dir.join(package_manifest.name),
        )?;

        self.find_new_extensions().await.unwrap();

        Ok(())
    }

    #[tracing::instrument(level = "trace", skip(self, package_name))]
    pub async fn remove_extension(&self, package_name: String) -> Result<()> {
        let ext_path = self.extensions_dir.join(package_name.clone());
        if ext_path.exists() {
            fs::remove_dir_all(ext_path)?;
            self.send_remove_extension(PackageNameArgs { package_name })
                .await?;
            self.find_new_extensions().await?;
            Ok(())
        } else {
            Err(MoosyncError::String("Extension not found".to_string()))
        }
    }

    #[tracing::instrument(level = "trace", skip(self, fetched_ext))]
    pub async fn download_extension(&self, fetched_ext: FetchedExtensionManifest) -> Result<()> {
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

        self.install_extension(file_path.to_string_lossy().to_string())
            .await?;

        Ok(())
    }

    async fn send_remove_extension(&self, package_name: PackageNameArgs) -> Result<()> {
        let mut inner = self.inner.lock().await;
        inner
            .handle_runner_command(RunnerCommand::RemoveExtension(package_name))
            .await?;
        Ok(())
    }

    pub async fn find_new_extensions(&self) -> Result<()> {
        let mut inner = self.inner.lock().await;
        inner
            .handle_runner_command(RunnerCommand::FindNewExtensions)
            .await?;
        Ok(())
    }

    #[tracing::instrument(level = "trace", skip(self))]
    pub async fn get_installed_extensions(&self) -> Result<Vec<ExtensionDetail>> {
        let mut inner = self.inner.lock().await;
        let ret = inner
            .handle_runner_command(RunnerCommand::GetInstalledExtensions)
            .await?;
        if let RunnerCommandResp::ExtensionList(list) = ret {
            return Ok(list);
        }
        Err("Failed to retrieve extensions list".into())
    }

    pub async fn get_extension_icon(&self, args: PackageNameArgs) -> Result<String> {
        let mut inner = self.inner.lock().await;
        let ret = inner
            .handle_runner_command(RunnerCommand::GetExtensionIcon(args))
            .await?;
        if let RunnerCommandResp::ExtensionIcon(Some(icon)) = ret {
            return Ok(icon);
        }
        Err("Could not find extension icon".into())
    }

    async fn send_extension_command(
        &self,
        command: ExtensionCommand,
        wait: bool,
    ) -> Result<ExtensionCommandResponse> {
        tracing::trace!("Sending extension command {:?}", command);
        let (tx, mut rx) = unbounded_channel();

        {
            let mut inner = self.inner.lock().await;
            if let Err(e) = inner.handle_extension_command(&command, tx).await {
                tracing::error!("Failed to execute command {:?}: {:?}", command, e);
                return Err(e);
            }
        }

        tracing::trace!("Should wait for response {}", wait);
        if wait {
            if let Some(resp) = rx.recv().await {
                return Ok(resp);
            }
        }

        Ok(ExtensionCommandResponse::Empty)
    }

    pub async fn send_extra_event(&self, args: ExtensionExtraEventArgs) -> Result<Value> {
        let package_name = args.package_name.clone();
        let resp = self
            .send_extension_command(
                ExtensionCommand::ExtraExtensionEvent(args),
                !package_name.is_empty(),
            )
            .await?;

        if !package_name.is_empty() {
            if let ExtensionCommandResponse::ExtraExtensionEvent(resp) = resp {
                return Ok(serde_json::to_value(resp).unwrap());
            }

            Err("Extension sent invalid reply".into())
        } else {
            Ok(Value::Null)
        }
    }

    pub async fn get_provider_scopes(
        &self,
        package_name: PackageNameArgs,
    ) -> Result<Vec<ExtensionProviderScope>> {
        let resp = self
            .send_extension_command(
                ExtensionCommand::GetProviderScopes(package_name.clone()),
                true,
            )
            .await?;
        if let ExtensionCommandResponse::GetProviderScopes(scopes) = resp {
            return Ok(scopes);
        }
        Ok(vec![])
    }

    pub async fn get_accounts(
        &self,
        package_name: PackageNameArgs,
    ) -> Result<Vec<ExtensionAccountDetail>> {
        let resp = self
            .send_extension_command(ExtensionCommand::GetAccounts(package_name.clone()), true)
            .await?;
        if let ExtensionCommandResponse::GetAccounts(accounts) = resp {
            return Ok(accounts);
        }
        Ok(vec![])
    }

    pub async fn account_login(&self, args: AccountLoginArgs) -> Result<()> {
        self.send_extension_command(ExtensionCommand::PerformAccountLogin(args), false)
            .await?;

        Ok(())
    }

    #[tracing::instrument(level = "trace", skip(self))]
    pub async fn get_extension_manifest(&self) -> Result<Vec<FetchedExtensionManifest>> {
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
            permissions: HashMap<String, Value>,
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
                        ret.push(FetchedExtensionManifest {
                            name: manifest.display_name,
                            package_name,
                            logo: manifest.icon.map(|icon| format!("https://raw.githubusercontent.com/Moosync/moosync-exts/refs/heads/v2/{}", icon)),
                            description: None,
                            url: asset.browser_download_url.clone(),
                            version: manifest.version,
                        })
                    }
                }
                break;
            }
        }

        Ok(ret)
    }
}
