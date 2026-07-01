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

use std::time::SystemTimeError;

#[derive(Debug, thiserror::Error)]
pub enum DatabaseError {
    #[error("SQLite query failed")]
    Query(#[source] rusqlite::Error),

    #[error("SQLite connection failed")]
    Connection(#[source] rusqlite::Error),

    #[error("SQLite migration failed")]
    Migration(#[source] rusqlite::Error),

    #[error("SQLite transaction failed")]
    Transaction(#[source] rusqlite::Error),

    #[error("JSON serialization failed: {0}")]
    Json(#[from] serde_json::Error),

    #[error("System time error: {0}")]
    SystemTime(#[from] SystemTimeError),

    #[error("Format error: {0}")]
    Fmt(#[from] std::fmt::Error),

    #[error("Pool error: {0}")]
    Pool(String),

    #[error("Cache expired")]
    CacheExpired,

    #[error("Cache invalidated")]
    InvalidatedCache,

    #[error("Playlist not found")]
    PlaylistNotFound,
}
