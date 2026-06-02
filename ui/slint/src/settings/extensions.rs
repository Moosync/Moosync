use crate::MainWindow;
use crate::pages::PageHandler;
use state_manager::StateManager;

pub struct ExtensionsPageHandler<'a> {
    main_window: &'a MainWindow,
    state_manager: &'a StateManager,
}

impl<'a> ExtensionsPageHandler<'a> {
    pub fn new(main_window: &'a MainWindow, state_manager: &'a StateManager) -> Self {
        Self {
            main_window,
            state_manager,
        }
    }
}

impl<'a> PageHandler for ExtensionsPageHandler<'a> {
    fn initialize(&self) {}

    fn on_show(&self) {}

    fn on_hide(&self) {}
}
