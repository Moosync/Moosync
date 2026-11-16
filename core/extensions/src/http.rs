use std::{collections::HashMap, time::Duration};

use extensions_proto::moosync::types::{
    BatchHttpRequest, BatchHttpResponse, HttpRequest, HttpResponse, HttpResult, http_result,
};
use futures::future::join_all;
use reqwest::{Client, Method, Url};
use tracing::Instrument;

use crate::errors::ExtensionError;

#[tracing::instrument(level = "trace", skip_all)]
pub(crate) fn is_host_allowed(host: &str, allowed_hosts: Option<&[String]>) -> bool {
    let Some(allowed) = allowed_hosts else {
        return false;
    };
    let host_lower = host.to_ascii_lowercase();
    for pattern in allowed {
        let pattern_lower = pattern.to_ascii_lowercase();
        if pattern_lower == "*" {
            return true;
        }
        if let Some(bare) = pattern_lower.strip_prefix("*.") {
            let suffix = &pattern_lower[1..];
            if host_lower.ends_with(suffix) || host_lower == bare {
                return true;
            }
        }
        if host_lower == pattern_lower {
            return true;
        }
    }
    false
}

#[tracing::instrument(level = "trace", skip_all)]
fn validate_request(request: &HttpRequest, allowed_hosts: Option<&[String]>) -> Result<(), String> {
    let url_parsed = Url::parse(&request.url).map_err(|error| format!("Invalid URL: {error}"))?;
    let host = url_parsed
        .host_str()
        .ok_or_else(|| "URL has no host".to_string())?;
    if !is_host_allowed(host, allowed_hosts) {
        return Err(format!(
            "Host '{host}' is not allowed by manifest permissions"
        ));
    }
    Ok(())
}

#[tracing::instrument(level = "debug", skip_all)]
pub(crate) async fn execute_single_request(
    client: &Client,
    request: HttpRequest,
    allowed_hosts: Option<&[String]>,
) -> Result<HttpResponse, String> {
    validate_request(&request, allowed_hosts)?;
    let url_parsed = Url::parse(&request.url).map_err(|error| format!("Invalid URL: {error}"))?;

    let method = match request.method.to_uppercase().as_str() {
        "POST" => Method::POST,
        "PUT" => Method::PUT,
        "DELETE" => Method::DELETE,
        "PATCH" => Method::PATCH,
        "HEAD" => Method::HEAD,
        "OPTIONS" => Method::OPTIONS,
        _ => Method::GET,
    };

    let mut builder = client.request(method, url_parsed);
    for (header_key, header_val) in request.headers {
        builder = builder.header(header_key, header_val);
    }

    if let Some(body) = request.body {
        builder = builder.body(body);
    }

    if let Some(timeout_ms) = request.timeout_ms
        && timeout_ms > 0
    {
        builder = builder.timeout(Duration::from_millis(timeout_ms));
    }

    let response = builder
        .send()
        .await
        .map_err(|error| format!("Network request failed: {error}"))?;

    let status_code = response.status().as_u16() as u32;
    let status_text = response
        .status()
        .canonical_reason()
        .unwrap_or("")
        .to_string();
    let mut headers = HashMap::new();
    for (header_key, header_val) in response.headers() {
        if let Ok(val_str) = header_val.to_str() {
            headers.insert(header_key.as_str().to_string(), val_str.to_string());
        }
    }

    let body_bytes = response
        .bytes()
        .await
        .map_err(|error| format!("Failed to read response body: {error}"))?;

    Ok(HttpResponse {
        status_code,
        status_text,
        headers,
        body: body_bytes.to_vec(),
    })
}

pub struct HttpManager {
    allowed_hosts: Option<Vec<String>>,
    client: Client,
}

impl HttpManager {
    #[tracing::instrument(level = "debug", skip_all)]
    pub fn new(allowed_hosts: Option<Vec<String>>) -> Self {
        let client = Client::builder()
            .pool_max_idle_per_host(10)
            .build()
            .unwrap_or_else(|error| {
                tracing::error!("Failed to build custom reqwest client: {}", error);
                Client::new()
            });
        Self {
            allowed_hosts,
            client,
        }
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub async fn batch_http_request(
        &self,
        request: BatchHttpRequest,
    ) -> Result<BatchHttpResponse, ExtensionError> {
        let requests = request.requests;

        for (index, item) in requests.iter().enumerate() {
            if let Err(error) = validate_request(item, self.allowed_hosts.as_deref()) {
                return Ok(BatchHttpResponse {
                    responses: Vec::new(),
                    error: Some(format!("Request #{index} invalid: {error}")),
                });
            }
        }

        let futures = requests
            .into_iter()
            .map(|item| execute_single_request(&self.client, item, self.allowed_hosts.as_deref()));
        let results = join_all(futures).in_current_span().await;
        let responses = results
            .into_iter()
            .map(|result| match result {
                Ok(response) => HttpResult {
                    result: Some(http_result::Result::Response(response)),
                },
                Err(error) => HttpResult {
                    result: Some(http_result::Result::Error(error)),
                },
            })
            .collect();

        Ok(BatchHttpResponse {
            responses,
            error: None,
        })
    }
}
