use std::{
    sync::{
        mpsc::{self, Receiver},
        Arc, Mutex,
    },
    time::Duration,
};

use types::{errors::Result, mpris::MprisPlayerDetails, ui::player_details::PlayerState};

pub struct MprisHolder {
    last_duration: Mutex<u64>,
    last_state: Mutex<PlayerState>,
    pub event_rx: Arc<Mutex<Receiver<MediaControlEvent>>>,
}

impl MprisHolder {
    #[tracing::instrument(level = "trace", skip())]
    pub fn new() -> Result<MprisHolder> {
        let (_event_tx, event_rx) = mpsc::channel();
        Ok(MprisHolder {
            last_duration: Mutex::new(0),
            last_state: Mutex::new(PlayerState::Stopped),
            event_rx: Arc::new(Mutex::new(event_rx)),
        })
    }

    #[tracing::instrument(level = "trace", skip(self, metadata))]
    pub fn set_metadata(&self, metadata: MprisPlayerDetails) -> Result<()> {
        Ok(())
    }

    #[tracing::instrument(level = "trace", skip(self, state))]
    pub fn set_playback_state(&self, state: PlayerState) -> Result<()> {
        Ok(())
    }

    #[tracing::instrument(level = "trace", skip(self, duration))]
    pub fn set_position(&self, duration: f64) -> Result<()> {
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
