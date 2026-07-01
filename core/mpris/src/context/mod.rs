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

#[cfg(not(target_os = "android"))]
use std::time::Duration;

use extensions_proto::moosync::types::PlayerState;
#[cfg(test)]
use mockall::mock;
#[cfg(not(target_os = "android"))]
use souvlaki::{
    MediaControls, MediaMetadata, MediaPlayback, MediaPosition as SouvlakiMediaPosition,
    PlatformConfig,
};

#[cfg(not(target_os = "android"))]
use crate::SeekDirection;
#[cfg(not(target_os = "android"))]
use crate::error::MprisError;
use crate::{MediaControlEvent, MprisPlayerDetails};

// ─────────────────────────────────────────────────────────────────────── //
//  MprisContext trait — platform-agnostic interface.                       //
//  Uses crate-level MediaControlEvent (defined in lib.rs).                //
// ─────────────────────────────────────────────────────────────────────── //

pub trait MprisContext: Send + Sync {
    fn attach(
        &mut self,
        sender: std::sync::mpsc::Sender<MediaControlEvent>,
    ) -> Result<(), MprisError>;
    fn set_metadata(&mut self, metadata: MprisPlayerDetails) -> Result<(), MprisError>;
    fn set_playback_state(&mut self, state: PlayerState, duration: u64) -> Result<(), MprisError>;
}

#[cfg(test)]
mock! {
    pub MprisContext {}
    impl MprisContext for MprisContext {
        fn attach(&mut self, sender: std::sync::mpsc::Sender<MediaControlEvent>) -> Result<(), MprisError>;
        fn set_metadata(&mut self, metadata: MprisPlayerDetails) -> Result<(), MprisError>;
        fn set_playback_state(&mut self, state: PlayerState, duration: u64) -> Result<(), MprisError>;
    }
}

// ─────────────────────────────────────────────────────────────────────── //
//  SouvlakiMprisContext — desktop implementation backed by souvlaki.     //
// ─────────────────────────────────────────────────────────────────────── //

#[cfg(not(target_os = "android"))]
pub struct SouvlakiMprisContext {
    controls: MediaControls,
}

#[cfg(not(target_os = "android"))]
impl SouvlakiMprisContext {
    #[tracing::instrument(level = "debug", skip_all)]
    pub fn new() -> Result<Self, MprisError> {
        #[cfg(not(target_os = "windows"))]
        let hwnd = None;

        #[cfg(target_os = "windows")]
        let (hwnd, _dummy_window) = {
            let dummy_window = crate::win32::DummyWindow::new().unwrap();
            let handle = Some(dummy_window.handle.0 as _);
            (handle, dummy_window)
        };

        let config = PlatformConfig {
            display_name: "Moosync",
            dbus_name: "moosync",
            hwnd,
        };

        let controls =
            MediaControls::new(config).map_err(|e| MprisError::InitFailed(format!("{:?}", e)))?;

        #[cfg(target_os = "windows")]
        std::thread::spawn(move || {
            loop {
                std::thread::sleep(std::time::Duration::from_millis(100));
                #[cfg(target_os = "windows")]
                crate::win32::pump_event_queue();
            }
        });

        Ok(Self { controls })
    }
}

/// Map our crate-level `MediaControlEvent` to souvlaki's event type for the
/// closure. souvlaki calls back with *its own* `MediaControlEvent`; we convert
/// to ours.
#[cfg(not(target_os = "android"))]
#[tracing::instrument(level = "debug", skip_all)]
fn from_souvlaki_event(e: souvlaki::MediaControlEvent) -> MediaControlEvent {
    match e {
        souvlaki::MediaControlEvent::Play => MediaControlEvent::Play,
        souvlaki::MediaControlEvent::Pause => MediaControlEvent::Pause,
        souvlaki::MediaControlEvent::Toggle => MediaControlEvent::Toggle,
        souvlaki::MediaControlEvent::Next => MediaControlEvent::Next,
        souvlaki::MediaControlEvent::Previous => MediaControlEvent::Previous,
        souvlaki::MediaControlEvent::Stop => MediaControlEvent::Stop,
        souvlaki::MediaControlEvent::Seek(d) => MediaControlEvent::Seek(match d {
            souvlaki::SeekDirection::Forward => SeekDirection::Forward,
            souvlaki::SeekDirection::Backward => SeekDirection::Backward,
        }),
        souvlaki::MediaControlEvent::SeekBy(d, dur) => MediaControlEvent::SeekBy(
            match d {
                souvlaki::SeekDirection::Forward => SeekDirection::Forward,
                souvlaki::SeekDirection::Backward => SeekDirection::Backward,
            },
            dur,
        ),
        souvlaki::MediaControlEvent::SetPosition(p) => {
            MediaControlEvent::SetPosition(crate::MediaPosition(p.0))
        }
        souvlaki::MediaControlEvent::SetVolume(v) => MediaControlEvent::SetVolume(v),
        souvlaki::MediaControlEvent::OpenUri(u) => MediaControlEvent::OpenUri(u),
        souvlaki::MediaControlEvent::Raise => MediaControlEvent::Raise,
        souvlaki::MediaControlEvent::Quit => MediaControlEvent::Quit,
    }
}

#[cfg(not(target_os = "android"))]
impl MprisContext for SouvlakiMprisContext {
    #[tracing::instrument(level = "debug", skip_all)]
    fn attach(
        &mut self,
        sender: std::sync::mpsc::Sender<MediaControlEvent>,
    ) -> Result<(), MprisError> {
        self.controls
            .attach(move |event| {
                let mapped = from_souvlaki_event(event);
                sender.send(mapped).unwrap();
            })
            .map_err(|e| MprisError::AttachFailed(format!("{:?}", e)))
    }

    #[tracing::instrument(level = "debug", skip_all)]
    fn set_metadata(&mut self, metadata: MprisPlayerDetails) -> Result<(), MprisError> {
        let duration = metadata.duration.map(|d| (d * 1000.0) as u64);
        self.controls
            .set_metadata(MediaMetadata {
                title: metadata.title.as_deref(),
                album: metadata.album_name.as_deref(),
                artist: metadata.artist_name.as_deref(),
                cover_url: metadata.thumbnail.as_deref(),
                duration: duration.map(Duration::from_millis),
            })
            .map_err(MprisError::Souvlaki)
    }

    #[tracing::instrument(level = "debug", skip_all)]
    fn set_playback_state(&mut self, state: PlayerState, duration: u64) -> Result<(), MprisError> {
        let parsed = match state {
            PlayerState::Playing => MediaPlayback::Playing {
                progress: Some(SouvlakiMediaPosition(Duration::from_millis(duration))),
            },
            PlayerState::Paused | PlayerState::Loading => MediaPlayback::Paused {
                progress: Some(SouvlakiMediaPosition(Duration::from_millis(duration))),
            },
            PlayerState::Stopped => MediaPlayback::Stopped,
        };

        self.controls
            .set_playback(parsed)
            .map_err(MprisError::Souvlaki)
    }
}

// ─────────────────────────────────────────────────────────────────────── //
//  DummyContext — used on Wine / unsupported platforms.                   //
// ─────────────────────────────────────────────────────────────────────── //

#[cfg(target_os = "windows")]
pub struct DummyContext {}

#[cfg(target_os = "windows")]
impl MprisContext for DummyContext {
    #[tracing::instrument(level = "debug", skip_all)]
    fn attach(&mut self, _: std::sync::mpsc::Sender<MediaControlEvent>) -> Result<(), MprisError> {
        Ok(())
    }

    #[tracing::instrument(level = "debug", skip_all)]
    fn set_metadata(&mut self, _: MprisPlayerDetails) -> Result<(), MprisError> { Ok(()) }

    #[tracing::instrument(level = "debug", skip_all)]
    fn set_playback_state(&mut self, _: PlayerState, _: u64) -> Result<(), MprisError> { Ok(()) }
}
