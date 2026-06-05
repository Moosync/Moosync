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

    use songs_proto::moosync::types::{
        Album, Artist, EntityResult, Genre, InnerSong, Playlist, Song, SongType, entity_result,
    };
    use themes_proto::moosync::types::{ThemeDetails, ThemeItem};

    use crate::errors::MoosyncError;

    pub trait EntityResultExt {
        fn get_albums(&self) -> Option<Vec<Album>>;
        fn get_artists(&self) -> Option<Vec<Artist>>;
        fn get_genres(&self) -> Option<Vec<Genre>>;
        fn get_playlists(&self) -> Option<Vec<Playlist>>;
    }

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
        fn get_id(&self) -> Option<String> { self.song.as_ref().and_then(|s| s.id.clone()) }
        fn get_title(&self) -> Option<String> { self.song.as_ref().and_then(|s| s.title.clone()) }
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
        fn get_path(&self) -> Option<String> { self.song.as_ref().and_then(|s| s.path.clone()) }
        fn get_extension(&self) -> Option<String> {
            self.song
                .as_ref()
                .and_then(|s| s.provider_extension.clone())
        }
        fn get_lyrics(&self) -> Option<String> { self.song.as_ref().and_then(|s| s.lyrics.clone()) }
        fn get_date(&self) -> Option<String> { self.song.as_ref().and_then(|s| s.date.clone()) }

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

    theme_macro::generate_theme_impl!("ui/slint/src/constants.slint");

    impl ThemeExt for ThemeDetails {
        fn get_theme_item_or_default(&self) -> ThemeItem {
            self.theme.clone().unwrap_or_else(get_default_theme_item)
        }
    }

    impl EntityResultExt for EntityResult {
        fn get_albums(&self) -> Option<Vec<Album>> {
            match &self.result {
                Some(entity_result::Result::Albums(album_list)) => Some(album_list.albums.clone()),
                _ => None,
            }
        }

        fn get_artists(&self) -> Option<Vec<Artist>> {
            match &self.result {
                Some(entity_result::Result::Artists(artist_list)) => {
                    Some(artist_list.artists.clone())
                }
                _ => None,
            }
        }

        fn get_genres(&self) -> Option<Vec<Genre>> {
            match &self.result {
                Some(entity_result::Result::Genres(genre_list)) => Some(genre_list.genres.clone()),
                _ => None,
            }
        }

        fn get_playlists(&self) -> Option<Vec<Playlist>> {
            match &self.result {
                Some(entity_result::Result::Playlists(playlist_list)) => {
                    Some(playlist_list.playlists.clone())
                }
                _ => None,
            }
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
