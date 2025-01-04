use serde::de::DeserializeOwned;
use tauri::{plugin::PluginApi, AppHandle, Runtime};

use types::errors::Result;

pub fn init<R: Runtime, C: DeserializeOwned>(
    app: &AppHandle<R>,
    _api: PluginApi<R, C>,
) -> Result<Audioplayer<R>> {
    Ok(Audioplayer(app.clone()))
}

/// Access to the audioplayer APIs.
pub struct Audioplayer<R: Runtime>(AppHandle<R>);
