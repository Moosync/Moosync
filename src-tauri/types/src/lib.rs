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

pub mod cache;
#[cfg(feature = "core")]
pub mod cache_schema;
pub mod canvaz;
pub mod common;
pub mod entities;
pub mod errors;

#[cfg(not(feature = "extensions"))]
pub mod mpris;
pub mod preferences;
#[cfg(feature = "core")]
pub mod schema;
pub mod songs;

pub mod providers;

pub mod ui;

#[cfg(feature = "core")]
pub mod oauth;

#[cfg(any(feature = "core", feature = "extensions"))]
pub mod extensions;

#[cfg(not(feature = "extensions"))]
pub mod themes;

#[cfg(not(feature = "extensions"))]
pub mod window;
