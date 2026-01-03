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

use std::{fmt::Display, str::FromStr};

use crate::errors::MoosyncError;
use bitcode::{Decode, Encode};
use serde::{Deserialize, Serialize};

use super::{
    common::{SearchByTerm, deserialize_default},
    entities::{Album, Artist, Genre, Playlist},
};

#[derive(Debug, Default, Deserialize, Serialize, Clone, PartialEq, Eq, Copy, Encode, Decode)]
pub enum SongType {
    #[default]
    LOCAL,
    URL,
    SPOTIFY,
    DASH,
    HLS,
}
impl Display for SongType {
    #[tracing::instrument(level = "debug", skip(self, f))]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let data = match self {
            SongType::Local => "LOCAL",
            SongType::Url => "URL",
            SongType::Spotify => "SPOTIFY",
            SongType::Dash => "DASH",
            SongType::Hls => "HLS",
        };
        write!(f, "{}", data)
    }
}

impl FromStr for SongType {
    type Err = MoosyncError;

    #[tracing::instrument(level = "debug", skip(s))]
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "LOCAL" => Ok(SongType::Local),
            "URL" => Ok(SongType::Url),
            "SPOTIFY" => Ok(SongType::Spotify),
            "DASH" => Ok(SongType::Dash),
            "HLS" => Ok(SongType::Hls),
            _ => Err(MoosyncError::String(format!("Invalid song type: {}", s))),
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Default, Clone, Encode, Decode)]
pub struct InnerSong {
    pub _id: Option<String>,
    pub path: Option<String>,
    pub size: Option<f64>,
    pub inode: Option<String>,
    pub deviceno: Option<String>,
    pub title: Option<String>,
    pub date: Option<String>,
    pub year: Option<String>,
    pub lyrics: Option<String>,
    #[serde(rename = "releaseType")]
    pub release_type: Option<String>,
    pub bitrate: Option<f64>,
    pub codec: Option<String>,
    pub container: Option<String>,
    pub duration: Option<f64>,
    #[serde(rename = "sampleRate")]
    pub sample_rate: Option<f64>,
    pub hash: Option<String>,
    #[serde(rename = "type")]
    pub type_: SongType,
    pub url: Option<String>,
    #[serde(rename = "song_coverPath_high")]
    pub song_cover_path_high: Option<String>,
    #[serde(rename = "playbackUrl")]
    pub playback_url: Option<String>,
    #[serde(rename = "song_coverPath_low")]
    pub song_cover_path_low: Option<String>,
    pub date_added: Option<i64>,
    pub provider_extension: Option<String>,
    pub icon: Option<String>,
    pub show_in_library: Option<bool>,
    pub track_no: Option<f64>,
    pub library_item: Option<bool>,
}

impl std::hash::Hash for InnerSong {
    #[tracing::instrument(level = "debug", skip(self, state))]
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self._id.hash(state);
    }
}

impl PartialEq for InnerSong {
    #[tracing::instrument(level = "debug", skip(self, other))]
    fn eq(&self, other: &Self) -> bool {
        self._id == other._id
    }
}

impl Eq for InnerSong {}

impl SearchByTerm for InnerSong {
    #[tracing::instrument(level = "debug", skip(term))]
    fn search_by_term(term: Option<String>) -> Self {
        let mut data = Self::default();
        data.title.clone_from(&term);
        data.path = term;

        data
    }
}

#[derive(Debug, Deserialize, Clone, Default, Serialize, PartialEq)]
pub struct SearchableSong {
    pub _id: Option<String>,
    pub path: Option<String>,
    pub title: Option<String>,

    pub sample_rate: Option<f64>,
    pub hash: Option<String>,
    pub type_: Option<SongType>,
    pub url: Option<String>,

    pub playback_url: Option<String>,
    pub provider_extension: Option<String>,
    pub show_in_library: Option<bool>,
}

#[derive(Debug, Deserialize, Clone, Serialize, Default, PartialEq)]
pub struct GetSongOptions {
    pub song: Option<SearchableSong>,
    pub artist: Option<Artist>,
    pub album: Option<Album>,
    pub genre: Option<Genre>,
    pub playlist: Option<Playlist>,
    pub inclusive: Option<bool>,
}

#[derive(Default, Deserialize, Serialize, Clone, PartialEq, Eq, Encode, Decode)]
pub struct Song {
    #[serde(flatten)]
    pub song: InnerSong,
    #[serde(default, deserialize_with = "deserialize_default")]
    pub album: Option<Album>,
    #[serde(default, deserialize_with = "deserialize_default")]
    pub artists: Option<Vec<Artist>>,
    #[serde(default, deserialize_with = "deserialize_default")]
    pub genre: Option<Vec<Genre>>,
}

impl std::fmt::Debug for Song {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let artist_names = self
            .artists
            .as_ref()
            .map(|artists| {
                artists
                    .iter()
                    .map(|a| a.artist_name.clone().unwrap_or_default())
                    .collect::<Vec<_>>()
                    .join(", ")
            })
            .unwrap_or_else(|| "No Artist".to_string());

        let title = self.song.title.as_deref().unwrap_or("No Title");
        let song_id = self.song.id.as_deref().unwrap_or("No ID");

        write!(f, "{} - {} ({})", artist_names, title, song_id)
    }
}

impl std::hash::Hash for Song {
    #[tracing::instrument(level = "debug", skip(self, state))]
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.song.id.hash(state);
    }
}

#[derive(Debug, Default, Deserialize, Serialize, Clone)]
pub struct AllAnalytics {
    pub total_listen_time: f64,
    pub songs: Vec<(String, f64)>,
}
