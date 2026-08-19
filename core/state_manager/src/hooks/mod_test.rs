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
use tempdir::TempDir;
use types::plugin::PluginContext;

use crate::{StateManager, hooks::Hook};

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

#[tokio::test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_hook_lifecycle_invocations() {
    let tmp = TempDir::new("moosync_sm_hooks_test").unwrap();
    let test_dir = tmp.path().to_path_buf();

    let context = PluginContext {
        data_dir: test_dir.clone(),
        cache_dir: test_dir.clone(),
        tmp_dir: test_dir.clone(),
        #[cfg(target_os = "android")]
        android_context: types::android::AndroidJNIContext::default(),
    };

    let sm = StateManager::new_with_context(context).unwrap();

    let events = Arc::new(Mutex::new(Vec::new()));
    let hook = TrackingHook {
        events: events.clone(),
    };

    assert!(hook.on_startup(&sm).await.is_ok());
    assert!(hook.on_delayed_startup(&sm).await.is_ok());
    assert!(hook.on_exit(&sm).await.is_ok());

    let recorded = events.lock().unwrap().clone();
    assert_eq!(recorded, vec!["startup", "delayed_startup", "exit"]);
}
