use rstest::{fixture, rstest};
use tempdir::TempDir;
use tracing_test::traced_test;
use types::plugin::PluginContext;

use crate::StateManager;

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

#[rstest]
#[tokio::test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_state_manager_oauth_delegation(sm_context: TestSmContext) {
    let TestSmContext { sm, .. } = sm_context;

    sm.get_oauth_manager()
        .register_path("test_pkg".to_string(), "auth_host".to_string());

    assert_eq!(
        sm.get_oauth_manager().find_package_for_host("auth_host"),
        Some("test_pkg".to_string())
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
    sm.get_oauth_manager()
        .register_path("non_existent_pkg".to_string(), "known-host.com".to_string());

    let res = sm.handle_oauth_callback("https://known-host.com/cb").await;

    assert!(res.is_err());
}
