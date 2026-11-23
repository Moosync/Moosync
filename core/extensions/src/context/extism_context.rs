use std::{
    collections::{HashMap, HashSet},
    env,
    fmt::Debug,
    fs,
    path::{Path, PathBuf},
    process,
    str::FromStr,
    sync::{Arc, Condvar, Mutex, OnceLock, atomic::Ordering},
    thread,
};

use cache::{CacheCategory, CacheManager};
use extensions_proto::moosync::types::{
    Error as MainCommandError, ExtensionCommand, ExtensionCommandResponse, MainCommand,
    MainCommandResponse, ManifestPermissions, main_command_response,
};
use extism::{
    CancelHandle, Error as ExtismError, Manifest, PTR, Plugin, PluginBuilder, UserData, Wasm,
    host_fn,
};
use extism_convert::Prost;
use tokio::sync::oneshot;

use crate::{
    cache::ExtensionMemoryCache,
    context::{DispatchCommand, ExtensionContext, HostCommandContext, ReplyHandler},
    errors::ExtensionError,
    extension::Extension,
    models::SanitizeCommand,
    resolve_env_vars,
    sanitize::Sanitize,
};

const HANDLE_EXTENSION_COMMAND: &str = "handle_extension_command";

struct MainCommandUserData {
    package_name: String,
    reply_handler: Arc<dyn ReplyHandler>,
    memory_cache: Arc<Mutex<ExtensionMemoryCache>>,
}

host_fn!(send_main_command(user_data: MainCommandUserData; command_wrapper: Prost<MainCommand>) {
    let user_data_arc = user_data.get()?;
    let (package_name, reply_handler, memory_cache) = {
        let data = user_data_arc.lock().unwrap();
        (
            data.package_name.clone(),
            data.reply_handler.clone(),
            data.memory_cache.clone(),
        )
    };

    let mut command = command_wrapper.0;
    if let Err(error) = command.sanitize(&package_name) {
        return Ok(Prost(MainCommandResponse {
            response: Some(main_command_response::Response::Error(MainCommandError {
                message: error.to_string()
            })),
        }));
    }

    tracing::debug!("Sending command {:?} from {}", command, package_name);
    let runtime_handle = reply_handler.get_runtime_handle();
    let _runtime_guard = runtime_handle.as_ref().map(|handle| handle.enter());
    let context = HostCommandContext {
        reply_handler: reply_handler.as_ref(),
        package_name: &package_name,
        memory_cache: &memory_cache,
    };
    let response = match command.command {
        Some(cmd) => cmd.dispatch(&context),
        None => Err(ExtensionError::MissingCommand),
    };

    let result = response.map(|resp| MainCommandResponse {
        response: Some(resp),
    });

    tracing::debug!("command returning to {}", package_name);

    match result {
        Ok(response) => Ok(Prost(response)),
        Err(error) => Ok(Prost(MainCommandResponse {
            response: Some(main_command_response::Response::Error(MainCommandError {
                message: error.to_string()
            })),
        })),
    }
});

static COMPILE_LIMIT: Mutex<usize> = Mutex::new(0);
static COMPILE_CONDVAR: Condvar = Condvar::new();
const MAX_CONCURRENT_COMPILATIONS: usize = 5;

pub struct ExtismContext {
    plugin: Arc<Mutex<Plugin>>,
    package_name: String,
    cancel_handle: CancelHandle,
}

impl Debug for ExtismContext {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ExtismContext")
            .field("package_name", &self.package_name)
            .finish()
    }
}

impl TryFrom<&Extension> for Arc<ExtismContext> {
    type Error = ExtensionError;

    #[tracing::instrument(level = "debug", skip_all)]
    fn try_from(extension: &Extension) -> Result<Self, Self::Error> {
        let manifest = &extension.manifest;
        let url = Wasm::file(manifest.extension_entry.clone());
        let mut plugin_manifest = Manifest::new([url]);

        let package_name = manifest.name.clone();
        let ext_cache_dir = extension.cache_path.join("extensions").join(&package_name);

        if let Err(error) = fs::create_dir_all(&ext_cache_dir) {
            tracing::error!(
                "Failed to create cache dir for extension {}: {:?}",
                package_name,
                error
            );
        }

        if let Some(permissions) = &manifest.permissions {
            let allowed_paths = ExtismContext::get_allowed_paths(permissions, &ext_cache_dir);
            plugin_manifest = plugin_manifest
                .with_allowed_hosts(permissions.hosts.iter().map(|h| h.host.clone()))
                .with_allowed_paths(allowed_paths.into_iter())
                .with_config_key("pid", format!("{}", process::id()));
        }

        plugin_manifest = plugin_manifest
            .with_config_key("os", env::consts::OS)
            .with_config_key("arch", env::consts::ARCH);

        let user_data = UserData::new(MainCommandUserData {
            package_name: package_name.clone(),
            reply_handler: extension.reply_handler.clone(),
            memory_cache: extension.memory_cache.clone(),
        });

        let plugin =
            match ExtismContext::build_plugin(&extension.cache_path, plugin_manifest, user_data) {
                Ok(plugin) => plugin,
                Err(error) => {
                    let err_msg = format!("Failed to build plugin: {}", error);
                    let _ = extension.error.set(err_msg.clone());
                    extension.context.set(Err(err_msg));
                    return Err(error);
                }
            };
        let cancel_handle = plugin.lock().unwrap().cancel_handle();
        let context = Arc::new(ExtismContext {
            plugin: plugin.clone(),
            package_name: package_name.clone(),
            cancel_handle,
        });

        let package_name_clone = package_name.clone();
        let reply_handler_clone = extension.reply_handler.clone();
        let error_holder = extension.error.clone();
        let context_holder = extension.context.clone();
        let has_started = extension.has_started.clone();
        let context_clone = context.clone();
        thread::spawn(move || {
            let mut plugin = plugin.lock().unwrap();
            tracing::debug!("Calling entry on {:?}", package_name_clone);
            if plugin.function_exists("entry")
                && let Err(error) = plugin.call::<(), ()>("entry", ())
            {
                tracing::error!("Failed to call extension entry: {:?}", error);
                let err_msg = format!("Failed to start: {}", error);
                let _ = error_holder.set(err_msg.clone());
                context_holder.set(Err(err_msg));
                let _ = reply_handler_clone.extensions_updated(&package_name_clone);
                return;
            }
            has_started.store(true, Ordering::SeqCst);
            context_holder.set(Ok(context_clone));
            let _ = reply_handler_clone.extensions_updated(&package_name_clone);
        });

        Ok(context)
    }
}

impl ExtismContext {
    #[tracing::instrument(level = "debug", skip_all)]
    pub(crate) fn get_allowed_paths(
        permissions: &ManifestPermissions,
        ext_cache_dir: &Path,
    ) -> HashMap<String, PathBuf> {
        let mut allowed_paths = HashMap::new();

        for p in &permissions.paths {
            let key = &p.source;
            let value = &p.target;
            let parsed = resolve_env_vars(key.as_str(), ext_cache_dir);

            let Ok(parsed_path) = PathBuf::from_str(&parsed);
            if !parsed_path.exists()
                && parsed_path.starts_with(ext_cache_dir)
                && let Err(e) = std::fs::create_dir_all(&parsed_path)
            {
                tracing::warn!("Failed to create cache path {:?}: {}", parsed_path, e);
            }
            if parsed_path.exists() {
                let Ok(value_path) = PathBuf::from_str(value);
                allowed_paths.insert(parsed, value_path);
            } else {
                tracing::warn!("Path {:?} does not exist", parsed_path);
            }
        }

        tracing::info!("Got allowed paths {:?}", allowed_paths);
        allowed_paths
    }

    #[tracing::instrument(level = "debug", skip_all)]
    fn build_plugin(
        cache_path: &Path,
        plugin_manifest: Manifest,
        user_data: UserData<MainCommandUserData>,
    ) -> Result<Arc<Mutex<Plugin>>, ExtensionError> {
        let config_path = cache_path.join("wasmtime").join("config.toml");
        static WRITTEN_PATHS: OnceLock<Mutex<HashSet<PathBuf>>> = OnceLock::new();
        let mutex = WRITTEN_PATHS.get_or_init(|| Mutex::new(HashSet::new()));
        {
            let mut paths = mutex.lock().unwrap();
            if !paths.contains(&config_path) {
                if !config_path.exists() {
                    let _ = fs::create_dir_all(config_path.parent().unwrap());
                }
                let config_contents = format!(
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
                );
                let _ = CacheManager::new(cache_path.to_path_buf()).write(
                    CacheCategory::Wasmtime,
                    "app",
                    &config_path,
                    config_contents.as_bytes(),
                );
                paths.insert(config_path.clone());
            }
        }

        let mut count = COMPILE_LIMIT.lock().unwrap();
        while *count >= MAX_CONCURRENT_COMPILATIONS {
            count = COMPILE_CONDVAR.wait(count).unwrap();
        }
        *count += 1;
        drop(count);

        let plugin_builder = PluginBuilder::new(plugin_manifest)
            .with_wasi(true)
            .with_cache_config(config_path)
            .with_function(
                "send_main_command",
                [PTR],
                [PTR],
                user_data,
                send_main_command,
            );

        let build_result = plugin_builder.build();

        let mut count = COMPILE_LIMIT.lock().unwrap();
        *count -= 1;
        COMPILE_CONDVAR.notify_one();
        drop(count);

        let plugin = build_result?;

        Ok(Arc::new(Mutex::new(plugin)))
    }
}

struct CancelGuard {
    handle: CancelHandle,
    active: bool,
}

impl CancelGuard {
    #[tracing::instrument(level = "trace", skip_all)]
    fn new(handle: CancelHandle) -> Self {
        Self {
            handle,
            active: true,
        }
    }

    #[tracing::instrument(level = "trace", skip_all)]
    fn defuse(&mut self) { self.active = false; }
}

impl Drop for CancelGuard {
    fn drop(&mut self) {
        if self.active {
            let _ = self.handle.cancel();
        }
    }
}

#[async_trait::async_trait]
impl ExtensionContext for ExtismContext {
    #[tracing::instrument(level = "debug", skip_all)]
    async fn execute_command(
        &self,
        command: ExtensionCommand,
    ) -> Result<ExtensionCommandResponse, ExtensionError> {
        let mut guard = CancelGuard::new(self.cancel_handle.clone());

        let plugin = self.plugin.clone();
        let package_name = self.package_name.clone();
        let (sender, receiver) = oneshot::channel();

        thread::spawn(move || {
            let run = move || {
                let mut plugin = plugin.lock().unwrap();
                tracing::debug!("Calling {:?} on {:?}", command, plugin.id);
                let response = plugin.call::<_, Prost<ExtensionCommandResponse>>(
                    HANDLE_EXTENSION_COMMAND,
                    Prost(command),
                )?;
                tracing::trace!("Finished calling on {:?}", plugin.id);
                tracing::trace!("Response: {:?}", response);

                let parsed_resp = response.0.sanitize(&package_name);
                Ok(parsed_resp)
            };
            let _ = sender.send(run());
        });

        let result = match receiver.await {
            Ok(res) => res,
            Err(error) => Err(ExtensionError::Extism(ExtismError::msg(error.to_string()))),
        };

        guard.defuse();
        result
    }
}
