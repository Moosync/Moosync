use futures::TryFutureExt;
use semver::{Version, VersionReq};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env;
use std::sync::Mutex;
use tauri::{AppHandle, Emitter, Manager, Runtime, State};
use tauri_plugin_self_update::{PlatformInfo, Release, SelfUpdateExt};
use types::errors::{error_helpers, Result};
use types::ui::updater::UpdateMetadata;

const URL: &'static str =
    "https://github.com/Moosync/Moosync/releases/latest/download/latest-android.json";

fn find_release(release_info: &Release) -> Option<PlatformInfo> {
    let arch = env::consts::ARCH;
    for (platform, info) in &release_info.platforms {
        if platform == &format!("android-{arch}") {
            return Some(info.clone());
        }
    }
    return None;
}

#[tauri::command]
#[tauri_invoke_proc::parse_tauri_command]
pub async fn fetch_update<R: Runtime>(
    app: AppHandle<R>,
    pending_update: State<'_, PendingUpdate>,
) -> Result<Option<UpdateMetadata>> {
    let latest_release = reqwest::get(URL)
        .await
        .map_err(error_helpers::to_network_error)?
        .bytes()
        .await
        .map_err(error_helpers::to_network_error)?;

    let parsed_release: Release =
        serde_json::from_slice(&latest_release).map_err(error_helpers::to_parse_error)?;

    let curr_version = app.config().version.clone().unwrap_or("0.0.0".into());
    let required_ver =
        VersionReq::parse(&format!(">{}", curr_version)).map_err(error_helpers::to_parse_error)?;
    let new_version =
        Version::parse(&parsed_release.version).map_err(error_helpers::to_parse_error)?;

    if required_ver.matches(&new_version) {
        if let Some(platform_info) = find_release(&parsed_release) {
            *pending_update.0.lock().unwrap() = Some(platform_info);
            return Ok(Some(UpdateMetadata {
                version: parsed_release.version,
                current_version: curr_version,
            }));
        }
    }

    return Ok(None);
}

#[tauri::command]
#[tauri_invoke_proc::parse_tauri_command]
pub async fn install_update<R: Runtime>(
    app: AppHandle<R>,
    pending_update: State<'_, PendingUpdate>,
) -> Result<()> {
    let Some(update) = pending_update.0.lock().unwrap().take() else {
        return Err("No pending update".into());
    };

    let self_update = app.self_update();

    self_update.download_and_install(update)?;

    // update
    //     .download_and_install(
    //         |chunk_length, content_length| {
    //             if !started {
    //                 let _ = app.emit("update_event", DownloadEvent::Started { content_length });
    //                 started = true;
    //             }

    //             let _ = app.emit("update_event", DownloadEvent::Progress { chunk_length });
    //         },
    //         || {
    //             let _ = app.emit("update_event", DownloadEvent::Finished);
    //             app.restart();
    //         },
    //     )
    //     .await
    //     .map_err(|e| MoosyncError::String(e.to_string()))?;

    Ok(())
}

pub struct PendingUpdate(Mutex<Option<PlatformInfo>>);

pub fn get_updater_state() -> PendingUpdate {
    PendingUpdate(Default::default())
}

#[cfg(test)]
mod tests {
    use tauri::Manager;

    use super::*;

    #[tokio::test]
    async fn test_fetch_release() {
        let app = tauri::test::mock_app();
        app.manage(get_updater_state());

        let state = app.state::<PendingUpdate>();

        let res = fetch_update(app.app_handle().clone(), state).await;
        println!("res {:?}", res);
    }
}
