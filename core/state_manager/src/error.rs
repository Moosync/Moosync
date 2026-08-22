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

#[derive(Debug, thiserror::Error)]
pub enum StateManagerError {
    #[error("Database error: {0}")]
    Database(#[from] database::error::DatabaseError),

    #[error("Extension error: {0}")]
    Extension(#[from] extensions::ExtensionError),

    #[error("File scanner error: {0}")]
    Scanner(#[from] file_scanner::error::ScannerError),

    #[error("Lyrics error: {0}")]
    Lyrics(#[from] lyrics::error::LyricsError),

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
