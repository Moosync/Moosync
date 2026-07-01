use state_manager::StateManager;

use crate::{MainWindow, pages::PageHandler};

pub struct ExplorePageHandler<'a> {
    main_window: &'a MainWindow,
    state_manager: &'a StateManager,
}

impl<'a> ExplorePageHandler<'a> {
    #[tracing::instrument(level = "debug", skip_all)]
    pub fn new(main_window: &'a MainWindow, state_manager: &'a StateManager) -> Self {
        Self {
            main_window,
            state_manager,
        }
    }
}

impl<'a> PageHandler for ExplorePageHandler<'a> {
    #[tracing::instrument(level = "debug", skip_all)]
    fn initialize(&self) {}
    #[tracing::instrument(level = "debug", skip_all)]
    fn on_show(&self) {}
    #[tracing::instrument(level = "debug", skip_all)]
    fn on_hide(&self) {}
}
