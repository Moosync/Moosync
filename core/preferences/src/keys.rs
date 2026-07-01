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

pub trait PreferenceKey {
    type Value: serde::Serialize + serde::de::DeserializeOwned + std::clone::Clone + std::fmt::Debug;
    fn key(&self) -> String;
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct ExtensionKey {
    pub package_name: String,
    pub key: String,
}

impl PreferenceKey for ExtensionKey {
    type Value = extensions_proto::struct_proto::google::protobuf::Value;
    #[tracing::instrument(level = "debug", skip_all)]
    fn key(&self) -> String { format!("extensions.{}.{}", self.package_name, self.key) }
}

macro_rules! define_keys {
    ($($name:ident: $val:ty = $key:expr;)*) => {
        $(
            pub struct $name;
            impl PreferenceKey for $name {
                type Value = $val;
                #[tracing::instrument(level = "debug", skip_all)]
                fn key(&self) -> String {
                    $key.to_string()
                }
            }
            impl From<$name> for String {
                fn from(_: $name) -> String {
                    $key.to_string()
                }
            }
            impl ::types::subscription::ToFilterKeys<String> for $name {
                #[tracing::instrument(level = "debug", skip_all)]
                fn to_filter_keys(self) -> Vec<String> {
                    vec![$key.to_string()]
                }
            }
            impl PartialEq<String> for $name {
                #[tracing::instrument(level = "debug", skip_all)]
                fn eq(&self, other: &String) -> bool {
                    $key == other
                }
            }
            impl PartialEq<str> for $name {
                #[tracing::instrument(level = "debug", skip_all)]
                fn eq(&self, _other: &str) -> bool {
                    $key == _other
                }
            }
            impl PartialEq<$name> for String {
                #[tracing::instrument(level = "debug", skip_all)]
                fn eq(&self, _other: &$name) -> bool {
                    self == $key
                }
            }
            impl PartialEq<$name> for str {
                #[tracing::instrument(level = "debug", skip_all)]
                fn eq(&self, _other: &$name) -> bool {
                    self == $key
                }
            }
        )*
    };
}

define_keys! {
    MusicPaths: Vec<String> = "music_paths";
    ExcludeMusicPaths: Vec<String> = "exclude_music_paths";
    ScanThreads: i32 = "scan_threads";
    ArtistSplitter: String = "artist_splitter";
    ScanInterval: i32 = "scan_interval";
    ThumbnailPath: String = "thumbnail_path";
    ArtworkPath: String = "artwork_path";
    AutoStartup: bool = "auto_startup";
    MinimizeToTray: bool = "minimize_to_tray";
    JukeboxMode: bool = "jukebox_mode";
    ClearQueue: bool = "clear_queue";
    VolumePersistMode: String = "volume_persist_mode";
    I18nLanguage: String = "i18n_language";
    ActiveThemeId: String = "active_theme_id";
}
