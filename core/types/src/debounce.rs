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
    future::Future,
    pin::Pin,
    sync::{Arc, Mutex},
    time::Duration,
};

use tokio::task::JoinHandle;
use tracing::Instrument;

#[derive(Clone, Default)]
pub struct DebounceHandle {
    task: Arc<Mutex<Option<JoinHandle<()>>>>,
}

impl DebounceHandle {
    #[tracing::instrument(level = "debug", skip_all)]
    pub fn new() -> Self {
        Self {
            task: Arc::new(Mutex::new(None)),
        }
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub fn debounce<Fut>(&self, delay: Duration, future: Fut)
    where
        Fut: Future<Output = ()> + Send + 'static,
    {
        let mut guard = self.task.lock().unwrap();
        if let Some(handle) = guard.take() {
            handle.abort();
        }
        *guard = Some(tokio::spawn(
            async move {
                tokio::time::sleep(delay).await;
                future.await;
            }
            .in_current_span(),
        ));
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub fn debounce_fn<F>(&self, delay: Duration, action: F)
    where
        F: FnOnce() + Send + 'static,
    {
        self.debounce(
            delay,
            async move {
                action();
            }
            .in_current_span(),
        );
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub fn cancel(&self) {
        let mut guard = self.task.lock().unwrap();
        if let Some(handle) = guard.take() {
            handle.abort();
        }
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub fn is_pending(&self) -> bool {
        let guard = self.task.lock().unwrap();
        guard.as_ref().is_some_and(|h| !h.is_finished())
    }
}

type DebounceCallback<T> =
    Arc<dyn Fn(T) -> Pin<Box<dyn Future<Output = ()> + Send>> + Send + Sync + 'static>;

pub struct Debouncer<T> {
    handle: DebounceHandle,
    delay: Duration,
    callback: DebounceCallback<T>,
}

impl<T: Send + 'static> Debouncer<T> {
    #[tracing::instrument(level = "debug", skip_all)]
    pub fn new<F, Fut>(delay: Duration, callback: F) -> Self
    where
        F: Fn(T) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = ()> + Send + 'static,
    {
        let boxed: DebounceCallback<T> = Arc::new(move |val| Box::pin(callback(val)));
        Self {
            handle: DebounceHandle::new(),
            delay,
            callback: boxed,
        }
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub fn new_sync<F>(delay: Duration, callback: F) -> Self
    where
        F: Fn(T) + Send + Sync + 'static,
    {
        let cb = Arc::new(callback);
        Self::new(delay, move |val| {
            let cb = cb.clone();
            async move {
                cb(val);
            }
            .in_current_span()
        })
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub fn call(&self, val: T) {
        let fut = (self.callback)(val);
        self.handle.debounce(self.delay, fut);
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub fn emit(&self, val: T) { self.call(val); }

    #[tracing::instrument(level = "debug", skip_all)]
    pub fn cancel(&self) { self.handle.cancel(); }

    #[tracing::instrument(level = "debug", skip_all)]
    pub fn is_pending(&self) -> bool { self.handle.is_pending() }
}

impl<T> Clone for Debouncer<T> {
    #[tracing::instrument(level = "debug", skip_all)]
    fn clone(&self) -> Self {
        Self {
            handle: self.handle.clone(),
            delay: self.delay,
            callback: self.callback.clone(),
        }
    }
}
