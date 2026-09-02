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

use std::{
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    time::Duration,
};

use assertables::assert_ok;
use rstest::{fixture, rstest};
use tempdir::TempDir;
use tracing_test::traced_test;
use types::plugin::PluginContext;

use crate::{
    StateManager,
    hooks::{Hook, extensions::ExtensionsHook},
};

struct TestSmContext {
    pub _temp_dir: TempDir,
    pub sm: StateManager,
}

#[fixture]
#[tracing::instrument(level = "debug", skip_all)]
fn sm_context() -> TestSmContext {
    let temp_dir = TempDir::new("moosync_sm_ext_test").expect("failed to create temp dir");
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
async fn test_extensions_hook_on_startup(sm_context: TestSmContext) {
    let TestSmContext { sm, .. } = sm_context;
    let hook = ExtensionsHook::new();
    assert_ok!(hook.on_startup(&sm).await);

    let pref = sm.get_preference_config().await;
    assert_ok!(pref.save(
        preferences::keys::ExtensionRegistries,
        vec!["https://example.com/custom_manifest.json".to_string()],
    ));
    tokio::time::sleep(Duration::from_millis(50)).await;

    let ext = sm.get_extension_handler().await;
    assert!(
        ext.get_registries()
            .contains("https://example.com/custom_manifest.json")
    );
}

#[rstest]
#[tokio::test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_extensions_hook_triggers_update_on_registry_change(sm_context: TestSmContext) {
    let TestSmContext { sm, .. } = sm_context;
    let hook = ExtensionsHook::new();
    assert_ok!(hook.on_startup(&sm).await);

    let updated_flag = Arc::new(AtomicBool::new(false));
    let ext = sm.get_extension_handler().await;
    let flag_clone = updated_flag.clone();
    let _cancel = ext.on_extensions_updated(move |_| {
        flag_clone.store(true, Ordering::SeqCst);
    });

    let pref = sm.get_preference_config().await;
    assert_ok!(pref.save(
        preferences::keys::ExtensionRegistries,
        vec!["https://example.com/new_registry.json".to_string()],
    ));
    tokio::time::sleep(Duration::from_millis(50)).await;

    assert!(updated_flag.load(Ordering::SeqCst));
}
