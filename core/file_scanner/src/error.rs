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
}
