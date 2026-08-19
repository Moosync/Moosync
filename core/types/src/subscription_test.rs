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

use crate::subscription::{SubscriberList, ToFilterKeys};

#[test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_subscriber_list_insert_and_run_all() {
    let list: SubscriberList<Box<dyn Fn(u32) + Send + Sync>> = SubscriberList::new();
    let counter = Arc::new(Mutex::new(0));

    let c_clone = counter.clone();
    let handle = list.insert(Box::new(move |val| {
        *c_clone.lock().unwrap() += val;
    }));

    list.run_all(|cb| cb(5));
    assert_eq!(*counter.lock().unwrap(), 5);

    handle.cancel();
    list.run_all(|cb| cb(10));
    assert_eq!(*counter.lock().unwrap(), 5);
}

#[test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_watch_immediate() {
    let list: SubscriberList<Box<dyn Fn(String) + Send + Sync>> = SubscriberList::new();
    let received = Arc::new(Mutex::new(String::new()));

    let r_clone = received.clone();
    let _handle = list.watch_immediate(
        Box::new(move |s| {
            *r_clone.lock().unwrap() = s;
        }),
        "initial".to_string(),
    );

    assert_eq!(*received.lock().unwrap(), "initial");
}

#[test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_to_filter_keys() {
    let single: u32 = 42;
    assert_eq!(ToFilterKeys::<u32>::to_filter_keys(single), vec![42]);

    let multiple: Vec<u32> = vec![1, 2, 3];
    assert_eq!(
        ToFilterKeys::<u32>::to_filter_keys(multiple),
        vec![1u32, 2, 3]
    );
}
