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

use crate::{OnPlaylistScanned, OnProgressUpdated, OnSongScanned, error::ScannerError};

#[allow(async_fn_in_trait)]
pub trait ScannerContext: Send + Sync {
    async fn start_scan(
        &self,
        on_song: &OnSongScanned,
        on_playlist: &OnPlaylistScanned,
        on_progress: &OnProgressUpdated,
    ) -> Result<(), ScannerError>;
}

#[cfg(target_os = "android")]
pub mod android;
#[cfg(not(any(target_os = "android", target_os = "ios")))]
pub mod desktop;
