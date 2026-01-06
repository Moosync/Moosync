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

use std::sync::Arc;

use crate::macros::generate_command_async;
use crate::macros::generate_command_async_cached;
use database::cache::CacheHolder;
use extensions::ExtensionHandler;
use extensions_proto::moosync::types::ExtensionDetail;
use extensions_proto::moosync::types::FetchedExtensionManifest;
use request_handler::ReplyHandler;
use tauri::AppHandle;
use tauri::Manager;
use tauri::State;
use types::errors::Result;

mod request_handler;

#[tracing::instrument(level = "debug", skip(app))]
pub fn get_extension_state(app: AppHandle) -> Result<ExtensionHandler> {
    let ext_path = app.path().app_data_dir().unwrap().join("extensions");
    let tmp_dir = app.path().temp_dir().unwrap();
    let cache_dir = app.path().cache_dir().unwrap();

    let ext_handler = ExtensionHandler::new(
        ext_path,
        tmp_dir,
        cache_dir,
        Arc::new(Box::new(move |ext, command| {
            let app = app.clone();
            let reply_handler = ReplyHandler::new(app);
            tauri::async_runtime::block_on(reply_handler.handle_request(ext, command))
        })),
    );

    Ok(ext_handler)
}

#[tracing::instrument(level = "debug", skip(app))]
pub fn get_extension_handler(app: &AppHandle) -> State<'_, ExtensionHandler> {
    app.state()
}

generate_command_async_cached!(
    get_extension_manifest,
    ExtensionHandler,
    Vec<FetchedExtensionManifest>
);
generate_command_async!(install_extension, ExtensionHandler, (), ext_path: String);
generate_command_async!(remove_extension, ExtensionHandler, (), ext_path: String);
generate_command_async!(download_extension, ExtensionHandler, (), fetched_ext: FetchedExtensionManifest);
generate_command_async!(
    get_installed_extensions,
    ExtensionHandler,
    Vec<ExtensionDetail>,
);
generate_command_async_cached!(get_extension_icon, ExtensionHandler, String, args: String);
