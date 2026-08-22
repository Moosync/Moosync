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
pub enum MprisError {
    #[error("Media controls initialization failed: {0}")]
    InitFailed(String),

    #[error("Failed to attach media controls: {0}")]
    AttachFailed(String),

    #[error("Failed to set metadata: {0}")]
    SetMetadataFailed(String),

    #[error("Failed to set playback state: {0}")]
    SetPlaybackFailed(String),

    #[error("Platform not supported")]
    Unsupported,

    #[cfg(not(target_os = "android"))]
    #[error("Souvlaki integration error: {0:?}")]
    Souvlaki(souvlaki::Error),
}

#[cfg(not(target_os = "android"))]
impl From<souvlaki::Error> for MprisError {
    #[tracing::instrument(level = "trace", skip_all)]
    fn from(err: souvlaki::Error) -> Self { MprisError::Souvlaki(err) }
}
