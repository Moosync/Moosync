use state_manager::StateManager;

use crate::{MainWindow, pages::PageHandler, settings::PreferenceHandler};

pub struct SystemPageHandler<'a> {
    main_window: &'a MainWindow,
    state_manager: &'a StateManager,
}

impl<'a> SystemPageHandler<'a> {
    #[tracing::instrument(level = "debug", skip_all)]
    pub fn new(main_window: &'a MainWindow, state_manager: &'a StateManager) -> Self {
        Self {
            main_window,
            state_manager,
        }
    }
}

pref_macro::generate_preferences!(
    "src/settings/system_prefs.yaml",
    system_items,
    SystemPageHandler
);

impl<'a> PageHandler for SystemPageHandler<'a> {
    #[tracing::instrument(level = "debug", skip_all)]
    fn initialize(&self) { self.init_preferences(); }

    #[tracing::instrument(level = "debug", skip_all)]
    fn on_show(&self) {}

    #[tracing::instrument(level = "debug", skip_all)]
    fn on_hide(&self) {}
}
