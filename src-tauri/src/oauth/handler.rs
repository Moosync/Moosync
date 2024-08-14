use std::{collections::HashMap, sync::Mutex};

use tauri::{AppHandle, Manager, State};
use url::Url;

use types::errors::errors::Result;

use crate::providers::handler::ProviderHandler;

pub struct OAuthHandler {
    pub oauth_map: Mutex<HashMap<String, String>>,
}

impl OAuthHandler {
    pub fn new() -> OAuthHandler {
        OAuthHandler {
            oauth_map: Mutex::new(HashMap::new()),
        }
    }

    pub fn register_oauth_path(&self, path: String, key: String) {
        let mut oauth_map = self.oauth_map.lock().unwrap();
        oauth_map.insert(path, key.clone());
    }

    pub fn unregister_oauth_path(&self, path: String) {
        let mut oauth_map = self.oauth_map.lock().unwrap();
        oauth_map.remove(&path);
    }

    pub fn handle_oauth(&self, app: AppHandle, url: String) -> Result<()> {
        let oauth_map = self.oauth_map.lock().unwrap();
        let url_parsed = Url::parse(url.as_str()).unwrap();
        let path = url_parsed.host_str().unwrap();
        if let Some(key) = oauth_map.get(path) {
            let app = app.clone();
            let key = key.clone();
            tauri::async_runtime::spawn(async move {
                println!("Authorizing {}", key);
                let provider_handler: State<ProviderHandler> = app.state();
                let err = provider_handler.provider_authorize(key.clone(), url).await;
                if let Err(err) = err {
                    println!("Error authorizing {}: {:?}", key, err);
                }
            });
        }

        Ok(())
    }
}

pub fn get_oauth_state() -> Result<OAuthHandler> {
    Ok(OAuthHandler::new())
}
