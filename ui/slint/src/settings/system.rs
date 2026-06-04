pref_macro::generate_preferences!("src/settings/system_prefs.yaml", system_items);

pub struct SystemPageHandler<'a> {
    main_window: &'a crate::MainWindow,
    state_manager: &'a state_manager::StateManager,
}

impl<'a> SystemPageHandler<'a> {
    pub fn new(
        main_window: &'a crate::MainWindow,
        state_manager: &'a state_manager::StateManager,
    ) -> Self {
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

impl<'a> crate::pages::PageHandler for SystemPageHandler<'a> {
    fn initialize(&self) { init(self.main_window, self.state_manager); }

    fn on_show(&self) {}
    fn on_hide(&self) {}
}
