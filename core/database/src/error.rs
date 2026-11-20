use std::time::SystemTimeError;

#[derive(Debug, thiserror::Error)]
pub enum DatabaseError {
    #[error("SQLite query failed")]
    Query(#[source] sqlx::Error),

    #[error("SQLite connection failed")]
    Connection(#[source] sqlx::Error),

    #[error("SQLite migration failed")]
    Migration(#[from] sqlx::migrate::MigrateError),

    #[error("SQLite transaction failed")]
    Transaction(#[source] sqlx::Error),

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
