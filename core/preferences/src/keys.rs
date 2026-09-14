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

use std::sync::LazyLock;

use preferences_proto::moosync::types::{
    PreferenceItem, PreferenceOption, PreferenceType, PreferenceValue, preference_value,
};

pub trait FromPreferenceValue: Sized {
    fn extract(item: &PreferenceItem) -> Option<Self>;
}

pub trait PreferenceItemExt {
    fn value<T: FromPreferenceValue>(&self) -> Option<T>;
}

impl PreferenceItemExt for PreferenceItem {
    #[tracing::instrument(level = "debug", skip_all)]
    fn value<T: FromPreferenceValue>(&self) -> Option<T> {
        if self.value.is_some() && self.default.is_some() {
            tracing::warn!(
                "Multiple values found for preference {}, returning first",
                self.id
            );
        }
        T::extract(self)
    }
}

impl FromPreferenceValue for bool {
    #[tracing::instrument(level = "debug", skip_all)]
    fn extract(item: &PreferenceItem) -> Option<Self> {
        let extract_val = |pv: &PreferenceValue| match &pv.value {
            Some(preference_value::Value::BoolValue(b)) => Some(*b),
            _ => None,
        };
        item.value
            .as_ref()
            .and_then(extract_val)
            .or_else(|| item.default.as_ref().and_then(extract_val))
    }
}

impl FromPreferenceValue for String {
    #[tracing::instrument(level = "debug", skip_all)]
    fn extract(item: &PreferenceItem) -> Option<Self> {
        let extract_val = |pv: &PreferenceValue| match &pv.value {
            Some(preference_value::Value::StringValue(s)) => Some(s.clone()),
            _ => None,
        };
        item.value
            .as_ref()
            .and_then(extract_val)
            .or_else(|| item.default.as_ref().and_then(extract_val))
    }
}

impl FromPreferenceValue for f32 {
    #[tracing::instrument(level = "debug", skip_all)]
    fn extract(item: &PreferenceItem) -> Option<Self> {
        let extract_val = |pv: &PreferenceValue| match &pv.value {
            Some(preference_value::Value::NumberValue(n)) => Some(*n),
            _ => None,
        };
        item.value
            .as_ref()
            .and_then(extract_val)
            .or_else(|| item.default.as_ref().and_then(extract_val))
    }
}

impl FromPreferenceValue for i32 {
    #[tracing::instrument(level = "debug", skip_all)]
    fn extract(item: &PreferenceItem) -> Option<Self> {
        let extract_val = |pv: &PreferenceValue| match &pv.value {
            Some(preference_value::Value::NumberValue(n)) => Some(*n as i32),
            _ => None,
        };
        item.value
            .as_ref()
            .and_then(extract_val)
            .or_else(|| item.default.as_ref().and_then(extract_val))
    }
}

impl FromPreferenceValue for u32 {
    #[tracing::instrument(level = "debug", skip_all)]
    fn extract(item: &PreferenceItem) -> Option<Self> {
        let extract_val = |pv: &PreferenceValue| match &pv.value {
            Some(preference_value::Value::NumberValue(n)) => Some(*n as u32),
            _ => None,
        };
        item.value
            .as_ref()
            .and_then(extract_val)
            .or_else(|| item.default.as_ref().and_then(extract_val))
    }
}

impl FromPreferenceValue for usize {
    #[tracing::instrument(level = "debug", skip_all)]
    fn extract(item: &PreferenceItem) -> Option<Self> {
        let extract_val = |pv: &PreferenceValue| match &pv.value {
            Some(preference_value::Value::NumberValue(n)) => Some(*n as usize),
            _ => None,
        };
        item.value
            .as_ref()
            .and_then(extract_val)
            .or_else(|| item.default.as_ref().and_then(extract_val))
    }
}

impl FromPreferenceValue for Vec<String> {
    #[tracing::instrument(level = "debug", skip_all)]
    fn extract(item: &PreferenceItem) -> Option<Self> {
        let extract_val = |pv: &PreferenceValue| match &pv.value {
            Some(preference_value::Value::ListValue(l)) => Some(l.values.clone()),
            _ => None,
        };
        item.value
            .as_ref()
            .and_then(extract_val)
            .or_else(|| item.default.as_ref().and_then(extract_val))
    }
}

macro_rules! define_preferences {
    ($(
        $NAME:ident = $key:literal => {
            type: $pref_type:ident,
            title: $title:literal,
            subtitle: $subtitle:literal
            $(, placeholder: $placeholder:literal)?
            $(, validation: $validation:literal)?
            $(, default_string: $default_string:literal)?
            $(, default_bool: $default_bool:literal)?
            $(, default_number: $default_number:literal)?
            $(, default_list: [$($default_item:literal),* $(,)?])?
            $(, options: [$(($opt_id:literal, $opt_label:literal)),* $(,)?])?
            $(,)?
        };
    )*) => {
        $(
            pub static $NAME: LazyLock<PreferenceItem> = LazyLock::new(|| {
                #[allow(unused_mut, unused_assignments)]
                let mut placeholder: Option<String> = None;
                $(
                    placeholder = Some($placeholder.to_string());
                )?

                #[allow(unused_mut, unused_assignments)]
                let mut validation: String = String::new();
                $(
                    validation = $validation.to_string();
                )?

                #[allow(unused_mut, unused_assignments)]
                let mut default: Option<PreferenceValue> = None;
                $(
                    default = Some(PreferenceValue {
                        value: Some(preference_value::Value::StringValue($default_string.to_string())),
                    });
                )?
                $(
                    default = Some(PreferenceValue {
                        value: Some(preference_value::Value::BoolValue($default_bool)),
                    });
                )?
                $(
                    default = Some(PreferenceValue {
                        value: Some(preference_value::Value::NumberValue($default_number as f32)),
                    });
                )?
                $(
                    default = Some(PreferenceValue {
                        value: Some(preference_value::Value::ListValue(StringList {
                            values: vec![$($default_item.to_string()),*],
                        })),
                    });
                )?

                #[allow(unused_mut, unused_assignments)]
                let mut options: Vec<PreferenceOption> = Vec::new();
                $(
                    options = vec![
                        $(
                            PreferenceOption {
                                id: $opt_id.to_string(),
                                label: $opt_label.to_string(),
                            },
                        )*
                    ];
                )?

                PreferenceItem {
                    id: $key.to_string(),
                    pref_type: PreferenceType::$pref_type as i32,
                    title: $title.to_string(),
                    subtitle: $subtitle.to_string(),
                    placeholder,
                    validation,
                    options,
                    value: None,
                    default,
                }
            });
        )*
    };
}

define_preferences! {
    MUSIC_PATHS = "music_paths" => {
        type: PathSelector,
        title: "Music Paths",
        subtitle: "Directories to scan for local audio files",
    };
    EXCLUDE_MUSIC_PATHS = "exclude_music_paths" => {
        type: PathSelector,
        title: "Exclude Music Paths",
        subtitle: "Directories to exclude from scanning",
    };
    SCAN_THREADS = "scan_threads" => {
        type: NumberInputGroup,
        title: "Scan Threads",
        subtitle: "Number of parallel worker threads used when scanning local directories",
        validation: ">= 0 && <= 128",
    };
    ARTIST_SPLITTER = "artist_splitter" => {
        type: TextInputGroup,
        title: "Artist Splitter",
        subtitle: "Character(s) used to separate multiple artists in track tags",
        placeholder: ",",
    };
    SCAN_INTERVAL = "scan_interval" => {
        type: NumberInputGroup,
        title: "Scan Interval",
        subtitle: "Interval in seconds between automatic library scans (0 to disable)",
        validation: ">= 0",
    };
    THUMBNAIL_PATH = "thumbnail_path" => {
        type: SingleFileInput,
        title: "Thumbnail Path",
        subtitle: "Custom path to store generated thumbnails and album art cache",
    };
    ARTWORK_PATH = "artwork_path" => {
        type: SingleFileInput,
        title: "Artwork Path",
        subtitle: "Custom path to store fetched artist and playlist artwork",
    };
    AUTO_STARTUP = "auto_startup" => {
        type: ToggleGroup,
        title: "Auto Startup",
        subtitle: "Launch Moosync automatically when logging into the system",
    };
    MINIMIZE_TO_TRAY = "minimize_to_tray" => {
        type: ToggleGroup,
        title: "Minimize to Tray",
        subtitle: "Keep Moosync running in the system tray when closing the window",
    };
    JUKEBOX_MODE = "jukebox_mode" => {
        type: ToggleGroup,
        title: "Jukebox Mode",
        subtitle: "Prevent users from modifying the playback queue or stopping playback",
    };
    CLEAR_QUEUE = "clear_queue" => {
        type: ToggleGroup,
        title: "Clear Queue on Exit",
        subtitle: "Automatically clear the playback queue when closing the application",
    };
    VOLUME_PERSIST_MODE = "volume_persist_mode" => {
        type: RadioGroup,
        title: "Volume Persistence",
        subtitle: "How player volume levels are preserved between sessions",
        options: [
            ("no_persist", "Do not persist volume"),
            ("persist_separate", "Persist separate volume per output device"),
            ("persist_clamp", "Persist volume and clamp on restart"),
        ],
    };
    I18N_LANGUAGE = "i18n_language" => {
        type: DropdownGroup,
        title: "Language",
        subtitle: "Application display language",
        options: [
            ("af_ZA", "Afrikaans"),
            ("ar_SA", "العربية"),
            ("ca_ES", "Català"),
            ("cs_CZ", "Čeština"),
            ("da_DK", "Dansk"),
            ("de_DE", "Deutsch"),
            ("el_GR", "Ελληνικά"),
            ("en_US", "English"),
            ("es_ES", "Español"),
            ("fi_FI", "Suomi"),
            ("fr_FR", "Français"),
            ("he_IL", "עברית"),
            ("hi_IN", "हिन्दी"),
            ("hu_HU", "Magyar"),
            ("it_IT", "Italiano"),
            ("ja_JP", "日本語"),
            ("ko_KR", "한국어"),
            ("nl_NL", "Nederlands"),
            ("no_NO", "Norsk"),
            ("pl_PL", "Polski"),
            ("pt_BR", "Português do Brasil"),
            ("pt_PT", "Português"),
            ("ro_RO", "Română"),
            ("ru_RU", "Русский"),
            ("sr_SP", "Srpski"),
            ("sv_SE", "Svenska"),
            ("tr_TR", "Türkçe"),
            ("uk_UA", "Українська"),
            ("vi_VN", "Tiếng Việt"),
            ("zh_CN", "简体中文"),
            ("zh_TW", "繁體中文"),
        ],
    };
    ACTIVE_THEME_ID = "active_theme_id" => {
        type: TextInputGroup,
        title: "Active Theme",
        subtitle: "Selected theme ID",
        default_string: "current",
    };
    EXTENSION_REGISTRIES = "extension_registries" => {
        type: TextArrayInput,
        title: "Extension Registries",
        subtitle: "Add or remove extension registry URLs (manifest.json)",
        placeholder: "https://example.com/manifest.json",
        validation: "^https?://.+",
    };
}
