use rstest::{fixture, rstest};
use tempdir::TempDir;
use tracing_test::traced_test;
use types::plugin::{Plugin, PluginContext};

use crate::preferences::PreferenceConfig;

struct PluginSmokeContext {
    _temp_dir: TempDir,
    pub context: PluginContext,
}

#[fixture]
fn smoke_context() -> PluginSmokeContext {
    let temp_dir = TempDir::new("moosync_prefs_plugin_smoke").expect("failed to create temp dir");
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
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_preferences_plugin_init(smoke_context: PluginSmokeContext) {
    let PluginSmokeContext { context, .. } = smoke_context;

    let pref_plugin = PreferenceConfig::init(&context);

    assert!(context.data_dir.join("preferences.bin").exists());
    assert!(pref_plugin.get("scan_threads").is_none());
}
