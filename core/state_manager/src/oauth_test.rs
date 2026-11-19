use assertables::{assert_none, assert_some_eq_x};
use rstest::{fixture, rstest};
use tempdir::TempDir;
use tracing_test::traced_test;
use types::plugin::PluginContext;

use crate::{
    StateManager,
    oauth::OAuthManager,
    oauth_markdown::{OauthContentPart, parse_and_split_oauth_markdown},
};

struct TestSmContext {
    pub _temp_dir: TempDir,
    pub sm: StateManager,
}

#[fixture]
#[tracing::instrument(level = "debug", skip_all)]
fn sm_context() -> TestSmContext {
    let temp_dir = TempDir::new("moosync_sm_oauth_test").expect("failed to create temp dir");
    let test_dir = temp_dir.path().to_path_buf();
    let context = PluginContext {
        data_dir: test_dir.clone(),
        cache_dir: test_dir.clone(),
        tmp_dir: test_dir.clone(),
        #[cfg(target_os = "android")]
        android_context: types::android::AndroidJNIContext::default(),
    };
    let sm = StateManager::new_with_context(context).expect("failed to create state manager");
    TestSmContext {
        _temp_dir: temp_dir,
        sm,
    }
}

#[test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_oauth_manager_register_and_find_host() {
    let manager = OAuthManager::new();

    manager.register_path("package_a".to_string(), "spotify".to_string());
    manager.register_path("package_b".to_string(), "moosync://youtube".to_string());

    assert_some_eq_x!(manager.find_package_for_host("spotify"), "package_a");
    assert_some_eq_x!(manager.find_package_for_host("youtube"), "package_b");
    assert_none!(manager.find_package_for_host("unknown"));
}

#[rstest]
#[tokio::test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_state_manager_oauth_delegation(sm_context: TestSmContext) {
    let TestSmContext { sm, .. } = sm_context;

    sm.register_oauth_path("test_pkg".to_string(), "auth_host".to_string());

    assert_eq!(
        sm.find_package_for_oauth_host("auth_host"),
        Some("test_pkg".to_string())
    );
}

#[test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_oauth_manager_register_path_fallback() {
    let manager = OAuthManager::new();

    manager.register_path("pkg_fallback".to_string(), "://fallback_host/".to_string());

    assert_some_eq_x!(
        manager.find_package_for_host("fallback_host"),
        "pkg_fallback"
    );
}

#[rstest]
#[tokio::test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_handle_oauth_callback_invalid_url(sm_context: TestSmContext) {
    let TestSmContext { sm, .. } = sm_context;

    let res = sm.handle_oauth_callback("not a valid url!").await;

    assert!(res.is_err());
}

#[rstest]
#[tokio::test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_handle_oauth_callback_no_host(sm_context: TestSmContext) {
    let TestSmContext { sm, .. } = sm_context;

    let res = sm.handle_oauth_callback("data:text/plain,hello").await;

    assert!(res.is_err());
}

#[rstest]
#[tokio::test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_handle_oauth_callback_unregistered_host(sm_context: TestSmContext) {
    let TestSmContext { sm, .. } = sm_context;

    let res = sm
        .handle_oauth_callback("https://unknown-host.com/cb")
        .await;

    assert!(res.is_err());
}

#[rstest]
#[tokio::test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_handle_oauth_callback_missing_extension(sm_context: TestSmContext) {
    let TestSmContext { sm, .. } = sm_context;
    sm.register_oauth_path("non_existent_pkg".to_string(), "known-host.com".to_string());

    let res = sm.handle_oauth_callback("https://known-host.com/cb").await;

    assert!(res.is_err());
}

#[test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_oauth_manager_preserves_first_host_for_package() {
    let manager = OAuthManager::new();

    manager.register_path("pkg_multi".to_string(), "first_host".to_string());
    manager.register_path("pkg_multi".to_string(), "second_host".to_string());

    assert_some_eq_x!(manager.find_host_for_package("pkg_multi"), "first_host");
    assert_some_eq_x!(manager.find_package_for_host("first_host"), "pkg_multi");
    assert_some_eq_x!(manager.find_package_for_host("second_host"), "pkg_multi");
    assert_none!(manager.find_host_for_package("unknown_pkg"));
}

#[test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_parse_and_split_oauth_markdown_extracts_title_and_interleaved_images() {
    let ext_dir = TempDir::new("moosync_ext_assets").expect("failed to create ext temp dir");
    let assets_dir = ext_dir.path().join("assets");
    std::fs::create_dir_all(&assets_dir).expect("failed to create assets dir");
    let img1_path = assets_dir.join("qr.png");
    let img2_path = assets_dir.join("step2.png");
    std::fs::write(&img1_path, b"png1").expect("failed to write img1");
    std::fs::write(&img2_path, b"png2").expect("failed to write img2");

    let md = "# Pair Your Device\n\nScan this code:\n\n![QR](assets/qr.png)\n\nThen visit [Setup](https://example.com/setup) or [Insecure](http://example.com/bad).\n\n![Step 2](assets/step2.png)\n\nAll done!";

    let processed = parse_and_split_oauth_markdown(md, Some(ext_dir.path()));

    assert_eq!(processed.title, Some("Pair Your Device".to_string()));
    assert_eq!(
        processed.parts,
        vec![
            OauthContentPart::Text("Scan this code:".to_string()),
            OauthContentPart::Image(img1_path.canonicalize().unwrap()),
            OauthContentPart::Text(
                "Then visit [Setup](https://example.com/setup) or Insecure.".to_string()
            ),
            OauthContentPart::Image(img2_path.canonicalize().unwrap()),
            OauthContentPart::Text("All done!".to_string()),
        ]
    );
}

#[test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_parse_and_split_oauth_markdown_rejects_traversal_and_external_images() {
    let parent_dir = TempDir::new("moosync_parent_dir").expect("failed to create parent dir");
    let ext_dir = parent_dir.path().join("ext");
    std::fs::create_dir_all(&ext_dir).expect("failed to create ext dir");
    let secret_file = parent_dir.path().join("secret.png");
    std::fs::write(&secret_file, b"secret").expect("failed to write secret");

    let md = "![Traversal](../secret.png)\n\n![External](https://evil.com/img.png)\n\n![Absolute](/etc/passwd)\n\n[Bad](javascript:alert(1))";

    let processed = parse_and_split_oauth_markdown(md, Some(&ext_dir));

    assert_eq!(processed.title, None);
    assert_eq!(
        processed.parts,
        vec![OauthContentPart::Text("Bad".to_string())]
    );
}
