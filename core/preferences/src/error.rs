use std::string::FromUtf8Error;

#[derive(Debug, thiserror::Error)]
pub enum PreferencesError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Keyring access failed: {0}")]
    Keyring(#[from] keyring::Error),

    #[error("Decryption failed: {0}")]
    Decryption(chacha20poly1305::Error),

    #[error("Encryption failed: {0}")]
    Encryption(chacha20poly1305::Error),

    #[error("Hex decode failed: {0}")]
    HexDecode(#[from] hex::FromHexError),

    #[error("UTF-8 decode failed: {0}")]
    Utf8(#[from] FromUtf8Error),

    #[error("Key not found: {0}")]
    KeyNotFound(String),

    #[error("Invalid extension key: {0}")]
    InvalidExtensionKey(String),

    #[error("Decode error: {0}")]
    Decode(#[from] prost::DecodeError),

    #[error("Encode error: {0}")]
    Encode(#[from] prost::EncodeError),
}
