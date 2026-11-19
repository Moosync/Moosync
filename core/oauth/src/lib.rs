use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};

use plugin_macro::generate;
use types::plugin::{Plugin, PluginContext};
use url::Url;

#[cfg(test)]
mod lib_test;

#[derive(Clone, Default)]
pub struct OauthManager {
    routes: Arc<RwLock<HashMap<String, String>>>,
}

#[generate]
impl OauthManager {
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

    #[tracing::instrument(level = "debug", skip_all)]
    pub fn find_host_for_package(&self, package_name: &str) -> Option<String> {
        let routes = self.routes.read().unwrap();
        routes
            .iter()
            .find(|(_, pkg)| pkg.as_str() == package_name)
            .map(|(host, _)| host.clone())
    }
}

impl Plugin for OauthManager {
    #[tracing::instrument(level = "debug", skip_all)]
    fn init(_context: &PluginContext) -> Arc<Self> { Arc::new(OauthManager::new()) }
}
