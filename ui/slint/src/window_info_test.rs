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
