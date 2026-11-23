use std::io;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum CacheError {
    #[error("Cache I/O error: {0}")]
    Io(#[from] io::Error),

    #[error("Cache write callback error: {0}")]
    WriteCallback(String),

    #[error("Cache database error: {0}")]
    Database(String),

    #[error("Cache backend not initialized")]
    BackendNotInitialized,
}
