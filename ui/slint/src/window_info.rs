use std::sync::Mutex;

use slint::Window;

pub type WindowResizeCallback = Box<dyn Fn(&Window)>;

pub struct WindowEvents {
    pub on_resize: Mutex<Vec<WindowResizeCallback>>,
}

impl Default for WindowEvents {
    fn default() -> Self { Self::new() }
}

impl WindowEvents {
    #[tracing::instrument(level = "debug", skip_all)]
    pub fn new() -> Self {
        Self {
            on_resize: Mutex::new(Vec::new()),
        }
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub fn on_resize(&self, callback: WindowResizeCallback) {
        self.on_resize.lock().unwrap().push(callback);
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub fn trigger_resize(&self, window: &Window) {
        self.on_resize
            .lock()
            .unwrap()
            .iter()
            .for_each(|cb| cb(window));
    }
}

thread_local! {
    pub static WINDOW_EVENTS: WindowEvents = WindowEvents::default();
}
