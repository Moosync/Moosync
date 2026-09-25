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

type SubscriberCallback<T> = Arc<dyn Fn(T) + Send + Sync + 'static>;
type SubscriberMap<T> = Arc<Mutex<HashMap<usize, SubscriberCallback<T>>>>;

pub struct SubscriberList<T> {
    subscribers: SubscriberMap<T>,
    next_id: Arc<Mutex<usize>>,
}

impl<T: Send + Sync + 'static> SubscriberList<T> {
    #[tracing::instrument(level = "debug", skip_all)]
    pub fn new() -> Self {
        Self {
            subscribers: Arc::new(Mutex::new(HashMap::new())),
            next_id: Arc::new(Mutex::new(0)),
        }
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub fn listen<F>(&self, subscriber: F) -> CancelHandle
    where
        F: Fn(T) + Send + Sync + 'static,
    {
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
    pub fn listen_immediate<F>(&self, subscriber: F, init_val: T) -> CancelHandle
    where
        F: Fn(T) + Send + Sync + 'static,
        T: Clone,
    {
        subscriber(init_val);
        self.listen(subscriber)
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub fn listen_filtered<F, K>(&self, subscriber: F, keys: K) -> CancelHandle
    where
        F: Fn(T) + Send + Sync + 'static,
        K: ToFilterKeys<T>,
        T: PartialEq + Clone,
    {
        let keys = keys.to_filter_keys();
        self.listen(move |val| {
            if keys.contains(&val) {
                subscriber(val);
            }
        })
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub fn listen_filtered_immediate<F, K>(&self, subscriber: F, keys: K) -> CancelHandle
    where
        F: Fn(T) + Send + Sync + 'static,
        K: ToFilterKeys<T>,
        T: PartialEq + Clone,
    {
        let keys = keys.to_filter_keys();
        for key in &keys {
            subscriber(key.clone());
        }
        self.listen(move |val| {
            if keys.contains(&val) {
                subscriber(val);
            }
        })
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub fn emit(&self, val: T)
    where
        T: Clone,
    {
        let subs: Vec<SubscriberCallback<T>> = {
            let subscribers = self.subscribers.lock().unwrap();
            subscribers.values().cloned().collect()
        };
        for sub in subs {
            sub(val.clone());
        }
    }
}

impl<T: Send + Sync + 'static> Default for SubscriberList<T> {
    fn default() -> Self { Self::new() }
}

impl<T: Send + Sync + 'static> Clone for SubscriberList<T> {
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

impl ToFilterKeys<String> for &str {
    #[tracing::instrument(level = "debug", skip_all)]
    fn to_filter_keys(self) -> Vec<String> { vec![self.to_string()] }
}

impl ToFilterKeys<String> for &String {
    #[tracing::instrument(level = "debug", skip_all)]
    fn to_filter_keys(self) -> Vec<String> { vec![self.clone()] }
}
