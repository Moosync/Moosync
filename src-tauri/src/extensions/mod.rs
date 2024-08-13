use std::collections::HashMap;

use database::cache::CacheHolder;
use extensions::ExtensionHandler;
use futures::SinkExt;
use futures::StreamExt;
use macros::generate_command_async;
use macros::generate_command_async_cached;
use request_handler::ReplyHandler;
use serde_json::Value;
use tauri::async_runtime;
use tauri::AppHandle;
use tauri::Manager;
use tauri::State;
use types::errors::errors::Result;
use types::extensions::ExtensionDetail;
use types::extensions::ExtensionExtraEventArgs;
use types::extensions::FetchedExtensionManifest;
use types::extensions::PackageNameArgs;

use crate::providers::handler::ProviderHandler;

mod request_handler;

async fn extension_runner_connected(app_handle: AppHandle) {
    let provider_handler: State<ProviderHandler> = app_handle.state();
    provider_handler
        .discover_provider_extensions()
        .await
        .unwrap();
}

pub fn get_extension_state(app: AppHandle) -> Result<ExtensionHandler> {
    let ext_path = app.path().app_data_dir().unwrap().join("extensions");
    let tmp_dir = app.path().temp_dir().unwrap();
    let ext_handler = ExtensionHandler::new(ext_path, tmp_dir);
    let mut rx_listen = ext_handler.listen_socket()?;

    async_runtime::spawn(async move {
        let app_handle = app.clone();
        loop {
            let rx_ext_command = rx_listen.next().await;
            if let Some(mut rx_ext_command) = rx_ext_command {
                let app_handle = app_handle.clone();
                let app_handle_1 = app_handle.clone();
                async_runtime::spawn(async move {
                    let request_handler = ReplyHandler::new(app_handle);
                    loop {
                        let message = rx_ext_command.next().await;
                        let request_handler = request_handler.clone();
                        async_runtime::spawn(async move {
                            if let Some((message, mut tx_reply)) = message {
                                let data = request_handler.handle_request(&message).await;
                                if let Ok(data) = data {
                                    tx_reply.send(data).await.unwrap();
                                }
                            }
                        });
                    }
                });
                extension_runner_connected(app_handle_1).await;
            }
        }
    });

    Ok(ext_handler)
}

pub fn get_extension_handler(app: &AppHandle) -> State<'_, ExtensionHandler> {
    let ext_state = app.state();
    ext_state
}

generate_command_async_cached!(
    get_extension_manifest,
    ExtensionHandler,
    Vec<FetchedExtensionManifest>,
);
generate_command_async!(install_extension, ExtensionHandler, (), ext_path: String);
generate_command_async!(remove_extension, ExtensionHandler, (), ext_path: String);
generate_command_async!(download_extension, ExtensionHandler, (), fetched_ext: FetchedExtensionManifest);
generate_command_async!(get_installed_extensions, ExtensionHandler, HashMap<String, Vec<ExtensionDetail>>, );
generate_command_async!(
    send_extra_event,
    ExtensionHandler,
    Value,
    args: ExtensionExtraEventArgs
);
generate_command_async_cached!(get_extension_icon, ExtensionHandler, String, args: PackageNameArgs);
