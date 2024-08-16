use std::{
    cell::{OnceCell, RefCell},
    rc::Rc,
    sync::Mutex,
};

#[derive(Debug, Clone)]
pub enum Modals {
    LoginModal(String, String),
    DiscoverExtensions,
    NewPlaylistModal,
}

#[derive(Clone, Default)]
pub struct ModalStore {
    pub active_modal: Option<Modals>,
    pub on_modal_close: Option<Rc<Box<dyn Fn()>>>,
}

impl ModalStore {
    pub fn set_active_modal(&mut self, modal: Modals) {
        self.clear_active_modal();
        self.active_modal = Some(modal);
    }

    pub fn clear_active_modal(&mut self) {
        self.active_modal = None;
        if let Some(cb) = self.on_modal_close.take() {
            cb();
        }
    }

    pub fn on_modal_close<T>(&mut self, cb: T)
    where
        T: Fn() + 'static,
    {
        self.on_modal_close = Some(Rc::new(Box::new(cb)));
    }
}
