use std::sync::Mutex;
use tauri::{AppHandle, Emitter, State};
use tauri_plugin_updater::{Update, UpdaterExt};
use types::{
    errors::{MoosyncError, Result},
    ui::updater::{DownloadEvent, UpdateMetadata},
};

#[tauri::command]
#[tauri_invoke_proc::parse_tauri_command]
pub async fn fetch_update(
    app: AppHandle,
    pending_update: State<'_, PendingUpdate>,
) -> Result<Option<UpdateMetadata>> {
    let update = app
        .updater_builder()
        .build()
        .map_err(|e| MoosyncError::String(e.to_string()))?
        .check()
        .await
        .map_err(|e| MoosyncError::String(e.to_string()))?;

    let update_metadata = update.as_ref().map(|update| UpdateMetadata {
        version: update.version.clone(),
        current_version: update.current_version.clone(),
    });

    *pending_update.0.lock().unwrap() = update;

    Ok(update_metadata)
}

#[tauri::command]
#[tauri_invoke_proc::parse_tauri_command]
pub async fn install_update(
    app: AppHandle,
    pending_update: State<'_, PendingUpdate>,
) -> Result<()> {
    let Some(update) = pending_update.0.lock().unwrap().take() else {
        return Err("No pending update".into());
    };

    let mut started = false;

    update
        .download_and_install(
            |chunk_length, content_length| {
                if !started {
                    let _ = app.emit("update_event", DownloadEvent::Started { content_length });
                    started = true;
                }

                let _ = app.emit("update_event", DownloadEvent::Progress { chunk_length });
            },
            || {
                let _ = app.emit("update_event", DownloadEvent::Finished);
                app.restart();
            },
        )
        .await
        .map_err(|e| MoosyncError::String(e.to_string()))?;

    Ok(())
}

pub struct PendingUpdate(Mutex<Option<Update>>);

pub fn get_updater_state() -> PendingUpdate {
    PendingUpdate(Mutex::new(None))
}
