use std::{fmt::Display, num::ParseIntError};

use types::errors::MoosyncError;

#[derive(Debug)]
pub enum ExtensionError {
    ExtismError(extism::Error),
    NotAnExtension,
    NoExtensionFound,
    InvalidResponse,
    NoExtensionIconFound(String),
    DuplicateExtension(String),
    IoError(Box<dyn std::error::Error + Send + Sync>),
    SerdeError(Box<dyn std::error::Error + Send + Sync>),
    ZipError(Box<dyn std::error::Error + Send + Sync>),
    ReqwestError(Box<dyn std::error::Error + Send + Sync>),
    ExtVersionError(Box<dyn std::error::Error + Send + Sync>),
    ParseError(Box<dyn std::error::Error + Send + Sync>),
}

impl Display for ExtensionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ExtensionError::ExtismError(e) => write!(f, "{}", e),

            ExtensionError::IoError(e) => write!(f, "IO Error: {}", e),
            ExtensionError::SerdeError(e) => write!(f, "Serde Error: {}", e),
            ExtensionError::ZipError(e) => write!(f, "Zip Error: {}", e),
            ExtensionError::ReqwestError(e) => write!(f, "Reqwest Error: {}", e),
            ExtensionError::ExtVersionError(e) => write!(f, "Version Error: {}", e),
            ExtensionError::ParseError(e) => write!(f, "Parse Error: {}", e),

            ExtensionError::NotAnExtension => write!(f, "Not an extension"),
            ExtensionError::NoExtensionFound => write!(f, "No extension found"),
            ExtensionError::InvalidResponse => write!(f, "Invalid response"),
            ExtensionError::NoExtensionIconFound(s) => write!(f, "No icon found: {}", s),
            ExtensionError::DuplicateExtension(s) => write!(f, "Duplicate extension: {}", s),
        }
    }
}

impl std::error::Error for ExtensionError {}

impl From<std::io::Error> for ExtensionError {
    #[tracing::instrument(level = "debug", skip(value))]
    fn from(value: std::io::Error) -> Self {
        Self::IoError(Box::new(value))
    }
}

impl From<fs_extra::error::Error> for ExtensionError {
    fn from(value: fs_extra::error::Error) -> Self {
        Self::IoError(Box::new(value))
    }
}

impl From<serde_json::Error> for ExtensionError {
    fn from(value: serde_json::Error) -> Self {
        Self::SerdeError(Box::new(value))
    }
}

impl From<reqwest::Error> for ExtensionError {
    fn from(value: reqwest::Error) -> Self {
        Self::ReqwestError(Box::new(value))
    }
}

impl From<extism::Error> for ExtensionError {
    fn from(value: extism::Error) -> Self {
        Self::ExtismError(value)
    }
}

impl From<ParseIntError> for ExtensionError {
    fn from(value: ParseIntError) -> Self {
        Self::ExtVersionError(Box::new(value))
    }
}

impl From<MoosyncError> for ExtensionError {
    fn from(value: MoosyncError) -> Self {
        Self::ParseError(Box::new(value))
    }
}

impl From<ExtensionError> for MoosyncError {
    fn from(value: ExtensionError) -> MoosyncError {
        MoosyncError::ExtensionError(Box::new(value))
    }
}
