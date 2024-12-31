use std::{
    collections::HashMap,
    env,
    fs::{self, File},
    io::Write,
    path::PathBuf,
    process::Command,
    str::FromStr,
    sync::Arc,
    thread,
};

use command_group::CommandGroup;
use fs_extra::dir::CopyOptions;
use futures::{
    channel::mpsc::{channel, unbounded, Receiver, UnboundedSender},
    lock::Mutex,
    SinkExt,
};
use futures::{future::join_all, StreamExt};
use interprocess::local_socket::{
    traits::tokio::Listener, GenericFilePath, GenericNamespaced, ListenerOptions, NameType,
    ToFsName, ToNsName,
};
use serde_json::Value;
use socket_handler::{ExtensionCommandReceiver, MainCommandSender, SocketHandler};
use types::{
    errors::{MoosyncError, Result},
    extensions::{
        AccountLoginArgs, ContextMenuActionArgs, ExtensionAccountDetail, ExtensionContextMenuItem,
        ExtensionDetail, ExtensionExtraEventArgs, ExtensionManifest, ExtensionProviderScope,
        FetchedExtensionManifest, GenericExtensionHostRequest, PackageNameArgs, ToggleExtArgs,
    },
};
use uuid::Uuid;
use zip_extensions::zip_extract;

mod socket_handler;

// const CREATE_NO_WINDOW: u32 = 0x08000000;

macro_rules! helper1 {
    // Internal helper macro to handle request creation and transformation
    ($self:ident, $req_type:expr, $data:expr) => {
        async {
            let value = GenericExtensionHostRequest {
                type_: $req_type.into(),
                channel: Uuid::new_v4().to_string(),
                data: $data,
            };
            $self.broadcast(serde_json::to_value(value)?).await
        }
    };

    // Any other type
    ($self:ident, $arg:ident, $res:expr, $ret_type:ty) => {{
        tracing::debug!("parsing response {:?} {:?}", $arg, $res);
        let package_name = if let Some(arg) = $arg {
            arg.package_name
        } else {
            String::new()
        };

        if package_name.is_empty() {
            return Ok(Default::default());
        }

        if let Ok(data) = $self
            .get_extension_response(package_name.clone(), &mut $res)
            .await
        {
            tracing::debug!("parsed response {:?}", data);
            if data.is_null() {
                return Ok(Default::default());
            }
            let parsed = serde_json::from_value(data)?;
            return Ok(parsed);
        }

        return Err(format!(
            "Failed to parse data from {} in {:?} as {}",
            package_name,
            $res,
            stringify!($ret_type)
        )
        .into());
    }};
}

macro_rules! helper {
    ($func_name:ident, $req_type:expr, $ret_type:ty, $arg:ty) => {
        #[tracing::instrument(level = "trace", skip(self))]
        pub async fn $func_name(&self, arg: $arg) -> Result<$ret_type> {
            let mut res = helper1!(self, $req_type, Some(arg.clone())).await?;

            let arg = Some(arg);
            helper1!(self, arg, res, $ret_type)
        }
    };

    ($func_name:ident, $req_type:expr, $ret_type:ty) => {
        #[tracing::instrument(level = "trace", skip(self))]
        pub async fn $func_name(&self) -> Result<$ret_type> {
            let mut res = helper1!(self, $req_type, None::<()>).await?;

            let arg = None::<PackageNameArgs>;
            helper1!(self, arg, res, $ret_type)
        }
    };
}

macro_rules! create_extension_function {
    (
            $(
                ($func_name:ident, $req_type:expr, $ret_type:ty $(, $arg:ty)?)
            ),+ $(,)?
        ) => {
            $(

                helper!($func_name, $req_type, $ret_type $(, $arg)?);
            )+

        }
}

type SenderMap = HashMap<String, UnboundedSender<(MainCommandSender, Value)>>;

#[derive(Debug)]
pub struct ExtensionHandler {
    pub ipc_path: PathBuf,
    pub extensions_dir: PathBuf,
    pub tmp_dir: PathBuf,
    sender_map: Arc<Mutex<SenderMap>>,
    extension_runner_map: Mutex<HashMap<String, String>>,
}

impl ExtensionHandler {
    #[tracing::instrument(level = "trace", skip(extensions_dir, tmp_dir))]
    pub fn new(extensions_dir: PathBuf, tmp_dir: PathBuf) -> Self {
        let ipc_path = extensions_dir.join("ipc/ipc.sock");
        if !ipc_path.parent().unwrap().exists() {
            fs::create_dir_all(ipc_path.parent().unwrap()).unwrap();
        }

        if ipc_path.exists() {
            fs::remove_file(ipc_path.clone()).unwrap();
        }

        Self {
            ipc_path,
            extensions_dir,
            tmp_dir,
            sender_map: Arc::new(Mutex::new(HashMap::new())),
            extension_runner_map: Mutex::new(HashMap::new()),
        }
    }

    fn get_builder(&self, exe_path: PathBuf) -> Command {
        let exe_path_clone = exe_path.clone();
        let mut builder = Command::new(exe_path_clone);
        builder.args([
            "-ipcPath",
            self.ipc_path.to_str().unwrap(),
            "-extensionPath",
            self.extensions_dir.to_str().unwrap(),
            "-installPath",
            exe_path.to_str().unwrap(),
        ]);

        // #[cfg(target_os = "windows")]
        // {
        //     builder.creation_flags(CREATE_NO_WINDOW);
        // }

        builder
    }

    #[tracing::instrument(level = "trace", skip(self))]
    fn spawn_ext_runners(&self) {
        #[cfg(not(any(target_os = "android", target_os = "ios")))]
        {
            let exe_path = env::current_exe().unwrap();
            let exe_path = exe_path.parent().unwrap().join("exthost-wasm");

            let mut builder = self.get_builder(exe_path.clone());
            builder.group_spawn().unwrap();
        }
    }

    #[tracing::instrument(level = "trace", skip(self))]
    pub fn listen_socket(&self) -> Result<Receiver<ExtensionCommandReceiver>> {
        let sender_map = self.sender_map.clone();
        let (tx_listen, rx_listen) = channel(1);

        let ipc_path = self.ipc_path.clone();
        thread::spawn(move || {
            let runtime = tokio::runtime::Builder::new_current_thread()
                .enable_io()
                .build()
                .unwrap();

            let ipc_path = if GenericNamespaced::is_supported() {
                ipc_path
                    .file_name()
                    .unwrap()
                    .to_ns_name::<GenericNamespaced>()
            } else {
                ipc_path.clone().to_fs_name::<GenericFilePath>()
            };

            runtime.block_on(async move {
                let opts = ListenerOptions::new().name(ipc_path.unwrap());
                let sock_listener = opts.create_tokio().unwrap();

                loop {
                    tracing::info!("Listening on socket");
                    let conn = sock_listener.accept().await;
                    let (tx_main_command, rx_main_command) = unbounded();
                    let (tx_ext_command, rx_ext_command) = unbounded();
                    match conn {
                        Ok(conn) => {
                            let sender_map = sender_map.clone();
                            let mut tx_listen = tx_listen.clone();
                            tokio::spawn(async move {
                                let handler =
                                    SocketHandler::new(conn, rx_main_command, tx_ext_command);
                                let uuid = uuid::Uuid::new_v4().to_string();
                                let mut sender_map_lock = sender_map.lock().await;
                                sender_map_lock.insert(uuid.clone(), tx_main_command);
                                drop(sender_map_lock);
                                tx_listen.send(rx_ext_command).await.unwrap();
                                tracing::info!("Handling extension socket connections");
                                tokio::select! {
                                    _ = handler.handle_main_command() => {}
                                    _ = handler.handle_connection() => {}
                                };
                                let mut sender_map = sender_map.lock().await;
                                sender_map.remove(&uuid)
                            });
                        }
                        Err(e) => tracing::info!("Extension socket failed to listen {}", e),
                    };
                }
            });
        });

        self.spawn_ext_runners();

        Ok(rx_listen)
    }

    #[tracing::instrument(level = "trace", skip(self))]
    pub async fn broadcast(&self, value: Value) -> Result<HashMap<String, Value>> {
        let mut to_remove_senders = vec![];
        let mut sender_map = self.sender_map.lock().await;
        tracing::trace!("Broadcasting command");
        let mut rx_map = HashMap::new();
        for (key, tx) in sender_map.iter() {
            tracing::debug!("Broadcasting command to {} {:?}", key, value);
            let (tx_r, rx_r) = channel(1);
            let res = tx.clone().send((tx_r, value.clone())).await.map_err(|_| {
                MoosyncError::String(format!(
                    "Failed to send command to extension backend {}",
                    key
                ))
            });
            if let Err(e) = res {
                tracing::error!("{}", e);
                to_remove_senders.push(key.clone());
                continue;
            }
            rx_map.insert(key.clone(), Mutex::new(rx_r));
            tracing::debug!("Broadcasted command");
        }

        if !to_remove_senders.is_empty() {
            for key in to_remove_senders {
                sender_map.remove(&key);
            }
        }

        drop(sender_map);

        let ret = Arc::new(Mutex::new(HashMap::new()));
        let mut promises = vec![];
        for (key, rx) in rx_map.iter() {
            let ret = ret.clone();
            let promise = async move {
                if let Some(res) = rx.lock().await.next().await {
                    tracing::trace!("Got reply from runner {}: {:?}", key, res);
                    match res {
                        Ok(res) => {
                            let mut ret = ret.lock().await;
                            ret.insert(key.clone(), res);
                        }
                        Err(e) => {
                            tracing::error!("Failed to get response from extension runner {}", e)
                        }
                    }
                }
            };
            promises.push(promise);
        }

        join_all(promises).await;

        let ret = ret.lock().await;
        tracing::trace!(result = ?ret, "Got extension runners response");
        Ok(ret.clone())
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

        tracing::info!("parsed url {}", parsed_url);

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

    #[tracing::instrument(level = "trace", skip(self))]
    pub async fn get_installed_extensions(&self) -> Result<HashMap<String, Vec<ExtensionDetail>>> {
        let args = serde_json::to_value(GenericExtensionHostRequest {
            type_: "getInstalledExtensions".to_string(),
            channel: Uuid::new_v4().to_string(),
            data: None::<()>,
        })
        .unwrap();
        let mut res = self.broadcast(args).await?;

        let mut extension_runner_map = self.extension_runner_map.lock().await;

        Ok(res
            .iter_mut()
            .filter_map(|(key, value)| {
                let value = serde_json::from_value::<Vec<ExtensionDetail>>(value.clone());
                if let Ok(mut value) = value {
                    for extension in value.iter_mut() {
                        // Record runner for each extension
                        extension_runner_map.insert(extension.package_name.clone(), key.clone());

                        for preferences in extension.preferences.iter_mut() {
                            if !preferences
                                .key
                                .starts_with(&format!("extension.{}", extension.package_name))
                            {
                                preferences.key = format!(
                                    "extension.{}.{}",
                                    extension.package_name, preferences.key
                                );
                            }
                        }
                    }

                    Some((key.clone(), value))
                } else {
                    tracing::info!("Error parsing extension detail {:?}", value);
                    None
                }
            })
            .collect())
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

    async fn get_extension_response(
        &self,
        ext_name: String,
        data: &mut HashMap<String, Value>,
    ) -> Result<Value> {
        let ext_runner_map = self.extension_runner_map.lock().await;
        if let Some(runner_id) = ext_runner_map.get(&ext_name) {
            if let Some(value) = data.remove(runner_id) {
                if value.is_null() {
                    return Ok(Value::Null);
                }
                let mut parsed_value: HashMap<String, Value> = serde_json::from_value(value)?;
                if let Some(value) = parsed_value.remove(&ext_name) {
                    return Ok(value);
                }
            }
        }

        Err("Cannot extract data".into())
    }

    create_extension_function!(
        (find_new_extensions, "findNewExtensions", Option<()>),
        (get_provider_scopes, "getExtensionProviderScopes", Vec<ExtensionProviderScope>, PackageNameArgs),
        (get_extension_icon, "getExtensionIcon", String, PackageNameArgs),
        (toggle_extension, "toggleExtensionStatus", Option<()>, ToggleExtArgs),
        (send_remove_extension, "removeExtension", Option<()>, PackageNameArgs),
        (stop_process, "stopProcess", Option<()>),
        (get_context_menu, "getExtensionContextMenu", Vec<ExtensionContextMenuItem>, PackageNameArgs),
        (fire_context_menu_action, "onClickedContextMenu", Option<()>, ContextMenuActionArgs),
        (get_accounts, "getAccounts", Vec<ExtensionAccountDetail>, PackageNameArgs),
        (account_login, "performAccountLogin", Option<()>, AccountLoginArgs),
        (get_display_name, "getDisplayName", String, PackageNameArgs),
        (send_extra_event, "extraExtensionEvents", Value, ExtensionExtraEventArgs),
    );
}
