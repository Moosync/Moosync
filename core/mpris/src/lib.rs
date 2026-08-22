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

use std::{
    sync::{
        Arc, Mutex,
        mpsc::{self, Receiver},
    },
    time::Duration,
};

use extensions_proto::moosync::types::PlayerState;
use serde::{Deserialize, Serialize};
#[cfg(target_os = "android")]
use types::android::AndroidJNIContext;
pub mod error;
#[cfg(not(target_os = "android"))]
use crate::context::{DummyContext, SouvlakiMprisContext};
use crate::{context::MprisContext, error::MprisError};

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

#[cfg(target_os = "android")]
mod mpris_android;

#[cfg(test)]
mod lib_test;

mod context;

#[cfg(target_os = "windows")]
mod win32;

pub struct MprisHolder {
    context: Mutex<Box<dyn MprisContext>>,
    pub event_rx: Arc<Mutex<Receiver<crate::MediaControlEvent>>>,
    last_duration: Mutex<u64>,
    last_state: Mutex<PlayerState>,
    #[cfg(target_os = "windows")]
    _dummy_window: Option<crate::win32::DummyWindow>,
}

#[plugin_macro::generate]
impl MprisHolder {
    #[tracing::instrument(level = "debug", skip_all)]
    #[cfg(target_os = "android")]
    pub fn new(android_context: AndroidJNIContext) -> Result<MprisHolder, MprisError> {
        let context = Box::new(mpris_android::AndroidMprisContext::new(android_context));
        Self::new_with_context(context)
    }

    #[cfg(not(target_os = "android"))]
    #[tracing::instrument(level = "debug", skip_all)]
    pub fn new() -> Result<MprisHolder, MprisError> {
        #[cfg(target_os = "windows")]
        {
            // If we cannot determine wine support, just default to assuming it is wine
            let is_wine = is_wine::try_is_wine().unwrap_or(true);
            if is_wine {
                let context = Box::new(DummyContext {});
                return Self::new_with_context(context);
            }
        }

        let context: Box<dyn MprisContext> = match SouvlakiMprisContext::new() {
            Ok(ctx) => Box::new(ctx),
            Err(e) => {
                tracing::warn!(
                    "Failed to create SouvlakiMprisContext: {:?}, using dummy context",
                    e
                );
                Box::new(DummyContext {})
            }
        };

        match Self::new_with_context(context) {
            Ok(holder) => Ok(holder),
            Err(e) => {
                tracing::warn!(
                    "Failed to attach SouvlakiMprisContext: {:?}, using dummy context",
                    e
                );
                Self::new_with_context(Box::new(DummyContext {}))
            }
        }
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub fn new_with_context(mut context: Box<dyn MprisContext>) -> Result<MprisHolder, MprisError> {
        let (event_tx, event_rx) = mpsc::channel();
        context.attach(event_tx)?;

        Ok(MprisHolder {
            context: Mutex::new(context),
            event_rx: Arc::new(Mutex::new(event_rx)),
            last_duration: Mutex::new(0),
            last_state: Mutex::new(PlayerState::Stopped),
            #[cfg(target_os = "windows")]
            _dummy_window: None,
        })
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub fn set_metadata(&self, metadata: MprisPlayerDetails) -> Result<(), MprisError> {
        let mut context = self.context.lock().unwrap();
        context.set_metadata(metadata)
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub fn set_playback_state(&self, state: PlayerState) -> Result<(), MprisError> {
        let last_duration = self.last_duration.lock().unwrap();
        let duration = *last_duration;
        drop(last_duration);

        let mut context = self.context.lock().unwrap();
        context.set_playback_state(state, duration)?;

        let mut last_state = self.last_state.lock().unwrap();
        *last_state = state;
        Ok(())
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub fn set_position(&self, duration: f64) -> Result<(), MprisError> {
        let mut last_duration = self.last_duration.lock().unwrap();
        *last_duration = (duration * 1000.0) as u64;
        drop(last_duration);

        #[allow(clippy::clone_on_copy)]
        let last_state = self.last_state.lock().unwrap().clone();
        self.set_playback_state(last_state)?;
        Ok(())
    }
}

impl types::plugin::Plugin for MprisHolder {
    #[tracing::instrument(level = "debug", skip_all)]
    fn init(
        _context: &types::plugin::PluginContext,
    ) -> types::plugin::Arc<types::plugin::RwLock<Self>> {
        types::plugin::Arc::new(types::plugin::RwLock::new(
            MprisHolder::new(
                #[cfg(target_os = "android")]
                _context.android_context.clone(),
            )
            .expect("Failed to initialize MprisHolder"),
        ))
    }
}
