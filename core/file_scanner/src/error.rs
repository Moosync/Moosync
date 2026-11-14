#[derive(Debug, thiserror::Error)]
pub enum ScannerError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Image processing error: {0}")]
    Image(#[from] image::ImageError),

    #[error("Audio metadata read error: {0}")]
    AudioMeta(#[from] lofty::error::LoftyError),

    #[error("Parse float error: {0}")]
    ParseFloat(#[from] std::num::ParseFloatError),

    #[error("Task join error: {0}")]
    Join(#[from] tokio::task::JoinError),

    #[error("Image buffer error: {0}")]
    ImageBuffer(#[from] fast_image_resize::ImageBufferError),

    #[error("Image resize error: {0}")]
    Resize(#[from] fast_image_resize::ResizeError),

    #[error("Protobuf decode error: {0}")]
    Decode(#[from] prost::DecodeError),

    #[error("JNI error: {0}")]
    Jni(String),

    #[error("Scan directories not configured")]
    ScanDirsNotConfigured,

    #[error("Thumbnail directory not configured")]
    ThumbnailDirNotConfigured,

    #[error("Song callback not configured")]
    SongCallbackNotConfigured,

    #[error("Playlist callback not configured")]
    PlaylistCallbackNotConfigured,

    #[error("Invalid image dimensions")]
    InvalidImageDimensions,

    #[error("Audio permission denied by user")]
    PermissionDenied,
}
