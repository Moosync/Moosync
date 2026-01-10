use std::io::Error;
use std::mem;

use windows::core::PCWSTR;
use windows::w;
use windows::Win32::Foundation::{HWND, LPARAM, LRESULT, WPARAM};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DefWindowProcW, DestroyWindow, DispatchMessageW, GetAncestor,
    IsDialogMessageW, PeekMessageW, RegisterClassExW, TranslateMessage, GA_ROOT, MSG, PM_REMOVE,
    WINDOW_EX_STYLE, WINDOW_STYLE, WM_QUIT, WNDCLASSEXW,
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
    extern "system" fn wnd_proc(hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
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
