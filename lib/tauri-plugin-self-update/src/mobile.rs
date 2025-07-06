use serde::{de::DeserializeOwned, Serialize};
use tauri::{
    plugin::{PluginApi, PluginHandle},
    AppHandle, Runtime,
};

use crate::PlatformInfo;
use types::errors::{MoosyncError, Result};

#[cfg(target_os = "ios")]
tauri::ios_plugin_binding!(init_plugin_self_update);

#[cfg(target_os = "android")]
const PLUGIN_IDENTIFIER: &str = "app.moosync.selfupdate";

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct UpdateArgs {
    payload: PlatformInfo,
}

// initializes the Kotlin or Swift plugin classes
pub fn init<R: Runtime, C: DeserializeOwned>(
    app: &AppHandle<R>,
    api: PluginApi<R, C>,
) -> Result<SelfUpdate<R>> {
    #[cfg(target_os = "android")]
    let handle = api
        .register_android_plugin(PLUGIN_IDENTIFIER, "SelfUpdatePlugin")
        .map_err(|e| MoosyncError::String(e.to_string()))?;

    #[cfg(target_os = "ios")]
    let handle = api
        .register_ios_plugin(init_plugin_self_update)
        .map_err(|e| MoosyncError::String(e.to_string()))?;

    Ok(SelfUpdate(handle))
}

/// Access to the self-update APIs.
pub struct SelfUpdate<R: Runtime>(PluginHandle<R>);

impl<R: Runtime> SelfUpdate<R> {
    pub fn download_and_install(&self, payload: PlatformInfo) -> Result<()> {
        self.0
            .run_mobile_plugin("download_and_install", UpdateArgs { payload })
            .map_err(|e| MoosyncError::String(e.to_string()))
    }
}
