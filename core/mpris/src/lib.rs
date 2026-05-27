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

use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct MprisPlayerDetails {
    pub id: Option<String>,
    pub title: Option<String>,
    #[serde(rename = "artistName")]
    pub artist_name: Option<String>,
    #[serde(rename = "albumName")]
    pub album_name: Option<String>,
    #[serde(rename = "albumArtist")]
    pub album_artist: Option<String>,
    pub genres: Option<Vec<String>>,
    pub duration: Option<f64>,
    pub thumbnail: Option<String>,
}

// ─────────────────────────────────────────────────────────────────────── //
//  Platform-agnostic MediaControlEvent.                                   //
//  On desktop, souvlaki re-exports the same type names so callers can     //
//  use them interchangeably.                                               //
// ─────────────────────────────────────────────────────────────────────── //

/// Events sent by the OS media controls.
///
/// Defined here (not re-exported from souvlaki) so Android builds compile
/// without the souvlaki dependency.
#[derive(Clone, PartialEq, Debug)]
pub enum MediaControlEvent {
    Play,
    Pause,
    Toggle,
    Next,
    Previous,
    Stop,

    /// Seek forward or backward by an undetermined amount.
    Seek(SeekDirection),
    /// Seek forward or backward by a certain amount.
    SeekBy(SeekDirection, Duration),
    /// Set the position/progress of the currently playing media item.
    SetPosition(MediaPosition),
    /// Sets the volume. The value is intended to be from 0.0 to 1.0.
    SetVolume(f64),
    /// Open the URI in the media player.
    OpenUri(String),
    /// Bring the media player's user interface to the front.
    Raise,
    /// Shut down the media player.
    Quit,
}

/// An instant in a media item.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct MediaPosition(pub Duration);

/// The direction to seek in.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum SeekDirection {
    Forward,
    Backward,
}

// ─────────────────────────────────────────────────────────────────────── //
//  Platform-specific modules.                                              //
// ─────────────────────────────────────────────────────────────────────── //

mod mpris;
pub use mpris::MprisHolder;

#[cfg(target_os = "android")]
mod mpris_android;

#[cfg(test)]
mod tests;

pub mod context;

#[cfg(target_os = "windows")]
mod win32;
