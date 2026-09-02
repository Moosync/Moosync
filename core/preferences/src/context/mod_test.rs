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

use assertables::assert_err;
use rstest::{fixture, rstest};
use tracing_test::traced_test;
use uuid::Uuid;

use crate::context::{Keyring, KeyringContext};

#[fixture]
fn fresh_keyring_context() -> KeyringContext {
    let service = format!("moosync_test_empty_{}", Uuid::new_v4());
    let user = format!("moosync_test_user_{}", Uuid::new_v4());
    KeyringContext::new(&service, &user).expect("failed to init fresh keyring context")
}

#[rstest]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_keyring_context_get_nonexistent_returns_no_entry(fresh_keyring_context: KeyringContext) {
    assert_err!(fresh_keyring_context.get_secret());
}
