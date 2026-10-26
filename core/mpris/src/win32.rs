use std::{io::Error, mem};

use windows::{
    Win32::{
        Foundation::{HWND, LPARAM, LRESULT, WPARAM},
        System::LibraryLoader::GetModuleHandleW,
        UI::WindowsAndMessaging::{
            CreateWindowExW, DefWindowProcW, DestroyWindow, DispatchMessageW, GA_ROOT, GetAncestor,
            IsDialogMessageW, MSG, PM_REMOVE, PeekMessageW, RegisterClassExW, TranslateMessage,
            WINDOW_EX_STYLE, WINDOW_STYLE, WM_QUIT, WNDCLASSEXW,
        },
    },
    core::w,
};

pub struct DummyWindow {
    pub handle: HWND,
}

impl DummyWindow {
    #[tracing::instrument(level = "debug", skip_all)]
    pub fn new() -> Result<DummyWindow, String> {
        let class_name = w!("SimpleTray");

        let handle_result = unsafe {
            let instance =
                GetModuleHandleW(None).map_err(|e| format!("Getting module handle failed: {e}"))?;

            let wnd_class = WNDCLASSEXW {
                cbSize: mem::size_of::<WNDCLASSEXW>() as u32,
                hInstance: instance.into(),
                lpszClassName: class_name,
                lpfnWndProc: Some(Self::wnd_proc),
                ..Default::default()
            };

            if RegisterClassExW(&wnd_class) == 0 {
                return Err(format!(
                    "Registering class failed: {}",
                    Error::last_os_error()
                ));
            }

            CreateWindowExW(
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
                Some(instance.into()),
                None,
            )
            .map_err(|e| format!("Message only window creation failed: {e}"))
        };

        handle_result.map(|handle| DummyWindow { handle })
    }

    #[tracing::instrument(level = "debug", skip_all)]
    extern "system" fn wnd_proc(hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
        unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) }
    }
}

impl Drop for DummyWindow {
    #[tracing::instrument(level = "debug", skip_all)]
    fn drop(&mut self) {
        unsafe {
            let _ = DestroyWindow(self.handle);
        }
    }
}

#[tracing::instrument(level = "debug", skip_all)]
pub fn pump_event_queue() -> bool {
    unsafe {
        let mut msg: MSG = std::mem::zeroed();
        let mut has_message = PeekMessageW(&mut msg, None, 0, 0, PM_REMOVE).as_bool();
        while msg.message != WM_QUIT && has_message {
            if !IsDialogMessageW(GetAncestor(msg.hwnd, GA_ROOT), &msg).as_bool() {
                let _ = TranslateMessage(&msg);
                DispatchMessageW(&msg);
            }

            has_message = PeekMessageW(&mut msg, None, 0, 0, PM_REMOVE).as_bool();
        }

        msg.message == WM_QUIT
    }
}
