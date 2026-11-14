#[derive(Debug, thiserror::Error)]
pub enum StateManagerError {
    #[error("Database error: {0}")]
    Database(#[from] database::error::DatabaseError),

    #[error("Extension error: {0}")]
    Extension(#[from] extensions::ExtensionError),

    #[error("File scanner error: {0}")]
    Scanner(#[from] file_scanner::error::ScannerError),

    #[error("Preferences error: {0}")]
    Preferences(#[from] preferences::error::PreferencesError),

    #[error("Player error: {0}")]
    Player(#[from] player::error::PlayerError),

    #[error("Themes error: {0}")]
    Themes(#[from] themes::error::ThemesError),

    #[error("MPRIS error: {0}")]
    Mpris(#[from] mpris::error::MprisError),

    #[error("Types parsing error: {0}")]
    Types(#[from] types::errors::TypesError),

    #[error("Standard IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("General error: {0}")]
    General(String),
}
