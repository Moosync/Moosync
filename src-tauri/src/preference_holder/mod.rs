use std::thread;

use database::database::Database;
use file_scanner::ScannerHolder;
use macros::generate_command;
use preferences::preferences::PreferenceConfig;
use serde_json::Value;
use tauri::{async_runtime, App, AppHandle, Emitter, Manager, State};
use types::{errors::Result, preferences::CheckboxPreference};

use crate::{
    providers::handler::ProviderHandler,
    scanner::{start_scan, ScanTask},
};

const UI_KEYS: &[&str] = &[
    "prefs.system_settings",
    "prefs.queue_settings",
    "prefs.audio_settings",
    "prefs.gapless_skip",
    "prefs.volume_persist_mode",
    "prefs.spotify.enable",
    "prefs.spotify.username",
    "prefs.spotify.password",
    "prefs.themes.active_theme",
    "prefs.i18n_language",
];

macro_rules! generate_states {
    ($app:expr, $( $state_type:ty ),*) => {
        {
            // Create a tuple to hold the state variables
            let tuple = ( $( $app.state::<$state_type>().clone() ),* );
            tuple
        }
    };
}

#[tracing::instrument(level = "trace", skip(app))]
pub fn handle_pref_changes(app: AppHandle) {
    async_runtime::spawn(async move {
        let pref_config: State<PreferenceConfig> = app.state::<PreferenceConfig>();
        let receiver = pref_config.get_receiver();
        for (key, value) in receiver {
            tracing::debug!("Received key: {} value: {}", key, value);
            if UI_KEYS.contains(&key.as_str()) {
                tracing::info!("Emitting preference-changed event");
                if let Err(e) = app.emit("preference-changed", (key.clone(), value.clone())) {
                    tracing::error!("Error emitting preference-changed event{}", e);
                } else {
                    tracing::info!("Emitted preference-changed event");
                }
            }

            if key == "prefs.music_paths" || key == "prefs.exclude_music_paths" {
                let app = app.clone();
                thread::spawn(move || {
                    let app = app.clone();
                    let (pref_config, scanner, database) =
                        generate_states!(app, PreferenceConfig, ScannerHolder, Database);
                    if let Err(e) = start_scan(scanner, database, pref_config, None, true) {
                        tracing::error!("{}", e);
                    }
                });
            }

            if key.starts_with("prefs.youtube") {
                let provider_state: State<ProviderHandler> = app.state();
                provider_state.initialize_provider("youtube".into()).await;
            }

            if key.starts_with("prefs.spotify") {
                let provider_state: State<ProviderHandler> = app.state();
                provider_state.initialize_provider("spotify".into()).await;
            }

            if key.starts_with("prefs.system_settings") {
                #[cfg(not(any(target_os = "android", target_os = "ios")))]
                {
                    let manager: State<tauri_plugin_autostart::AutoLaunchManager> = app.state();

                    let auto_start = pref_config.load_selective_array::<CheckboxPreference>(
                        "system_settings.auto_startup".into(),
                    );
                    tracing::info!("Setting autolaunch {:?}", auto_start);
                    if let Ok(auto_start) = auto_start {
                        let res = if auto_start.enabled {
                            manager.enable()
                        } else {
                            manager.disable()
                        };

                        if let Err(e) = res {
                            tracing::error!("Error toggling autostart {:?}", e);
                        }
                    }
                }
            }

            if key.starts_with("prefs.scan_interval") {
                let scan_task: State<ScanTask> = app.state();
                scan_task.spawn_scan_task(app.clone(), value.as_u64().unwrap());
            }
        }
    });
}

#[tracing::instrument(level = "trace", skip(app))]
pub fn get_preference_state(app: &mut App) -> Result<PreferenceConfig> {
    let data_dir = app.path().app_config_dir()?;
    PreferenceConfig::new(data_dir)
}

#[tracing::instrument(level = "trace", skip(app))]
pub fn initial(app: &mut App) {
    let pref_config: State<PreferenceConfig> = app.state();
    if !pref_config.has_key("thumbnail_path") {
        let path = app.path().app_local_data_dir().unwrap().join("thumbnails");
        let _ = pref_config.save_selective("thumbnail_path".to_string(), Some(path));
    }

    if !pref_config.has_key("artwork_path") {
        let path = app.path().app_local_data_dir().unwrap().join("artwork");
        let _ = pref_config.save_selective("artwork_path".to_string(), Some(path));
    }

    if !pref_config.has_key("i18n_language") {
        let _ = pref_config.save_selective(
            "i18n_language".to_string(),
            Some(
                [
                    "af_ZA", "ar_SA", "ca_ES", "cs_CZ", "da_DK", "de_DE", "el_GR", "en_US",
                    "es_ES", "fi_FI", "fr_FR", "he_IL", "hi_IN", "hu_HU", "it_IT", "ja_JP",
                    "ko_KR", "nl_NL", "no_NO", "pl_PL", "pt_BR", "pt_PT", "ro_RO", "ru_RU",
                    "sr_SP", "sv_SE", "tr_TR", "uk_UA", "vi_VN", "zh_CN", "zh_TW",
                ]
                .into_iter()
                .map(|locale| CheckboxPreference {
                    key: locale.to_string(),
                    enabled: locale == "en_US",
                })
                .collect::<Vec<_>>(),
            ),
        );
    }

    // Spawn scan task
    let scan_task: State<ScanTask> = app.state();
    let scan_duration = pref_config.load_selective("scan_interval".into());
    if let Ok(scan_duration) = scan_duration {
        scan_task.spawn_scan_task(app.handle().clone(), scan_duration);
    } else {
        tracing::warn!("Could not spawn scan task, no / invalid duration found");
    }
}

generate_command!(load_selective, PreferenceConfig, Value, key: String);
generate_command!(save_selective, PreferenceConfig, (), key: String, value: Option<Value>);
generate_command!(get_secure, PreferenceConfig, Value, key: String);
generate_command!(set_secure, PreferenceConfig, (), key: String, value: Option<Value>);
generate_command!(load_selective_array, PreferenceConfig, Value, key: String);
