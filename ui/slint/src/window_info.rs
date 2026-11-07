use std::sync::Mutex;

use slint::Window;

pub type WindowResizeCallback = Box<dyn Fn(&Window)>;

pub struct WindowEvents {
    pub on_resize: Mutex<Vec<WindowResizeCallback>>,
    pub last_size: Mutex<Option<(usize, usize)>>,
}

impl Default for WindowEvents {
    fn default() -> Self { Self::new() }
}

impl WindowEvents {
    #[tracing::instrument(level = "debug", skip_all)]
    pub fn new() -> Self {
        Self {
            on_resize: Mutex::new(Vec::new()),
            last_size: Mutex::new(None),
        }
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub fn on_resize(&self, callback: WindowResizeCallback) {
        self.on_resize.lock().unwrap().push(callback);
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub fn set_last_size(&self, width: usize, height: usize) {
        *self.last_size.lock().unwrap() = Some((width, height));
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub fn get_last_size(&self) -> Option<(usize, usize)> { *self.last_size.lock().unwrap() }

    #[tracing::instrument(level = "debug", skip_all)]
    pub fn trigger_resize(&self, window: &Window) {
        let scale = window.scale_factor();
        let width = (window.size().width as f32 / scale) as usize;
        let height = (window.size().height as f32 / scale) as usize;
        self.set_last_size(width, height);

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
