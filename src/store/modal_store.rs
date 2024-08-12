#[derive(Debug, Clone)]
pub enum Modals {
    LoginModal(String),
    DiscoverExtensions,
}

#[derive(Debug, Clone, Default)]
pub struct ModalStore {
    pub active_modal: Option<Modals>,
}

impl ModalStore {
    pub fn set_active_modal(&mut self, modal: Modals) {
        self.active_modal = Some(modal);
    }

    pub fn clear_active_modal(&mut self) {
        self.active_modal = None;
    }
}
