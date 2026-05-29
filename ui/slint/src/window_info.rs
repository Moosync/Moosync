use slint::Window;
use std::sync::Mutex;

pub struct WindowEvents {
    pub on_resize: Mutex<Vec<Box<dyn Fn(&Window)>>>,
}

impl WindowEvents {
    pub fn new() -> Self {
        Self {
            on_resize: Mutex::new(Vec::new()),
        }
    }

    pub fn on_resize(&self, callback: Box<dyn Fn(&Window)>) {
        self.on_resize.lock().unwrap().push(callback);
    }

    pub fn trigger_resize(&self, window: &Window) {
        self.on_resize
            .lock()
            .unwrap()
            .iter()
            .for_each(|cb| cb(window));
    }
}

thread_local! {
    pub static WINDOW_EVENTS: WindowEvents = WindowEvents::new();
}
