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
pub mod common;
// pub mod entities;
pub mod errors;

pub mod mpris;
pub mod preferences;
// pub mod songs;

pub mod providers;

pub mod extensions;
pub mod ui;
pub mod window;

#[cfg(test)]
mod tests;

pub mod prelude {
    use songs_proto::moosync::types::{InnerSong, Song, SongType};
    use themes_proto::moosync::types::{ThemeDetails, ThemeItem};

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

    pub trait SongsExt {
        fn get_id(&self) -> Option<String>;
        fn get_title(&self) -> Option<String>;
        fn get_duration_or_default(&self) -> f64;
        fn get_cover(&self) -> Option<String>;
        fn get_playback_url(&self) -> Option<String>;
        fn get_type_or_default(&self) -> SongType;
        fn get_path(&self) -> Option<String>;
        fn get_extension(&self) -> Option<String>;
        fn get_lyrics(&self) -> Option<String>;
        fn get_date(&self) -> Option<String>;
    }

    impl SongsExt for Song {
        fn get_id(&self) -> Option<String> {
            self.song.as_ref().and_then(|s| s.id.clone())
        }
        fn get_title(&self) -> Option<String> {
            self.song.as_ref().and_then(|s| s.title.clone())
        }
        fn get_duration_or_default(&self) -> f64 {
            self.song.as_ref().and_then(|s| s.duration).unwrap_or(-1f64)
        }
        fn get_cover(&self) -> Option<String> {
            self.song
                .as_ref()
                .and_then(|s| s.song_cover_path_high.clone())
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
    }

    impl ThemeExt for ThemeDetails {
        fn get_theme_item_or_default(&self) -> ThemeItem {
            self.theme.clone().unwrap_or(ThemeItem {
                primary: "#212121".into(),
                secondary: "#282828".into(),
                tertiary: "#151515".into(),
                text_primary: "#ffffff".into(),
                text_secondary: "#565656".into(),
                text_inverse: "#000000".into(),
                accent: "#65CB88".into(),
                divider: "rgba(79, 79, 79, 0.67)".into(),
                custom_css: None,
            })
        }
    }
}
