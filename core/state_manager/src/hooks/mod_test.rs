use std::sync::{Arc, Mutex};

use assertables::assert_ok;
use async_trait::async_trait;
use rstest::{fixture, rstest};
use tempdir::TempDir;
use tracing_test::traced_test;
use types::plugin::PluginContext;

use crate::{StateManager, hooks::Hook};

struct TestSmContext {
    pub _temp_dir: TempDir,
    pub sm: StateManager,
}

#[fixture]
#[tracing::instrument(level = "debug", skip_all)]
fn sm_context() -> TestSmContext {
    let temp_dir = TempDir::new("moosync_sm_hooks_test").expect("failed to create temp dir");
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

struct TrackingHook {
    events: Arc<Mutex<Vec<&'static str>>>,
}

#[async_trait]
impl Hook for TrackingHook {
    #[tracing::instrument(level = "debug", skip_all)]
    async fn on_startup(
        &self,
        _state_manager: &StateManager,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        self.events.lock().unwrap().push("startup");
        Ok(())
    }

    #[tracing::instrument(level = "debug", skip_all)]
    async fn on_delayed_startup(
        &self,
        _state_manager: &StateManager,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        self.events.lock().unwrap().push("delayed_startup");
        Ok(())
    }

    #[tracing::instrument(level = "debug", skip_all)]
    async fn on_exit(
        &self,
        _state_manager: &StateManager,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        self.events.lock().unwrap().push("exit");
        Ok(())
    }
}

#[rstest]
#[tokio::test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_hook_lifecycle_invocations(sm_context: TestSmContext) {
    let TestSmContext { sm, .. } = sm_context;
    let events = Arc::new(Mutex::new(Vec::new()));
    let hook = TrackingHook {
        events: events.clone(),
    };

    assert_ok!(hook.on_startup(&sm).await);
    assert_ok!(hook.on_delayed_startup(&sm).await);
    assert_ok!(hook.on_exit(&sm).await);

    assert_eq!(
        *events.lock().unwrap(),
        vec!["startup", "delayed_startup", "exit"]
    );
}
