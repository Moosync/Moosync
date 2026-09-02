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

use rstest::fixture;
use state_manager::StateManager;
use tempdir::TempDir;
use types::plugin::PluginContext;

use crate::MainWindow;

#[fixture]
#[tracing::instrument(level = "debug", skip_all)]
pub fn main_window() -> MainWindow {
    i_slint_backend_testing::init_no_event_loop();
    MainWindow::new().expect("failed to create MainWindow")
}

pub struct TestSlintSmContext {
    pub _temp_dir: TempDir,
    pub sm: StateManager,
}

#[fixture]
#[tracing::instrument(level = "debug", skip_all)]
pub fn state_manager_fixture() -> TestSlintSmContext {
    let temp_dir = TempDir::new("moosync_slint_test").expect("failed to create temp dir");
    let test_dir = temp_dir.path().to_path_buf();
    let context = PluginContext {
        data_dir: test_dir.clone(),
        cache_dir: test_dir.clone(),
        tmp_dir: test_dir.clone(),
        #[cfg(target_os = "android")]
        android_context: types::android::AndroidJNIContext::default(),
    };
    let sm = StateManager::new_with_context(context).expect("failed to create state manager");
    TestSlintSmContext {
        _temp_dir: temp_dir,
        sm,
    }
}
