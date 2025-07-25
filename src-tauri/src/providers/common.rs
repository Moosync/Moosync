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

use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

use oauth2::{basic::BasicClient, AuthUrl, ClientId, ClientSecret, RedirectUrl, TokenUrl};
use oauth2::{AuthorizationCode, CsrfToken, PkceCodeChallenge, RefreshToken, Scope, TokenResponse};
use preferences::preferences::PreferenceConfig;
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager, State};
use types::errors::{MoosyncError, Result};
use types::oauth::{OAuth2Client, OAuth2Verifier, OAuthTokenResponse};
use url::Url;
use types::errors::error_helpers;

use crate::window::handler::WindowHandler;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenHolder {
    pub access_token: String,
    pub refresh_token: String,
    pub expires_in: u64,
    pub expires_at: i64,
}

pub struct OAuthClientArgs {
    pub auth_url: String,
    pub token_url: String,
    pub redirect_url: String,
    pub client_id: String,
    pub client_secret: String,
}

#[tracing::instrument(level = "debug", skip(config))]
pub fn get_oauth_client(config: OAuthClientArgs) -> OAuth2Client {
    let mut client = BasicClient::new(ClientId::new(config.client_id.clone()))
        .set_auth_uri(AuthUrl::new(config.auth_url.clone()).unwrap())
        .set_token_uri(TokenUrl::new(config.token_url.clone()).unwrap())
        .set_redirect_uri(RedirectUrl::new(config.redirect_url.clone()).unwrap());

    if !config.client_secret.is_empty() {
        client = client.set_client_secret(ClientSecret::new(config.client_secret));
    };

    client
}

#[tracing::instrument(level = "debug", skip(key, app, res, default_refresh))]
pub fn set_tokens(
    key: &str,
    app: &AppHandle,
    res: OAuthTokenResponse,
    default_refresh: Option<String>,
) -> Result<TokenHolder> {
    let refresh_token = res
        .refresh_token()
        .map(|r| r.secret().clone())
        .unwrap_or(default_refresh.unwrap_or_default());
    let expires_in = res.expires_in().unwrap_or_default();
    let token_holder = TokenHolder {
        access_token: res.access_token().secret().clone(),
        refresh_token: refresh_token.clone(),
        expires_in: expires_in.as_secs(),
        expires_at: (SystemTime::now().duration_since(UNIX_EPOCH).unwrap() + expires_in).as_millis()
            as i64,
    };

    let preferences: State<PreferenceConfig> = app.state();
    preferences.set_secure(key.into(), Some(refresh_token))?;

    Ok(token_holder)
}

#[tracing::instrument(level = "debug", skip(key, client, app))]
pub async fn refresh_login(
    key: &str,
    client: OAuth2Client,
    app: &AppHandle,
) -> Result<TokenHolder> {
    let preferences: State<PreferenceConfig> = app.state();
    let refresh_token: Result<String> = preferences.inner().get_secure(key.into());
    if refresh_token.is_err() {
        tracing::error!("Error fetching refresh token {:?}", refresh_token);
        let preferences: State<PreferenceConfig> = app.state();
        let res = preferences.inner().set_secure::<String>(key.into(), None);
        tracing::info!("Set secure token: {:?}", res);
        return Err(format!(
            "Refresh token not found or corrupted: {:?}",
            refresh_token.unwrap_err()
        )
        .into());
    }

    let refresh_token = refresh_token.unwrap();
    if !refresh_token.is_empty() {
        let http_client = reqwest::ClientBuilder::new()
            .redirect(reqwest::redirect::Policy::none())
            .build()
            .expect("Client should build");

        let res = client
            .exchange_refresh_token(&RefreshToken::new(refresh_token.to_string()))
            .request_async(&http_client)
            .await
            .map_err(|err| match err {
                oauth2::RequestTokenError::ServerResponse(e) => MoosyncError::String(format!(
                    "{:?}: {:?} {:?}",
                    e.error(),
                    e.error_description(),
                    serde_json::to_string(&e)
                )),
                oauth2::RequestTokenError::Request(e) => error_helpers::to_network_error(e),
                oauth2::RequestTokenError::Parse(e, _) => MoosyncError::String(format!("Parse error: {e}")),
                oauth2::RequestTokenError::Other(e) => MoosyncError::String(e.to_string()),
            })?;

        return set_tokens(key, app, res, Some(refresh_token.to_string()));
    }

    Err("Refresh token not found".into())
}

#[allow(dead_code)]
pub struct LoginArgs {
    pub client_id: Option<String>,
    pub client_secret: Option<String>,
    pub scopes: Vec<&'static str>,
    pub extra_params: Option<HashMap<&'static str, &'static str>>,
}

#[tracing::instrument(level = "debug", skip(config, client, app))]
pub fn login(
    config: LoginArgs,
    client: OAuth2Client,
    app: &AppHandle,
) -> Result<(String, OAuth2Verifier)> {
    if config.client_id.is_none() {
        return Err("Client ID not set".into());
    }

    let (pkce_challenge, pkce_verifier) = PkceCodeChallenge::new_random_sha256();

    let mut builder = client
        .authorize_url(CsrfToken::new_random)
        .add_scopes(config.scopes.iter().map(|s| Scope::new(s.to_string())))
        .set_pkce_challenge(pkce_challenge);

    if let Some(extra_params) = config.extra_params {
        for (key, value) in extra_params {
            builder = builder.add_extra_param(key, value);
        }
    }

    let (auth_url, csrf_token) = builder.url();

    let verifier = Some((client, pkce_verifier, csrf_token.clone()));
    let window: State<WindowHandler> = app.state();

    tracing::info!("Opening url {:?}", auth_url);
    if let Err(e) = window
        .inner()
        .open_external(app.clone(), auth_url.to_string())
    {
        tracing::error!("Error opening URL: {:?}", e);
    }
    Ok((auth_url.to_string(), verifier))
}

#[tracing::instrument(level = "debug", skip(key, code, verifier, app))]
pub async fn authorize(
    key: &str,
    code: String,
    verifier: OAuth2Verifier,
    app: &AppHandle,
) -> Result<TokenHolder> {
    if verifier.is_none() {
        return Err("OAuth not initiated".into());
    }

    let parsed_code = if code.starts_with("?") {
        Url::parse(format!("https://moosync.app/login{code}").as_str()).unwrap()
    } else {
        Url::parse(code.as_str()).unwrap()
    };
    let code = parsed_code
        .query_pairs()
        .find(|p| p.0 == "code")
        .unwrap()
        .1
        .to_string();

    let (client, verifier, _csrf) = verifier.unwrap();

    let http_client = reqwest::ClientBuilder::new()
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .expect("Client should build");
    let res = client
        .exchange_code(AuthorizationCode::new(code))
        .set_pkce_verifier(verifier)
        .request_async(&http_client)
        .await
        .map_err(|err| match err {
            oauth2::RequestTokenError::ServerResponse(e) => MoosyncError::String(format!(
                "{:?}: {:?} {:?}",
                e.error(),
                e.error_description(),
                serde_json::to_string(&e)
            )),
            oauth2::RequestTokenError::Request(e) => error_helpers::to_network_error(e),
            oauth2::RequestTokenError::Parse(e, _) => MoosyncError::String(format!("Parse error: {e}")),
            oauth2::RequestTokenError::Other(e) => MoosyncError::String(e.to_string()),
        })?;

    set_tokens(key, app, res, None)
}
