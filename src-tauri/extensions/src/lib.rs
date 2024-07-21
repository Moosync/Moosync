use std::{
    collections::HashMap,
    env,
    fs::{self, File},
    io::{BufRead, ErrorKind, Write},
    path::PathBuf,
    process::Command,
    str::FromStr,
    sync::{
        mpsc::{self, Sender},
        Mutex,
    },
    thread, u64,
};

use fs_extra::dir::CopyOptions;
use futures::StreamExt;
use interprocess::local_socket::{
    traits::ListenerExt, GenericFilePath,
    ListenerOptions, ToFsName,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use socket_handler::{CommandSender, SocketHandler};
use tauri::{AppHandle, Manager};
use types::errors::errors::{MoosyncError, Result};
use zip_extensions::zip_extract;

mod request_handler;
mod socket_handler;

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FetchedExtensionManifest {
    pub name: String,
    pub package_name: String,
    pub logo: Option<String>,
    pub description: Option<String>,
    pub url: String,
    pub release: Release,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Release {
    pub r#type: Option<String>,
    pub url: String,
    pub version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ExtensionManifest {
    moosync_extension: bool,
    display_name: String,
    extension_entry: String,
    name: String,
    version: String,
}

type SenderMap = HashMap<String, Sender<(CommandSender, Value)>>;

pub struct ExtensionHandler {
    pub ipc_path: PathBuf,
    pub app_handle: AppHandle,
    pub extensions_dir: PathBuf,
    sender_map: Mutex<SenderMap>,
}

impl ExtensionHandler {
    pub fn new(app_handle: AppHandle) -> Self {
        let ipc_path = app_handle
            .path()
            .app_data_dir()
            .unwrap()
            .join("extensions/ipc/ipc.sock");
        if !ipc_path.parent().unwrap().exists() {
            fs::create_dir_all(ipc_path.parent().unwrap()).unwrap();
        }

        if ipc_path.exists() {
            fs::remove_file(ipc_path.clone()).unwrap();
        }

        let extensions_dir = app_handle.path().app_data_dir().unwrap().join("extensions");
        if !extensions_dir.exists() {
            fs::create_dir_all(extensions_dir.clone()).unwrap();
        }

        Self {
            ipc_path,
            app_handle,
            extensions_dir,
            sender_map: Mutex::new(HashMap::new()),
        }
    }

    pub fn listen_socket(&self) -> Result<()> {
        let opts = ListenerOptions::new()
            .name(self.ipc_path.clone().to_fs_name::<GenericFilePath>()?)
            .nonblocking(interprocess::local_socket::ListenerNonblockingMode::Both);
        let sock_listener = opts.create_sync()?;
        let app_handler = self.app_handle.clone();

        let (tx_command, rx_command) = mpsc::channel::<(CommandSender, Value)>();

        thread::spawn(move || {
            for conn in sock_listener.incoming() {
                match conn {
                    Ok(conn) => {
                        let mut handler =
                            SocketHandler::new(conn, app_handler.clone(), &rx_command);
                        handler.handle_connection();
                    }
                    Err(e) => {
                        if e.kind() == ErrorKind::WouldBlock {
                            if let Ok((recv, _)) = rx_command.try_recv() {
                                recv.send(Err(MoosyncError::String(
                                    "Extension backend not yet connected".to_string(),
                                )))
                                .unwrap();
                            }
                        }
                    }
                };
            }
        });

        let exe_path = env::current_exe().unwrap();
        let _handle = Command::new(exe_path.clone().parent().unwrap().join("exthost"))
            .args([
                "-extensionPath",
                self.extensions_dir.to_str().unwrap(),
                "-logPath",
                self.app_handle
                    .path()
                    .app_log_dir()
                    .unwrap()
                    .to_str()
                    .unwrap(),
                "-installPath",
                exe_path.to_str().unwrap(),
            ])
            .spawn()
            .unwrap();

        let mut sender_map = self.sender_map.lock().unwrap();
        sender_map.insert(uuid::Uuid::new_v4().to_string(), tx_command);
        Ok(())
    }

    pub fn broadcast(&self, value: Value) -> Result<HashMap<String, Value>> {
        let sender_map = self.sender_map.lock().unwrap();
        let mut rx_map = HashMap::new();
        for (key, tx) in sender_map.iter() {
            let (tx_r, rx_r) = mpsc::channel();
            tx.send((tx_r, value.clone())).map_err(|_| {
                MoosyncError::String("Failed to send command to extension backend".to_string())
            })?;
            rx_map.insert(key.clone(), rx_r);
        }

        drop(sender_map);

        let mut ret = HashMap::new();
        for (key, rx) in rx_map.iter() {
            if let Ok(res) = rx.recv() {
                let res = res?;
                ret.insert(key.clone(), res);
            }
        }

        Ok(ret)
    }

    fn get_extension_version(&self, ext_path: PathBuf) -> Result<String> {
        let manifest_path = ext_path.join("package.json");
        if manifest_path.exists() {
            let package_manifest: ExtensionManifest =
                serde_json::from_slice(&fs::read(manifest_path)?)?;

            return Ok(package_manifest.version);
        }

        Err(MoosyncError::String("No extension found".into()))
    }

    fn get_ext_version(&self, version: String) -> Result<u64> {
        Ok(u64::from_str(
            &version.split('.').collect::<Vec<&str>>().join(""),
        )?)
    }

    pub fn install_extension(&self, ext_path: String) -> Result<()> {
        println!("ext path {}", ext_path);
        let ext_path =
            PathBuf::from_str(&ext_path).map_err(|e| MoosyncError::String(e.to_string()))?;

        let tmp_dir = self
            .app_handle
            .path()
            .temp_dir()
            .unwrap()
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
        println!(
            "Moving items from {:?} to {:?}",
            tmp_dir.clone(),
            parent_dir
        );
        fs_extra::move_items(&[tmp_dir.clone()], parent_dir, &options)?;

        println!(
            "Renaming {:?} to {:?}",
            parent_dir.join(tmp_dir.file_name().unwrap()),
            parent_dir.join(package_manifest.name.clone())
        );
        fs::rename(
            parent_dir.join(tmp_dir.file_name().unwrap()),
            parent_dir.join(package_manifest.name),
        )?;

        Ok(())
    }

    pub async fn download_extension(&self, fetched_ext: FetchedExtensionManifest) -> Result<()> {
        let parsed_url = fetched_ext
            .release
            .url
            .replace("{version}", &fetched_ext.release.version)
            .replace("{platform}", env::consts::OS)
            .replace("{arch}", env::consts::ARCH);
        let file_path = self.app_handle.path().temp_dir().unwrap().join(format!(
            "{}-{}.msox",
            fetched_ext.package_name,
            uuid::Uuid::new_v4()
        ));

        println!("parsed url {}", parsed_url);

        let mut stream = reqwest::get(parsed_url).await?.bytes_stream();
        let mut file = File::create(file_path.clone())?;

        while let Some(chunk_result) = stream.next().await {
            let chunk = chunk_result?;
            file.write_all(&chunk)?;
        }

        println!("Wrote file");

        self.install_extension(file_path.to_string_lossy().to_string())?;

        Ok(())
    }
}
