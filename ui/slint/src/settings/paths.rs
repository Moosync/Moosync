pref_macro::generate_preferences!("src/settings/paths_prefs.yaml", paths_items);

pub struct PathsPageHandler<'a> {
    main_window: &'a crate::MainWindow,
    state_manager: &'a state_manager::StateManager,
}

impl<'a> PathsPageHandler<'a> {
    pub fn new(main_window: &'a crate::MainWindow, state_manager: &'a state_manager::StateManager) -> Self {
        Self {
            main_window,
            state_manager,
        }
    }

    pub fn handle_change(
        change: &crate::PreferenceChange,
        main_window_weak: &slint::Weak<crate::MainWindow>,
        state_manager: &state_manager::StateManager,
    ) -> bool {
        handle_change(change, main_window_weak, state_manager)
    }
}

impl<'a> crate::pages::PageHandler for PathsPageHandler<'a> {
    fn initialize(&self) {
        init(self.main_window, self.state_manager);
    }

    fn on_show(&self) {}
    fn on_hide(&self) {}
}
