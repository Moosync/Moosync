use std::sync::LazyLock;

use preferences::keys::{
    AUTO_STARTUP, CLEAR_QUEUE, I18N_LANGUAGE, JUKEBOX_MODE, MINIMIZE_TO_TRAY,
    PERSIST_PLAYBACK_STATE, VOLUME_PERSIST_MODE,
};
use preferences_proto::moosync::types::PreferenceItem as ProtoPreferenceItem;
use slint::ComponentHandle;
use state_manager::StateManager;
use tracing::Instrument;

use super::utils::{clone_lazy_preferences, to_preference_model, update_preferences_in_ui};
use crate::{AppPreferences, MainWindow, pages::PageHandler};

pub static PREFERENCES: &[&LazyLock<ProtoPreferenceItem>] = &[
    &AUTO_STARTUP,
    &MINIMIZE_TO_TRAY,
    &JUKEBOX_MODE,
    &CLEAR_QUEUE,
    &PERSIST_PLAYBACK_STATE,
    &VOLUME_PERSIST_MODE,
    &I18N_LANGUAGE,
];

pub struct SystemPageHandler {
    main_window: MainWindow,
    state_manager: StateManager,
}

impl SystemPageHandler {
    #[tracing::instrument(level = "debug", skip_all)]
    pub fn new(main_window: &MainWindow, state_manager: &StateManager) -> Self {
        Self {
            main_window: main_window.clone_strong(),
            state_manager: state_manager.clone(),
        }
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub fn get_preferences() -> Vec<ProtoPreferenceItem> { clone_lazy_preferences(PREFERENCES) }

    #[tracing::instrument(level = "debug", skip_all)]
    pub async fn refresh_preferences(
        main_window_weak: slint::Weak<MainWindow>,
        state_manager: StateManager,
    ) {
        update_preferences_in_ui(
            main_window_weak,
            state_manager,
            PREFERENCES,
            "system",
            |main_window, items| {
                main_window
                    .global::<AppPreferences>()
                    .set_system_items(to_preference_model(items));
            },
        )
        .await;
    }
}

impl PageHandler for SystemPageHandler {
    #[tracing::instrument(level = "debug", skip_all)]
    fn initialize(&self) { self.on_show(); }

    #[tracing::instrument(level = "debug", skip_all)]
    fn on_show(&self) {
        let main_window_weak = self.main_window.as_weak();
        let state_manager = self.state_manager.clone();

        tokio::spawn(
            async move {
                Self::refresh_preferences(main_window_weak, state_manager).await;
            }
            .in_current_span(),
        );
    }
}
