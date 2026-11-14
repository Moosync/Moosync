use std::{collections::HashSet, fs};

use assertables::{assert_err, assert_is_empty, assert_len_eq_x, assert_ok};
use prost::Message;
use rstest::{fixture, rstest};
use tempdir::TempDir;
use themes_proto::moosync::types::{ThemeItem, ThemeManifest, ThemeRegistryManifest};
use tracing_test::traced_test;
use wiremock::{
    Mock, MockServer, ResponseTemplate,
    matchers::{method, path},
};

use crate::remote::RemoteThemes;

struct TestRemoteThemeContext {
    pub _temp_dir: TempDir,
    pub remote: RemoteThemes,
}

#[fixture]
#[tracing::instrument(level = "debug", skip_all)]
fn remote_context() -> TestRemoteThemeContext {
    let temp_dir = TempDir::new("moosync_theme_remote_test").expect("failed to create temp dir");
    let tmp_dir = temp_dir.path().join("tmp");
    fs::create_dir_all(&tmp_dir).unwrap();

    let remote = RemoteThemes::new(tmp_dir);
    TestRemoteThemeContext {
        _temp_dir: temp_dir,
        remote,
    }
}

#[rstest]
#[tokio::test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_fetch_theme_registry_success(remote_context: TestRemoteThemeContext) {
    let server = MockServer::start().await;
    let manifest = ThemeRegistryManifest {
        name: "test_themes_registry".to_string(),
        themes: vec![ThemeManifest {
            id: "gruvbox_theme".to_string(),
            name: "Gruvbox Dark".to_string(),
            author: Some("Tester".to_string()),
            description: Some("Retro groove".to_string()),
            url: Some("gruvbox.mstx".to_string()),
            registry: None,
            theme: Some(ThemeItem {
                primary: "#282828".to_string(),
                accent: "#b8bb26".to_string(),
                ..Default::default()
            }),
        }],
    };

    Mock::given(method("GET"))
        .and(path("/manifest.pb"))
        .respond_with(ResponseTemplate::new(200).set_body_bytes(manifest.encode_to_vec()))
        .mount(&server)
        .await;

    let TestRemoteThemeContext { remote, .. } = remote_context;
    let mut registries = HashSet::new();
    let registry_url = format!("{}/manifest.pb", server.uri());
    registries.insert(registry_url.clone());

    let results = remote.get_theme_manifests(&registries).await;
    assert_ok!(results.as_ref());
    let results = results.unwrap();
    assert_len_eq_x!(&results, 1);

    let item = &results[0];
    assert_eq!(item.id, "gruvbox_theme");
    assert_eq!(item.name, "Gruvbox Dark");
    assert_eq!(item.registry, Some("test_themes_registry".to_string()));
    assert_eq!(
        item.url.as_deref(),
        Some(format!("{}/gruvbox.mstx", server.uri()).as_str())
    );
}

#[rstest]
#[tokio::test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_fetch_theme_registry_malformed(remote_context: TestRemoteThemeContext) {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/malformed.pb"))
        .respond_with(ResponseTemplate::new(200).set_body_bytes(vec![0xff, 0xff, 0xff]))
        .mount(&server)
        .await;

    let TestRemoteThemeContext { remote, .. } = remote_context;
    let mut registries = HashSet::new();
    registries.insert(format!("{}/malformed.pb", server.uri()));

    let results = remote.get_theme_manifests(&registries).await;
    assert_ok!(results.as_ref());
    assert_is_empty!(&results.unwrap());
}

#[rstest]
#[tokio::test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_fetch_theme_registry_not_found(remote_context: TestRemoteThemeContext) {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/not_found.pb"))
        .respond_with(ResponseTemplate::new(404))
        .mount(&server)
        .await;

    let TestRemoteThemeContext { remote, .. } = remote_context;
    let mut registries = HashSet::new();
    registries.insert(format!("{}/not_found.pb", server.uri()));

    let results = remote.get_theme_manifests(&registries).await;
    assert_ok!(results.as_ref());
    assert_is_empty!(&results.unwrap());
}

#[rstest]
#[tokio::test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_download_theme_file_success(remote_context: TestRemoteThemeContext) {
    let server = MockServer::start().await;
    let dummy_mstx = b"PK\x03\x04test_theme_archive_content";

    Mock::given(method("GET"))
        .and(path("/theme.mstx"))
        .respond_with(ResponseTemplate::new(200).set_body_bytes(dummy_mstx.to_vec()))
        .mount(&server)
        .await;

    let TestRemoteThemeContext { remote, .. } = remote_context;
    let theme_manifest = ThemeManifest {
        id: "theme1".to_string(),
        name: "Theme 1".to_string(),
        author: None,
        description: None,
        url: Some(format!("{}/theme.mstx", server.uri())),
        registry: Some("reg".to_string()),
        theme: None,
    };

    let result = remote.download_theme_file(&theme_manifest).await;
    assert_ok!(result.as_ref());
    let path = result.unwrap();
    assert!(path.exists());
    let read_back = fs::read(&path).unwrap();
    assert_eq!(read_back, dummy_mstx);
}

#[rstest]
#[tokio::test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_download_theme_file_missing_url(remote_context: TestRemoteThemeContext) {
    let TestRemoteThemeContext { remote, .. } = remote_context;
    let theme_manifest = ThemeManifest {
        id: "theme1".to_string(),
        name: "Theme 1".to_string(),
        author: None,
        description: None,
        url: None,
        registry: None,
        theme: None,
    };

    let result = remote.download_theme_file(&theme_manifest).await;
    assert_err!(result.as_ref());
}
