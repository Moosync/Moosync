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

pub use souvlaki::MediaControlEvent;

use std::sync::{
    mpsc::{self, Receiver},
    Arc, Mutex,
};

use types::{
    errors::Result,
    mpris::MprisPlayerDetails,
    ui::player_details::PlayerState,
};

use crate::context::{MprisContext, SouvlakiMprisContext};

pub struct MprisHolder {
    context: Mutex<Box<dyn MprisContext>>,
    pub event_rx: Arc<Mutex<Receiver<MediaControlEvent>>>,
    last_duration: Mutex<u64>,
    last_state: Mutex<PlayerState>,
    #[cfg(target_os = "windows")]
    _dummy_window: Option<windows::DummyWindow>,
}

impl MprisHolder {
    #[tracing::instrument(level = "debug", skip())]
    pub fn new() -> Result<MprisHolder> {
        let context = Box::new(SouvlakiMprisContext::new()?);
        Self::new_with_context(context)
    }

    pub fn new_with_context(mut context: Box<dyn MprisContext>) -> Result<MprisHolder> {
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

    #[tracing::instrument(level = "debug", skip(self, metadata))]
    pub fn set_metadata(&self, metadata: MprisPlayerDetails) -> Result<()> {
        let mut context = self.context.lock().unwrap();
        context.set_metadata(metadata)
    }

    #[tracing::instrument(level = "debug", skip(self, state))]
    pub fn set_playback_state(&self, state: PlayerState) -> Result<()> {
        let last_duration = self.last_duration.lock().unwrap();
        let duration = *last_duration;
        drop(last_duration);

        let mut context = self.context.lock().unwrap();
        context.set_playback_state(state, duration)?;

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
