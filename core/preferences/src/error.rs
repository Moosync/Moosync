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

use std::string::FromUtf8Error;

#[derive(Debug, thiserror::Error)]
pub enum PreferencesError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Keyring access failed: {0}")]
    Keyring(#[from] keyring::Error),

    #[error("Decryption failed: {0}")]
    Decryption(chacha20poly1305::aead::Error),

    #[error("Encryption failed: {0}")]
    Encryption(chacha20poly1305::aead::Error),

    #[error("Hex decode failed: {0}")]
    HexDecode(#[from] hex::FromHexError),

    #[error("UTF-8 decode failed: {0}")]
    Utf8(#[from] FromUtf8Error),

    #[error("Key not found: {0}")]
    KeyNotFound(String),
}
