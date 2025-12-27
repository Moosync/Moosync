use souvlaki::{MediaControlEvent, MediaControls, MediaMetadata, MediaPlayback, MediaPosition, PlatformConfig};
#[cfg(test)]
use mockall::mock;

use std::time::Duration;

use types::{
    errors::{MoosyncError, Result},
    mpris::MprisPlayerDetails,
    ui::player_details::PlayerState,
};

pub trait MprisContext: Send + Sync {
    fn attach(&mut self, sender: std::sync::mpsc::Sender<MediaControlEvent>) -> Result<()>;
    fn set_metadata(&mut self, metadata: MprisPlayerDetails) -> Result<()>;
    fn set_playback_state(&mut self, state: PlayerState, duration: u64) -> Result<()>;
}

#[cfg(test)]
mock! {
    pub MprisContext {}
    impl MprisContext for MprisContext {
        fn attach(&mut self, sender: std::sync::mpsc::Sender<MediaControlEvent>) -> Result<()>;
        fn set_metadata(&mut self, metadata: MprisPlayerDetails) -> Result<()>;
        fn set_playback_state(&mut self, state: PlayerState, duration: u64) -> Result<()>;
    }
}

pub struct SouvlakiMprisContext {
    controls: MediaControls,
}

impl SouvlakiMprisContext {
    pub fn new() -> Result<Self> {
        #[cfg(not(target_os = "windows"))]
        let hwnd = None;

        #[cfg(target_os = "windows")]
        let (hwnd, _dummy_window) = {
            let dummy_window = windows::DummyWindow::new().unwrap();
            let handle = Some(dummy_window.handle.0 as _);
            (handle, dummy_window)
        };

        let config = PlatformConfig {
            display_name: "Moosync",
            dbus_name: "moosync",
            hwnd,
        };

        let controls =
            MediaControls::new(config).map_err(|e| MoosyncError::String(format!("{:?}", e)))?;

        #[cfg(target_os = "windows")]
        std::thread::spawn(move || {
            loop {
                std::thread::sleep(std::time::Duration::from_millis(100));
                #[cfg(target_os = "windows")]
                windows::pump_event_queue();
            }
        });

        Ok(Self { controls })
    }
}

impl MprisContext for SouvlakiMprisContext {
    fn attach(&mut self, sender: std::sync::mpsc::Sender<MediaControlEvent>) -> Result<()> {
        self.controls
            .attach(move |event| {
                sender.send(event).unwrap();
            })
            .map_err(|e| MoosyncError::String(format!("{:?}", e)))
    }

    fn set_metadata(&mut self, metadata: MprisPlayerDetails) -> Result<()> {
        let duration = metadata.duration.map(|d| (d * 1000.0) as u64);
        self.controls
            .set_metadata(MediaMetadata {
                title: metadata.title.as_deref(),
                album: metadata.album_name.as_deref(),
                artist: metadata.artist_name.as_deref(),
                cover_url: metadata.thumbnail.as_deref(),
                duration: duration.map(Duration::from_millis),
            })
            .map_err(|e| {
                #[cfg(any(target_os = "macos", target_os = "windows"))]
                {
                    MoosyncError::String("Failed to set metadata".into())
                }
                #[cfg(all(not(target_os = "macos"), not(target_os = "windows")))]
                {
                    MoosyncError::MprisError(Box::new(e))
                }
            })
    }

    fn set_playback_state(&mut self, state: PlayerState, duration: u64) -> Result<()> {
        let parsed = match state {
            PlayerState::Playing => MediaPlayback::Playing {
                progress: Some(MediaPosition(Duration::from_millis(
                    duration,
                ))),
            },
            PlayerState::Paused | PlayerState::Loading => MediaPlayback::Paused {
                progress: Some(MediaPosition(Duration::from_millis(
                    duration,
                ))),
            },
            PlayerState::Stopped => MediaPlayback::Stopped,
        };

        self.controls.set_playback(parsed).map_err(|e| {
            #[cfg(any(target_os = "macos", target_os = "windows"))]
            {
                MoosyncError::String("Failed to set playback state".into())
            }
            #[cfg(all(not(target_os = "macos"), not(target_os = "windows")))]
            {
                MoosyncError::MprisError(Box::new(e))
            }
        })
    }
}
