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
// GNU Goueneral Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <http://www.gnu.org/licenses/>.

#[cfg(not(test))]
pub use souvlaki::MediaControlEvent;
#[cfg(test)]
pub use crate::mock_souvlaki::souvlaki::{MediaControlEvent, MediaControls};

use std::{
    sync::{
        mpsc::{self, Receiver},
        Arc, Mutex,
    },
    time::Duration,
};

#[cfg(not(test))]
use souvlaki::{MediaMetadata, MediaPlayback, MediaPosition, PlatformConfig};
#[cfg(test)]
use crate::mock_souvlaki::souvlaki::{MediaMetadata, MediaPlayback, MediaPosition, PlatformConfig};
use types::{
    errors::{MoosyncError, Result},
    mpris::MprisPlayerDetails,
    ui::player_details::PlayerState,
};

pub struct MprisHolder {
    controls: Mutex<MediaControls>,
    pub event_rx: Arc<Mutex<Receiver<MediaControlEvent>>>,
    last_duration: Mutex<u64>,
    last_state: Mutex<PlayerState>,
    #[cfg(target_os = "windows")]
    _dummy_window: Option<windows::DummyWindow>,
}

impl MprisHolder {
    #[tracing::instrument(level = "debug", skip())]
    pub fn new() -> Result<MprisHolder> {
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

        let mut controls =
            MediaControls::new(config).map_err(|e| MoosyncError::String(format!("{:?}", e)))?;

        let (event_tx, event_rx) = mpsc::channel();
        controls
            .attach(move |event| {
                event_tx.send(event).unwrap();
            })
            .map_err(|e| MoosyncError::String(format!("{:?}", e)))?;

        #[cfg(target_os = "windows")]
        std::thread::spawn(move || {
            loop {
                std::thread::sleep(std::time::Duration::from_millis(100));

                // this must be run repeatedly by your program to ensure
                // the Windows event queue is processed by your application
                #[cfg(target_os = "windows")]
                windows::pump_event_queue();
            }
        });

        Ok(MprisHolder {
            controls: Mutex::new(controls),
            event_rx: Arc::new(Mutex::new(event_rx)),
            last_duration: Mutex::new(0),
            last_state: Mutex::new(PlayerState::Stopped),
            #[cfg(target_os = "windows")]
            _dummy_window: Some(_dummy_window),
        })
    }

    #[tracing::instrument(level = "debug", skip(self, metadata))]
    pub fn set_metadata(&self, metadata: MprisPlayerDetails) -> Result<()> {
        let mut controls = self.controls.lock().unwrap();
        let duration = metadata.duration.map(|d| (d * 1000.0) as u64);
        controls
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
            })?;

        Ok(())
    }

    #[tracing::instrument(level = "debug", skip(self, state))]
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
        drop(last_duration);

        let mut controls = self.controls.lock().unwrap();
        controls.set_playback(parsed).map_err(|e| {
            #[cfg(any(target_os = "macos", target_os = "windows"))]
            {
                MoosyncError::String("Failed to set playback state".into())
            }
            #[cfg(all(not(target_os = "macos"), not(target_os = "windows")))]
            {
                MoosyncError::MprisError(Box::new(e))
            }
        })?;
        drop(controls);

        let mut last_state = self.last_state.lock().unwrap();
        *last_state = state;
        Ok(())
    }

    #[tracing::instrument(level = "debug", skip(self, duration))]
    pub fn set_position(&self, duration: f64) -> Result<()> {
        let mut last_duration = self.last_duration.lock().unwrap();
        *last_duration = (duration * 1000.0) as u64;
        drop(last_duration);

        #[allow(clippy::clone_on_copy)]
        let last_state = self.last_state.lock().unwrap().clone();
        self.set_playback_state(last_state)?;
        Ok(())
    }
}

#[cfg(target_os = "windows")]
mod windows {
    use std::io::Error;
    use std::mem;

    use windows::core::PCWSTR;
    use windows::w;
    use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, WPARAM};
    use windows::Win32::System::LibraryLoader::GetModuleHandleW;
    use windows::Win32::UI::WindowsAndMessaging::{
        CreateWindowExW, DefWindowProcW, DestroyWindow, DispatchMessageW, GetAncestor,
        IsDialogMessageW, PeekMessageW, RegisterClassExW, TranslateMessage, GA_ROOT, MSG,
        PM_REMOVE, WINDOW_EX_STYLE, WINDOW_STYLE, WM_QUIT, WNDCLASSEXW,
    };

    pub struct DummyWindow {
        pub handle: HWND,
    }

    impl DummyWindow {
        #[tracing::instrument(level = "debug", skip())]
        pub fn new() -> Result<DummyWindow, String> {
            let class_name = w!("SimpleTray");

            let handle_result = unsafe {
                let instance = GetModuleHandleW(None)
                    .map_err(|e| (format!("Getting module handle failed: {e}")))?;

                let wnd_class = WNDCLASSEXW {
                    cbSize: mem::size_of::<WNDCLASSEXW>() as u32,
                    hInstance: instance,
                    lpszClassName: PCWSTR::from(class_name),
                    lpfnWndProc: Some(Self::wnd_proc),
                    ..Default::default()
                };

                if RegisterClassExW(&wnd_class) == 0 {
                    return Err(format!(
                        "Registering class failed: {}",
                        Error::last_os_error()
                    ));
                }

                let handle = CreateWindowExW(
                    WINDOW_EX_STYLE::default(),
                    class_name,
                    w!(""),
                    WINDOW_STYLE::default(),
                    0,
                    0,
                    0,
                    0,
                    None,
                    None,
                    instance,
                    None,
                );

                if handle.0 == 0 {
                    Err(format!(
                        "Message only window creation failed: {}",
                        Error::last_os_error()
                    ))
                } else {
                    Ok(handle)
                }
            };

            handle_result.map(|handle| DummyWindow { handle })
        }
        #[tracing::instrument(level = "debug", skip(hwnd, msg, wparam, lparam))]
        extern "system" fn wnd_proc(
            hwnd: HWND,
            msg: u32,
            wparam: WPARAM,
            lparam: LPARAM,
        ) -> LRESULT {
            unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) }
        }
    }

    impl Drop for DummyWindow {
        #[tracing::instrument(level = "debug", skip(self))]
        fn drop(&mut self) {
            unsafe {
                DestroyWindow(self.handle);
            }
        }
    }

    #[tracing::instrument(level = "debug", skip())]
    pub fn pump_event_queue() -> bool {
        unsafe {
            let mut msg: MSG = std::mem::zeroed();
            let mut has_message = PeekMessageW(&mut msg, None, 0, 0, PM_REMOVE).as_bool();
            while msg.message != WM_QUIT && has_message {
                if !IsDialogMessageW(GetAncestor(msg.hwnd, GA_ROOT), &msg).as_bool() {
                    TranslateMessage(&msg);
                    DispatchMessageW(&msg);
                }

                has_message = PeekMessageW(&mut msg, None, 0, 0, PM_REMOVE).as_bool();
            }

            msg.message == WM_QUIT
        }
    }
}
