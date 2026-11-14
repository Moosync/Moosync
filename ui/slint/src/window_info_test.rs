use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};

use i_slint_backend_testing::init_no_event_loop;
use slint::ComponentHandle;
use tracing_test::traced_test;

use crate::{MainWindow, window_info::WindowEvents};

#[test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_window_events_on_resize_registration_and_trigger() {
    init_no_event_loop();
    let main_window = MainWindow::new().unwrap();
    let events = WindowEvents::new();
    let resize_called = Arc::new(AtomicBool::new(false));
    let flag = resize_called.clone();

    events.on_resize(Box::new(move |_| {
        flag.store(true, Ordering::SeqCst);
    }));

    assert_eq!(events.on_resize.lock().unwrap().len(), 1);
    events.trigger_resize(main_window.window());
    assert!(resize_called.load(Ordering::SeqCst));
}
