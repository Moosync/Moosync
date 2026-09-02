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

use assertables::{assert_ok, assert_ok_eq_x};
use rstest::{fixture, rstest};
use tracing_test::traced_test;
use uuid::Uuid;

use crate::context::{Keyring, KeyringContext};

#[fixture]
fn keyring_context() -> KeyringContext {
    let service = format!("moosync_test_service_{}", Uuid::new_v4());
    let user = format!("moosync_test_user_{}", Uuid::new_v4());
    KeyringContext::new(&service, &user).expect("failed to init keyring context")
}

#[rstest]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_keyring_context_set_and_get_secret(keyring_context: KeyringContext) {
    let secret = b"my_secret_token_12345";

    assert_ok!(keyring_context.set_secret(secret));
    assert_ok_eq_x!(keyring_context.get_secret().as_deref(), secret.as_slice());
}
