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
    collections::HashMap,
    sync::{
        mpsc::{self, Receiver, Sender},
        Arc, Mutex,
    },
    time::Duration,
};

use serde_json::Value;
use tauri::{AppHandle, Listener};
use tauri_plugin_audioplayer::AudioplayerExt;
use types::{errors::Result, mpris::MprisPlayerDetails, ui::player_details::PlayerState};

pub struct MprisHolder {
    last_duration: Mutex<u64>,
    last_state: Mutex<PlayerState>,
    pub event_rx: Arc<Mutex<Receiver<MediaControlEvent>>>,
    pub event_tx: Arc<Mutex<Sender<MediaControlEvent>>>,
    pub app_handle: Mutex<Option<AppHandle>>,
}

impl MprisHolder {
    #[tracing::instrument(level = "debug", skip())]
    pub fn new() -> Result<MprisHolder> {
        let (event_tx, event_rx) = mpsc::channel();
        Ok(MprisHolder {
            last_duration: Mutex::new(0),
            last_state: Mutex::new(PlayerState::Stopped),
            event_rx: Arc::new(Mutex::new(event_rx)),
            event_tx: Arc::new(Mutex::new(event_tx)),
            app_handle: Default::default(),
        })
    }

    pub fn set_app_handle(&self, app: AppHandle) {
        let ev_tx = self.event_tx.clone();
        app.listen("MediaSessionCallback", move |event| {
            let mut payload: HashMap<String, Value> =
                serde_json::from_str(event.payload()).unwrap();
            let event: String =
                serde_json::from_value(payload.get_mut("event").unwrap().take()).unwrap();
            let media_control_ev = match event.as_str() {
                "onPlay" => Some(MediaControlEvent::Play),
                "onPause" => Some(MediaControlEvent::Pause),
                "onStop" => Some(MediaControlEvent::Stop),
                "onSeekTo" => {
                    let millis =
                        serde_json::from_value(payload.get_mut("pos").unwrap().take()).unwrap();
                    Some(MediaControlEvent::SetPosition(MediaPosition(
                        Duration::from_millis(millis),
                    )))
                }
                _ => None,
            };

            if let Some(ev) = media_control_ev {
                let ev_tx = ev_tx.lock().unwrap();
                ev_tx.send(ev).unwrap();
            }
        });

        let mut app_handle = self.app_handle.lock().unwrap();
        *app_handle = Some(app)
    }

    #[tracing::instrument(level = "debug", skip(self, metadata))]
    pub fn set_metadata(&self, mut metadata: MprisPlayerDetails) -> Result<()> {
        let app = self.app_handle.lock().unwrap();
        if let Some(app) = app.as_ref() {
            let player = app.audioplayer();

            metadata.duration = metadata.duration.map(|d| d * 1000f64);
            player.update_notification(metadata)?;
        }
        Ok(())
    }

    #[tracing::instrument(level = "debug", skip(self, state))]
    pub fn set_playback_state(&self, state: PlayerState) -> Result<()> {
        let app = self.app_handle.lock().unwrap();
        if let Some(app) = app.as_ref() {
            {
                let mut last_state = self.last_state.lock().unwrap();
                *last_state = state;
            }
            let last_duration = self.last_duration.lock().unwrap();
            let player = app.audioplayer();
            player.update_notification_state(state == PlayerState::Playing, *last_duration)?;
        }
        Ok(())
    }

    #[tracing::instrument(level = "debug", skip(self, duration))]
    pub fn set_position(&self, duration: f64) -> Result<()> {
        let app = self.app_handle.lock().unwrap();
        if let Some(app) = app.as_ref() {
            {
                let mut last_duration = self.last_duration.lock().unwrap();
                *last_duration = (duration * 1000f64) as u64;
            }

            let last_state = self.last_state.lock().unwrap();
            let player = app.audioplayer();
            player.update_notification_state(
                *last_state == PlayerState::Playing,
                (duration * 1000f64) as u64,
            )?;
        }
        Ok(())
    }
}

/// Events sent by the OS media controls.
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
    /// But other values are also accepted. **It is up to the user to
    /// set constraints on this value.**
    /// **NOTE**: If the volume event was received and correctly handled,
    /// the user must call [`MediaControls::set_volume`]. Note that
    /// this must be done only with the MPRIS backend.
    SetVolume(f64),
    /// Open the URI in the media player.
    OpenUri(String),

    /// Bring the media player's user interface to the front using any appropriate mechanism available.
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
