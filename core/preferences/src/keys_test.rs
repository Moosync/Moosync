// Moosync
// Copyright (C) 2024, 2025  Moosync <support@moosync.app>
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <http://www.gnu.org/licenses/>.

use preferences_proto::moosync::types::{
    PreferenceItem, PreferenceType, PreferenceValue, StringList, preference_value,
};
use tracing_test::traced_test;

use crate::keys::{
    ACTIVE_THEME_ID, ARTIST_SPLITTER, ARTWORK_PATH, AUTO_STARTUP, CLEAR_QUEUE, EXCLUDE_MUSIC_PATHS,
    EXTENSION_REGISTRIES, I18N_LANGUAGE, JUKEBOX_MODE, MINIMIZE_TO_TRAY, MUSIC_PATHS,
    PreferenceItemExt, SCAN_INTERVAL, SCAN_THREADS, THUMBNAIL_PATH, VOLUME_PERSIST_MODE,
};

#[test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_preference_constants_definitions() {
    assert_eq!(MUSIC_PATHS.id, "music_paths");
    assert_eq!(MUSIC_PATHS.pref_type, PreferenceType::PathSelector as i32);
    assert_eq!(MUSIC_PATHS.title, "Music Paths");

    assert_eq!(EXCLUDE_MUSIC_PATHS.id, "exclude_music_paths");
    assert_eq!(
        EXCLUDE_MUSIC_PATHS.pref_type,
        PreferenceType::PathSelector as i32
    );

    assert_eq!(SCAN_THREADS.id, "scan_threads");
    assert_eq!(
        SCAN_THREADS.pref_type,
        PreferenceType::NumberInputGroup as i32
    );
    assert_eq!(SCAN_THREADS.validation, ">= 0 && <= 128");

    assert_eq!(ARTIST_SPLITTER.id, "artist_splitter");
    assert_eq!(
        ARTIST_SPLITTER.pref_type,
        PreferenceType::TextInputGroup as i32
    );
    assert_eq!(ARTIST_SPLITTER.placeholder.as_deref(), Some(","));

    assert_eq!(SCAN_INTERVAL.id, "scan_interval");
    assert_eq!(
        SCAN_INTERVAL.pref_type,
        PreferenceType::NumberInputGroup as i32
    );
    assert_eq!(SCAN_INTERVAL.validation, ">= 0");

    assert_eq!(THUMBNAIL_PATH.id, "thumbnail_path");
    assert_eq!(
        THUMBNAIL_PATH.pref_type,
        PreferenceType::SingleFileInput as i32
    );

    assert_eq!(ARTWORK_PATH.id, "artwork_path");
    assert_eq!(
        ARTWORK_PATH.pref_type,
        PreferenceType::SingleFileInput as i32
    );

    assert_eq!(AUTO_STARTUP.id, "auto_startup");
    assert_eq!(AUTO_STARTUP.pref_type, PreferenceType::ToggleGroup as i32);

    assert_eq!(MINIMIZE_TO_TRAY.id, "minimize_to_tray");
    assert_eq!(
        MINIMIZE_TO_TRAY.pref_type,
        PreferenceType::ToggleGroup as i32
    );

    assert_eq!(JUKEBOX_MODE.id, "jukebox_mode");
    assert_eq!(JUKEBOX_MODE.pref_type, PreferenceType::ToggleGroup as i32);

    assert_eq!(CLEAR_QUEUE.id, "clear_queue");
    assert_eq!(CLEAR_QUEUE.pref_type, PreferenceType::ToggleGroup as i32);

    assert_eq!(VOLUME_PERSIST_MODE.id, "volume_persist_mode");
    assert_eq!(
        VOLUME_PERSIST_MODE.pref_type,
        PreferenceType::RadioGroup as i32
    );
    assert_eq!(VOLUME_PERSIST_MODE.options.len(), 3);

    assert_eq!(I18N_LANGUAGE.id, "i18n_language");
    assert_eq!(
        I18N_LANGUAGE.pref_type,
        PreferenceType::DropdownGroup as i32
    );
    assert_eq!(I18N_LANGUAGE.options.len(), 31);

    assert_eq!(ACTIVE_THEME_ID.id, "active_theme_id");
    assert_eq!(
        ACTIVE_THEME_ID.pref_type,
        PreferenceType::TextInputGroup as i32
    );
    assert_eq!(
        ACTIVE_THEME_ID.value::<String>().as_deref(),
        Some("current")
    );

    assert_eq!(EXTENSION_REGISTRIES.id, "extension_registries");
    assert_eq!(
        EXTENSION_REGISTRIES.pref_type,
        PreferenceType::TextArrayInput as i32
    );
}

#[test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_preference_item_value_extraction() {
    let item_bool = PreferenceItem {
        id: "test_bool".to_string(),
        value: Some(PreferenceValue {
            value: Some(preference_value::Value::BoolValue(true)),
        }),
        ..Default::default()
    };
    assert_eq!(item_bool.value::<bool>(), Some(true));

    let item_string = PreferenceItem {
        id: "test_str".to_string(),
        value: Some(PreferenceValue {
            value: Some(preference_value::Value::StringValue("test_val".to_string())),
        }),
        ..Default::default()
    };
    assert_eq!(item_string.value::<String>().as_deref(), Some("test_val"));

    let item_number = PreferenceItem {
        id: "test_num".to_string(),
        value: Some(PreferenceValue {
            value: Some(preference_value::Value::NumberValue(42.0)),
        }),
        ..Default::default()
    };
    assert_eq!(item_number.value::<f32>(), Some(42.0));
    assert_eq!(item_number.value::<i32>(), Some(42));
    assert_eq!(item_number.value::<usize>(), Some(42));

    let item_list = PreferenceItem {
        id: "test_list".to_string(),
        value: Some(PreferenceValue {
            value: Some(preference_value::Value::ListValue(StringList {
                values: vec!["a".to_string(), "b".to_string()],
            })),
        }),
        ..Default::default()
    };
    assert_eq!(
        item_list.value::<Vec<String>>(),
        Some(vec!["a".to_string(), "b".to_string()])
    );
}

#[test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_preference_item_value_fallback_to_default() {
    let item_default = PreferenceItem {
        id: "test_fallback".to_string(),
        value: None,
        default: Some(PreferenceValue {
            value: Some(preference_value::Value::StringValue("fallback".to_string())),
        }),
        ..Default::default()
    };
    assert_eq!(item_default.value::<String>().as_deref(), Some("fallback"));
}

#[test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_preference_item_value_multiple_warning() {
    let item_both = PreferenceItem {
        id: "test_both".to_string(),
        value: Some(PreferenceValue {
            value: Some(preference_value::Value::StringValue("active".to_string())),
        }),
        default: Some(PreferenceValue {
            value: Some(preference_value::Value::StringValue("default".to_string())),
        }),
        ..Default::default()
    };
    assert_eq!(item_both.value::<String>().as_deref(), Some("active"));
}
