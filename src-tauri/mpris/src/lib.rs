pub use souvlaki::{MediaControlEvent, SeekDirection};
use std::{
    sync::{
        mpsc::{self, Receiver},
        Arc, Mutex,
    },
    time::Duration,
};

use souvlaki::{MediaControls, MediaMetadata, MediaPlayback, MediaPosition, PlatformConfig};
use types::{errors::errors::Result, mpris::MprisPlayerDetails, ui::player_details::PlayerState};

#[derive(Debug)]
pub struct MprisHolder {
    controls: Mutex<MediaControls>,
    pub event_rx: Arc<Mutex<Receiver<MediaControlEvent>>>,
    last_duration: Mutex<u64>,
    last_state: Mutex<PlayerState>,
}

impl MprisHolder {
    #[tracing::instrument(level = "trace", skip())]
    pub fn new() -> Result<MprisHolder> {
        #[cfg(not(target_os = "windows"))]
        let hwnd = None;

        #[cfg(target_os = "windows")]
        let hwnd = {
            use raw_window_handle::{HasRawWindowHandle, RawWindowHandle};

            let handle = match window.raw_window_handle() {
                RawWindowHandle::Win32(h) => h,
                _ => unreachable!(),
            };
            Some(handle.hwnd)
        };

        let config = PlatformConfig {
            display_name: "Moosync",
            dbus_name: "moosync",
            hwnd,
        };

        let mut controls = MediaControls::new(config)?;

        let (event_tx, event_rx) = mpsc::channel();
        controls.attach(move |event| {
            event_tx.send(event).unwrap();
        })?;

        Ok(MprisHolder {
            controls: Mutex::new(controls),
            event_rx: Arc::new(Mutex::new(event_rx)),
            last_duration: Mutex::new(0),
            last_state: Mutex::new(PlayerState::Stopped),
        })
    }

    #[tracing::instrument(level = "trace", skip(self, metadata))]
    pub fn set_metadata(&self, metadata: MprisPlayerDetails) -> Result<()> {
        let mut controls = self.controls.lock().unwrap();
        let duration = metadata.duration.map(|d| (d * 1000.0) as u64);
        controls.set_metadata(MediaMetadata {
            title: metadata.title.as_deref(),
            album: metadata.album_name.as_deref(),
            artist: metadata.artist_name.as_deref(),
            cover_url: metadata.thumbnail.as_deref(),
            duration: duration.map(Duration::from_millis),
        })?;

        Ok(())
    }

    #[tracing::instrument(level = "trace", skip(self, state))]
    pub fn set_playback_state(&self, state: PlayerState) -> Result<()> {
        let last_duration = self.last_duration.lock().unwrap();
        let parsed = match state {
            PlayerState::Playing => MediaPlayback::Playing {
                progress: Some(MediaPosition(Duration::from_millis(
                    last_duration.to_owned(),
                ))),
            },
            PlayerState::Paused | PlayerState::Loading => MediaPlayback::Paused {
                progress: Some(MediaPosition(Duration::from_millis(
                    last_duration.to_owned(),
                ))),
            },
            PlayerState::Stopped => MediaPlayback::Stopped,
        };

        let mut controls = self.controls.lock().unwrap();
        controls.set_playback(parsed)?;
        drop(controls);

        let mut last_state = self.last_state.lock().unwrap();
        *last_state = state;
        Ok(())
    }

    #[tracing::instrument(level = "trace", skip(self, duration))]
    pub fn set_position(&self, duration: f64) -> Result<()> {
        let mut last_duration = self.last_duration.lock().unwrap();
        *last_duration = (duration * 1000.0) as u64;
        drop(last_duration);

        let last_state = self.last_state.lock().unwrap();
        #[allow(clippy::clone_on_copy)]
        self.set_playback_state(last_state.clone())?;
        Ok(())
    }
}
