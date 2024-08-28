use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

use oauth2::reqwest::async_http_client;
use oauth2::{basic::BasicClient, AuthUrl, ClientId, ClientSecret, RedirectUrl, TokenUrl};
use oauth2::{AuthorizationCode, CsrfToken, PkceCodeChallenge, RefreshToken, Scope, TokenResponse};
use preferences::preferences::PreferenceConfig;
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager, State};
use types::errors::{MoosyncError, Result};
use types::oauth::{OAuth2Client, OAuth2Verifier, OAuthTokenResponse};
use url::Url;

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

#[tracing::instrument(level = "trace", skip(config))]
pub fn get_oauth_client(config: OAuthClientArgs) -> OAuth2Client {
    BasicClient::new(
        ClientId::new(config.client_id),
        Some(ClientSecret::new(config.client_secret)),
        AuthUrl::new(config.auth_url).unwrap(),
        Some(TokenUrl::new(config.token_url).unwrap()),
    )
    .set_redirect_uri(RedirectUrl::new(config.redirect_url).unwrap())
}

#[tracing::instrument(level = "trace", skip(key, app, res, default_refresh))]
pub fn set_tokens(
    key: &str,
    app: &AppHandle,
    res: OAuthTokenResponse,
    default_refresh: Option<String>,
) -> Result<TokenHolder> {
    tracing::info!("token response: {:?}", res);
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

#[tracing::instrument(level = "trace", skip(key, client, app))]
pub async fn refresh_login(
    key: &str,
    client: OAuth2Client,
    app: &AppHandle,
) -> Result<TokenHolder> {
    let preferences: State<PreferenceConfig> = app.state();
    let refresh_token: Result<String> = preferences.inner().get_secure(key.into());
    if refresh_token.is_err() {
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
        let res = client
            .exchange_refresh_token(&RefreshToken::new(refresh_token.to_string()))
            .request_async(async_http_client)
            .await
            .map_err(|err| match err {
                oauth2::RequestTokenError::ServerResponse(e) => MoosyncError::String(e.to_string()),
                oauth2::RequestTokenError::Request(_) => todo!(),
                oauth2::RequestTokenError::Parse(_, _) => todo!(),
                oauth2::RequestTokenError::Other(_) => todo!(),
            })?;

        return set_tokens(key, app, res, Some(refresh_token.to_string()));
    }

    Err("Refresh token not found".into())
}

pub struct LoginArgs {
    pub client_id: Option<String>,
    pub client_secret: Option<String>,
    pub scopes: Vec<&'static str>,
    pub extra_params: Option<HashMap<&'static str, &'static str>>,
}

#[tracing::instrument(level = "trace", skip(config, client, app))]
pub fn login(config: LoginArgs, client: OAuth2Client, app: &AppHandle) -> Result<OAuth2Verifier> {
    if config.client_id.is_none() || config.client_secret.is_none() {
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
    if let Err(e) = window.inner().open_external(auth_url.to_string()) {
        tracing::error!("Error opening URL: {:?}", e);
    }
    Ok(verifier)
}

#[tracing::instrument(level = "trace", skip(key, code, verifier, app))]
pub async fn authorize(
    key: &str,
    code: String,
    verifier: &mut OAuth2Verifier,
    app: &AppHandle,
) -> Result<TokenHolder> {
    if verifier.is_none() {
        return Err("OAuth not initiated".into());
    }

    let parsed_code = if code.starts_with("?") {
        Url::parse(format!("https://moosync.app/login{}", code).as_str()).unwrap()
    } else {
        Url::parse(code.as_str()).unwrap()
    };
    let code = parsed_code
        .query_pairs()
        .find(|p| p.0 == "code")
        .unwrap()
        .1
        .to_string();

    let (client, verifier, _csrf) = verifier.take().unwrap();

    let res = client
        .exchange_code(AuthorizationCode::new(code))
        .set_pkce_verifier(verifier)
        .request_async(async_http_client)
        .await
        .map_err(|err| match err {
            oauth2::RequestTokenError::ServerResponse(e) => MoosyncError::String(format!(
                "{:?}: {:?} {:?}",
                e.error(),
                e.error_description(),
                serde_json::to_string(&e)
            )),
            oauth2::RequestTokenError::Request(_) => todo!(),
            oauth2::RequestTokenError::Parse(_, _) => todo!(),
            oauth2::RequestTokenError::Other(_) => todo!(),
        })?;

    set_tokens(key, app, res, None)
}
