use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};

use assertables::assert_ok;
use preferences::keys::{EXTENSION_REGISTRIES, THEME_REGISTRIES};
use preferences_proto::moosync::types::{
    PreferenceItem, PreferenceValue, StringList, preference_value,
};
use rstest::{fixture, rstest};
use tempdir::TempDir;
use tracing_test::traced_test;
use types::plugin::PluginContext;

use crate::{
    StateManager,
    hooks::{
        Hook,
        registries::{ExtensionRegistriesHook, ThemeRegistriesHook},
    },
};

struct TestSmContext {
    pub _temp_dir: TempDir,
    pub sm: StateManager,
}

#[fixture]
#[tracing::instrument(level = "debug", skip_all)]
fn sm_context() -> TestSmContext {
    let temp_dir = TempDir::new("moosync_sm_registries_test").expect("failed to create temp dir");
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
async fn test_registries_hook_on_startup(sm_context: TestSmContext) {
    let TestSmContext { sm, .. } = sm_context;
    let hook = ExtensionRegistriesHook::new();
    let preference_config = sm.get_preference_config();
    let fetched = Arc::new(AtomicBool::new(false));
    let fetched_clone = fetched.clone();

    let _handle = sm
        .get_extension_handler()
        .on_extensions_updated
        .listen(move |_| {
            fetched_clone.store(true, Ordering::SeqCst);
        });

    assert_ok!(hook.on_startup(&sm).await);

    preference_config
        .save(PreferenceItem {
            id: EXTENSION_REGISTRIES.id.clone(),
            value: Some(PreferenceValue {
                value: Some(preference_value::Value::ListValue(StringList {
                    values: vec!["https://example.com/registry.json".to_string()],
                })),
            }),
            ..EXTENSION_REGISTRIES.clone()
        })
        .unwrap();

    tokio::time::sleep(std::time::Duration::from_millis(100)).await;

    assert_ok!(preference_config.remove(&EXTENSION_REGISTRIES));
}

#[rstest]
#[tokio::test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_theme_registries_hook_on_startup(sm_context: TestSmContext) {
    let TestSmContext { sm, .. } = sm_context;
    let hook = ThemeRegistriesHook::new();
    let preference_config = sm.get_preference_config();
    let fetched = Arc::new(AtomicBool::new(false));
    let fetched_clone = fetched.clone();

    let _handle = sm.get_theme_holder().on_themes_updated.listen(move |_| {
        fetched_clone.store(true, Ordering::SeqCst);
    });

    assert_ok!(hook.on_startup(&sm).await);

    preference_config
        .save(PreferenceItem {
            id: THEME_REGISTRIES.id.clone(),
            value: Some(PreferenceValue {
                value: Some(preference_value::Value::ListValue(StringList {
                    values: vec!["https://example.com/themes_manifest.pb".to_string()],
                })),
            }),
            ..THEME_REGISTRIES.clone()
        })
        .unwrap();

    tokio::time::sleep(std::time::Duration::from_millis(100)).await;

    assert_ok!(preference_config.remove(&THEME_REGISTRIES));
}
