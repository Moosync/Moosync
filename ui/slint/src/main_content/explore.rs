use crate::{MainWindow, pages::PageHandler};
use state_manager::StateManager;

pub struct ExplorePageHandler<'a> {
    main_window: &'a MainWindow,
    state_manager: &'a StateManager,
}

impl<'a> ExplorePageHandler<'a> {
    pub fn new(main_window: &'a MainWindow, state_manager: &'a StateManager) -> Self {
        Self {
            main_window,
            state_manager,
        }
    }
}

impl<'a> PageHandler for ExplorePageHandler<'a> {
    fn initialize(&self) {}
    fn on_show(&self) {}
    fn on_hide(&self) {}
}
