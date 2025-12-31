use std::{
    collections::{BTreeMap, HashMap},
    env,
    fmt::Debug,
    fs,
    io::{Read, Write},
    path::PathBuf,
    process,
    str::FromStr,
    sync::{Arc, Mutex},
    thread,
    time::{SystemTime, UNIX_EPOCH},
};

use crypto::{
    digest::Digest,
    sha1::Sha1,
    sha2::{Sha256, Sha512},
};
use extism::{Error, ValType::I64};
use extism::{Manifest, PTR, Plugin, PluginBuilder, UserData, Wasm, host_fn};
use extism_convert::Json;
use interprocess::local_socket::{
    GenericFilePath, GenericNamespaced, NameType, Stream as LocalSocketStream, ToFsName, ToNsName,
    traits::Stream,
};
use regex::{Captures, Regex};
use serde_json::Value;
use types::extensions::{ExtensionManifest, ManifestPermissions};

use crate::{
    context::{Extism, MainCommandUserData, ReplyHandler, SocketUserData},
    errors::ExtensionError,
    models::{ExtensionCommand, ExtensionCommandResponse},
};
use types::extensions::MainCommand;

host_fn!(send_main_command(user_data: MainCommandUserData; command_wrapper: Json<MainCommand>) -> Option<Value> {
    let user_data = user_data.get()?;
    let user_data = user_data.lock().unwrap();

    let mut command = command_wrapper.0;
    command.sanitize_command(&user_data.package_name);

    let response = user_data.reply_handler
    .clone()
    .as_ref()(&user_data.package_name, command)
    .map_err(|e| Error::msg(e.to_string()))?;

    Ok(Some(Json(response)))
});

host_fn!(system_time() -> u64 {
    let start = SystemTime::now();
    let since_the_epoch = start
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards");
   Ok(since_the_epoch.as_secs())
});

host_fn!(open_clientfd(user_data: SocketUserData; sock_path: String) -> i64 {
    let user_data = user_data.get()?;
    let mut user_data = user_data.lock().unwrap();

    if user_data.socks.len() > u8::MAX as usize {
        tracing::error!("Cannot open more sockets");
        return Ok(-1);
    }


    // Check if path is allowed
    if user_data.allowed_paths.is_none() {
        tracing::error!("Not enough permissions to access {}", sock_path);
        return Ok(-1)
    }

    let sock_path_parsed = PathBuf::from_str(sock_path.as_str())?;
    if let Some(allowed_paths) = user_data.allowed_paths.as_ref() {
        for (key, value) in allowed_paths {
            if let Some(sock_path) = sock_path_parsed.to_str() {
                if let Some(allowed_path) = value.to_str() {
                    tracing::debug!("Checking {:?}, {:?}", sock_path, key);
                    if sock_path.starts_with(allowed_path) {
                        // Resultant path is the mapped_path + (passed path - prefix)
                        let mapped_path = PathBuf::from_str(format!("{}/{}", key, sock_path.replacen(allowed_path, "", 1)).as_str())?;
                        if !mapped_path.exists() {
                            tracing::debug!("Path {:?} does not exist", mapped_path);
                            continue;
                        }

                        let mapped_path_name = if GenericNamespaced::is_supported() && key.starts_with("\\\\.\\pipe\\") {
                            mapped_path.file_name().unwrap()
                                .to_ns_name::<GenericNamespaced>()
                        } else {
                            mapped_path.to_fs_name::<GenericFilePath>()
                        }?;

                        if let Ok(sock) = LocalSocketStream::connect(mapped_path_name) {
                            user_data.socks.push(sock);
                            return Ok((user_data.socks.len() - 1) as i64);
                        }
                    }
                } else {
                   tracing::error!("Failed to convert mapped path: {:?} to string", value);
                }
            } else {
                tracing::error!("Failed to convert passed path to string");
                return Ok(-1);
            }
        }
    }

    tracing::error!("Sock path not specified in allowed_paths");
    Ok(-1)

});

host_fn!(write_sock(user_data: SocketUserData; sock_id: i64, buf: Vec<u8>) -> i64 {
    let user_data = user_data.get()?;
    let mut user_data = user_data.lock().unwrap();

    let sock = user_data.socks.get_mut(sock_id as usize);
    if let Some(sock) = sock {
        tracing::info!("Writing {:?}", buf);
        let res = sock.write_all(&buf);
        if let Err(e) = res {
            tracing::error!("Failed to write data to sock {}", e);
            return Ok(-1);
        } else {
            tracing::info!("Wrote all");
            return Ok(-1);
        }
    }

    tracing::error!("Invalid sock id");
    return Ok(-1);
});

host_fn!(read_sock(user_data: SocketUserData; sock_id: i64, read_len: u64) -> Vec<u8> {
    let user_data = user_data.get()?;
    let mut user_data = user_data.lock().unwrap();

    let sock = user_data.socks.get_mut(sock_id as usize);
    if let Some(sock) = sock {
        let mut read_len = read_len;
        if read_len == 0 || read_len > 1024 {
            read_len = 1024
        }

        tracing::info!("Reading {}", read_len);
        let mut ret = vec![0; read_len as usize];
        let read = sock.read(&mut ret);
        if let Ok(read) = read {
            if read >= 1024 {
                tracing::error!("Read out of bounds");
                return Ok(vec![]);
            }
            let mut ret = ret.to_vec();
            ret.truncate(read);
            return Ok(ret);
        }
    }

    tracing::error!("Invalid sock id");
    return Ok(vec![]);
});

host_fn!(hash(hash_type: String, data: Vec<u8>) -> Vec<u8> {
    tracing::info!("Calling hash function {} type {:?}", hash_type, data);
    let mut hasher: Box<dyn Digest> = match hash_type.as_str() {
        "SHA256" => {
            Box::new(Sha256::new())
        },
        "SHA512" => {
            Box::new(Sha512::new())
        },
        _ => {
            Box::new(Sha1::new())
        },
    };

    hasher.input(&data);
    let mut buf = vec![0u8; hasher.output_bytes()];
    hasher.result(&mut buf);
    return Ok(buf);
});

pub struct ExtismContext {
    cache_path: PathBuf,
    reply_handler: ReplyHandler,
}

impl Debug for ExtismContext {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ExtismContext")
            .field("cache_path", &self.cache_path)
            .finish()
    }
}

impl ExtismContext {
    pub fn new(cache_path: PathBuf, reply_handler: ReplyHandler) -> Self {
        Self {
            cache_path,
            reply_handler,
        }
    }

    fn get_allowed_paths(
        &self,
        permissions: &ManifestPermissions,
        package_name: &str,
    ) -> HashMap<String, PathBuf> {
        let re = Regex::new(r"\{([A-Z_][A-Z0-9_]*)\}").unwrap();
        let mut allowed_paths = HashMap::new();
        let ext_cache_dir = self.cache_path.join("extensions").join(package_name);

        if let Err(e) = fs::create_dir_all(&ext_cache_dir) {
            tracing::error!(
                "Failed to create cache dir for extension {}: {:?}",
                package_name,
                e
            );
        }

        for (key, value) in &permissions.paths {
            // Replace all matches with corresponding env variable values
            let parsed = re
                .replace_all(key.as_str(), |caps: &Captures| {
                    let var_name = &caps[1];
                    if var_name == "CACHE_DIR" {
                        return ext_cache_dir.to_string_lossy().to_string();
                    }
                    env::var(var_name).unwrap_or_else(|_| "".to_string())
                })
                .to_string();

            let Ok(parsed_path) = PathBuf::from_str(&parsed);
            if !parsed_path.exists() {
                tracing::warn!("Path {:?} does not exist", parsed_path);
                continue;
            }
            allowed_paths.insert(parsed, value.clone());
        }

        tracing::info!("Got allowed paths {:?}", allowed_paths);
        allowed_paths
    }

    fn get_user_data(
        &self,
        package_name: String,
        allowed_paths: Option<BTreeMap<String, PathBuf>>,
    ) -> (UserData<MainCommandUserData>, UserData<SocketUserData>) {
        let user_data = UserData::new(MainCommandUserData {
            package_name,
            reply_handler: self.reply_handler.clone(),
        });

        let sock_data = UserData::new(SocketUserData {
            socks: vec![],
            allowed_paths,
        });

        (user_data, sock_data)
    }

    fn build_plugin(
        &self,
        plugin_manifest: Manifest,
        user_data: UserData<MainCommandUserData>,
        sock_data: UserData<SocketUserData>,
    ) -> Arc<Mutex<Plugin>> {
        #[allow(unused_mut)]
        let mut plugin_builder = PluginBuilder::new(plugin_manifest)
            .with_wasi(true)
            .with_function(
                "send_main_command",
                [PTR],
                [PTR],
                user_data,
                send_main_command,
            )
            .with_function("system_time", [], [PTR], UserData::default(), system_time)
            .with_function(
                "open_clientfd",
                [PTR],
                [I64],
                sock_data.clone(),
                open_clientfd,
            )
            .with_function(
                "write_sock",
                [I64, PTR],
                [I64],
                sock_data.clone(),
                write_sock,
            )
            .with_function("read_sock", [I64, I64], [PTR], sock_data, read_sock)
            .with_function("hash", [PTR, PTR], [PTR], UserData::default(), hash);

        #[cfg(any(target_os = "android", target_os = "ios"))]
        {
            let cache_path = self.cache_path.join("wasmtime").join("config.toml");
            if !cache_path.exists() {
                fs::create_dir_all(cache_path.parent().unwrap()).unwrap();
            }
            fs::write(
                &cache_path,
                format!(
                    r#"
            [cache]
            enabled = true
            directory = "{}"
            cleanup-interval = "30m"
            files-total-size-soft-limit = "1Gi"
            "#,
                    cache_path.parent().unwrap().join("cache").to_string_lossy()
                ),
            )
            .unwrap();

            plugin_builder = plugin_builder.with_cache_config(cache_path);
        }

        let plugin = plugin_builder.build().unwrap();

        Arc::new(Mutex::new(plugin))
    }
}

#[async_trait::async_trait]
impl Extism for ExtismContext {
    fn spawn_extension(&self, manifest: &ExtensionManifest) -> Arc<Mutex<Plugin>> {
        let url = Wasm::file(manifest.extension_entry.clone());
        let mut plugin_manifest = Manifest::new([url]);

        if let Some(permissions) = &manifest.permissions {
            let allowed_paths = self.get_allowed_paths(permissions, &manifest.name);
            plugin_manifest = plugin_manifest
                .with_allowed_hosts(permissions.hosts.clone().into_iter())
                .with_allowed_paths(allowed_paths.into_iter())
                .with_config_key("pid", format!("{}", process::id()));
        }

        let (user_data, sock_data) =
            self.get_user_data(manifest.name.clone(), plugin_manifest.allowed_paths.clone());

        let plugin = self.build_plugin(plugin_manifest, user_data, sock_data);
        let plugin_clone = plugin.clone();
        let package_name = manifest.name.clone();
        let reply_handler = self.reply_handler.clone();
        thread::spawn(move || {
            {
                let mut plugin = plugin.lock().unwrap();
                tracing::trace!("Calling entry");
                if let Err(e) = plugin.call::<(), ()>("entry", ()) {
                    tracing::error!("Failed to called extension entry: {:?}", e);
                }
            }
            let _ = reply_handler.as_ref()(&package_name, MainCommand::ExtensionsUpdated());
        });

        plugin_clone
    }

    async fn execute_command(
        &self,
        package_name: String,
        plugin: Arc<Mutex<Plugin>>,
        command: ExtensionCommand,
    ) -> Result<ExtensionCommandResponse, ExtensionError> {
        tokio::task::spawn_blocking(move || {
            let (fn_name, args) = command.to_plugin_call();
            let mut plugin = plugin.lock().unwrap();
            tracing::debug!("Calling {} on {:?}", fn_name, plugin.id);
            let res = plugin.call::<_, Value>(fn_name, args.clone())?;
            tracing::trace!("Finished calling {} on {:?}", fn_name, plugin.id);
            let mut parsed_resp = command.parse_response(res)?;
            parsed_resp.sanitize(&package_name);
            Ok(parsed_resp)
        })
        .await
        .unwrap()
    }
}
