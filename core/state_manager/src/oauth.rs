use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};

use extensions::ExtensionError;
use extensions_proto::moosync::types::OauthCallbackRequest;
use url::Url;

use crate::StateManager;

#[derive(Clone, Default)]
pub struct OAuthManager {
    pub(crate) routes: Arc<RwLock<HashMap<String, String>>>,
}

impl OAuthManager {
    #[tracing::instrument(level = "debug", skip_all)]
    pub fn new() -> Self {
        Self {
            routes: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub fn register_path(&self, package_name: String, url: String) {
        let host = if let Ok(parsed) = Url::parse(&url) {
            parsed.host_str().unwrap_or(&url).to_string()
        } else {
            url.trim_start_matches("://").trim_matches('/').to_string()
        };
        let mut routes = self.routes.write().unwrap();
        routes.insert(host, package_name);
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub fn find_package_for_host(&self, host: &str) -> Option<String> {
        let routes = self.routes.read().unwrap();
        routes.get(host).cloned()
    }
}

impl StateManager {
    #[tracing::instrument(level = "debug", skip_all)]
    pub fn register_oauth_path(&self, package_name: String, path: String) {
        self.oauth.register_path(package_name, path);
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub fn find_package_for_oauth_host(&self, host: &str) -> Option<String> {
        self.oauth.find_package_for_host(host)
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub async fn handle_oauth_callback(&self, callback_url: &str) -> Result<(), ExtensionError> {
        let parsed_url = Url::parse(callback_url).map_err(|e| {
            ExtensionError::Sanitize(format!("Invalid oauth callback URL '{callback_url}': {e}"))
        })?;

        let host = parsed_url.host_str().ok_or_else(|| {
            ExtensionError::Sanitize(format!(
                "No host found in oauth callback URL: {callback_url}"
            ))
        })?;

        let package_name = self.find_package_for_oauth_host(host).ok_or_else(|| {
            ExtensionError::Sanitize(format!("No extension registered for oauth host: {host}"))
        })?;

        let extensions = self.get_extension_handler().await;
        let extension = extensions.get_extension(&package_name)?;
        extension
            .oauth_callback(OauthCallbackRequest {
                callback_uri: callback_url.to_string(),
            })
            .await?;
        Ok(())
    }
}
