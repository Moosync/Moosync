use std::{
    sync::{Arc, Mutex},
    time::Duration,
};

use tracing_test::traced_test;

use crate::debounce::{DebounceHandle, Debouncer};

#[tokio::test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_debounce_handle_executes_after_delay() {
    let handle = DebounceHandle::new();
    let executed = Arc::new(Mutex::new(false));
    let executed_clone = executed.clone();

    handle.debounce(Duration::from_millis(50), async move {
        *executed_clone.lock().unwrap() = true;
    });

    assert!(!*executed.lock().unwrap());
    assert!(handle.is_pending());

    tokio::time::sleep(Duration::from_millis(100)).await;

    assert!(*executed.lock().unwrap());
    assert!(!handle.is_pending());
}

#[tokio::test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_debounce_handle_cancels_previous() {
    let handle = DebounceHandle::new();
    let results = Arc::new(Mutex::new(Vec::new()));

    for i in 1..=3 {
        let results_clone = results.clone();
        handle.debounce(Duration::from_millis(50), async move {
            results_clone.lock().unwrap().push(i);
        });
        tokio::time::sleep(Duration::from_millis(10)).await;
    }

    tokio::time::sleep(Duration::from_millis(100)).await;

    let recorded = results.lock().unwrap().clone();
    assert_eq!(recorded, vec![3]);
}

#[tokio::test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_debounce_handle_explicit_cancel() {
    let handle = DebounceHandle::new();
    let executed = Arc::new(Mutex::new(false));
    let executed_clone = executed.clone();

    handle.debounce(Duration::from_millis(50), async move {
        *executed_clone.lock().unwrap() = true;
    });

    handle.cancel();

    tokio::time::sleep(Duration::from_millis(100)).await;

    assert!(!*executed.lock().unwrap());
    assert!(!handle.is_pending());
}

#[tokio::test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_debouncer_call_and_emit() {
    let results = Arc::new(Mutex::new(Vec::new()));
    let results_clone = results.clone();

    let debouncer = Debouncer::new(Duration::from_millis(50), move |val: u32| {
        let results_inner = results_clone.clone();
        async move {
            results_inner.lock().unwrap().push(val);
        }
    });

    debouncer.call(10);
    debouncer.emit(20);
    debouncer.call(30);

    assert!(results.lock().unwrap().is_empty());

    tokio::time::sleep(Duration::from_millis(100)).await;

    let recorded = results.lock().unwrap().clone();
    assert_eq!(recorded, vec![30]);
}

#[tokio::test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_debouncer_new_sync() {
    let results = Arc::new(Mutex::new(Vec::new()));
    let results_clone = results.clone();

    let debouncer = Debouncer::new_sync(Duration::from_millis(50), move |val: String| {
        results_clone.lock().unwrap().push(val);
    });

    debouncer.call("hello".to_string());
    debouncer.call("world".to_string());

    tokio::time::sleep(Duration::from_millis(100)).await;

    let recorded = results.lock().unwrap().clone();
    assert_eq!(recorded, vec!["world".to_string()]);
}

#[tokio::test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_debouncer_cancel() {
    let results = Arc::new(Mutex::new(Vec::new()));
    let results_clone = results.clone();

    let debouncer = Debouncer::new_sync(Duration::from_millis(50), move |val: i32| {
        results_clone.lock().unwrap().push(val);
    });

    debouncer.call(42);
    debouncer.cancel();

    tokio::time::sleep(Duration::from_millis(100)).await;

    assert!(results.lock().unwrap().is_empty());
}
