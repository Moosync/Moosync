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
    collections::HashMap,
    sync::{Arc, Mutex},
};

pub struct CancelHandle {
    cancel_fn: Mutex<Option<Box<dyn FnOnce() + Send + Sync + 'static>>>,
}

impl CancelHandle {
    #[tracing::instrument(level = "debug", skip_all)]
    pub fn new<F>(cancel_fn: F) -> Self
    where
        F: FnOnce() + Send + Sync + 'static,
    {
        Self {
            cancel_fn: Mutex::new(Some(Box::new(cancel_fn))),
        }
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub fn cancel(&self) {
        let mut guard = self.cancel_fn.lock().unwrap();
        if let Some(f) = guard.take() {
            f();
        }
    }
}

pub struct SubscriberList<F> {
    subscribers: Arc<Mutex<HashMap<usize, Arc<F>>>>,
    next_id: Arc<Mutex<usize>>,
}

impl<F: Send + Sync + 'static> SubscriberList<F> {
    #[tracing::instrument(level = "debug", skip_all)]
    pub fn new() -> Self {
        Self {
            subscribers: Arc::new(Mutex::new(HashMap::new())),
            next_id: Arc::new(Mutex::new(0)),
        }
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub fn insert(&self, subscriber: F) -> CancelHandle {
        let mut id_guard = self.next_id.lock().unwrap();
        let id = *id_guard;
        *id_guard += 1;
        drop(id_guard);

        self.subscribers
            .lock()
            .unwrap()
            .insert(id, Arc::new(subscriber));

        let weak_subscribers = Arc::downgrade(&self.subscribers);

        CancelHandle::new(move || {
            if let Some(map) = weak_subscribers.upgrade() {
                map.lock().unwrap().remove(&id);
            }
        })
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub fn run_all<G>(&self, mut f: G)
    where
        G: FnMut(&F),
    {
        let subs: Vec<Arc<F>> = {
            let subscribers = self.subscribers.lock().unwrap();
            subscribers.values().cloned().collect()
        };
        for sub in subs {
            f(&sub);
        }
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub fn watch_immediate<A>(&self, subscriber: F, init_val: A) -> CancelHandle
    where
        F: std::ops::Fn(A),
    {
        subscriber(init_val);
        self.insert(subscriber)
    }
}

impl<F: Send + Sync + 'static> Default for SubscriberList<F> {
    fn default() -> Self { Self::new() }
}

impl<F: Send + Sync + 'static> Clone for SubscriberList<F> {
    fn clone(&self) -> Self {
        Self {
            subscribers: self.subscribers.clone(),
            next_id: self.next_id.clone(),
        }
    }
}

pub trait ToFilterKeys<T> {
    fn to_filter_keys(self) -> Vec<T>;
}

impl<T> ToFilterKeys<T> for T {
    #[tracing::instrument(level = "debug", skip_all)]
    fn to_filter_keys(self) -> Vec<T> { vec![self] }
}

impl<T> ToFilterKeys<T> for Vec<T> {
    #[tracing::instrument(level = "debug", skip_all)]
    fn to_filter_keys(self) -> Vec<T> { self }
}

#[macro_export]
macro_rules! generate_on_event_impl {
    ($struct_name:ident; $($name:ident, $watch_name:ident, $arg:ty, $trait_name:path);* $(;)?) => {
        impl $struct_name {
            $(
                pub fn $name<F, K>(&self, callback: F, keys: K) -> $crate::subscription::CancelHandle
                where
                    F: Fn($arg) + Send + Sync + 'static,
                    K: $trait_name,
                {
                    let keys = keys.to_filter_keys();
                    self.$name.insert(Box::new(move |val| {
                        if keys.contains(&val) {
                            callback(val);
                        }
                    }))
                }

                pub fn $watch_name<F, K>(&self, callback: F, keys: K) -> $crate::subscription::CancelHandle
                where
                    F: Fn($arg) + Send + Sync + 'static,
                    K: $trait_name,
                {
                    let keys = keys.to_filter_keys();
                    for key in &keys {
                        callback(key.clone());
                    }
                    self.$name.insert(Box::new(move |val| {
                        if keys.contains(&val) {
                            callback(val);
                        }
                    }))
                }
            )*
        }
    };

    ($struct_name:ident; $($name:ident, $watch_name:ident, $arg:ty);* $(;)?) => {
        impl $struct_name {
            $(
                pub fn $name<F>(&self, callback: F) -> $crate::subscription::CancelHandle
                where
                    F: Fn($arg) + Send + Sync + 'static,
                {
                    self.$name.insert(Box::new(callback))
                }

                pub fn $watch_name<F>(&self, callback: F, init_val: $arg) -> $crate::subscription::CancelHandle
                where
                    F: Fn($arg) + Send + Sync + 'static,
                {
                    self.$name.watch_immediate(Box::new(callback), init_val)
                }
            )*
        }
    };

    ($struct_name:ident; $($name:ident, $arg:ty);* $(;)?) => {
        impl $struct_name {
            $(
                pub fn $name<F>(&self, callback: F) -> $crate::subscription::CancelHandle
                where
                    F: Fn($arg) + Send + Sync + 'static,
                {
                    self.$name.insert(Box::new(callback))
                }
            )*
        }
    };
}
