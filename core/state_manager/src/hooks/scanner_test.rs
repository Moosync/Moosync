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

use std::path::PathBuf;

use file_scanner::PlaylistSongId;

use crate::{
    StateManager,
    hooks::{Hook, scanner::ScannerHook},
};

#[tokio::test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_scanner_hook_on_startup() {
    let sm = StateManager::new(
        #[cfg(target_os = "android")]
        types::android::AndroidJNIContext::default(),
    )
    .unwrap();

    let hook = ScannerHook::new();
    let res = hook.on_startup(&sm).await;
    assert!(res.is_ok());
}
