use assertables::{assert_none, assert_some_eq_x};
use rstest::{fixture, rstest};
use tempdir::TempDir;
use tracing_test::traced_test;
use types::plugin::PluginContext;

use crate::{StateManager, oauth::OAuthManager};

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
