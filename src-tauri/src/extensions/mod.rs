use std::collections::HashMap;


use extensions::ExtensionHandler;
use extensions::FetchedExtensionManifest;
use macros::generate_command;
use macros::generate_command_async;
use serde_json::Value;
use tauri::AppHandle;
use tauri::State;
use types::errors::errors::Result;

pub fn get_extension_state(app: AppHandle) -> Result<ExtensionHandler> {
    let ext_handler = ExtensionHandler::new(app);
    ext_handler.listen_socket()?;
    Ok(ext_handler)
}

generate_command!(broadcast, ExtensionHandler, HashMap<String, Value>, value: Value);
generate_command!(install_extension, ExtensionHandler, (), ext_path: String);
generate_command_async!(download_extension, ExtensionHandler, (), fetched_ext: FetchedExtensionManifest);
