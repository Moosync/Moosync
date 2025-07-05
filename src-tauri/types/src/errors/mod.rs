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

#[cfg(not(feature = "extensions"))]
use std::{
    fmt::Error as FmtError,
    num::{ParseFloatError, ParseIntError},
    string::FromUtf8Error,
    time::SystemTimeError,
};

#[cfg(all(
    not(feature = "extensions"),
    any(feature = "core", feature = "extensions-core")
))]
use std::io;

#[cfg(all(not(feature = "extensions"), feature = "ui"))]
use serde_json::Value;

#[cfg(all(not(feature = "extensions"), feature = "ui"))]
use wasm_bindgen::JsValue;

#[cfg(all(not(feature = "extensions"), feature = "core"))]
use core::str;


#[cfg(not(feature = "extensions"))]
#[derive(Debug, thiserror::Error)]
pub enum MoosyncError {
    #[cfg_attr(any(feature = "core", feature = "extensions-core"), error(transparent))]
    #[cfg(any(feature = "core", feature = "extensions-core"))]
    IO(#[from] io::Error),
    #[error(transparent)]
    Json(#[from] serde_json::Error),

    #[error("Playback error: {0}")]
    PlaybackError(Box<dyn std::error::Error + Send + Sync>),
    #[error("Database error: {0}")]
    DatabaseError(Box<dyn std::error::Error + Send + Sync>),
    #[error("Network error: {0}")]
    NetworkError(Box<dyn std::error::Error + Send + Sync>),
    #[error("Authentication error: {0}")]
    AuthError(Box<dyn std::error::Error + Send + Sync>),
    #[error("File system error: {0}")]
    FileSystemError(Box<dyn std::error::Error + Send + Sync>),
    #[error("Media error: {0}")]
    MediaError(Box<dyn std::error::Error + Send + Sync>),
    #[error("Configuration error: {0}")]
    ConfigError(Box<dyn std::error::Error + Send + Sync>),
    #[error("Parse error: {0}")]
    ParseError(Box<dyn std::error::Error + Send + Sync>),
    #[error("Validation error: {0}")]
    ValidationError(Box<dyn std::error::Error + Send + Sync>),
    #[error("Provider error: {0}")]
    ProviderError(Box<dyn std::error::Error + Send + Sync>),
    #[error("Extension error: {0}")]
    ExtensionError(Box<dyn std::error::Error + Send + Sync>),
    #[error("Cache error: {0}")]
    CacheError(Box<dyn std::error::Error + Send + Sync>),
    #[error("Webview error: {0}")]
    WebviewError(Box<dyn std::error::Error + Send + Sync>),
    #[error("Plugin error: {0}")]
    PluginError(Box<dyn std::error::Error + Send + Sync>),
    #[error("MPRIS error: {0}")]
    MprisError(Box<dyn std::error::Error + Send + Sync>),
    #[error("{0}")]
    String(String),
    #[cfg(feature = "core")]
    #[error("Transfer control to provider: {0}")]
    SwitchProviders(String),
    #[error("Invalidated cache")]
    InvalidatedCache,
}

#[cfg(all(not(feature = "extensions"), feature = "ui"))]
impl From<serde_wasm_bindgen::Error> for MoosyncError {
    #[tracing::instrument(level = "debug", skip(value))]
    fn from(value: serde_wasm_bindgen::Error) -> Self {
        Self::String(value.to_string())
    }
}

#[cfg(all(not(feature = "extensions"), feature = "ui"))]
impl From<JsValue> for MoosyncError {
    #[tracing::instrument(level = "debug", skip(value))]
    fn from(value: JsValue) -> Self {
        let parsed: Value = serde_wasm_bindgen::from_value(value).unwrap();
        Self::String(format!("{}", parsed))
    }
}

impl From<&'static str> for MoosyncError {
    #[tracing::instrument(level = "debug", skip(value))]
    fn from(value: &'static str) -> Self {
        Self::String(value.to_string())
    }
}

impl From<String> for MoosyncError {
    #[tracing::instrument(level = "debug", skip(value))]
    fn from(value: String) -> Self {
        Self::String(value)
    }
}

#[cfg(not(feature = "extensions"))]
impl From<FmtError> for MoosyncError {
    #[tracing::instrument(level = "debug", skip(value))]
    fn from(value: FmtError) -> Self {
        Self::String(value.to_string())
    }
}

#[cfg(not(feature = "extensions"))]
impl From<ParseFloatError> for MoosyncError {
    #[tracing::instrument(level = "debug", skip(value))]
    fn from(value: ParseFloatError) -> Self {
        Self::ParseError(Box::new(value))
    }
}

#[cfg(not(feature = "extensions"))]
impl From<ParseIntError> for MoosyncError {
    #[tracing::instrument(level = "debug", skip(value))]
    fn from(value: ParseIntError) -> Self {
        Self::ParseError(Box::new(value))
    }
}

#[cfg(not(feature = "extensions"))]
impl From<FromUtf8Error> for MoosyncError {
    #[tracing::instrument(level = "debug", skip(value))]
    fn from(value: FromUtf8Error) -> Self {
        Self::ParseError(Box::new(value))
    }
}

#[cfg(not(feature = "extensions"))]
impl From<SystemTimeError> for MoosyncError {
    #[tracing::instrument(level = "debug", skip(value))]
    fn from(value: SystemTimeError) -> Self {
        Self::FileSystemError(Box::new(value))
    }
}

impl serde::Serialize for MoosyncError {
    #[tracing::instrument(level = "debug", skip(self, serializer))]
    fn serialize<S>(&self, serializer: S) -> core::result::Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer,
    {
        serializer.serialize_str(self.to_string().as_ref())
    }
}

#[cfg(feature = "extensions")]
#[derive(Debug, thiserror::Error)]
pub enum MoosyncError {
    #[error("{0}")]
    String(String),
    #[error(transparent)]
    Json(#[from] serde_json::Error),
}

pub type Result<T> = std::result::Result<T, MoosyncError>;

/// Helper functions for converting errors to MoosyncError variants
/// These can be used with .map_err() directly
pub mod error_helpers {
    use super::MoosyncError;

    pub fn to_playback_error<E: std::error::Error + Send + Sync + 'static>(e: E) -> MoosyncError {
        MoosyncError::PlaybackError(Box::new(e))
    }

    pub fn to_database_error<E: std::error::Error + Send + Sync + 'static>(e: E) -> MoosyncError {
        MoosyncError::DatabaseError(Box::new(e))
    }

    pub fn to_network_error<E: std::error::Error + Send + Sync + 'static>(e: E) -> MoosyncError {
        MoosyncError::NetworkError(Box::new(e))
    }

    pub fn to_auth_error<E: std::error::Error + Send + Sync + 'static>(e: E) -> MoosyncError {
        MoosyncError::AuthError(Box::new(e))
    }

    pub fn to_file_system_error<E: std::error::Error + Send + Sync + 'static>(e: E) -> MoosyncError {
        MoosyncError::FileSystemError(Box::new(e))
    }

    pub fn to_media_error<E: std::error::Error + Send + Sync + 'static>(e: E) -> MoosyncError {
        MoosyncError::MediaError(Box::new(e))
    }

    pub fn to_config_error<E: std::error::Error + Send + Sync + 'static>(e: E) -> MoosyncError {
        MoosyncError::ConfigError(Box::new(e))
    }

    pub fn to_parse_error<E: std::error::Error + Send + Sync + 'static>(e: E) -> MoosyncError {
        MoosyncError::ParseError(Box::new(e))
    }

    pub fn to_validation_error<E: std::error::Error + Send + Sync + 'static>(e: E) -> MoosyncError {
        MoosyncError::ValidationError(Box::new(e))
    }

    pub fn to_provider_error<E: std::error::Error + Send + Sync + 'static>(e: E) -> MoosyncError {
        MoosyncError::ProviderError(Box::new(e))
    }

    pub fn to_extension_error<E: std::error::Error + Send + Sync + 'static>(e: E) -> MoosyncError {
        MoosyncError::ExtensionError(Box::new(e))
    }

    pub fn to_cache_error<E: std::error::Error + Send + Sync + 'static>(e: E) -> MoosyncError {
        MoosyncError::CacheError(Box::new(e))
    }

    pub fn to_webview_error<E: std::error::Error + Send + Sync + 'static>(e: E) -> MoosyncError {
        MoosyncError::WebviewError(Box::new(e))
    }

    pub fn to_plugin_error<E: std::error::Error + Send + Sync + 'static>(e: E) -> MoosyncError {
        MoosyncError::PluginError(Box::new(e))
    }

    pub fn to_mpris_error<E: std::error::Error + Send + Sync + 'static>(e: E) -> MoosyncError {
        MoosyncError::MprisError(Box::new(e))
    }
}

#[macro_export]
macro_rules! moosync_err {
    ($variant:ident, $err:expr) => {
        Err($crate::errors::MoosyncError::$variant(Box::new($err)))
    };
}
