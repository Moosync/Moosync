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

use base64::Engine;
use fs_extra::dir::CopyOptions;
use futures::StreamExt;
use futures::{
    channel::mpsc::{channel, unbounded, Receiver, UnboundedSender},
    lock::Mutex,
    SinkExt,
};
use interprocess::local_socket::{
    traits::tokio::Listener, GenericFilePath, ListenerOptions, ToFsName,
};
use serde_json::Value;
use socket_handler::{ExtensionCommandReceiver, MainCommandSender, SocketHandler};
use tokio::join;
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

macro_rules! helper1 {
    // Internal helper macro to handle request creation and transformation
    ($self:ident, $req_type:expr, $data:expr) => {
        async {
            let value = GenericExtensionHostRequest {
                type_: $req_type.into(),
                channel: Uuid::new_v4().to_string(),
                data: $data,
            };
            $self.broadcast(serde_json::to_value(value).unwrap()).await
        }
    };

    // Any other type
    ($arg:ident, $res:expr, $ret_type:ty) => {{
        let package_name = if let Some(arg) = $arg {
            arg.package_name
        } else {
            String::new()
        };

        let first = $res.values().next().map(|v| v);
        if let Some(first) = first {
            if first.is_null() {
                return Ok(Default::default());
            }

            let parsed = serde_json::from_value::<HashMap<String, Value>>(first.clone());
            if let Ok(parsed) = parsed {
                // if package_name.is_empty() {
                //     return Err(format!("Need package name in args to get back response").into());
                // }

                let first_result = parsed.get(&package_name);
                if let Some(first_result) = first_result {
                    let parsed = serde_json::from_value(first_result.clone()).unwrap();
                    return Ok(parsed);
                } else {
                    tracing::info!("Extension  did not reply");
                    return Ok(Default::default());
                }
            }

            return Err(format!("Failed to parse {:?} as {}", first, stringify!($ret_type)).into());
        }
        Err("Received null from ext host".into())
    }};
}

macro_rules! helper {
    ($func_name:ident, $req_type:expr, $ret_type:ty, $arg:ty) => {
        #[tracing::instrument(level = "trace", skip(self))]
        pub async fn $func_name(&self, arg: $arg) -> Result<$ret_type> {
            let res = helper1!(self, $req_type, Some(arg.clone())).await?;

            let arg = Some(arg);
            helper1!(arg, res, $ret_type)
        }
    };

    ($func_name:ident, $req_type:expr, $ret_type:ty) => {
        #[tracing::instrument(level = "trace", skip(self))]
        pub async fn $func_name(&self) -> Result<$ret_type> {
            let res = helper1!(self, $req_type, None::<()>).await?;

            let arg = None::<PackageNameArgs>;
            helper1!(arg, res, $ret_type)
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
            let ipc_path = ipc_path.clone();
            tracing::info!("Inside thread");
            runtime.block_on(async move {
                tracing::info!("inside async runtime");
                let opts =
                    ListenerOptions::new().name(ipc_path.to_fs_name::<GenericFilePath>().unwrap());
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
                                let mut sender_map = sender_map.lock().await;
                                sender_map
                                    .insert(uuid::Uuid::new_v4().to_string(), tx_main_command);
                                drop(sender_map);
                                tx_listen.send(rx_ext_command).await.unwrap();
                                tracing::info!("Handling extension socket connections");
                                join!(handler.handle_main_command(), handler.handle_connection());
                            });
                        }
                        Err(e) => tracing::info!("Extension socket failed to listen {}", e),
                    };
                }
            });
        });

        let exe_path = env::current_exe().unwrap();
        let _handle = Command::new(exe_path.clone().parent().unwrap().join("exthost"))
            .args([
                "-ipcPath",
                self.ipc_path.to_str().unwrap(),
                "-extensionPath",
                self.extensions_dir.to_str().unwrap(),
                "-installPath",
                exe_path.to_str().unwrap(),
            ])
            .spawn()
            .unwrap();

        Ok(rx_listen)
    }

    #[tracing::instrument(level = "trace", skip(self))]
    pub async fn broadcast(&self, value: Value) -> Result<HashMap<String, Value>> {
        let sender_map = self.sender_map.lock().await;
        tracing::trace!("Broadcasting command");
        let mut rx_map = HashMap::new();
        for (key, tx) in sender_map.iter() {
            tracing::info!("Broadcasting command {:?}", value);
            let (tx_r, rx_r) = channel(1);
            tx.clone().send((tx_r, value.clone())).await.map_err(|_| {
                MoosyncError::String(format!(
                    "Failed to send command to extension backend {}",
                    key
                ))
            })?;
            rx_map.insert(key.clone(), Mutex::new(rx_r));
            tracing::info!("Broadcasted command");
        }

        drop(sender_map);

        let mut ret = HashMap::new();
        for (key, rx) in rx_map.iter() {
            if let Some(res) = rx.lock().await.next().await {
                let res = res?;
                ret.insert(key.clone(), res);
            }
        }

        tracing::trace!(result = ?ret, "Got extension runner response");

        Ok(ret)
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
        tracing::info!("ext path {}", ext_path);
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
        tracing::info!(
            "Moving items from {:?} to {:?}",
            tmp_dir.clone(),
            parent_dir
        );
        fs_extra::move_items(&[tmp_dir.clone()], parent_dir, &options)?;

        tracing::info!(
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
        let parsed_url = fetched_ext
            .release
            .url
            .replace("{version}", &fetched_ext.release.version)
            .replace("{platform}", env::consts::OS)
            .replace("{arch}", env::consts::ARCH);
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

        Ok(res
            .iter_mut()
            .filter_map(|(key, value)| {
                let value = serde_json::from_value::<Vec<ExtensionDetail>>(value.clone());
                if let Ok(mut value) = value {
                    for extension in value.iter_mut() {
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
                    return Some((key.clone(), value));
                } else {
                    tracing::info!("Error parsing extension detail {:?}", value);
                }
                None
            })
            .collect())
    }

    #[tracing::instrument(level = "trace", skip(self))]
    pub async fn get_extension_manifest(&self) -> Result<Vec<FetchedExtensionManifest>> {
        #[derive(serde::Deserialize, Debug)]
        struct GithubTreeItem {
            path: String,
            r#type: String,
            url: String,
        }

        #[derive(serde::Deserialize, Debug)]
        struct GithubTreeResponse {
            tree: Vec<GithubTreeItem>,
        }

        #[derive(serde::Deserialize, Debug)]
        struct BlockResponse {
            content: String,
            encoding: String,
        }

        tracing::info!("Getting extension manifest");
        let client = reqwest::Client::new();
        let res = client.get(
            "https://api.github.com/repos/Moosync/moosync-exts/git/trees/main?recursive=1",
        )
        .header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/58.0.3029.110 Safari/537.3")
        .header("Accept", "application/json")
        .send()
        .await?;
        let res = res.json::<GithubTreeResponse>().await?;

        let mut ret = vec![];
        for item in res.tree {
            if item.path.ends_with("/extension.yml") && item.r#type == "blob" {
                let res = client.get(&item.url).header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/58.0.3029.110 Safari/537.3")
                .header("Accept", "application/json")
                .send().await?;
                let res = res.json::<BlockResponse>().await?;

                if res.encoding == "base64" {
                    let content = base64::prelude::BASE64_STANDARD
                        .decode(res.content.replace("\n", ""))
                        .unwrap();

                    let content = serde_yaml::from_slice::<FetchedExtensionManifest>(&content);
                    if let Ok(content) = content {
                        ret.push(content);
                    }
                }
            }
        }

        Ok(ret)
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
        (account_logout, "getDisplayName", String, PackageNameArgs),
        (send_extra_event, "extraExtensionEvents", Value, ExtensionExtraEventArgs),
    );
}
