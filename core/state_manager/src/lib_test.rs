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

use std::sync::{Arc, Mutex};

use async_trait::async_trait;
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
    let temp_dir = TempDir::new("moosync_sm_lib_test").expect("failed to create temp dir");
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
impl crate::hooks::Hook for TrackingHook {
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
async fn test_state_manager_lifecycle_methods(sm_context: TestSmContext) {
    let TestSmContext { sm, .. } = sm_context;
    let events = Arc::new(Mutex::new(Vec::new()));
    let hook = Arc::new(TrackingHook {
        events: events.clone(),
    });
    sm.register_hook(hook).await;

    assert!(!sm.get_cache_dir().as_os_str().is_empty());

    sm.setup().await;
    sm.delayed_setup().await;
    sm.shutdown().await;

    assert_eq!(
        *events.lock().unwrap(),
        vec!["startup", "delayed_startup", "exit"]
    );
}

struct FailingHook;

#[async_trait]
impl crate::hooks::Hook for FailingHook {
    #[tracing::instrument(level = "debug", skip_all)]
    async fn on_startup(
        &self,
        _state_manager: &StateManager,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        Err("startup failure".into())
    }

    #[tracing::instrument(level = "debug", skip_all)]
    async fn on_delayed_startup(
        &self,
        _state_manager: &StateManager,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        Err("delayed startup failure".into())
    }

    #[tracing::instrument(level = "debug", skip_all)]
    async fn on_exit(
        &self,
        _state_manager: &StateManager,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        Err("exit failure".into())
    }
}

#[rstest]
#[tokio::test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_state_manager_hook_error_handling(sm_context: TestSmContext) {
    let TestSmContext { sm, .. } = sm_context;
    sm.register_hook(Arc::new(FailingHook)).await;

    sm.setup().await;
    sm.delayed_setup().await;
    sm.shutdown().await;
}

#[tokio::test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_state_manager_new_with_custom_env() {
    let temp_dir = TempDir::new("moosync_env_test").unwrap();
    let old_data = std::env::var_os("XDG_DATA_HOME");
    let old_cache = std::env::var_os("XDG_CACHE_HOME");

    unsafe {
        std::env::set_var("XDG_DATA_HOME", temp_dir.path().join("data"));
        std::env::set_var("XDG_CACHE_HOME", temp_dir.path().join("cache"));
    }

    #[cfg(not(target_os = "android"))]
    let sm = StateManager::new();
    #[cfg(target_os = "android")]
    let sm = StateManager::new(types::android::AndroidJNIContext::default());

    unsafe {
        match old_data {
            Some(v) => std::env::set_var("XDG_DATA_HOME", v),
            None => std::env::remove_var("XDG_DATA_HOME"),
        }
        match old_cache {
            Some(v) => std::env::set_var("XDG_CACHE_HOME", v),
            None => std::env::remove_var("XDG_CACHE_HOME"),
        }
    }

    assert!(sm.is_ok());
    let sm = sm.unwrap();
    assert!(!sm.get_cache_dir().as_os_str().is_empty());
}
