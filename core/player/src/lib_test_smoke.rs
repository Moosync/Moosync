use rstest::{fixture, rstest};
use tempdir::TempDir;
use tracing_test::traced_test;
use types::plugin::{Plugin, PluginContext};

use crate::PlayerHandler;

struct PluginSmokeContext {
    pub _temp_dir: TempDir,
    pub context: PluginContext,
}

#[fixture]
#[tracing::instrument(level = "debug", skip_all)]
fn smoke_context() -> PluginSmokeContext {
    let temp_dir = TempDir::new("moosync_player_plugin_smoke").expect("failed to create temp dir");
    let test_dir = temp_dir.path().to_path_buf();
    let context = PluginContext {
        data_dir: test_dir.clone(),
        cache_dir: test_dir.clone(),
        tmp_dir: test_dir.clone(),
        #[cfg(target_os = "android")]
        android_context: types::android::AndroidJNIContext::default(),
    };
    PluginSmokeContext {
        _temp_dir: temp_dir,
        context,
    }
}

#[rstest]
#[tokio::test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_player_plugin_init(smoke_context: PluginSmokeContext) {
    let PluginSmokeContext { context, .. } = smoke_context;

    let plugin = PlayerHandler::init(&context);
    let _ = plugin.get_queue();
}
