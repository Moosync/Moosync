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

use std::sync::{OnceLock, mpsc};

type Task = Box<dyn FnOnce() + Send + 'static>;

static TASK_SENDER: OnceLock<mpsc::Sender<Task>> = OnceLock::new();
static INIT_LOGGING: OnceLock<()> = OnceLock::new();

#[tracing::instrument(level = "debug", skip_all)]
pub fn init_test_logging() {
    INIT_LOGGING.get_or_init(|| {
        let filter = tracing_subscriber::EnvFilter::try_from_default_env()
            .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("debug"));
        let _ = tracing_subscriber::fmt()
            .with_env_filter(filter)
            .with_test_writer()
            .try_init();
    });
}

#[tracing::instrument(level = "debug", skip_all)]
fn get_worker() -> &'static mpsc::Sender<Task> {
    TASK_SENDER.get_or_init(|| {
        init_test_logging();
        let (tx, rx) = mpsc::channel::<Task>();
        std::thread::Builder::new()
            .name("slint_test_worker".into())
            .spawn(move || {
                i_slint_backend_testing::init_integration_test_with_system_time();
                while let Ok(task) = rx.recv() {
                    let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(task));
                }
            })
            .expect("failed to spawn slint test worker");
        tx
    })
}

#[tracing::instrument(level = "debug", skip_all)]
pub fn run_test<F, R>(f: F) -> R
where
    F: FnOnce() -> R + Send + 'static,
    R: Send + 'static,
{
    let (res_tx, res_rx) = mpsc::channel();
    get_worker()
        .send(Box::new(move || {
            let res = std::panic::catch_unwind(std::panic::AssertUnwindSafe(f));
            let _ = res_tx.send(res);
        }))
        .expect("failed to send task to slint test worker");
    match res_rx.recv().expect("worker thread died") {
        Ok(r) => r,
        Err(e) => std::panic::resume_unwind(e),
    }
}

#[tracing::instrument(level = "debug", skip_all)]
pub fn run_async_test<F, Fut, R>(f: F) -> R
where
    F: FnOnce() -> Fut + Send + 'static,
    Fut: std::future::Future<Output = R> + 'static,
    R: Send + 'static,
{
    run_test(move || {
        let (tx, rx) = std::sync::mpsc::channel();
        let rt = tokio::runtime::Builder::new_multi_thread()
            .worker_threads(2)
            .enable_all()
            .build()
            .unwrap();
        let _guard = rt.enter();

        let spawn_res = slint::spawn_local(async move {
            let res = f().await;
            let _ = tx.send(res);
            let _ = slint::quit_event_loop();
        });
        if spawn_res.is_err() {
            panic!("failed to spawn local task on slint event loop");
        }

        let _ = slint::run_event_loop();
        rx.recv()
            .expect("test failed or hung without quitting event loop")
    })
}
