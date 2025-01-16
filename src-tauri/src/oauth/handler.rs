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

use std::{collections::HashMap, sync::Mutex};

use tauri::{AppHandle, Manager, State};
use url::Url;

use types::errors::Result;

use crate::providers::handler::ProviderHandler;

pub struct OAuthHandler {
    pub oauth_map: Mutex<HashMap<String, String>>,
}

impl OAuthHandler {
    #[tracing::instrument(level = "trace", skip())]
    pub fn new() -> OAuthHandler {
        OAuthHandler {
            oauth_map: Mutex::new(HashMap::new()),
        }
    }

    #[tracing::instrument(level = "trace", skip(self, path, key))]
    pub fn register_oauth_path(&self, path: String, key: String) {
        let mut oauth_map = self.oauth_map.lock().unwrap();
        oauth_map.insert(path, key.clone());
    }

    #[tracing::instrument(level = "trace", skip(self, path))]
    pub fn unregister_oauth_path(&self, path: String) {
        let mut oauth_map = self.oauth_map.lock().unwrap();
        oauth_map.remove(&path);
    }

    #[tracing::instrument(level = "trace", skip(self, app, url))]
    pub fn handle_oauth(&self, app: AppHandle, url: String) -> Result<()> {
        let oauth_map = self.oauth_map.lock().unwrap();
        let url_parsed = Url::parse(url.as_str()).unwrap();
        let path = url_parsed.host_str().unwrap();
        if let Some(key) = oauth_map.get(path) {
            let app = app.clone();
            let key = key.clone();
            tauri::async_runtime::spawn(async move {
                tracing::info!("Authorizing {}", key);
                let provider_handler: State<ProviderHandler> = app.state();
                let err = provider_handler.provider_authorize(key.clone(), url).await;
                if let Err(err) = err {
                    tracing::error!("Error authorizing {}: {:?}", key, err);
                }
            });
        }

        Ok(())
    }
}

#[tracing::instrument(level = "trace", skip())]
pub fn get_oauth_state() -> Result<OAuthHandler> {
    Ok(OAuthHandler::new())
}
