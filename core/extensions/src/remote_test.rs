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

use std::{collections::HashSet, fs};

use extensions_proto::moosync::types::FetchedExtensionManifest;
use tempdir::TempDir;
use wiremock::{
    Mock, MockServer, ResponseTemplate,
    matchers::{method, path},
};

use crate::remote::RemoteExtensions;

#[tokio::test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_fetch_registry_success_and_caching() {
    let server = MockServer::start().await;
    let manifest_body = serde_json::json!({
        "displayName": "Test Community Registry",
        "extensions": {
            "org.test.discord": {
                "displayName": "Discord Integration",
                "version": "1.2.3",
                "icon": "assets/discord.svg",
                "url": "discord.msox",
                "description": "Rich presence for discord"
            }
        }
    });

    Mock::given(method("GET"))
        .and(path("/manifest.json"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&manifest_body))
        .mount(&server)
        .await;

    let tmp = TempDir::new("test_remote_success").unwrap();
    let ext_dir = tmp.path().join("exts");
    let tmp_dir = tmp.path().join("tmp");
    let cache_dir = tmp.path().join("cache");
    fs::create_dir_all(&ext_dir).unwrap();
    fs::create_dir_all(&tmp_dir).unwrap();
    fs::create_dir_all(&cache_dir).unwrap();

    let remote = RemoteExtensions::new(ext_dir, tmp_dir, cache_dir.clone());
    let mut registries = HashSet::new();
    let registry_url = format!("{}/manifest.json", server.uri());
    registries.insert(registry_url.clone());

    let results = remote.get_extension_manifest(&registries).await.unwrap();
    assert_eq!(results.len(), 1);

    let item = results
        .iter()
        .find(|i| i.package_name == "org.test.discord")
        .unwrap();
    assert_eq!(item.name, "Discord Integration");
    assert_eq!(item.version, "1.2.3");
    assert_eq!(item.registry, Some("Test Community Registry".to_string()));
    assert_eq!(item.url, format!("{}/discord.msox", server.uri()));
    assert_eq!(
        item.logo,
        Some(format!("{}/assets/discord.svg", server.uri()))
    );

    assert!(cache_dir.join("remote_manifest_cache.json").exists());
}

#[tokio::test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_fetch_registry_rejects_missing_name() {
    let server = MockServer::start().await;
    let manifest_no_name = serde_json::json!({
        "extensions": {
            "org.test.ext": {
                "displayName": "Ext",
                "version": "1.0.0"
            }
        }
    });

    Mock::given(method("GET"))
        .and(path("/no_name.json"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&manifest_no_name))
        .mount(&server)
        .await;

    let tmp = TempDir::new("test_remote_no_name").unwrap();
    let remote = RemoteExtensions::new(
        tmp.path().join("exts"),
        tmp.path().join("tmp"),
        tmp.path().join("cache"),
    );

    let mut registries = HashSet::new();
    registries.insert(format!("{}/no_name.json", server.uri()));

    let results = remote.get_extension_manifest(&registries).await.unwrap();
    assert!(results.is_empty());
}

#[tokio::test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_fetch_registry_rejects_missing_version() {
    let server = MockServer::start().await;
    let manifest_body = serde_json::json!({
        "name": "Verified Registry",
        "extensions": {
            "valid.ext": {
                "displayName": "Valid Ext",
                "version": "2.0.0"
            },
            "invalid.ext": {
                "displayName": "No Version Ext"
            }
        }
    });

    Mock::given(method("GET"))
        .and(path("/version_check.json"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&manifest_body))
        .mount(&server)
        .await;

    let tmp = TempDir::new("test_remote_version").unwrap();
    let remote = RemoteExtensions::new(
        tmp.path().join("exts"),
        tmp.path().join("tmp"),
        tmp.path().join("cache"),
    );

    let mut registries = HashSet::new();
    registries.insert(format!("{}/version_check.json", server.uri()));

    let results = remote.get_extension_manifest(&registries).await.unwrap();
    assert_eq!(results.len(), 1);
    let item = results.into_iter().next().unwrap();
    assert_eq!(item.package_name, "valid.ext");
}

#[tokio::test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_fetch_multiple_registries_and_download() {
    let server = MockServer::start().await;
    let reg1_body = serde_json::json!({
        "displayName": "Registry 1",
        "extensions": {
            "ext.one": {
                "displayName": "Extension One",
                "version": "1.0.0",
                "url": "pkg1.msox"
            }
        }
    });
    let reg2_body = serde_json::json!({
        "displayName": "Registry 2",
        "extensions": {
            "ext.two": {
                "displayName": "Extension Two",
                "version": "1.0.0",
                "url": "pkg2.msox"
            }
        }
    });

    Mock::given(method("GET"))
        .and(path("/reg1.json"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&reg1_body))
        .mount(&server)
        .await;

    Mock::given(method("GET"))
        .and(path("/reg2.json"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&reg2_body))
        .mount(&server)
        .await;

    let dummy_pkg = b"dummy zip payload";
    Mock::given(method("GET"))
        .and(path("/pkg1.msox"))
        .respond_with(ResponseTemplate::new(200).set_body_bytes(dummy_pkg.to_vec()))
        .mount(&server)
        .await;

    let tmp = TempDir::new("test_remote_multi").unwrap();
    let ext_dir = tmp.path().join("exts");
    let tmp_dir = tmp.path().join("tmp");
    let cache_dir = tmp.path().join("cache");
    fs::create_dir_all(&ext_dir).unwrap();
    fs::create_dir_all(&tmp_dir).unwrap();
    fs::create_dir_all(&cache_dir).unwrap();

    let remote = RemoteExtensions::new(ext_dir, tmp_dir, cache_dir);
    let mut registries = HashSet::new();
    registries.insert(format!("{}/reg1.json", server.uri()));
    registries.insert(format!("{}/reg2.json", server.uri()));

    let results = remote.get_extension_manifest(&registries).await.unwrap();
    assert_eq!(results.len(), 2);

    let ext_one = results
        .iter()
        .find(|i| i.package_name == "ext.one")
        .unwrap();
    let downloaded_path = remote.download_extension(ext_one.clone()).await.unwrap();

    assert!(downloaded_path.exists());
    assert_eq!(fs::read(downloaded_path).unwrap(), dummy_pkg);
}

#[tokio::test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_fetch_registry_malformed_json() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/malformed.json"))
        .respond_with(ResponseTemplate::new(200).set_body_string("{ this is not valid json {"))
        .mount(&server)
        .await;

    let tmp = TempDir::new("test_remote_malformed").unwrap();
    let remote = RemoteExtensions::new(
        tmp.path().join("exts"),
        tmp.path().join("tmp"),
        tmp.path().join("cache"),
    );

    let mut registries = HashSet::new();
    registries.insert(format!("{}/malformed.json", server.uri()));

    let results = remote.get_extension_manifest(&registries).await.unwrap();

    assert!(results.is_empty());
}

#[tokio::test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_fetch_registry_http_404_not_found() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/not_found.json"))
        .respond_with(ResponseTemplate::new(404))
        .mount(&server)
        .await;

    let tmp = TempDir::new("test_remote_404").unwrap();
    let remote = RemoteExtensions::new(
        tmp.path().join("exts"),
        tmp.path().join("tmp"),
        tmp.path().join("cache"),
    );

    let mut registries = HashSet::new();
    registries.insert(format!("{}/not_found.json", server.uri()));

    let results = remote.get_extension_manifest(&registries).await.unwrap();

    assert!(results.is_empty());
}

#[tokio::test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_fetch_registry_http_500_server_error() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/server_error.json"))
        .respond_with(ResponseTemplate::new(500).set_body_string("Internal Server Error"))
        .mount(&server)
        .await;

    let tmp = TempDir::new("test_remote_500").unwrap();
    let remote = RemoteExtensions::new(
        tmp.path().join("exts"),
        tmp.path().join("tmp"),
        tmp.path().join("cache"),
    );

    let mut registries = HashSet::new();
    registries.insert(format!("{}/server_error.json", server.uri()));

    let results = remote.get_extension_manifest(&registries).await.unwrap();

    assert!(results.is_empty());
}

#[tokio::test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_fetch_registry_network_connection_error() {
    let tmp = TempDir::new("test_remote_unreachable").unwrap();
    let remote = RemoteExtensions::new(
        tmp.path().join("exts"),
        tmp.path().join("tmp"),
        tmp.path().join("cache"),
    );

    let mut registries = HashSet::new();
    registries.insert("http://127.0.0.1:1/non_existent.json".to_string());

    let results = remote.get_extension_manifest(&registries).await.unwrap();

    assert!(results.is_empty());
}

#[tokio::test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_download_extension_http_404_error() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/broken_pkg.msox"))
        .respond_with(ResponseTemplate::new(404))
        .mount(&server)
        .await;

    let tmp = TempDir::new("test_download_404").unwrap();
    let tmp_dir = tmp.path().join("tmp");
    fs::create_dir_all(&tmp_dir).unwrap();
    let remote = RemoteExtensions::new(tmp.path().join("exts"), tmp_dir, tmp.path().join("cache"));

    let fake_ext = FetchedExtensionManifest {
        name: "Broken Extension".to_string(),
        package_name: "broken.ext".to_string(),
        logo: None,
        description: Some("Has a broken 404 download link".to_string()),
        url: format!("{}/broken_pkg.msox", server.uri()),
        version: "1.0.0".to_string(),
        registry: Some("Test Registry".to_string()),
    };

    let result = remote.download_extension(fake_ext).await;

    assert!(result.is_err());
}

#[tokio::test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_download_extension_http_500_error() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/crash_pkg.msox"))
        .respond_with(ResponseTemplate::new(500).set_body_string("Server Crashed"))
        .mount(&server)
        .await;

    let tmp = TempDir::new("test_download_500").unwrap();
    let tmp_dir = tmp.path().join("tmp");
    fs::create_dir_all(&tmp_dir).unwrap();
    let remote = RemoteExtensions::new(tmp.path().join("exts"), tmp_dir, tmp.path().join("cache"));

    let fake_ext = FetchedExtensionManifest {
        name: "Crash Extension".to_string(),
        package_name: "crash.ext".to_string(),
        logo: None,
        description: Some("Fails with 500 error".to_string()),
        url: format!("{}/crash_pkg.msox", server.uri()),
        version: "1.0.0".to_string(),
        registry: Some("Test Registry".to_string()),
    };

    let result = remote.download_extension(fake_ext).await;

    assert!(result.is_err());
}

#[tokio::test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_fetch_registry_with_mixed_and_broken_extensions() {
    let server = MockServer::start().await;

    // Fake manifest with a healthy extension, an extension with missing version,
    // and one with absolute url
    let manifest_body = serde_json::json!({
        "displayName": "Mock Fake Server Registry",
        "name": "fake_server",
        "extensions": {
            "app.fake.spotify": {
                "displayName": "Fake Spotify Integration",
                "name": "spotify_plugin",
                "version": "2.1.0",
                "desc": "Stream metadata and controls from fake spotify",
                "logo": "icons/spotify_logo.png",
                "downloadUrl": "downloads/spotify.msox"
            },
            "app.fake.broken_version": {
                "displayName": "Broken Missing Version",
                "url": "broken.msox"
            },
            "app.fake.absolute_urls": {
                "name": "Absolute URLs Extension",
                "version": "3.0.0",
                "icon": "https://cdn.example.com/icon.svg",
                "url": "https://cdn.example.com/download.msox"
            }
        }
    });

    Mock::given(method("GET"))
        .and(path("/manifest.json"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&manifest_body))
        .mount(&server)
        .await;

    let tmp = TempDir::new("test_fake_server").unwrap();
    let remote = RemoteExtensions::new(
        tmp.path().join("exts"),
        tmp.path().join("tmp"),
        tmp.path().join("cache"),
    );

    let mut registries = HashSet::new();
    registries.insert(format!("{}/manifest.json", server.uri()));

    let results = remote.get_extension_manifest(&registries).await.unwrap();

    assert_eq!(results.len(), 2);

    let spotify = results
        .iter()
        .find(|i| i.package_name == "app.fake.spotify")
        .unwrap();
    assert_eq!(spotify.name, "Fake Spotify Integration");
    assert_eq!(spotify.version, "2.1.0");
    assert_eq!(
        spotify.description,
        Some("Stream metadata and controls from fake spotify".to_string())
    );
    assert_eq!(
        spotify.logo,
        Some(format!("{}/icons/spotify_logo.png", server.uri()))
    );
    assert_eq!(
        spotify.url,
        format!("{}/downloads/spotify.msox", server.uri())
    );

    let absolute = results
        .iter()
        .find(|i| i.package_name == "app.fake.absolute_urls")
        .unwrap();
    assert_eq!(absolute.name, "Absolute URLs Extension");
    assert_eq!(
        absolute.logo,
        Some("https://cdn.example.com/icon.svg".to_string())
    );
    assert_eq!(absolute.url, "https://cdn.example.com/download.msox");
}
