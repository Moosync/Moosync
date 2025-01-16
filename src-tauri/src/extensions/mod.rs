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

// Moosync
// Copyright (C) 2025 Moosync
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
// along with this program. If not, see <http://www.gnu.org/licenses/>.

use database::cache::CacheHolder;
use extensions::ExtensionHandler;
use macros::generate_command_async;
use macros::generate_command_async_cached;
use request_handler::ReplyHandler;
use serde_json::Value;
use tauri::async_runtime;
use tauri::AppHandle;
use tauri::Manager;
use tauri::State;
use types::errors::Result;
use types::extensions::GenericExtensionHostRequest;
use types::ui::extensions::ExtensionDetail;
use types::ui::extensions::ExtensionExtraEventArgs;
use types::ui::extensions::FetchedExtensionManifest;
use types::ui::extensions::PackageNameArgs;

use crate::providers::handler::ProviderHandler;

mod request_handler;

#[tracing::instrument(level = "trace", skip(app_handle))]
async fn extension_runner_connected(app_handle: AppHandle) {
    let provider_handler: State<ProviderHandler> = app_handle.state();
    provider_handler
        .discover_provider_extensions()
        .await
        .unwrap();
}

#[tracing::instrument(level = "trace", skip(app))]
pub fn get_extension_state(app: AppHandle) -> Result<ExtensionHandler> {
    let ext_path = app.path().app_data_dir().unwrap().join("extensions");
    let tmp_dir = app.path().temp_dir().unwrap();
    let cache_dir = app.path().cache_dir().unwrap();
    let (ext_handler, mut ui_request_rx, ui_reply_tx) =
        ExtensionHandler::new(ext_path, cache_dir, tmp_dir);

    let app_clone = app.clone();
    async_runtime::spawn(async move {
        let app_handle = app.clone();
        let reply_handler = ReplyHandler::new(app_handle);
        loop {
            tracing::trace!("Waiting for extension UI requests");
            if let Some(resp) = ui_request_rx.recv().await {
                if let Some(data) = resp.data {
                    tracing::debug!("Got main command {:?}", data);
                    match reply_handler.handle_request(data).await {
                        Ok(reply) => {
                            ui_reply_tx
                                .send(GenericExtensionHostRequest {
                                    channel: resp.channel,
                                    data: Some(reply),
                                })
                                .unwrap();
                        }
                        Err(e) => {
                            tracing::error!("Failed to handle extension command {:?}", e)
                        }
                    }
                }
            }
        }
    });
    async_runtime::spawn(extension_runner_connected(app_clone));

    Ok(ext_handler)
}

#[tracing::instrument(level = "trace", skip(app))]
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
generate_command_async!(
    get_installed_extensions,
    ExtensionHandler,
    Vec<ExtensionDetail>,
);
generate_command_async!(
    send_extra_event,
    ExtensionHandler,
    Value,
    args: ExtensionExtraEventArgs
);
generate_command_async_cached!(get_extension_icon, ExtensionHandler, String, args: PackageNameArgs);
