use std::path::PathBuf;

use extensions::ExtensionError;
use extensions_proto::moosync::types::OauthCallbackRequest;
use url::Url;

use crate::StateManager;

#[derive(Debug, Clone, Default, PartialEq)]
pub struct ProcessedOauthResponse {
    pub extra_content: Option<String>,
    pub extension_path: PathBuf,
}

impl StateManager {
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

        let package_name = self
            .get_oauth_manager()
            .find_package_for_host(host)
            .ok_or_else(|| {
                ExtensionError::Sanitize(format!("No extension registered for oauth host: {host}"))
            })?;

        let extensions = self.get_extension_handler();
        let extension = extensions.get_extension(&package_name)?;
        let resp = extension
            .oauth_callback(OauthCallbackRequest {
                callback_uri: callback_url.to_string(),
            })
            .await?;

        let extra_content = resp.extra_content.filter(|raw| !raw.trim().is_empty());

        Ok(ProcessedOauthResponse {
            extra_content,
            extension_path: extension.extension_path().to_path_buf(),
        })
    }
}
