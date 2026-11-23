use std::sync::LazyLock;

use logging::LogController;
use preferences::keys::{
    LOG_COMPONENT_CACHE, LOG_COMPONENT_DATABASE, LOG_COMPONENT_EXTENSIONS,
    LOG_COMPONENT_FILE_SCANNER, LOG_COMPONENT_LOGGING, LOG_COMPONENT_MPRIS, LOG_COMPONENT_OAUTH,
    LOG_COMPONENT_PLAYER, LOG_COMPONENT_PREFERENCES, LOG_COMPONENT_SLINT_APP,
    LOG_COMPONENT_SPOTIFY_PLAYER, LOG_COMPONENT_STATE_MANAGER, LOG_COMPONENT_THEMES,
    LOG_COMPONENT_TYPES, LOG_CUSTOM_FILTER, LOG_LEVEL, PreferenceItemExt, TRACY_PROFILING,
};
use preferences_proto::moosync::types::PreferenceItem as ProtoPreferenceItem;
use slint::ComponentHandle;
use state_manager::StateManager;
use tracing::Instrument;

use super::utils::{clone_lazy_preferences, to_preference_model, update_preferences_in_ui};
use crate::{AppPreferences, MainWindow, pages::PageHandler};

pub static PREFERENCES: &[&LazyLock<ProtoPreferenceItem>] = &[
    &LOG_LEVEL,
    &LOG_CUSTOM_FILTER,
    &TRACY_PROFILING,
    &LOG_COMPONENT_SLINT_APP,
    &LOG_COMPONENT_STATE_MANAGER,
    &LOG_COMPONENT_PLAYER,
    &LOG_COMPONENT_DATABASE,
    &LOG_COMPONENT_EXTENSIONS,
    &LOG_COMPONENT_FILE_SCANNER,
    &LOG_COMPONENT_CACHE,
    &LOG_COMPONENT_MPRIS,
    &LOG_COMPONENT_THEMES,
    &LOG_COMPONENT_PREFERENCES,
    &LOG_COMPONENT_SPOTIFY_PLAYER,
    &LOG_COMPONENT_OAUTH,
    &LOG_COMPONENT_TYPES,
    &LOG_COMPONENT_LOGGING,
];

const COMPONENT_PREFS: &[(&str, &LazyLock<ProtoPreferenceItem>)] = &[
    ("slint_app", &LOG_COMPONENT_SLINT_APP),
    ("state_manager", &LOG_COMPONENT_STATE_MANAGER),
    ("player", &LOG_COMPONENT_PLAYER),
    ("database", &LOG_COMPONENT_DATABASE),
    ("extensions", &LOG_COMPONENT_EXTENSIONS),
    ("file_scanner", &LOG_COMPONENT_FILE_SCANNER),
    ("cache", &LOG_COMPONENT_CACHE),
    ("mpris", &LOG_COMPONENT_MPRIS),
    ("themes", &LOG_COMPONENT_THEMES),
    ("preferences", &LOG_COMPONENT_PREFERENCES),
    ("spotify_player", &LOG_COMPONENT_SPOTIFY_PLAYER),
    ("oauth", &LOG_COMPONENT_OAUTH),
    ("types", &LOG_COMPONENT_TYPES),
    ("logging", &LOG_COMPONENT_LOGGING),
];

#[tracing::instrument(level = "debug", skip_all)]
pub fn apply_saved_logging(state_manager: &StateManager) {
    let config = state_manager.get_preference_config();
    let global_level = config
        .load(&LOG_LEVEL)
        .value::<String>()
        .unwrap_or_else(|| "info".to_string());
    let custom_filter = config
        .load(&LOG_CUSTOM_FILTER)
        .value::<String>()
        .unwrap_or_default();

    let component_levels: Vec<(&str, String)> = COMPONENT_PREFS
        .iter()
        .map(|(crate_name, pref)| {
            let level = config
                .load(pref)
                .value::<String>()
                .unwrap_or_else(|| "inherit".to_string());
            (*crate_name, level)
        })
        .collect();
    let component_refs: Vec<(&str, &str)> = component_levels
        .iter()
        .map(|(name, level)| (*name, level.as_str()))
        .collect();

    let filter =
        LogController::build_env_filter_string(&global_level, &component_refs, &custom_filter);
    if let Err(err) = state_manager.get_log_controller().set_log_level(&filter) {
        tracing::error!("Failed to apply saved log level '{}': {}", filter, err);
    }

    let tracy_enabled = config
        .load(&TRACY_PROFILING)
        .value::<bool>()
        .unwrap_or(false);
    if let Err(err) = state_manager
        .get_log_controller()
        .set_tracy_profiling(tracy_enabled)
    {
        tracing::error!("Failed to apply saved Tracy profiling state: {}", err);
    }
}

pub struct LogsPageHandler {
    main_window: MainWindow,
    state_manager: StateManager,
}

impl LogsPageHandler {
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
        let controller = state_manager.get_log_controller();
        let log_dir = controller.get_log_dir();
        let tracy_status = controller.get_tracy_status();

        update_preferences_in_ui(
            main_window_weak,
            state_manager,
            PREFERENCES,
            "logs",
            move |main_window, mut items| {
                for item in &mut items {
                    if item.id == LOG_LEVEL.id {
                        item.subtitle = format!("{} (Log dir: {})", item.subtitle, log_dir);
                    } else if item.id == TRACY_PROFILING.id {
                        item.subtitle = tracy_status.clone();
                    }
                }
                main_window
                    .global::<AppPreferences>()
                    .set_logs_items(to_preference_model(items));
            },
        )
        .await;
    }
}

impl PageHandler for LogsPageHandler {
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

    #[tracing::instrument(level = "debug", skip_all)]
    fn on_hide(&self) {}
}
