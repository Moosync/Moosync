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

#[tracing::instrument(level = "debug", skip_all)]
fn get_worker() -> &'static mpsc::Sender<Task> {
    TASK_SENDER.get_or_init(|| {
        let (tx, rx) = mpsc::channel::<Task>();
        std::thread::Builder::new()
            .name("slint_test_worker".into())
            .spawn(move || {
                i_slint_backend_testing::init_no_event_loop();
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
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();
        let local = tokio::task::LocalSet::new();
        local.block_on(&rt, f())
    })
}
