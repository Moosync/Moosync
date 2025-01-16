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

// Moosync
// Copyright (C) 2025 Moosync
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
// along with this program. If not, see <http://www.gnu.org/licenses/>.

#[cfg(not(any(target_os = "android", target_os = "ios")))]
mod playlist_scanner;
#[cfg(not(any(target_os = "android", target_os = "ios")))]
mod scanner;
#[cfg(not(any(target_os = "android", target_os = "ios")))]
pub use scanner::{ScanState, ScannerHolder};
#[cfg(not(any(target_os = "android", target_os = "ios")))]
mod song_scanner;
mod types;
mod utils;

#[cfg(target_os = "android")]
mod scanner_android;
#[cfg(target_os = "android")]
pub use scanner_android::{ScanState, ScannerHolder};
