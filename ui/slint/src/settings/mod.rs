use extensions_proto::moosync::types::{PreferenceArgs, PreferenceChangedRequest};
use preferences::keys::{
    AUTO_STARTUP, I18N_LANGUAGE, LOG_CUSTOM_FILTER, LOG_LEVEL, PreferenceItemExt, THEME_REGISTRIES,
    TRACY_PROFILING,
};
use preferences_proto::moosync::types::{
    PreferenceItem as ProtoPreferenceItem, PreferenceType, PreferenceValue,
};
#[cfg(not(target_os = "android"))]
use rfd::FileDialog;
use slint::{ComponentHandle, Model};
use state_manager::StateManager;
use tracing::Instrument;

#[cfg(target_os = "android")]
use crate::utils::android::get_android_context;
use crate::{
    AppCallbacks, MainWindow,
    pages::PageHandler,
    settings::{
        extensions::ExtensionsPageHandler, logs::LogsPageHandler, paths::PathsPageHandler,
        system::SystemPageHandler, themes::ThemesPageHandler,
    },
    utils::autostart::update_auto_startup,
};

pub mod cache;
pub mod extensions;
pub mod logs;
pub mod paths;
pub mod system;
pub mod themes;
pub mod utils;

#[cfg(test)]
mod cache_test;
#[cfg(test)]
mod extensions_test;
#[cfg(test)]
mod logs_test;
#[cfg(test)]
mod paths_test;
#[cfg(test)]
mod system_test;
#[cfg(test)]
mod themes_test;
#[cfg(test)]
mod utils_test;

#[cfg(not(target_os = "android"))]
#[tracing::instrument(level = "debug", skip_all)]
fn select_directory() -> String {
    if let Some(path) = FileDialog::new().pick_folder() {
        tracing::info!("Selected directory: {:?}", path);
        path.to_string_lossy().to_string()
    } else {
        String::new()
    }
}

#[cfg(not(target_os = "android"))]
#[tracing::instrument(level = "debug", skip_all)]
fn select_file(filter: String) -> String {
    let filter_items: Vec<&str> = filter.split(',').collect();
    if let Some(path) = FileDialog::new()
        .add_filter("Custom files", &filter_items)
        .pick_file()
    {
        tracing::info!("Selected file: {:?}", path);
        path.to_string_lossy().to_string()
    } else {
        String::new()
    }
}

#[cfg(target_os = "android")]
static DIRECTORY_PICKER_TX: std::sync::Mutex<Option<std::sync::mpsc::Sender<String>>> =
    std::sync::Mutex::new(None);

#[cfg(target_os = "android")]
static FILE_PICKER_TX: std::sync::Mutex<Option<std::sync::mpsc::Sender<String>>> =
    std::sync::Mutex::new(None);

#[cfg(target_os = "android")]
#[tracing::instrument(level = "debug", skip_all)]
fn select_directory() -> String {
    let (tx, rx) = std::sync::mpsc::channel();
    {
        let mut guard = DIRECTORY_PICKER_TX.lock().unwrap();
        *guard = Some(tx);
    }

    let context = get_android_context();
    if let Err(err) = context.call_activity::<_, ()>("openDirectoryPicker", ()) {
        tracing::error!("Failed to invoke openDirectoryPicker via JNI: {:?}", err);
        panic!("openDirectoryPicker failed: {:?}", err);
    }

    rx.recv_timeout(std::time::Duration::from_secs(120))
        .unwrap_or_default()
}

#[cfg(target_os = "android")]
#[tracing::instrument(level = "debug", skip_all)]
fn select_file(filter: String) -> String {
    let (tx, rx) = std::sync::mpsc::channel();
    {
        let mut guard = FILE_PICKER_TX.lock().unwrap();
        *guard = Some(tx);
    }

    let context = get_android_context();
    if let Err(err) = context.call_activity::<_, ()>("openFilePicker", filter.as_str()) {
        tracing::error!("Failed to invoke openFilePicker via JNI: {:?}", err);
        panic!("openFilePicker failed: {:?}", err);
    }

    rx.recv_timeout(std::time::Duration::from_secs(120))
        .unwrap_or_default()
}

#[cfg(target_os = "android")]
#[tracing::instrument(level = "debug", skip_all)]
pub fn on_directory_picked(path: String) {
    tracing::info!("on_directory_picked: {}", path);
    match DIRECTORY_PICKER_TX.lock() {
        Ok(mut guard) => {
            if let Some(tx) = guard.take() {
                if let Err(err) = tx.send(path) {
                    tracing::error!("Failed to send picked directory to channel: {:?}", err);
                }
            } else {
                tracing::warn!("No pending directory picker receiver found");
            }
        }
        Err(err) => {
            tracing::error!("DIRECTORY_PICKER_TX lock poisoned: {:?}", err);
        }
    }
}

#[cfg(target_os = "android")]
#[tracing::instrument(level = "debug", skip_all)]
pub fn on_file_picked(path: String) {
    tracing::info!("on_file_picked: {}", path);
    match FILE_PICKER_TX.lock() {
        Ok(mut guard) => {
            if let Some(tx) = guard.take() {
                if let Err(err) = tx.send(path) {
                    tracing::error!("Failed to send picked file to channel: {:?}", err);
                }
            } else {
                tracing::warn!("No pending file picker receiver found");
            }
        }
        Err(err) => {
            tracing::error!("FILE_PICKER_TX lock poisoned: {:?}", err);
        }
    }
}

#[tracing::instrument(level = "debug", skip_all)]
pub fn get_all_static_preferences() -> Vec<ProtoPreferenceItem> {
    let mut prefs = PathsPageHandler::get_preferences();
    prefs.extend(SystemPageHandler::get_preferences());
    prefs.extend(ExtensionsPageHandler::get_preferences());
    prefs.extend(ThemesPageHandler::get_preferences());
    prefs.extend(LogsPageHandler::get_preferences());
    prefs
}

#[tracing::instrument(level = "debug", skip_all)]
pub(crate) fn find_preference_item(
    key: &str,
    state_manager: &StateManager,
) -> Option<ProtoPreferenceItem> {
    let config = state_manager.get_preference_config();
    if let Some(item) = config.get(key) {
        return Some(item);
    }

    if let Some(item) = get_all_static_preferences()
        .into_iter()
        .find(|p| p.id == key)
    {
        return Some(item);
    }

    if key.starts_with("ext:") {
        let ext_handler = state_manager.get_extension_handler();
        for ext in ext_handler.get_installed_extensions() {
            for mut p in ext.preferences {
                let scoped_id = format!("ext:{}:{}", ext.package_name, p.id);
                if scoped_id == key {
                    p.id = scoped_id;
                    return Some(p);
                }
            }
        }
    }

    None
}

#[tracing::instrument(level = "debug", skip_all)]
pub(crate) fn compute_list_value(
    item: &ProtoPreferenceItem,
    value_string: String,
    value_list: Vec<String>,
) -> preferences_proto::moosync::types::preference_value::Value {
    let list_to_save = if !value_list.is_empty() {
        value_list
    } else if !value_string.is_empty() {
        let mut current_list = item.value::<Vec<String>>().unwrap_or_default();
        if let Some(pos) = current_list.iter().position(|s| s == &value_string) {
            current_list.remove(pos);
        } else {
            current_list.push(value_string);
        }
        current_list
    } else {
        Vec::new()
    };

    preferences_proto::moosync::types::preference_value::Value::ListValue(
        preferences_proto::moosync::types::StringList {
            values: list_to_save,
        },
    )
}

#[tracing::instrument(level = "debug", skip_all)]
pub(crate) fn compute_updated_preference_value(
    item: &ProtoPreferenceItem,
    value_string: String,
    value_bool: bool,
    value_number: f32,
    value_list: Vec<String>,
) -> preferences_proto::moosync::types::preference_value::Value {
    let pref_type = preferences_proto::moosync::types::PreferenceType::try_from(item.pref_type)
        .unwrap_or(preferences_proto::moosync::types::PreferenceType::TextInputGroup);

    match pref_type {
        preferences_proto::moosync::types::PreferenceType::ToggleGroup => {
            preferences_proto::moosync::types::preference_value::Value::BoolValue(value_bool)
        }
        preferences_proto::moosync::types::PreferenceType::NumberInputGroup => {
            preferences_proto::moosync::types::preference_value::Value::NumberValue(value_number)
        }
        preferences_proto::moosync::types::PreferenceType::PathSelector
        | preferences_proto::moosync::types::PreferenceType::TextArrayInput => {
            compute_list_value(item, value_string, value_list)
        }
        _ => preferences_proto::moosync::types::preference_value::Value::StringValue(value_string),
    }
}

#[tracing::instrument(level = "debug", skip_all)]
async fn refresh_relevant_page(
    key: &str,
    main_window_weak: slint::Weak<MainWindow>,
    state_manager: StateManager,
) {
    if paths::PREFERENCES.iter().any(|p| p.id == key) {
        PathsPageHandler::refresh_preferences(main_window_weak, state_manager).await;
    } else if system::PREFERENCES.iter().any(|p| p.id == key) {
        SystemPageHandler::refresh_preferences(main_window_weak, state_manager).await;
    } else if extensions::PREFERENCES.iter().any(|p| p.id == key) || key.starts_with("ext:") {
        ExtensionsPageHandler::refresh_preferences(main_window_weak, state_manager).await;
    } else if key == THEME_REGISTRIES.id {
        ThemesPageHandler::refresh_preferences(main_window_weak, &state_manager);
    } else if logs::PREFERENCES.iter().any(|p| p.id == key) {
        LogsPageHandler::refresh_preferences(main_window_weak, state_manager).await;
    }
}

#[tracing::instrument(level = "debug", skip_all)]
async fn notify_extension_preference_changed(key: &str, state_manager: &StateManager) {
    let Some(rest) = key.strip_prefix("ext:") else {
        return;
    };
    let Some((pkg, ext_key)) = rest.split_once(':') else {
        return;
    };

    let ext_handler = state_manager.get_extension_handler();
    match ext_handler.get_extension(pkg) {
        Ok(ext) => {
            if let Err(e) = ext
                .preference_changed(PreferenceChangedRequest {
                    preference: Some(PreferenceArgs {
                        key: ext_key.to_string(),
                        value: Default::default(),
                    }),
                })
                .await
            {
                tracing::error!("Failed to notify extension {}: {:?}", pkg, e);
            }
        }
        Err(e) => {
            tracing::error!("Failed to get extension {} for pref change: {:?}", pkg, e);
        }
    }
}

#[tracing::instrument(level = "debug", skip_all)]
pub async fn handle_preference_change(
    key: String,
    value_string: String,
    value_bool: bool,
    value_number: f32,
    value_list: Vec<String>,
    state_manager: StateManager,
    main_window_weak: slint::Weak<MainWindow>,
) {
    tracing::debug!(
        "handle_preference_change: received update for key = {}",
        key
    );
    let config = state_manager.get_preference_config();

    let Some(mut item) = find_preference_item(&key, &state_manager) else {
        tracing::error!("Preference {} not found in store or static registry", key);
        return;
    };

    if key == I18N_LANGUAGE.id {
        let lang = value_string.clone();
        let _ = slint::invoke_from_event_loop(move || {
            if let Err(e) = slint::select_bundled_translation(&lang) {
                tracing::error!("Failed to select bundled translation {}: {:?}", lang, e);
            }
        });
    }
    if key == AUTO_STARTUP.id {
        update_auto_startup(value_bool);
    }

    let pref_type =
        PreferenceType::try_from(item.pref_type).unwrap_or(PreferenceType::TextInputGroup);

    let val =
        compute_updated_preference_value(&item, value_string, value_bool, value_number, value_list);
    item.value = Some(PreferenceValue { value: Some(val) });

    if let Err(e) = config.save(item) {
        tracing::error!("Failed to save preference for {}: {:?}", key, e);
    } else {
        tracing::debug!("Successfully applied preference change: key = {}", key);
    }

    if key == LOG_LEVEL.id || key == LOG_CUSTOM_FILTER.id || key.starts_with("log_component_") {
        logs::apply_saved_logging(&state_manager);
    } else if key == TRACY_PROFILING.id {
        if let Err(err) = state_manager
            .get_log_controller()
            .set_tracy_profiling(value_bool)
        {
            tracing::error!("Failed to toggle Tracy profiling: {}", err);
        }
    }

    if pref_type != PreferenceType::TextInputGroup && pref_type != PreferenceType::NumberInputGroup
    {
        refresh_relevant_page(&key, main_window_weak, state_manager.clone()).await;
    }
    notify_extension_preference_changed(&key, &state_manager).await;
}

#[tracing::instrument(level = "debug", skip_all)]
pub fn apply_saved_language(state_manager: &StateManager) {
    let lang = state_manager
        .get_preference_config()
        .load(&I18N_LANGUAGE)
        .value::<String>()
        .unwrap_or_else(|| "en_US".to_string());
    if let Err(e) = slint::select_bundled_translation(&lang) {
        tracing::error!("Failed to select bundled translation {}: {:?}", lang, e);
    }
}

#[tracing::instrument(level = "debug", skip_all)]
pub fn setup_settings(main_window: &MainWindow, state_manager: &StateManager) {
    apply_saved_language(state_manager);
    logs::apply_saved_logging(state_manager);

    let auto_startup_enabled = state_manager
        .get_preference_config()
        .load(&AUTO_STARTUP)
        .value::<bool>()
        .unwrap_or(false);
    update_auto_startup(auto_startup_enabled);

    let paths_handler = PathsPageHandler::new(main_window, state_manager);
    let system_handler = SystemPageHandler::new(main_window, state_manager);
    let extensions_handler = ExtensionsPageHandler::new(main_window, state_manager);

    paths_handler.initialize();
    system_handler.initialize();
    extensions_handler.initialize();

    let main_window_weak = main_window.as_weak();
    let sm = state_manager.clone();
    main_window
        .global::<AppCallbacks>()
        .on_preference_changed(move |change| {
            let main_window_weak = main_window_weak.clone();
            let key = change.id.to_string();
            let value_string = change.value_string.to_string();
            let value_bool = change.value_bool;
            let value_number = change.value_number;
            let value_list: Vec<String> = change.value_list.iter().map(|s| s.to_string()).collect();
            let state_manager = sm.clone();

            tokio::spawn(
                async move {
                    handle_preference_change(
                        key,
                        value_string,
                        value_bool,
                        value_number,
                        value_list,
                        state_manager,
                        main_window_weak,
                    )
                    .await;
                }
                .in_current_span(),
            );
        });

    main_window
        .global::<AppCallbacks>()
        .on_open_directory_picker(move || select_directory().into());

    main_window
        .global::<AppCallbacks>()
        .on_open_file_picker(move |filter| select_file(filter.into()).into());
}
