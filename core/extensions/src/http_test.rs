use std::collections::HashMap;

use assertables::{assert_err, assert_ok, assert_some};
use extensions_proto::moosync::types::{BatchHttpRequest, HttpRequest, http_result};
use reqwest::Client;
use tracing_test::traced_test;
use wiremock::{
    Mock, MockServer, ResponseTemplate,
    matchers::{method, path},
};

use crate::http::{HttpManager, execute_single_request};

#[tracing::instrument(level = "debug", skip_all)]
fn make_request(url: &str) -> HttpRequest {
    HttpRequest {
        url: url.to_string(),
        method: "GET".to_string(),
        headers: HashMap::new(),
        body: None,
        timeout_ms: None,
    }
}

#[tokio::test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_execute_single_request_disallowed_host() {
    let client = Client::new();
    let request = make_request("https://unauthorized.domain.com/api/test");
    let allowed = vec!["api.spotify.com".to_string()];

    let result = execute_single_request(&client, request, Some(&allowed)).await;

    assert_err!(result.as_ref());
    assert!(result.unwrap_err().contains("unauthorized.domain.com"));
}

#[tokio::test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_execute_single_request_invalid_url() {
    let client = Client::new();
    let request = make_request("not a valid url");
    let allowed = vec!["*".to_string()];

    let result = execute_single_request(&client, request, Some(&allowed)).await;

    assert_err!(result.as_ref());
    assert!(result.unwrap_err().contains("Invalid URL"));
}

#[tokio::test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_execute_single_request_none_allowed() {
    let client = Client::new();
    let request = make_request("https://api.spotify.com/v1");

    let result = execute_single_request(&client, request, None).await;

    assert_err!(result.as_ref());
    assert!(result.unwrap_err().contains("not allowed"));
}

#[tokio::test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_execute_single_request_disallowed_subdomain() {
    let client = Client::new();
    let allowed = vec!["*.spotify.com".to_string()];
    let request = make_request("https://notspotify.com/tracks");

    let result = execute_single_request(&client, request, Some(&allowed)).await;

    assert_err!(result.as_ref());
    assert!(result.unwrap_err().contains("notspotify.com"));
}

#[tokio::test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_execute_single_request_success_with_wiremock() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/api/echo"))
        .respond_with(ResponseTemplate::new(200).set_body_bytes(b"hello server".to_vec()))
        .mount(&server)
        .await;
    let client = Client::new();
    let mut headers = HashMap::new();
    headers.insert("content-type".to_string(), "text/plain".to_string());
    let request = HttpRequest {
        url: format!("{}/api/echo", server.uri()),
        method: "POST".to_string(),
        headers,
        body: Some(b"ping data".to_vec()),
        timeout_ms: Some(5000),
    };
    let allowed = vec!["127.0.0.1".to_string()];

    let result = execute_single_request(&client, request, Some(&allowed)).await;

    assert_ok!(result.as_ref());
    let response = result.unwrap();
    assert_eq!(response.status_code, 200);
    assert_eq!(response.body, b"hello server");
}

#[tokio::test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_http_manager_batch_http_request() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/api/items"))
        .respond_with(ResponseTemplate::new(200).set_body_bytes(b"items_body".to_vec()))
        .mount(&server)
        .await;
    let manager = HttpManager::new(Some(vec!["127.0.0.1".to_string()]));
    let batch_request = BatchHttpRequest {
        requests: vec![make_request(&format!("{}/api/items", server.uri()))],
    };

    let result = manager.batch_http_request(batch_request).await;

    assert_ok!(result.as_ref());
    let response = result.unwrap();
    assert!(response.error.is_none());
    assert_eq!(response.responses.len(), 1);
    let first = response.responses.into_iter().next().unwrap();
    assert_some!(first.result.as_ref());
    match first.result.unwrap() {
        http_result::Result::Response(http_response) => {
            assert_eq!(http_response.status_code, 200);
            assert_eq!(http_response.body, b"items_body");
        }
        other => panic!("Unexpected HTTP result: {:?}", other),
    }
}
