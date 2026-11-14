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

    #[error("Zip extraction error: {0}")]
    Zip(String),

    #[error("HTTP error: {0}")]
    Reqwest(#[from] reqwest::Error),

    #[error("Extension version parse error: {0}")]
    VersionParse(#[from] semver::Error),

    #[error("Protobuf decode error: {0}")]
    Decode(#[from] prost::DecodeError),

    #[error("Response sanitization error: {0}")]
    Sanitize(String),

    #[error("Extension update failed: {0}")]
    UpdateFailed(String),

    #[error("Cache error: {0}")]
    Cache(String),

    #[error("No allowed paths configured for extension")]
    ProcessNoAllowedPaths,

    #[error("Binary not found at: {0}")]
    ProcessBinaryNotFound(std::path::PathBuf),

    #[error("Binary at {0} is outside allowed extension paths")]
    ProcessBinaryNotPermitted(std::path::PathBuf),

    #[error("Binary hash {0} is not declared in manifest permissions")]
    ProcessHashNotPermitted(String),

    #[error("Failed to read binary file at {0}: {1}")]
    ProcessFailedToReadBinary(std::path::PathBuf, String),

    #[error("Failed to spawn process: {0}")]
    ProcessFailedToSpawn(String),

    #[error("Process {0} not found")]
    ProcessNotFound(usize),

    #[error("Process {0} stdin is not open")]
    ProcessStdinNotOpen(usize),

    #[error("Failed to write to process {0} stdin: {1}")]
    ProcessFailedToWrite(usize, String),

    #[error("Failed to flush process {0} stdin: {1}")]
    ProcessFailedToFlush(usize, String),

    #[error("Process error: {0}")]
    Process(String),

    #[error("WebSocket host '{0}' is not allowed by manifest permissions")]
    WsHostNotPermitted(String),

    #[error("Failed to connect to WebSocket '{0}': {1}")]
    WsFailedToConnect(String, String),

    #[error("WebSocket connection {0} not found")]
    WsConnectionNotFound(usize),

    #[error("WebSocket connection {0} is closed")]
    WsConnectionClosed(usize),

    #[error("Failed to send WebSocket message on connection {0}: {1}")]
    WsFailedToSend(usize, String),

    #[error("WebSocket error: {0}")]
    Ws(String),
}
