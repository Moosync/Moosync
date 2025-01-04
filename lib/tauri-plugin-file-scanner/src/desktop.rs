use tauri::{plugin::PluginHandle, Runtime};

pub struct FileScanner<R: Runtime>(PluginHandle<R>);
