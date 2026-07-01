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

#[derive(Debug, thiserror::Error)]
pub enum ExtensionError {
    #[error("Extism runtime error: {0}")]
    Extism(#[from] extism::Error),

    #[error("Not implemented")]
    NotImplemented,

    #[error("Missing song in request")]
    MissingSong,

    #[error("Missing command")]
    MissingCommand,

    #[error("Missing playlist in request")]
    MissingPlaylist,

    #[error("Not a valid extension")]
    NotAnExtension,

    #[error("Extension not found")]
    NoExtensionFound,

    #[error("No extension context")]
    NoContext,

    #[error("Invalid extension response")]
    InvalidResponse,

    #[error("Extension icon not found: {0}")]
    NoExtensionIconFound(String),

    #[error("Duplicate extension: {0}")]
    DuplicateExtension(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Filesystem copy error: {0}")]
    FsExtra(#[from] fs_extra::error::Error),

    #[error("JSON error: {0}")]
    Serde(#[from] serde_json::Error),

    #[error("Zip extraction error: {0}")]
    Zip(String),

    #[error("HTTP error: {0}")]
    Reqwest(#[from] reqwest::Error),

    #[error("Extension version parse error: {0}")]
    VersionParse(#[from] std::num::ParseIntError),

    #[error("Response sanitization error: {0}")]
    Sanitize(String),
}
