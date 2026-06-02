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

pub mod canvaz;
pub mod errors;
pub mod mpris;
pub mod plugin;
pub mod preferences;
pub mod providers;
pub mod ui;
pub mod window;

#[cfg(target_os = "android")]
pub mod android;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(non_camel_case_types)]
pub enum ScanProgress {
    STOPPED,
    PROGRESS(u8),
}

pub mod prelude {
    use std::time::Duration;

    use songs_proto::moosync::types::{InnerSong, Song, SongType};
    use themes_proto::moosync::types::{ThemeDetails, ThemeItem};

    use crate::errors::MoosyncError;

    pub trait ThemeExt {
        fn get_theme_item_or_default(&self) -> ThemeItem;
    }

    pub trait InnerSongExt {
        fn get_type_or_default(&self) -> SongType;
    }

    impl InnerSongExt for InnerSong {
        fn get_type_or_default(&self) -> SongType {
            SongType::try_from(self.r#type).unwrap_or_else(|e| {
                tracing::error!(
                    "Failed to parse song type for song: {:?}, {}. Using SongType::Local",
                    self,
                    e
                );
                SongType::Local
            })
        }
    }

    pub fn format_duration(secs: i64) -> String {
        let minutes = secs / 60;
        let seconds = secs % 60;
        format!("{:02}:{:02}", minutes, seconds)
    }

    pub trait SongsExt {
        fn get_id(&self) -> Option<String>;
        fn get_title(&self) -> Option<String>;
        fn get_duration_or_default(&self) -> std::time::Duration;
        fn get_cover_high(&self) -> Option<String>;
        fn get_cover_low(&self) -> Option<String>;
        fn get_playback_url(&self) -> Option<String>;
        fn get_type_or_default(&self) -> SongType;
        fn get_path(&self) -> Option<String>;
        fn get_extension(&self) -> Option<String>;
        fn get_lyrics(&self) -> Option<String>;
        fn get_date(&self) -> Option<String>;
        fn get_artist_string(&self) -> Option<String>;
        fn get_album_string(&self) -> Option<String>;
        fn format_duration(&self) -> String {
            let duration = self.get_duration_or_default();
            if duration == std::time::Duration::ZERO {
                return "Unknown".to_string();
            }
            format_duration(duration.as_secs() as i64)
        }
    }

    impl SongsExt for Song {
        fn get_id(&self) -> Option<String> {
            self.song.as_ref().and_then(|s| s.id.clone())
        }
        fn get_title(&self) -> Option<String> {
            self.song.as_ref().and_then(|s| s.title.clone())
        }
        fn get_duration_or_default(&self) -> std::time::Duration {
            self.song
                .as_ref()
                .and_then(|s| {
                    s.duration
                        .as_ref()
                        .map(|d| proto_duration_to_core(d.clone()).unwrap_or_default())
                })
                .unwrap_or(std::time::Duration::ZERO)
        }
        fn get_cover_high(&self) -> Option<String> {
            self.song
                .as_ref()
                .and_then(|s| s.song_cover_path_high.clone())
        }
        fn get_cover_low(&self) -> Option<String> {
            let cover_low = self
                .song
                .as_ref()
                .and_then(|s| s.song_cover_path_low.clone());
            if cover_low.is_none() {
                return self.get_cover_high();
            }
            cover_low
        }
        fn get_playback_url(&self) -> Option<String> {
            self.song.as_ref().and_then(|s| s.playback_url.clone())
        }
        fn get_type_or_default(&self) -> SongType {
            self.song
                .as_ref()
                .and_then(|s| match SongType::try_from(s.r#type) {
                    Ok(t) => Some(t),
                    Err(e) => {
                        tracing::error!(
                            "Failed to parse song type for song: {:?}, {}. Using SongType::Local",
                            s,
                            e
                        );
                        None
                    }
                })
                .unwrap_or(SongType::Local)
        }
        fn get_path(&self) -> Option<String> {
            self.song.as_ref().and_then(|s| s.path.clone())
        }
        fn get_extension(&self) -> Option<String> {
            self.song
                .as_ref()
                .and_then(|s| s.provider_extension.clone())
        }
        fn get_lyrics(&self) -> Option<String> {
            self.song.as_ref().and_then(|s| s.lyrics.clone())
        }
        fn get_date(&self) -> Option<String> {
            self.song.as_ref().and_then(|s| s.date.clone())
        }

        fn get_artist_string(&self) -> Option<String> {
            if self.artists.is_empty() {
                return None;
            }

            Some(
                self.artists
                    .iter()
                    .map(|a| a.artist_name())
                    .collect::<Vec<&str>>()
                    .join(","),
            )
        }
        fn get_album_string(&self) -> Option<String> {
            self.album.as_ref().and_then(|a| a.album_name.clone())
        }
    }

    pub trait ThemeItemExt {
        fn get_constant(&self, key: &str) -> Option<String>;
        fn set_constant(&mut self, key: &str, value: String);
        fn get_all_keys(&self) -> std::collections::HashSet<String>;
    }

    impl ThemeItemExt for ThemeItem {
        fn get_constant(&self, key: &str) -> Option<String> {
            match key {
                "primary" => if !self.primary.is_empty() { Some(self.primary.clone()) } else { self.constants.get(key).cloned() },
                "secondary" => if !self.secondary.is_empty() { Some(self.secondary.clone()) } else { self.constants.get(key).cloned() },
                "tertiary" => if !self.tertiary.is_empty() { Some(self.tertiary.clone()) } else { self.constants.get(key).cloned() },
                "textPrimary" => if !self.text_primary.is_empty() { Some(self.text_primary.clone()) } else { self.constants.get(key).cloned() },
                "textSecondary" => if !self.text_secondary.is_empty() { Some(self.text_secondary.clone()) } else { self.constants.get(key).cloned() },
                "textInverse" => if !self.text_inverse.is_empty() { Some(self.text_inverse.clone()) } else { self.constants.get(key).cloned() },
                "accent" => if !self.accent.is_empty() { Some(self.accent.clone()) } else { self.constants.get(key).cloned() },
                "divider" => if !self.divider.is_empty() { Some(self.divider.clone()) } else { self.constants.get(key).cloned() },
                _ => self.constants.get(key).cloned(),
            }
        }

        fn set_constant(&mut self, key: &str, value: String) {
            match key {
                "primary" => {
                    self.primary = value;
                    self.constants.remove("primary");
                }
                "secondary" => {
                    self.secondary = value;
                    self.constants.remove("secondary");
                }
                "tertiary" => {
                    self.tertiary = value;
                    self.constants.remove("tertiary");
                }
                "textPrimary" => {
                    self.text_primary = value;
                    self.constants.remove("textPrimary");
                    self.constants.remove("text_primary");
                }
                "textSecondary" => {
                    self.text_secondary = value;
                    self.constants.remove("textSecondary");
                    self.constants.remove("text_secondary");
                }
                "textInverse" => {
                    self.text_inverse = value;
                    self.constants.remove("textInverse");
                    self.constants.remove("text_inverse");
                }
                "accent" => {
                    self.accent = value;
                    self.constants.remove("accent");
                }
                "divider" => {
                    self.divider = value;
                    self.constants.remove("divider");
                }
                _ => {
                    self.constants.insert(key.to_string(), value);
                }
            }
        }

        fn get_all_keys(&self) -> std::collections::HashSet<String> {
            let mut keys = std::collections::HashSet::new();
            keys.insert("primary".to_string());
            keys.insert("secondary".to_string());
            keys.insert("tertiary".to_string());
            keys.insert("textPrimary".to_string());
            keys.insert("textSecondary".to_string());
            keys.insert("textInverse".to_string());
            keys.insert("accent".to_string());
            keys.insert("divider".to_string());
            for k in self.constants.keys() {
                keys.insert(k.clone());
            }
            keys
        }
    }

    // TODO: Find a way to generate this default theme item from the Slint constants directly
    pub fn get_default_theme_item() -> ThemeItem {
        let mut item = ThemeItem::default();

        // Colors
        item.set_constant("primary", "#212121".into());
        item.set_constant("secondary", "#282828".into());
        item.set_constant("tertiary", "#151515".into());
        item.set_constant("textPrimary", "#ffffff".into());
        item.set_constant("textSecondary", "#565656".into());
        item.set_constant("textInverse", "#000000".into());
        item.set_constant("accent", "#65cb88".into());
        item.set_constant("divider", "rgba(0, 0, 0, 0.0)".into());

        // Font Sizes
        item.set_constant("pageHeader", "32px".into());
        item.set_constant("modelTitle", "16px".into());
        item.set_constant("subtitle", "14px".into());
        item.set_constant("body", "13px".into());
        item.set_constant("caption", "11px".into());
        item.set_constant("extraSmall", "10px".into());

        // Spacing
        item.set_constant("spacingXxl", "30px".into());
        item.set_constant("spacingXl", "20px".into());
        item.set_constant("spacingLg", "16px".into());
        item.set_constant("spacingMd", "12px".into());
        item.set_constant("spacingSm", "10px".into());
        item.set_constant("spacingXs", "8px".into());
        item.set_constant("spacingXxs", "4px".into());
        item.set_constant("spacingTiny", "2px".into());

        // Paddings
        item.set_constant("paddingLg", "16px".into());
        item.set_constant("paddingMd", "14px".into());
        item.set_constant("paddingSm", "10px".into());
        item.set_constant("paddingXs", "8px".into());

        // Border Radius
        item.set_constant("borderRadiusXl", "16px".into());
        item.set_constant("borderRadiusLg", "8px".into());
        item.set_constant("borderRadiusMd", "6px".into());
        item.set_constant("borderRadiusSm", "1.5px".into());

        // Border
        item.set_constant("borderWidth", "1px".into());

        // Icon Sizes
        item.set_constant("iconSizeXl", "24px".into());
        item.set_constant("iconSizeLg", "20px".into());
        item.set_constant("iconSizeMd", "16px".into());
        item.set_constant("iconSizeSm", "13px".into());

        // Component Dimensions
        item.set_constant("cardWidth", "200px".into());
        item.set_constant("sidebarWidth", "261px".into());
        item.set_constant("bottombarHeight", "72px".into());
        item.set_constant("topbarHeight", "48px".into());
        item.set_constant("sidebarButtonHeight", "48px".into());
        item.set_constant("sidebarHeaderHeight", "48px".into());
        item.set_constant("songListItemHeight", "60px".into());
        item.set_constant("trackInfoThumbnailSize", "44px".into());
        item.set_constant("songListItemThumbnailSize", "44px".into());
        item.set_constant("songDetailsThumbnailSize", "250px".into());
        item.set_constant("exploreThumbnailSize", "200px".into());
        item.set_constant("playbackControlsIconSize", "18px".into());
        item.set_constant("playbackButtonSize", "32px".into());
        item.set_constant("navButtonSize", "28px".into());
        item.set_constant("topbarIconSize", "32px".into());
        item.set_constant("sidebarHeaderToggleSize", "32px".into());
        item.set_constant("searchBarHeight", "36px".into());
        item.set_constant("playbackSliderTrackHeight", "3px".into());
        item.set_constant("volumeSliderTrackHeight", "3px".into());
        item.set_constant("sliderThumbSize", "12px".into());
        item.set_constant("sliderThumbHoverSize", "14px".into());
        item.set_constant("bottombarProgressBarHeight", "14px".into());
        item.set_constant("volumeControlProgressBarHeight", "14px".into());
        item.set_constant("trackInfoWidth", "260px".into());
        item.set_constant("volumeSliderWidth", "80px".into());
        item.set_constant("volumeControlWidth", "100px".into());

        // Settings
        item.set_constant("settingsSidebarWidth", "200px".into());
        item.set_constant("settingsCloseButtonSize", "24px".into());

        // Settings Component Details
        item.set_constant("dropdownHeight", "35px".into());
        item.set_constant("dropdownOptionHeight", "25px".into());
        item.set_constant("numberInputHeight", "35px".into());
        item.set_constant("numberInputWidth", "150px".into());
        item.set_constant("textInputHeight", "35px".into());
        item.set_constant("fileInputHeight", "35px".into());
        item.set_constant("rgbaSwatchWidth", "60px".into());
        item.set_constant("rgbaSwatchHeight", "35px".into());
        item.set_constant("rgbaPopupWidth", "260px".into());
        item.set_constant("themeCardWidth", "220px".into());
        item.set_constant("themeCardPreviewHeight", "110px".into());
        item.set_constant("quickAccentSwatchSize", "32px".into());
        item.set_constant("advancedAccordionHeaderHeight", "40px".into());
        item.set_constant("themeSavePopupWidth", "450px".into());
        item.set_constant("toggleWidth", "44px".into());
        item.set_constant("toggleHeight", "24px".into());
        item.set_constant("toggleThumbSize", "20px".into());
        item.set_constant("radioOptionHeight", "30px".into());
        item.set_constant("radioOptionCircleSize", "16px".into());

        // Modal Sizes
        item.set_constant("modalWidthXl", "800px".into());
        item.set_constant("modalHeightXl", "800px".into());
        item.set_constant("modalWidthLg", "600px".into());
        item.set_constant("modalHeightLg", "600px".into());
        item.set_constant("modalWidthMd", "400px".into());
        item.set_constant("modalHeightMd", "400px".into());
        item.set_constant("modalWidthSm", "200px".into());
        item.set_constant("modalHeightSm", "200px".into());

        item
    }

    impl ThemeExt for ThemeDetails {
        fn get_theme_item_or_default(&self) -> ThemeItem {
            self.theme.clone().unwrap_or_else(get_default_theme_item)
        }
    }

    fn format_position(duration: Duration) -> String {
        let secs = duration.as_secs();
        let minutes = secs / 60;
        let seconds = secs % 60;
        format!("{:02}:{:02}", minutes, seconds)
    }

    // Assuming your generated module is `pb` and the struct is `pb::Duration`
    fn proto_duration_to_core(
        proto_dur: songs_proto::duration_proto::google::protobuf::Duration,
    ) -> Result<std::time::Duration, MoosyncError> {
        if proto_dur.seconds < 0 || proto_dur.nanos < 0 {
            return Err("Cannot convert negative protobuf duration to core::time::Duration".into());
        }

        // 3. Safely cast to unsigned integers and create the standard Duration
        Ok(std::time::Duration::new(
            proto_dur.seconds as u64,
            proto_dur.nanos as u32,
        ))
    }

    pub fn core_to_proto_duration(
        rust_dur: std::time::Duration,
    ) -> songs_proto::duration_proto::google::protobuf::Duration {
        songs_proto::duration_proto::google::protobuf::Duration {
            seconds: rust_dur.as_secs() as i64,
            nanos: rust_dur.subsec_nanos() as i32,
        }
    }
}
