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

use tempdir::TempDir;
use types::plugin::PluginContext;

use crate::StateManager;

#[tokio::test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_state_manager_lifecycle_methods() {
    let tmp = TempDir::new("moosync_sm_lib_test").unwrap();
    let test_dir = tmp.path().to_path_buf();

    let context = PluginContext {
        data_dir: test_dir.clone(),
        cache_dir: test_dir.clone(),
        tmp_dir: test_dir.clone(),
        #[cfg(target_os = "android")]
        android_context: types::android::AndroidJNIContext::default(),
    };

    let sm = StateManager::new_with_context(context).unwrap();

    assert!(!sm.get_cache_dir().as_os_str().is_empty());

    sm.setup().await;
    sm.delayed_setup().await;
    sm.shutdown().await;
}
