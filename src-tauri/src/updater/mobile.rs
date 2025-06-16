use tauri::{AppHandle, Emitter, State};
use types::errors::Result;
use types::ui::updater::UpdateMetadata;

#[tauri::command]
#[tauri_invoke_proc::parse_tauri_command]
pub async fn fetch_update(
    app: AppHandle,
    pending_update: State<'_, PendingUpdate>,
) -> Result<Option<UpdateMetadata>> {
    Ok(None)
}

#[tauri::command]
#[tauri_invoke_proc::parse_tauri_command]
pub async fn install_update(
    app: AppHandle,
    pending_update: State<'_, PendingUpdate>,
) -> Result<()> {
    Ok(())
}

pub struct PendingUpdate();

pub fn get_updater_state() -> PendingUpdate {
    PendingUpdate()
}
