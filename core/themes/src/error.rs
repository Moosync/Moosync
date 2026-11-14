#[derive(Debug, thiserror::Error)]
pub enum ThemesError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Protobuf decode error: {0}")]
    Decode(#[from] prost::DecodeError),

    #[error("HTTP request failed: {0}")]
    Network(#[from] reqwest::Error),

    #[error("File system extra error: {0}")]
    FsExtra(#[from] fs_extra::error::Error),

    #[error("Zip error: {0}")]
    Zip(String),

    #[error("Theme not found")]
    ThemeNotFound,

    #[error("Failed to parse theme")]
    ParseThemeFailed,

    #[error("Invalid theme response: {0}")]
    InvalidResponse(String),
}
