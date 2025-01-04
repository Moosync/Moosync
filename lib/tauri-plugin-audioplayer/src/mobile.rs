use serde::{de::DeserializeOwned, Serialize};
use tauri::{
    plugin::{PluginApi, PluginHandle},
    AppHandle, Runtime,
};
use types::errors::{MoosyncError, Result};

#[cfg(target_os = "ios")]
tauri::ios_plugin_binding!(init_plugin_audioplayer);

// initializes the Kotlin or Swift plugin classes
pub fn init<R: Runtime, C: DeserializeOwned>(
    _app: &AppHandle<R>,
    api: PluginApi<R, C>,
) -> Result<Audioplayer<R>> {
    #[cfg(target_os = "android")]
    let handle = api
        .register_android_plugin("app.moosync.audioplayer", "AudioPlayerPlugin")
        .map_err(|e| MoosyncError::String(e.to_string()))?;
    #[cfg(target_os = "ios")]
    let handle = api
        .register_ios_plugin(init_plugin_audioplayer)
        .map_err(|e| MoosyncError::String(e.to_string()))?;
    Ok(Audioplayer(handle))
}

/// Access to the audioplayer APIs.
pub struct Audioplayer<R: Runtime>(PluginHandle<R>);

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct LoadArgs {
    src: String,
    autoplay: bool,
    key: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct KeyArgs {
    key: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct SeekArgs {
    key: String,
    seek: f64,
}

impl<R: Runtime> Audioplayer<R> {
    pub fn load(&self, key: String, src: String, autoplay: bool) -> Result<()> {
        let res: serde_json::Value = self
            .0
            .run_mobile_plugin("load", LoadArgs { src, autoplay, key })
            .map_err(|e| MoosyncError::String(e.to_string()))?;
        Ok(())
    }

    pub fn play(&self, key: String) -> Result<()> {
        let res: serde_json::Value = self
            .0
            .run_mobile_plugin("play", KeyArgs { key })
            .map_err(|e| MoosyncError::String(e.to_string()))?;
        Ok(())
    }

    pub fn pause(&self, key: String) -> Result<()> {
        let res: serde_json::Value = self
            .0
            .run_mobile_plugin("pause", KeyArgs { key })
            .map_err(|e| MoosyncError::String(e.to_string()))?;
        Ok(())
    }

    pub fn stop(&self, key: String) -> Result<()> {
        let res: serde_json::Value = self
            .0
            .run_mobile_plugin("stop", KeyArgs { key })
            .map_err(|e| MoosyncError::String(e.to_string()))?;
        Ok(())
    }

    pub fn seek(&self, key: String, seek: f64) -> Result<()> {
        let res: serde_json::Value = self
            .0
            .run_mobile_plugin("seek", SeekArgs { key, seek })
            .map_err(|e| MoosyncError::String(e.to_string()))?;
        Ok(())
    }
}
