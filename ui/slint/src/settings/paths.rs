use std::sync::LazyLock;

use preferences::keys::{
    ARTIST_SPLITTER, ARTWORK_PATH, EXCLUDE_MUSIC_PATHS, MUSIC_PATHS, SCAN_INTERVAL, SCAN_THREADS,
    THUMBNAIL_PATH,
};
use preferences_proto::moosync::types::PreferenceItem as ProtoPreferenceItem;
use slint::ComponentHandle;
use state_manager::StateManager;
use tracing::Instrument;

use super::utils::{clone_lazy_preferences, to_preference_model, update_preferences_in_ui};
use crate::{AppPreferences, MainWindow, pages::PageHandler};

pub static PREFERENCES: &[&LazyLock<ProtoPreferenceItem>] = &[
    &MUSIC_PATHS,
    &EXCLUDE_MUSIC_PATHS,
    &SCAN_THREADS,
    &ARTIST_SPLITTER,
    &SCAN_INTERVAL,
    &THUMBNAIL_PATH,
    &ARTWORK_PATH,
];

pub struct PathsPageHandler {
    main_window: &'static MainWindow,
    state_manager: &'static StateManager,
}

impl PathsPageHandler {
    #[tracing::instrument(level = "debug", skip_all)]
    pub fn new(main_window: &'static MainWindow, state_manager: &'static StateManager) -> Self {
        Self {
            main_window,
            state_manager,
        }
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub fn get_preferences() -> Vec<ProtoPreferenceItem> { clone_lazy_preferences(PREFERENCES) }

    #[tracing::instrument(level = "debug", skip_all)]
    pub async fn refresh_preferences(
        main_window_weak: slint::Weak<MainWindow>,
        state_manager: &'static StateManager,
    ) {
        update_preferences_in_ui(
            main_window_weak,
            state_manager,
            PREFERENCES,
            "paths",
            |main_window, items| {
                main_window
                    .global::<AppPreferences>()
                    .set_paths_items(to_preference_model(items));
            },
        )
        .await;
    }
}

impl PageHandler for PathsPageHandler {
    #[tracing::instrument(level = "debug", skip_all)]
    fn initialize(&self) { self.on_show(); }

    #[tracing::instrument(level = "debug", skip_all)]
    fn on_show(&self) {
        let main_window_weak = self.main_window.as_weak();
        let state_manager = self.state_manager;

        tokio::spawn(
            async move {
                Self::refresh_preferences(main_window_weak, state_manager).await;
            }
            .in_current_span(),
        );
    }
}
