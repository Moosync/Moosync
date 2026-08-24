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

use extensions_proto::moosync::types::HttpRequest;

use crate::context::extism_context::execute_single_request;

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
#[tracing::instrument(level = "debug", skip_all)]
async fn test_execute_single_request_disallowed_host() {
    let client = reqwest::Client::new();
    let req = make_req("https://unauthorized.domain.com/api/test");
    let allowed = vec!["api.spotify.com".to_string()];

    let res = execute_single_request(&client, req, Some(&allowed)).await;

    assert!(res.is_err());
    assert!(res.unwrap_err().contains("unauthorized.domain.com"));
}

#[tokio::test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_execute_single_request_invalid_url() {
    let client = reqwest::Client::new();
    let req = make_req("not a valid url");
    let allowed = vec!["*".to_string()];

    let res = execute_single_request(&client, req, Some(&allowed)).await;

    assert!(res.is_err());
    assert!(res.unwrap_err().contains("Invalid URL"));
}

#[tokio::test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_execute_single_request_none_allowed() {
    let client = reqwest::Client::new();
    let req = make_req("https://api.spotify.com/v1");

    let res = execute_single_request(&client, req, None).await;

    assert!(res.is_err());
    assert!(res.unwrap_err().contains("not allowed"));
}

#[tokio::test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_execute_single_request_disallowed_subdomain() {
    let client = reqwest::Client::new();
    let allowed = vec!["*.spotify.com".to_string()];

    let diff_req = make_req("https://notspotify.com/tracks");
    let res = execute_single_request(&client, diff_req, Some(&allowed)).await;

    assert!(res.is_err());
    assert!(res.unwrap_err().contains("notspotify.com"));
}
