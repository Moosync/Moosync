use tauri::{AppHandle, Runtime};

/// Access to the self-update APIs.
pub struct SelfUpdate<R: Runtime>(AppHandle<R>);
