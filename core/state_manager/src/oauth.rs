use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};

use extensions::ExtensionError;
use extensions_proto::moosync::types::OauthCallbackRequest;
use url::Url;

use crate::{
    StateManager,
    oauth_markdown::{ProcessedOauthMarkdown, parse_and_split_oauth_markdown},
};

#[derive(Debug, Clone, Default, PartialEq)]
pub struct ProcessedOauthResponse {
    pub extra_content: Option<ProcessedOauthMarkdown>,
}

#[derive(Clone, Default)]
pub struct OAuthManager {
    pub(crate) routes: Arc<RwLock<HashMap<String, String>>>,
    pub(crate) package_hosts: Arc<RwLock<HashMap<String, String>>>,
}

impl OAuthManager {
    #[tracing::instrument(level = "debug", skip_all)]
    pub fn new() -> Self {
        Self {
            routes: Arc::new(RwLock::new(HashMap::new())),
            package_hosts: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub fn register_path(&self, package_name: String, url: String) {
        let host = if let Ok(parsed) = Url::parse(&url) {
            parsed.host_str().unwrap_or(&url).to_string()
        } else {
            url.trim_start_matches("://").trim_matches('/').to_string()
        };
        if let Ok(mut routes) = self.routes.write() {
            routes.insert(host.clone(), package_name.clone());
        } else {
            tracing::error!("OAuthManager routes write lock poisoned");
        }
        if let Ok(mut package_hosts) = self.package_hosts.write() {
            package_hosts.entry(package_name).or_insert(host);
        } else {
            tracing::error!("OAuthManager package_hosts write lock poisoned");
        }
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub fn find_package_for_host(&self, host: &str) -> Option<String> {
        match self.routes.read() {
            Ok(routes) => routes.get(host).cloned(),
            Err(err) => {
                tracing::error!("OAuthManager routes read lock poisoned: {:?}", err);
                None
            }
        }
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub fn find_host_for_package(&self, package_name: &str) -> Option<String> {
        match self.package_hosts.read() {
            Ok(package_hosts) => package_hosts.get(package_name).cloned(),
            Err(err) => {
                tracing::error!("OAuthManager package_hosts read lock poisoned: {:?}", err);
                None
            }
        }
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
    pub fn find_oauth_host_for_package(&self, package_name: &str) -> Option<String> {
        self.oauth.find_host_for_package(package_name)
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub async fn handle_oauth_callback(
        &self,
        callback_url: &str,
    ) -> Result<ProcessedOauthResponse, ExtensionError> {
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

        let extensions = self.get_extension_handler();
        let extension = extensions.get_extension(&package_name)?;
        let resp = extension
            .oauth_callback(OauthCallbackRequest {
                callback_uri: callback_url.to_string(),
            })
            .await?;

        let extra_content = resp.extra_content.and_then(|raw| {
            if raw.trim().is_empty() {
                None
            } else {
                Some(parse_and_split_oauth_markdown(
                    &raw,
                    Some(extension.extension_path()),
                ))
            }
        });

        Ok(ProcessedOauthResponse { extra_content })
    }
}
