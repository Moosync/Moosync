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

use assertables::assert_len_eq_x;
use tracing_test::traced_test;

use crate::subscription::{SubscriberList, ToFilterKeys};

#[test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_subscriber_list_insert_and_run_all() {
    let list: SubscriberList<Box<dyn Fn(u32) + Send + Sync>> = SubscriberList::new();
    let counter = Arc::new(Mutex::new(0));
    let counter_clone = counter.clone();
    let _handle = list.insert(Box::new(move |val| {
        *counter_clone.lock().unwrap() += val;
    }));

    list.run_all(|callback| callback(5));

    assert_eq!(*counter.lock().unwrap(), 5);
}

#[test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_subscriber_list_cancel() {
    let list: SubscriberList<Box<dyn Fn(u32) + Send + Sync>> = SubscriberList::new();
    let counter = Arc::new(Mutex::new(0));
    let counter_clone = counter.clone();
    let handle = list.insert(Box::new(move |val| {
        *counter_clone.lock().unwrap() += val;
    }));

    list.run_all(|callback| callback(5));
    handle.cancel();
    list.run_all(|callback| callback(10));

    assert_eq!(*counter.lock().unwrap(), 5);
}

#[test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_watch_immediate() {
    let list: SubscriberList<Box<dyn Fn(String) + Send + Sync>> = SubscriberList::new();
    let received = Arc::new(Mutex::new(String::new()));
    let received_clone = received.clone();

    let _handle = list.watch_immediate(
        Box::new(move |s| {
            *received_clone.lock().unwrap() = s;
        }),
        "initial".to_string(),
    );

    assert_eq!(*received.lock().unwrap(), "initial");

    list.run_all(|callback| callback("updated".to_string()));
    assert_eq!(*received.lock().unwrap(), "updated");
}

#[test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_to_filter_keys_single() {
    let single: u32 = 42;

    let keys = ToFilterKeys::<u32>::to_filter_keys(single);

    assert_len_eq_x!(&keys, 1);
    assert_eq!(keys, vec![42]);
}

#[test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_to_filter_keys_multiple() {
    let multiple: Vec<u32> = vec![1, 2, 3];

    let keys = ToFilterKeys::<u32>::to_filter_keys(multiple);

    assert_len_eq_x!(&keys, 3);
    assert_eq!(keys, vec![1, 2, 3]);
}
