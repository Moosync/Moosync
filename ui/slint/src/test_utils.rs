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

use std::time::Duration;

use rstest::fixture;
use state_manager::StateManager;
use tempdir::TempDir;
use types::plugin::PluginContext;

use crate::MainWindow;

type Task = Box<dyn FnOnce() + Send + 'static>;
static RUNNER: std::sync::OnceLock<std::sync::mpsc::Sender<Task>> = std::sync::OnceLock::new();

#[tracing::instrument(level = "debug", skip_all)]
pub fn runtime() -> &'static tokio::runtime::Runtime {
    static RUNTIME: std::sync::OnceLock<tokio::runtime::Runtime> = std::sync::OnceLock::new();
    RUNTIME.get_or_init(|| {
        tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .unwrap()
    })
}

#[tracing::instrument(level = "debug", skip_all)]
pub fn runner() -> &'static std::sync::mpsc::Sender<Task> {
    RUNNER.get_or_init(|| {
        let (tx, rx) = std::sync::mpsc::channel::<Task>();
        std::thread::Builder::new()
            .name("slint_test_runner".into())
            .spawn(move || {
                let _guard = runtime().enter();
                i_slint_backend_testing::init_integration_test_with_system_time();
                while let Ok(task) = rx.recv() {
                    let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(task));
                }
            })
            .expect("failed to spawn slint runner");
        tx
    })
}

#[tracing::instrument(level = "debug", skip_all)]
pub fn run_slint_test<F, Fut>(test_fn: F)
where
    F: FnOnce(&'static MainWindow, TestSlintSmContext) -> Fut + Send + 'static,
    Fut: std::future::Future<Output = ()> + 'static,
{
    let (tx, rx) = std::sync::mpsc::channel();
    runner()
        .send(Box::new(move || {
            let state_manager_fixture = state_manager_fixture();
            let main_window: &'static MainWindow = Box::leak(Box::new(MainWindow::new().unwrap()));

            slint::spawn_local(async move {
                test_fn(main_window, state_manager_fixture).await;
                let _ = slint::quit_event_loop();
            })
            .expect("failed to spawn local task on slint event loop");

            slint::run_event_loop().expect("failed to run slint event loop");
            let _ = tx.send(());
        }))
        .expect("failed to send task to slint runner");
    rx.recv().expect("test failed or runner panicked");
}

#[tracing::instrument(level = "debug", skip_all)]
pub async fn wait_until<F>(mut condition: F) -> bool
where
    F: FnMut() -> bool,
{
    for _ in 0..100 {
        if condition() {
            return true;
        }
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
    false
}

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
