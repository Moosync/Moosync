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

use crate::window_info::WindowEvents;

#[test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_window_events_on_resize_registration() {
    let events = WindowEvents::new();
    events.on_resize(Box::new(|_| {}));
    assert_eq!(events.on_resize.lock().unwrap().len(), 1);
}
