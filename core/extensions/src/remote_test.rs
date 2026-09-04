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

use assertables::{assert_err, assert_is_empty, assert_len_eq_x, assert_ok};
use extensions_proto::moosync::types::{ExtensionRegistryManifest, FetchedExtensionManifest};
use prost::Message;
use rstest::{fixture, rstest};
use tempdir::TempDir;
use tracing_test::traced_test;
use wiremock::{
    Mock, MockServer, ResponseTemplate,
    matchers::{method, path},
};

use crate::remote::RemoteExtensions;

struct TestRemoteContext {
    pub _temp_dir: TempDir,
    pub remote: RemoteExtensions,
}

#[fixture]
#[tracing::instrument(level = "debug", skip_all)]
fn remote_context() -> TestRemoteContext {
    let temp_dir = TempDir::new("moosync_remote_test").expect("failed to create temp dir");
    let tmp_dir = temp_dir.path().join("tmp");
    fs::create_dir_all(&tmp_dir).unwrap();

    let remote = RemoteExtensions::new(tmp_dir);
    TestRemoteContext {
        _temp_dir: temp_dir,
        remote,
    }
}

#[rstest]
#[tokio::test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_fetch_registry_success_and_caching(remote_context: TestRemoteContext) {
    let server = MockServer::start().await;
    let manifest = ExtensionRegistryManifest {
        name: "test_community_registry".to_string(),
        extensions: vec![FetchedExtensionManifest {
            name: "Discord Integration".to_string(),
            package_name: "org.test.discord".to_string(),
            logo: Some("assets/discord.svg".to_string()),
            description: Some("Rich presence for discord".to_string()),
            url: "discord.msox".to_string(),
            version: "1.2.3".to_string(),
            registry: None,
        }],
    };

    Mock::given(method("GET"))
        .and(path("/manifest.pb"))
        .respond_with(ResponseTemplate::new(200).set_body_bytes(manifest.encode_to_vec()))
        .mount(&server)
        .await;

    let TestRemoteContext {
        remote, _temp_dir, ..
    } = remote_context;
    let mut registries = HashSet::new();
    let registry_url = format!("{}/manifest.pb", server.uri());
    registries.insert(registry_url.clone());

    let results = remote.get_extension_manifest(&registries).await;

    assert_ok!(results.as_ref());
    let results = results.unwrap();
    assert_len_eq_x!(&results, 1);

    let item = results
        .iter()
        .find(|i| i.package_name == "org.test.discord")
        .unwrap();
    assert_eq!(item.name, "Discord Integration");
    assert_eq!(item.version, "1.2.3");
    assert_eq!(item.registry, Some("test_community_registry".to_string()));
    assert_eq!(item.url, format!("{}/discord.msox", server.uri()));
    assert_eq!(
        item.logo,
        Some(format!("{}/assets/discord.svg", server.uri()))
    );
}

#[rstest]
#[tokio::test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_fetch_registry_fallback_to_name(remote_context: TestRemoteContext) {
    let server = MockServer::start().await;
    let manifest = ExtensionRegistryManifest {
        name: "raw_registry_name".to_string(),
        extensions: vec![FetchedExtensionManifest {
            name: "Ext".to_string(),
            package_name: "org.test.ext".to_string(),
            logo: None,
            description: None,
            url: "ext.msox".to_string(),
            version: "1.0.0".to_string(),
            registry: None,
        }],
    };

    Mock::given(method("GET"))
        .and(path("/fallback.pb"))
        .respond_with(ResponseTemplate::new(200).set_body_bytes(manifest.encode_to_vec()))
        .mount(&server)
        .await;

    let TestRemoteContext { remote, .. } = remote_context;
    let mut registries = HashSet::new();
    registries.insert(format!("{}/fallback.pb", server.uri()));

    let results = remote.get_extension_manifest(&registries).await;

    assert_ok!(results.as_ref());
    let results = results.unwrap();
    assert_len_eq_x!(&results, 1);
    let item = results.into_iter().next().unwrap();
    assert_eq!(item.registry, Some("raw_registry_name".to_string()));
}

#[rstest]
#[tokio::test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_fetch_registry_rejects_missing_version(remote_context: TestRemoteContext) {
    let server = MockServer::start().await;
    let manifest = ExtensionRegistryManifest {
        name: "Verified Registry".to_string(),
        extensions: vec![
            FetchedExtensionManifest {
                name: "Valid Ext".to_string(),
                package_name: "valid.ext".to_string(),
                logo: None,
                description: None,
                url: "valid.msox".to_string(),
                version: "2.0.0".to_string(),
                registry: None,
            },
            FetchedExtensionManifest {
                name: "No Version Ext".to_string(),
                package_name: "invalid.ext".to_string(),
                logo: None,
                description: None,
                url: "invalid.msox".to_string(),
                version: "".to_string(),
                registry: None,
            },
        ],
    };

    Mock::given(method("GET"))
        .and(path("/version_check.pb"))
        .respond_with(ResponseTemplate::new(200).set_body_bytes(manifest.encode_to_vec()))
        .mount(&server)
        .await;

    let TestRemoteContext { remote, .. } = remote_context;
    let mut registries = HashSet::new();
    registries.insert(format!("{}/version_check.pb", server.uri()));

    let results = remote.get_extension_manifest(&registries).await;

    assert_ok!(results.as_ref());
    let results = results.unwrap();
    assert_len_eq_x!(&results, 1);
    let item = results.into_iter().next().unwrap();
    assert_eq!(item.package_name, "valid.ext");
}

#[rstest]
#[tokio::test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_fetch_multiple_registries_and_download(remote_context: TestRemoteContext) {
    let server = MockServer::start().await;
    let reg1 = ExtensionRegistryManifest {
        name: "reg1".to_string(),
        extensions: vec![FetchedExtensionManifest {
            name: "Extension One".to_string(),
            package_name: "ext.one".to_string(),
            logo: None,
            description: None,
            url: "pkg1.msox".to_string(),
            version: "1.0.0".to_string(),
            registry: None,
        }],
    };
    let reg2 = ExtensionRegistryManifest {
        name: "reg2".to_string(),
        extensions: vec![FetchedExtensionManifest {
            name: "Extension Two".to_string(),
            package_name: "ext.two".to_string(),
            logo: None,
            description: None,
            url: "pkg2.msox".to_string(),
            version: "1.0.0".to_string(),
            registry: None,
        }],
    };

    Mock::given(method("GET"))
        .and(path("/reg1.pb"))
        .respond_with(ResponseTemplate::new(200).set_body_bytes(reg1.encode_to_vec()))
        .mount(&server)
        .await;

    Mock::given(method("GET"))
        .and(path("/reg2.pb"))
        .respond_with(ResponseTemplate::new(200).set_body_bytes(reg2.encode_to_vec()))
        .mount(&server)
        .await;

    let dummy_pkg = b"dummy zip payload";
    Mock::given(method("GET"))
        .and(path("/pkg1.msox"))
        .respond_with(ResponseTemplate::new(200).set_body_bytes(dummy_pkg.to_vec()))
        .mount(&server)
        .await;

    let TestRemoteContext { remote, .. } = remote_context;
    let mut registries = HashSet::new();
    registries.insert(format!("{}/reg1.pb", server.uri()));
    registries.insert(format!("{}/reg2.pb", server.uri()));

    let results = remote.get_extension_manifest(&registries).await;
    assert_ok!(results.as_ref());
    let results = results.unwrap();
    assert_len_eq_x!(&results, 2);

    let ext_one = results
        .iter()
        .find(|i| i.package_name == "ext.one")
        .unwrap();
    let downloaded_path = remote.download_extension(ext_one.clone()).await;

    assert_ok!(downloaded_path.as_ref());
    let downloaded_path = downloaded_path.unwrap();
    assert!(downloaded_path.exists());
    assert_eq!(fs::read(downloaded_path).unwrap(), dummy_pkg);
}

#[rstest]
#[tokio::test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_fetch_registry_malformed_protobuf(remote_context: TestRemoteContext) {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/malformed.pb"))
        .respond_with(ResponseTemplate::new(200).set_body_bytes(vec![0xFF, 0xFF, 0xFF]))
        .mount(&server)
        .await;

    let TestRemoteContext { remote, .. } = remote_context;
    let mut registries = HashSet::new();
    registries.insert(format!("{}/malformed.pb", server.uri()));

    let results = remote.get_extension_manifest(&registries).await;

    assert_ok!(results.as_ref());
    assert_is_empty!(&results.unwrap());
}

#[rstest]
#[tokio::test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_fetch_registry_http_404_not_found(remote_context: TestRemoteContext) {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/not_found.pb"))
        .respond_with(ResponseTemplate::new(404))
        .mount(&server)
        .await;

    let TestRemoteContext { remote, .. } = remote_context;
    let mut registries = HashSet::new();
    registries.insert(format!("{}/not_found.pb", server.uri()));

    let results = remote.get_extension_manifest(&registries).await;

    assert_ok!(results.as_ref());
    assert_is_empty!(&results.unwrap());
}

#[rstest]
#[tokio::test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_fetch_registry_http_500_server_error(remote_context: TestRemoteContext) {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/server_error.pb"))
        .respond_with(ResponseTemplate::new(500).set_body_string("Internal Server Error"))
        .mount(&server)
        .await;

    let TestRemoteContext { remote, .. } = remote_context;
    let mut registries = HashSet::new();
    registries.insert(format!("{}/server_error.pb", server.uri()));

    let results = remote.get_extension_manifest(&registries).await;

    assert_ok!(results.as_ref());
    assert_is_empty!(&results.unwrap());
}

#[rstest]
#[tokio::test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_fetch_registry_network_connection_error(remote_context: TestRemoteContext) {
    let TestRemoteContext { remote, .. } = remote_context;
    let mut registries = HashSet::new();
    registries.insert("http://127.0.0.1:1/non_existent.pb".to_string());

    let results = remote.get_extension_manifest(&registries).await;

    assert_ok!(results.as_ref());
    assert_is_empty!(&results.unwrap());
}

#[rstest]
#[tokio::test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_download_extension_http_404_error(remote_context: TestRemoteContext) {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/broken_pkg.msox"))
        .respond_with(ResponseTemplate::new(404))
        .mount(&server)
        .await;

    let TestRemoteContext { remote, .. } = remote_context;
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

    assert_err!(result.as_ref());
}

#[rstest]
#[tokio::test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_download_extension_http_500_error(remote_context: TestRemoteContext) {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/crash_pkg.msox"))
        .respond_with(ResponseTemplate::new(500).set_body_string("Server Crashed"))
        .mount(&server)
        .await;

    let TestRemoteContext { remote, .. } = remote_context;
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

    assert_err!(result.as_ref());
}

#[rstest]
#[tokio::test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_fetch_registry_with_mixed_and_broken_extensions(remote_context: TestRemoteContext) {
    let server = MockServer::start().await;

    let manifest = ExtensionRegistryManifest {
        name: "fake_server".to_string(),
        extensions: vec![
            FetchedExtensionManifest {
                name: "Fake Spotify Integration".to_string(),
                package_name: "app.fake.spotify".to_string(),
                logo: Some("icons/spotify_logo.png".to_string()),
                description: Some("Stream metadata and controls from fake spotify".to_string()),
                url: "downloads/spotify.msox".to_string(),
                version: "2.1.0".to_string(),
                registry: None,
            },
            FetchedExtensionManifest {
                name: "Broken Missing Version".to_string(),
                package_name: "app.fake.broken_version".to_string(),
                logo: None,
                description: None,
                url: "broken.msox".to_string(),
                version: "".to_string(),
                registry: None,
            },
            FetchedExtensionManifest {
                name: "Absolute URLs Extension".to_string(),
                package_name: "app.fake.absolute_urls".to_string(),
                logo: Some("https://cdn.example.com/icon.svg".to_string()),
                description: None,
                url: "https://cdn.example.com/download.msox".to_string(),
                version: "3.0.0".to_string(),
                registry: None,
            },
        ],
    };

    Mock::given(method("GET"))
        .and(path("/manifest.pb"))
        .respond_with(ResponseTemplate::new(200).set_body_bytes(manifest.encode_to_vec()))
        .mount(&server)
        .await;

    let TestRemoteContext { remote, .. } = remote_context;
    let mut registries = HashSet::new();
    registries.insert(format!("{}/manifest.pb", server.uri()));

    let results = remote.get_extension_manifest(&registries).await;

    assert_ok!(results.as_ref());
    let results = results.unwrap();
    assert_len_eq_x!(&results, 2);

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
