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

use uuid::Uuid;

use crate::context::{Keyring, KeyringContext};

#[test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_keyring_context_new_and_methods() {
    let service = format!("moosync_test_service_{}", Uuid::new_v4());
    let user = format!("moosync_test_user_{}", Uuid::new_v4());

    let ctx = KeyringContext::new(&service, &user);
    assert!(ctx.is_ok());

    let keyring = ctx.unwrap();

    let secret = b"my_secret_token_12345";
    let set_res = keyring.set_secret(secret);
    assert!(set_res.is_ok());

    let get_res = keyring.get_secret();
    assert!(get_res.is_ok());
    assert_eq!(get_res.unwrap(), secret);
}
