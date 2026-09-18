use std::sync::LazyLock;

use preferences::keys::{
    ARTIST_SPLITTER, ARTWORK_PATH, AUTO_STARTUP, CLEAR_QUEUE, EXCLUDE_MUSIC_PATHS,
    EXTENSION_REGISTRIES, I18N_LANGUAGE, MINIMIZE_TO_TRAY, MUSIC_PATHS, PreferenceItemExt,
    SCAN_INTERVAL, SCAN_THREADS, THUMBNAIL_PATH, VOLUME_PERSIST_MODE,
};
use preferences_proto::moosync::types::PreferenceItem;
use slint::{ComponentHandle, Model, ModelRc};
use slint_app::{
    AppCallbacks, AppPreferences, ExtensionsPreferenceProps, MainWindow, PreferenceChange,
    SettingsPages, SettingsState,
    test_utils::integration::{integration_test, parameterized_test, wait_until},
};
use state_manager::StateManager;

#[tracing::instrument(level = "debug", skip_all)]
async fn do_preference_paths_list(
    main_window: &'static MainWindow,
    state_manager: &'static StateManager,
    pref_id: &'static str,
    pref_key: &'static LazyLock<PreferenceItem>,
    sample_path: &'static str,
) {
    main_window
        .global::<SettingsState>()
        .set_show_settings(true);
    main_window
        .global::<AppCallbacks>()
        .invoke_settings_toggled(true);
    main_window
        .global::<AppCallbacks>()
        .invoke_settings_active_page_changed(SettingsPages::Paths);

    let paths_loaded = wait_until(|| {
        let items = main_window.global::<AppPreferences>().get_paths_items();
        items.row_count() == 7
    })
    .await;
    assert!(paths_loaded);

    main_window
        .global::<AppCallbacks>()
        .invoke_preference_changed(PreferenceChange {
            id: pref_id.into(),
            value_string: sample_path.into(),
            value_bool: false,
            value_number: 0.0,
            value_list: ModelRc::default(),
        });

    let config = state_manager.get_preference_config().await;
    let path_saved = wait_until(|| {
        config
            .load(pref_key)
            .value::<Vec<String>>()
            .is_some_and(|paths| paths.contains(&sample_path.to_string()))
    })
    .await;
    assert!(path_saved);
}

#[tracing::instrument(level = "debug", skip_all)]
async fn do_preference_scan_number(
    main_window: &'static MainWindow,
    state_manager: &'static StateManager,
    pref_id: &'static str,
    pref_key: &'static LazyLock<PreferenceItem>,
    number_val: u32,
) {
    main_window
        .global::<SettingsState>()
        .set_show_settings(true);
    main_window
        .global::<AppCallbacks>()
        .invoke_settings_toggled(true);
    main_window
        .global::<AppCallbacks>()
        .invoke_settings_active_page_changed(SettingsPages::Paths);

    let _ = wait_until(|| {
        main_window
            .global::<AppPreferences>()
            .get_paths_items()
            .row_count()
            == 7
    })
    .await;

    main_window
        .global::<AppCallbacks>()
        .invoke_preference_changed(PreferenceChange {
            id: pref_id.into(),
            value_string: number_val.to_string().into(),
            value_bool: false,
            value_number: number_val as f32,
            value_list: ModelRc::default(),
        });

    let config = state_manager.get_preference_config().await;
    let saved = wait_until(|| {
        config
            .load(pref_key)
            .value::<u32>()
            .is_some_and(|val| val == number_val)
    })
    .await;
    assert!(saved);
}

#[tracing::instrument(level = "debug", skip_all)]
async fn do_preference_path_string(
    main_window: &'static MainWindow,
    state_manager: &'static StateManager,
    pref_id: &'static str,
    pref_key: &'static LazyLock<PreferenceItem>,
    path_val: &'static str,
) {
    main_window
        .global::<SettingsState>()
        .set_show_settings(true);
    main_window
        .global::<AppCallbacks>()
        .invoke_settings_toggled(true);
    main_window
        .global::<AppCallbacks>()
        .invoke_settings_active_page_changed(SettingsPages::Paths);

    let _ = wait_until(|| {
        main_window
            .global::<AppPreferences>()
            .get_paths_items()
            .row_count()
            == 7
    })
    .await;

    main_window
        .global::<AppCallbacks>()
        .invoke_preference_changed(PreferenceChange {
            id: pref_id.into(),
            value_string: path_val.into(),
            value_bool: false,
            value_number: 0.0,
            value_list: ModelRc::default(),
        });

    let config = state_manager.get_preference_config().await;
    let saved = wait_until(|| {
        config
            .load(pref_key)
            .value::<String>()
            .is_some_and(|val| val == path_val)
    })
    .await;
    assert!(saved);
}

#[tracing::instrument(level = "debug", skip_all)]
async fn do_preference_system_bool(
    main_window: &'static MainWindow,
    state_manager: &'static StateManager,
    pref_id: &'static str,
    pref_key: &'static LazyLock<PreferenceItem>,
    bool_val: bool,
) {
    main_window
        .global::<SettingsState>()
        .set_show_settings(true);
    main_window
        .global::<AppCallbacks>()
        .invoke_settings_toggled(true);
    main_window
        .global::<AppCallbacks>()
        .invoke_settings_active_page_changed(SettingsPages::System);

    let system_loaded = wait_until(|| {
        let items = main_window.global::<AppPreferences>().get_system_items();
        items.row_count() == 6
    })
    .await;
    assert!(system_loaded);

    main_window
        .global::<AppCallbacks>()
        .invoke_preference_changed(PreferenceChange {
            id: pref_id.into(),
            value_string: "".into(),
            value_bool: bool_val,
            value_number: 0.0,
            value_list: ModelRc::default(),
        });

    let config = state_manager.get_preference_config().await;
    let saved = wait_until(|| {
        config
            .load(pref_key)
            .value::<bool>()
            .is_some_and(|val| val == bool_val)
    })
    .await;
    assert!(saved);
}

#[tracing::instrument(level = "debug", skip_all)]
async fn do_preference_system_string(
    main_window: &'static MainWindow,
    state_manager: &'static StateManager,
    pref_id: &'static str,
    pref_key: &'static LazyLock<PreferenceItem>,
    str_val: &'static str,
) {
    main_window
        .global::<SettingsState>()
        .set_show_settings(true);
    main_window
        .global::<AppCallbacks>()
        .invoke_settings_toggled(true);
    main_window
        .global::<AppCallbacks>()
        .invoke_settings_active_page_changed(SettingsPages::System);

    let _ = wait_until(|| {
        main_window
            .global::<AppPreferences>()
            .get_system_items()
            .row_count()
            == 6
    })
    .await;

    main_window
        .global::<AppCallbacks>()
        .invoke_preference_changed(PreferenceChange {
            id: pref_id.into(),
            value_string: str_val.into(),
            value_bool: false,
            value_number: 0.0,
            value_list: ModelRc::default(),
        });

    let config = state_manager.get_preference_config().await;
    let saved = wait_until(|| {
        config
            .load(pref_key)
            .value::<String>()
            .is_some_and(|val| val == str_val)
    })
    .await;
    assert!(saved);
}

#[tracing::instrument(level = "debug", skip_all)]
async fn do_preference_extension_registries(
    main_window: &'static MainWindow,
    state_manager: &'static StateManager,
) {
    main_window
        .global::<SettingsState>()
        .set_show_settings(true);
    main_window
        .global::<AppCallbacks>()
        .invoke_settings_toggled(true);
    main_window
        .global::<AppCallbacks>()
        .invoke_settings_active_page_changed(SettingsPages::Extensions);

    let ext_loaded = wait_until(|| {
        let items = main_window
            .global::<ExtensionsPreferenceProps>()
            .get_static_preferences();
        items.row_count() == 1
    })
    .await;
    assert!(ext_loaded);

    main_window
        .global::<AppCallbacks>()
        .invoke_preference_changed(PreferenceChange {
            id: "extension_registries".into(),
            value_string: "https://example.com/manifest.json".into(),
            value_bool: false,
            value_number: 0.0,
            value_list: ModelRc::default(),
        });

    let config = state_manager.get_preference_config().await;
    let registry_saved = wait_until(|| {
        config
            .load(&EXTENSION_REGISTRIES)
            .value::<Vec<String>>()
            .is_some_and(|urls| urls.contains(&"https://example.com/manifest.json".to_string()))
    })
    .await;
    assert!(registry_saved);

    main_window
        .global::<AppCallbacks>()
        .invoke_preference_changed(PreferenceChange {
            id: "extension_registries".into(),
            value_string: "https://example.com/manifest.json".into(),
            value_bool: false,
            value_number: 0.0,
            value_list: ModelRc::default(),
        });

    let registry_removed = wait_until(|| {
        config
            .load(&EXTENSION_REGISTRIES)
            .value::<Vec<String>>()
            .is_some_and(|urls| !urls.contains(&"https://example.com/manifest.json".to_string()))
    })
    .await;
    assert!(registry_removed);
}

parameterized_test!(
    do_preference_paths_list,
    (
        test_preference_music_paths,
        "music_paths",
        &MUSIC_PATHS,
        "/music/test_dir"
    ),
    (
        test_preference_exclude_music_paths,
        "exclude_music_paths",
        &EXCLUDE_MUSIC_PATHS,
        "/music/excluded_dir"
    ),
);

parameterized_test!(
    do_preference_scan_number,
    (
        test_preference_scan_interval,
        "scan_interval",
        &SCAN_INTERVAL,
        3600
    ),
    (
        test_preference_scan_threads,
        "scan_threads",
        &SCAN_THREADS,
        8
    ),
);

parameterized_test!(
    do_preference_path_string,
    (
        test_preference_artwork_path,
        "artwork_path",
        &ARTWORK_PATH,
        "/custom/artwork"
    ),
    (
        test_preference_thumbnail_path,
        "thumbnail_path",
        &THUMBNAIL_PATH,
        "/custom/thumbnails"
    ),
    (
        test_preference_artist_splitter,
        "artist_splitter",
        &ARTIST_SPLITTER,
        ";,"
    ),
);

parameterized_test!(
    do_preference_system_bool,
    (
        test_preference_auto_startup,
        "auto_startup",
        &AUTO_STARTUP,
        true
    ),
    (
        test_preference_minimize_to_tray,
        "minimize_to_tray",
        &MINIMIZE_TO_TRAY,
        true
    ),
    (
        test_preference_clear_queue,
        "clear_queue",
        &CLEAR_QUEUE,
        true
    ),
);

parameterized_test!(
    do_preference_system_string,
    (
        test_preference_volume_persist_mode,
        "volume_persist_mode",
        &VOLUME_PERSIST_MODE,
        "PerTrack"
    ),
    (
        test_preference_i18n_language,
        "i18n_language",
        &I18N_LANGUAGE,
        "en_US"
    ),
);

integration_test!(
    test_preference_extension_registries => do_preference_extension_registries,
);
