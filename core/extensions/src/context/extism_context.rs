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
    collections::{BTreeMap, HashMap},
    env,
    fmt::Debug,
    fs,
    io::{Read, Write},
    path::{Path, PathBuf},
    process,
    str::FromStr,
    sync::{Arc, LazyLock, Mutex},
    thread,
    time::{SystemTime, UNIX_EPOCH},
};

use crypto::{
    digest::Digest,
    sha1::Sha1,
    sha2::{Sha256, Sha512},
};
use extensions_proto::moosync::types::{
    BatchHttpRequest, BatchHttpResponse, Error as MainCommandError, ExtensionCommand,
    ExtensionCommandResponse, ExtensionManifest, HttpRequest, HttpResponse, HttpResult,
    MainCommand, MainCommandResponse, ManifestPermissions, http_result, main_command_response,
};
use extism::{Manifest, PTR, Plugin, PluginBuilder, UserData, ValType::I64, Wasm, host_fn};
use extism_convert::Prost;
use interprocess::local_socket::{
    GenericFilePath, GenericNamespaced, NameType, Stream as LocalSocketStream, ToFsName, ToNsName,
    traits::Stream,
};
use regex::{Captures, Regex};

use crate::{
    context::{DispatchCommand, ExtensionContext, ReplyHandler},
    errors::ExtensionError,
    models::SanitizeCommand,
};

static ENV_VAR_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\{([A-Z_][A-Z0-9_]*)\}").unwrap());

struct MainCommandUserData {
    package_name: String,
    reply_handler: Arc<dyn ReplyHandler>,
}

struct SocketUserData {
    socks: Vec<LocalSocketStream>,
    allowed_paths: Option<BTreeMap<String, PathBuf>>,
}

struct HttpUserData {
    allowed_hosts: Option<Vec<String>>,
    client: reqwest::Client,
}

host_fn!(send_main_command(user_data: MainCommandUserData; command_wrapper: Prost<MainCommand>) {
    let user_data_arc = user_data.get()?;
    let (package_name, reply_handler) = {
        let data = user_data_arc.lock().unwrap();
        (data.package_name.clone(), data.reply_handler.clone())
    };

    let mut command = command_wrapper.0;
    if let Err(e) = command.sanitize(&package_name) {
        return Ok(Prost(MainCommandResponse {
            response: Some(main_command_response::Response::Error(MainCommandError {
                message: e.to_string()
            })),
        }));
    }

    let response = match command.command {
        Some(cmd) => cmd.dispatch(reply_handler.as_ref(), &package_name),
        None => Err(ExtensionError::MissingCommand),
    };

    let result = response.map(|resp| MainCommandResponse {
        response: Some(resp),
    });

    match result {
        Ok(response) => {
            Ok(Prost(response))
        }
        Err(e) => {
            Ok(Prost(MainCommandResponse {
                response: Some(main_command_response::Response::Error(MainCommandError {
                    message: e.to_string()
                })),
            }))
        }
    }
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
    let Some(allowed_paths) = user_data.allowed_paths.as_ref() else {
        tracing::error!("Not enough permissions to access {}", sock_path);
        return Ok(-1);
    };

    let sock_path_parsed = PathBuf::from_str(sock_path.as_str())?;
    let Some(sock_path_str) = sock_path_parsed.to_str() else {
        tracing::error!("Failed to convert passed path to string");
        return Ok(-1);
    };

    for (key, value) in allowed_paths {
        let allowed_path = value.to_str();
        if allowed_path.is_none() {
            tracing::error!("Failed to convert mapped path: {:?} to string", value);
        }

        let is_match = allowed_path.map(|path| sock_path_str.starts_with(path)).unwrap_or(false);
        if is_match {
            let path_val = allowed_path.unwrap();
            let mapped_path = PathBuf::from_str(format!("{}/{}", key, sock_path_str.replacen(path_val, "", 1)).as_str())?;
            if mapped_path.exists() {
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
            } else {
                tracing::debug!("Path {:?} does not exist", mapped_path);
            }
        }
    }

    tracing::error!("Sock path not specified in allowed_paths");
    Ok(-1)

});

host_fn!(write_sock(user_data: SocketUserData; sock_id: i64, buf: Vec<u8>) -> i64 {
    let user_data = user_data.get()?;
    let mut user_data = user_data.lock().unwrap();

    let Some(sock) = user_data.socks.get_mut(sock_id as usize) else {
        tracing::error!("Invalid sock id");
        return Ok(-1);
    };

    tracing::info!("Writing {:?}", buf);
    if let Err(e) = sock.write_all(&buf) {
        tracing::error!("Failed to write data to sock {}", e);
        return Ok(-1);
    }
    tracing::info!("Wrote all");
    Ok(-1)
});

host_fn!(read_sock(user_data: SocketUserData; sock_id: i64, read_len: u64) -> Vec<u8> {
    let user_data = user_data.get()?;
    let mut user_data = user_data.lock().unwrap();

    let Some(sock) = user_data.socks.get_mut(sock_id as usize) else {
        tracing::error!("Invalid sock id");
        return Ok(vec![]);
    };

    let mut read_len = read_len;
    if read_len == 0 || read_len > 1024 {
        read_len = 1024;
    }

    tracing::info!("Reading {}", read_len);
    let mut ret = vec![0; read_len as usize];
    let Ok(read) = sock.read(&mut ret) else {
        return Ok(vec![]);
    };

    if read >= 1024 {
        tracing::error!("Read out of bounds");
        return Ok(vec![]);
    }
    ret.truncate(read);
    Ok(ret)
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
        }
    };

    hasher.input(&data);
    let mut buf = vec![0u8; hasher.output_bytes()];
    hasher.result(&mut buf);
    Ok(buf)
});

#[tracing::instrument(level = "trace", skip_all)]
fn is_host_allowed(host: &str, allowed_hosts: Option<&[String]>) -> bool {
    let Some(allowed) = allowed_hosts else {
        return false;
    };
    let host_lower = host.to_ascii_lowercase();
    for pattern in allowed {
        let pattern_lower = pattern.to_ascii_lowercase();
        if pattern_lower == "*" {
            return true;
        }
        if let Some(bare) = pattern_lower.strip_prefix("*.") {
            let suffix = &pattern_lower[1..];
            if host_lower.ends_with(suffix) || host_lower == bare {
                return true;
            }
        }
        if host_lower == pattern_lower {
            return true;
        }
    }
    false
}

#[tracing::instrument(level = "trace", skip_all)]
fn validate_request(req: &HttpRequest, allowed_hosts: Option<&[String]>) -> Result<(), String> {
    let url_parsed = reqwest::Url::parse(&req.url).map_err(|e| format!("Invalid URL: {e}"))?;
    let host = url_parsed
        .host_str()
        .ok_or_else(|| "URL has no host".to_string())?;
    if !is_host_allowed(host, allowed_hosts) {
        return Err(format!(
            "Host '{host}' is not allowed by manifest permissions"
        ));
    }
    Ok(())
}

#[tracing::instrument(level = "debug", skip_all)]
pub(crate) async fn execute_single_request(
    client: &reqwest::Client,
    req: HttpRequest,
    allowed_hosts: Option<&[String]>,
) -> Result<HttpResponse, String> {
    validate_request(&req, allowed_hosts)?;
    let url_parsed = reqwest::Url::parse(&req.url).map_err(|e| format!("Invalid URL: {e}"))?;

    let method = match req.method.to_uppercase().as_str() {
        "POST" => reqwest::Method::POST,
        "PUT" => reqwest::Method::PUT,
        "DELETE" => reqwest::Method::DELETE,
        "PATCH" => reqwest::Method::PATCH,
        "HEAD" => reqwest::Method::HEAD,
        "OPTIONS" => reqwest::Method::OPTIONS,
        _ => reqwest::Method::GET,
    };

    let mut builder = client.request(method, url_parsed);
    for (k, v) in req.headers {
        builder = builder.header(k, v);
    }

    if let Some(body) = req.body {
        builder = builder.body(body);
    }

    if req.timeout_ms.unwrap_or(0) > 0 {
        let timeout_ms = req.timeout_ms.unwrap();
        builder = builder.timeout(std::time::Duration::from_millis(timeout_ms));
    }

    let resp = builder
        .send()
        .await
        .map_err(|e| format!("Network request failed: {e}"))?;

    let status_code = resp.status().as_u16() as u32;
    let status_text = resp.status().canonical_reason().unwrap_or("").to_string();
    let mut headers = HashMap::new();
    for (k, v) in resp.headers() {
        if let Ok(v_str) = v.to_str() {
            headers.insert(k.as_str().to_string(), v_str.to_string());
        }
    }

    let body_bytes = resp
        .bytes()
        .await
        .map_err(|e| format!("Failed to read response body: {e}"))?;

    Ok(HttpResponse {
        status_code,
        status_text,
        headers,
        body: body_bytes.to_vec(),
    })
}

host_fn!(batch_http_request(user_data: HttpUserData; req: Prost<BatchHttpRequest>) -> Prost<BatchHttpResponse> {
    let requests = req.0.requests;
    let (client, allowed_hosts) = {
        let user_data = user_data.get()?;
        let data = user_data.lock().unwrap();
        (data.client.clone(), data.allowed_hosts.clone())
    };

    // Upfront validation: if ANY request is invalid or disallowed, abort and do not execute any request
    for (idx, r) in requests.iter().enumerate() {
        if let Err(err) = validate_request(r, allowed_hosts.as_deref()) {
            return Ok(Prost(BatchHttpResponse {
                responses: Vec::new(),
                error: Some(format!("Request #{idx} invalid: {err}")),
            }));
        }
    }

    let handle = tokio::runtime::Handle::current();
    let responses = tokio::task::block_in_place(|| {
        handle.block_on(async {
            let futures = requests
                .into_iter()
                .map(|r| execute_single_request(&client, r, allowed_hosts.as_deref()));
            let results = futures::future::join_all(futures).await;
            results
                .into_iter()
                .map(|res| match res {
                    Ok(response) => HttpResult {
                        result: Some(http_result::Result::Response(response)),
                    },
                    Err(error) => HttpResult {
                        result: Some(http_result::Result::Error(error)),
                    },
                })
                .collect()
        })
    });

    Ok(Prost(BatchHttpResponse {
        responses,
        error: None,
    }))
});

static COMPILE_LIMIT: Mutex<usize> = Mutex::new(0);
static COMPILE_CONDVAR: std::sync::Condvar = std::sync::Condvar::new();
const MAX_CONCURRENT_COMPILATIONS: usize = 5;

pub struct ExtismContext {
    plugin: Arc<Mutex<Plugin>>,
    package_name: String,
}

impl Debug for ExtismContext {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ExtismContext")
            .field("package_name", &self.package_name)
            .finish()
    }
}

impl ExtismContext {
    #[tracing::instrument(level = "debug", skip_all)]
    pub fn new(
        manifest: &ExtensionManifest,
        has_started: Arc<std::sync::atomic::AtomicBool>,
        cache_path: &Path,
        reply_handler: Arc<dyn ReplyHandler>,
    ) -> Self {
        let url = Wasm::file(manifest.extension_entry.clone());
        let mut plugin_manifest = Manifest::new([url]);

        let package_name = manifest.name.clone();
        let ext_cache_dir = cache_path.join("extensions").join(&package_name);

        if let Err(e) = fs::create_dir_all(&ext_cache_dir) {
            tracing::error!(
                "Failed to create cache dir for extension {}: {:?}",
                package_name,
                e
            );
        }

        if let Some(permissions) = &manifest.permissions {
            let allowed_paths = Self::get_allowed_paths(permissions, &ext_cache_dir);
            plugin_manifest = plugin_manifest
                .with_allowed_hosts(permissions.hosts.clone().into_iter())
                .with_allowed_paths(allowed_paths.into_iter())
                .with_config_key("pid", format!("{}", process::id()));
        }

        let (user_data, sock_data, http_data) = Self::get_user_data(
            package_name.clone(),
            reply_handler.clone(),
            plugin_manifest.allowed_paths.clone(),
            manifest.permissions.as_ref().map(|p| p.hosts.clone()),
        );

        let plugin =
            Self::build_plugin(cache_path, plugin_manifest, user_data, sock_data, http_data);
        let plugin_clone = plugin.clone();
        let package_name_clone = package_name.clone();
        let reply_handler_clone = reply_handler.clone();
        let extension_entry = manifest.extension_entry.clone();
        thread::spawn(move || {
            {
                let mut plugin = plugin_clone.lock().unwrap();
                println!("Calling entry");
                if let Err(e) = plugin.call::<(), ()>("entry", ()) {
                    println!("Failed to called extension entry: {:?}", e);
                    if let Some(parent) = PathBuf::from(&extension_entry).parent() {
                        let lock_path = parent.join("extension.lock");
                        let mut lock_data = if let Ok(bytes) = fs::read(&lock_path)
                            && let Ok(data) = serde_json::from_slice::<
                                crate::extension::ExtensionLockData,
                            >(&bytes)
                        {
                            data
                        } else {
                            crate::extension::ExtensionLockData {
                                registry: "local".to_string(),
                                disabled: false,
                            }
                        };
                        lock_data.disabled = true;
                        if let Ok(bytes) = serde_json::to_vec_pretty(&lock_data) {
                            let _ = fs::write(lock_path, bytes);
                        }
                    }
                }
            }
            has_started.store(true, std::sync::atomic::Ordering::SeqCst);
            let _ = reply_handler_clone.extensions_updated(&package_name_clone);
        });

        Self {
            plugin,
            package_name,
        }
    }

    #[tracing::instrument(level = "debug", skip_all)]
    fn get_allowed_paths(
        permissions: &ManifestPermissions,
        ext_cache_dir: &Path,
    ) -> HashMap<String, PathBuf> {
        let mut allowed_paths = HashMap::new();

        for (key, value) in &permissions.paths {
            // Replace all matches with corresponding env variable values
            let parsed = ENV_VAR_REGEX
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

            let Ok(value_path) = PathBuf::from_str(value);
            allowed_paths.insert(parsed, value_path);
        }

        tracing::info!("Got allowed paths {:?}", allowed_paths);
        allowed_paths
    }

    #[tracing::instrument(level = "debug", skip_all)]
    fn get_user_data(
        package_name: String,
        reply_handler: Arc<dyn ReplyHandler>,
        allowed_paths: Option<BTreeMap<String, PathBuf>>,
        allowed_hosts: Option<Vec<String>>,
    ) -> (
        UserData<MainCommandUserData>,
        UserData<SocketUserData>,
        UserData<HttpUserData>,
    ) {
        let user_data = UserData::new(MainCommandUserData {
            package_name,
            reply_handler,
        });

        let sock_data = UserData::new(SocketUserData {
            socks: vec![],
            allowed_paths,
        });

        let http_data = UserData::new(HttpUserData {
            allowed_hosts,
            client: reqwest::Client::builder()
                .pool_max_idle_per_host(10)
                .build()
                .unwrap_or_else(|_| reqwest::Client::new()),
        });

        (user_data, sock_data, http_data)
    }

    #[tracing::instrument(level = "debug", skip_all)]
    fn build_plugin(
        cache_path: &Path,
        plugin_manifest: Manifest,
        user_data: UserData<MainCommandUserData>,
        sock_data: UserData<SocketUserData>,
        http_data: UserData<HttpUserData>,
    ) -> Arc<Mutex<Plugin>> {
        let config_path = cache_path.join("wasmtime").join("config.toml");
        static WRITTEN_PATHS: std::sync::OnceLock<
            std::sync::Mutex<std::collections::HashSet<PathBuf>>,
        > = std::sync::OnceLock::new();
        let mutex =
            WRITTEN_PATHS.get_or_init(|| std::sync::Mutex::new(std::collections::HashSet::new()));
        {
            let mut paths = mutex.lock().unwrap();
            if !paths.contains(&config_path) {
                if !config_path.exists() {
                    fs::create_dir_all(config_path.parent().unwrap()).unwrap();
                }
                fs::write(
                    &config_path,
                    format!(
                        r#"[cache]
directory = "{}"
cleanup-interval = "30m"
files-total-size-soft-limit = "1Gi"
"#,
                        config_path
                            .parent()
                            .unwrap()
                            .join("cache")
                            .to_string_lossy()
                    ),
                )
                .unwrap();
                paths.insert(config_path.clone());
            }
        }

        let mut count = COMPILE_LIMIT.lock().unwrap();
        while *count >= MAX_CONCURRENT_COMPILATIONS {
            count = COMPILE_CONDVAR.wait(count).unwrap();
        }
        *count += 1;
        drop(count);

        let plugin_result = std::panic::catch_unwind(|| {
            #[allow(unused_mut)]
            let mut plugin_builder = PluginBuilder::new(plugin_manifest)
                .with_wasi(true)
                .with_cache_config(config_path)
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
                .with_function("hash", [PTR, PTR], [PTR], UserData::default(), hash)
                .with_function(
                    "batch_http_request",
                    [PTR],
                    [PTR],
                    http_data,
                    batch_http_request,
                );

            plugin_builder.build().unwrap()
        });

        let mut count = COMPILE_LIMIT.lock().unwrap();
        *count -= 1;
        COMPILE_CONDVAR.notify_one();
        drop(count);

        let plugin = match plugin_result {
            Ok(p) => p,
            Err(err) => std::panic::resume_unwind(err),
        };

        Arc::new(Mutex::new(plugin))
    }
}

#[async_trait::async_trait]
impl ExtensionContext for ExtismContext {
    #[tracing::instrument(level = "debug", skip_all)]
    async fn execute_command(
        &self,
        command: ExtensionCommand,
    ) -> Result<ExtensionCommandResponse, ExtensionError> {
        let plugin = self.plugin.clone();
        let package_name = self.package_name.clone();
        tokio::task::spawn_blocking(move || {
            let mut plugin = plugin.lock().unwrap();
            tracing::debug!("Calling {:?} on {:?}", command, plugin.id);
            let res = plugin.call::<_, Prost<ExtensionCommandResponse>>(
                "handle_extension_command",
                Prost(command),
            )?;
            tracing::trace!("Finished calling on {:?}", plugin.id);
            tracing::trace!("Response: {:?}", res);

            let mut parsed_resp = res.0;
            parsed_resp.sanitize(&package_name)?;
            Ok(parsed_resp)
        })
        .await
        .unwrap()
    }

    #[tracing::instrument(level = "debug", skip_all)]
    fn kill(&self) -> Result<(), ExtensionError> {
        let plugin = self.plugin.lock().unwrap();
        plugin.cancel_handle().cancel()?;
        Ok(())
    }
}
