#[derive(Debug, thiserror::Error)]
pub enum UiError {
    #[error("Database error: {0}")]
    Database(#[from] database::error::DatabaseError),

    #[error("Extension error: {0}")]
    Extension(#[from] extensions::ExtensionError),

    #[error("Failed to parse entity")]
    EntityParseFailed,
}
