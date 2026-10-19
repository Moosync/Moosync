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

use std::{collections::HashMap, fs};

use assertables::{assert_err, assert_ok, assert_some};
use extensions_proto::moosync::types::{HttpRequest, ManifestPermissions};
use tempdir::TempDir;
use tracing_test::traced_test;
use wiremock::{
    Mock, MockServer, ResponseTemplate,
    matchers::{method, path},
};

use crate::context::extism_context::{ExtismContext, execute_single_request};

#[tracing::instrument(level = "debug", skip_all)]
fn make_req(url: &str) -> HttpRequest {
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
    let client = reqwest::Client::new();
    let req = make_req("https://unauthorized.domain.com/api/test");
    let allowed = vec!["api.spotify.com".to_string()];

    let res = execute_single_request(&client, req, Some(&allowed)).await;

    assert_err!(res.as_ref());
    assert!(res.unwrap_err().contains("unauthorized.domain.com"));
}

#[tokio::test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_execute_single_request_invalid_url() {
    let client = reqwest::Client::new();
    let req = make_req("not a valid url");
    let allowed = vec!["*".to_string()];

    let res = execute_single_request(&client, req, Some(&allowed)).await;

    assert_err!(res.as_ref());
    assert!(res.unwrap_err().contains("Invalid URL"));
}

#[tokio::test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_execute_single_request_none_allowed() {
    let client = reqwest::Client::new();
    let req = make_req("https://api.spotify.com/v1");

    let res = execute_single_request(&client, req, None).await;

    assert_err!(res.as_ref());
    assert!(res.unwrap_err().contains("not allowed"));
}

#[tokio::test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_execute_single_request_disallowed_subdomain() {
    let client = reqwest::Client::new();
    let allowed = vec!["*.spotify.com".to_string()];
    let diff_req = make_req("https://notspotify.com/tracks");

    let res = execute_single_request(&client, diff_req, Some(&allowed)).await;

    assert_err!(res.as_ref());
    assert!(res.unwrap_err().contains("notspotify.com"));
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

    let client = reqwest::Client::new();
    let mut headers = HashMap::new();
    headers.insert("content-type".to_string(), "text/plain".to_string());
    let req = HttpRequest {
        url: format!("{}/api/echo", server.uri()),
        method: "POST".to_string(),
        headers,
        body: Some(b"ping data".to_vec()),
        timeout_ms: Some(5000),
    };
    let allowed = vec!["127.0.0.1".to_string()];

    let res = execute_single_request(&client, req, Some(&allowed)).await;

    assert_ok!(res.as_ref());
    let response = res.unwrap();
    assert_eq!(response.status_code, 200);
    assert_eq!(response.body, b"hello server");
}

#[test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_get_allowed_paths() {
    let temp_dir = TempDir::new("test_allowed_paths").unwrap();
    let cache_dir = temp_dir.path().join("cache");
    let existing_sub = cache_dir.join("sub_dir");
    fs::create_dir_all(&existing_sub).unwrap();

    let env_sub = temp_dir.path().join("env_dir");
    fs::create_dir_all(&env_sub).unwrap();
    unsafe {
        std::env::set_var("MOOSYNC_TEST_PATH_VAR", &env_sub);
    }

    let mut paths_map = HashMap::new();
    paths_map.insert(
        format!("{}/sub_dir", "{CACHE_DIR}"),
        "/guest/cache_sub".to_string(),
    );
    paths_map.insert(
        "{MOOSYNC_TEST_PATH_VAR}".to_string(),
        "/guest/env_dir".to_string(),
    );
    paths_map.insert(
        "/non/existent/path/for/sure".to_string(),
        "/guest/invalid".to_string(),
    );

    let permissions = ManifestPermissions {
        paths: paths_map,
        hosts: vec![],
    };

    let allowed = ExtismContext::get_allowed_paths(&permissions, &cache_dir);

    assert_some!(allowed.get(&existing_sub.to_string_lossy().to_string()));
    assert_some!(allowed.get(&env_sub.to_string_lossy().to_string()));
    assert_eq!(allowed.len(), 2);
}
