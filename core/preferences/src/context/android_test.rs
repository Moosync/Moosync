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

use super::{AndroidKeyringContext, Keyring};

#[test]
fn test_android_keyring_fallback() {
    #[cfg(target_os = "android")]
    let context = AndroidKeyringContext::new(types::android::AndroidJNIContext::default());
    #[cfg(not(target_os = "android"))]
    let context = AndroidKeyringContext::new();

    assert!(context.get_secret().is_err());

    let test_secret = b"12345678901234567890123456789012";
    context.set_secret(test_secret).unwrap();

    let fetched = context.get_secret().unwrap();
    assert_eq!(fetched, test_secret);
}
