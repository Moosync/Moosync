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

use std::collections::HashMap;

use serde::{de::DeserializeOwned, Deserialize, Serialize};
use tauri::plugin::PermissionState;
use tauri::{
    ipc::Channel,
    plugin::{PluginApi, PluginHandle},
    utils::acl::Value,
    AppHandle, Emitter, Runtime,
};
use types::{
    errors::{MoosyncError, Result},
    mpris::MprisPlayerDetails,
    songs::Song,
};

#[cfg(target_os = "ios")]
tauri::ios_plugin_binding!(init_plugin_audioplayer);

// initializes the Kotlin or Swift plugin classes
pub fn init<R: Runtime, C: DeserializeOwned>(
    app: &AppHandle<R>,
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

    let ret = Audioplayer(handle);
    ret.register_media_callback(app.clone());

    Ok(ret)
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

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct UpdateNotificationArgs {
    metadata: MprisPlayerDetails,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct UpdateNotificationStateArgs {
    playing: bool,
    pos: u64,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EventHandler {
    pub handler: Channel,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PermissionResponse {
    pub read_media: PermissionState,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RequestPermission {
    read_media: bool,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct TokenArgs {
    token: String,
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

    pub fn update_notification(&self, metadata: MprisPlayerDetails) -> Result<()> {
        let res: serde_json::Value = self
            .0
            .run_mobile_plugin("updateNotification", UpdateNotificationArgs { metadata })
            .map_err(|e| MoosyncError::String(e.to_string()))?;
        Ok(())
    }

    pub fn update_notification_state(&self, playing: bool, pos: u64) -> Result<()> {
        let res: serde_json::Value = self
            .0
            .run_mobile_plugin(
                "updateNotificationState",
                UpdateNotificationStateArgs { playing, pos },
            )
            .map_err(|e| MoosyncError::String(e.to_string()))?;
        Ok(())
    }

    fn register_media_callback(&self, app: AppHandle<R>) -> Result<()> {
        self.0.run_mobile_plugin::<()>(
            "setEventHandler",
            EventHandler {
                handler: Channel::new(move |event| match event {
                    tauri::ipc::InvokeResponseBody::Json(payload) => {
                        app.emit(
                            "MediaSessionCallback",
                            serde_json::from_str::<HashMap<String, Value>>(&payload).unwrap(),
                        );
                        Ok(())
                    }
                    _ => Ok(()),
                }),
            },
        );

        Ok(())
    }

    pub fn request_read_media_permission(&self) -> Result<PermissionState> {
        self.0
            .run_mobile_plugin::<PermissionResponse>(
                "requestPermissions",
                RequestPermission { read_media: true },
            )
            .map(|r| r.read_media)
            .map_err(|e| MoosyncError::String(e.to_string()))
    }

    pub fn check_permissions(&self) -> Result<PermissionResponse> {
        self.0
            .run_mobile_plugin::<PermissionResponse>("checkPermissions", ())
            .map_err(|e| MoosyncError::String(e.to_string()))
    }
}
