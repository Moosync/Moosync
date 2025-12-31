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

use bitcode::{Decode, Encode};
use serde::{Deserialize, Serialize};

use super::{
    common::{SearchByTerm, deserialize_default},
    songs::Song,
};

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Encode, Decode)]
pub struct EntityInfo(pub String);

#[derive(Deserialize, Serialize, Default, Clone, Debug, Encode, Decode)]

pub struct Album {
    pub album_id: Option<String>,
    pub album_name: Option<String>,
    pub album_artist: Option<String>,
    #[serde(rename = "album_coverPath_high")]
    pub album_coverpath_high: Option<String>,
    #[serde(default)]
    pub album_song_count: f64,
    pub year: Option<String>,
    #[serde(rename = "album_coverPath_low")]
    pub album_coverpath_low: Option<String>,
    pub album_extra_info: Option<EntityInfo>,
}

impl std::hash::Hash for Album {
    #[tracing::instrument(level = "debug", skip(self, state))]
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.album_id.hash(state);
    }
}

impl PartialEq for Album {
    #[tracing::instrument(level = "debug", skip(self, other))]
    fn eq(&self, other: &Self) -> bool {
        self.album_id == other.album_id
    }
}

impl Eq for Album {}

impl Ord for Album {
    #[tracing::instrument(level = "debug", skip(self, other))]
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        let album_name = self
            .album_name
            .as_ref()
            .unwrap_or(&String::new())
            .to_lowercase();
        let other_album_name = other
            .album_name
            .as_ref()
            .unwrap_or(&String::new())
            .to_lowercase();
        album_name.cmp(&other_album_name)
    }
}

impl PartialOrd for Album {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl SearchByTerm for Album {
    #[tracing::instrument(level = "debug", skip(term))]
    fn search_by_term(term: Option<String>) -> Self {
        Self {
            album_name: term,
            ..Default::default()
        }
    }
}

#[derive(Deserialize, Serialize, Default, Clone, Debug, Encode, Decode)]
pub struct Artist {
    pub artist_id: Option<String>,
    pub artist_mbid: Option<String>,
    pub artist_name: Option<String>,
    #[serde(rename = "artist_coverPath")]
    pub artist_coverpath: Option<String>,
    #[serde(default)]
    pub artist_song_count: f64,
    pub artist_extra_info: Option<EntityInfo>,
    pub sanitized_artist_name: Option<String>,
}

impl std::hash::Hash for Artist {
    #[tracing::instrument(level = "debug", skip(self, state))]
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.artist_id.hash(state);
    }
}

impl PartialEq for Artist {
    #[tracing::instrument(level = "debug", skip(self, other))]
    fn eq(&self, other: &Self) -> bool {
        self.artist_id == other.artist_id
    }
}

impl Eq for Artist {}

impl PartialOrd for Artist {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Artist {
    #[tracing::instrument(level = "debug", skip(self, other))]
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        let artist_name = self
            .artist_name
            .as_ref()
            .unwrap_or(&String::new())
            .to_lowercase();
        let other_artist_name = other
            .artist_name
            .as_ref()
            .unwrap_or(&String::new())
            .to_lowercase();
        artist_name.cmp(&other_artist_name)
    }
}

impl SearchByTerm for Artist {
    #[tracing::instrument(level = "debug", skip(term))]
    fn search_by_term(term: Option<String>) -> Self {
        Self {
            artist_name: term,
            ..Default::default()
        }
    }
}

#[derive(Deserialize, Serialize, Default, Clone, Debug, Encode, Decode)]
pub struct Genre {
    pub genre_id: Option<String>,
    pub genre_name: Option<String>,
    #[serde(default)]
    pub genre_song_count: f64,
}

impl std::hash::Hash for Genre {
    #[tracing::instrument(level = "debug", skip(self, state))]
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.genre_id.hash(state);
    }
}

impl PartialEq for Genre {
    #[tracing::instrument(level = "debug", skip(self, other))]
    fn eq(&self, other: &Self) -> bool {
        self.genre_id == other.genre_id
    }
}

impl Eq for Genre {}

impl PartialOrd for Genre {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Genre {
    #[tracing::instrument(level = "debug", skip(self, other))]
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        let genre_name = self
            .genre_name
            .as_ref()
            .unwrap_or(&String::new())
            .to_lowercase();
        let other_genre_name = other
            .genre_name
            .as_ref()
            .unwrap_or(&String::new())
            .to_lowercase();
        genre_name.cmp(&other_genre_name)
    }
}

impl SearchByTerm for Genre {
    #[tracing::instrument(level = "debug", skip(term))]
    fn search_by_term(term: Option<String>) -> Self {
        Self {
            genre_name: term,
            ..Default::default()
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone, Default, PartialEq)]
pub struct GetEntityOptions {
    pub artist: Option<Artist>,
    pub album: Option<Album>,
    pub genre: Option<Genre>,
    pub playlist: Option<Playlist>,
    pub inclusive: Option<bool>,
}

#[derive(Deserialize, Serialize, Default, Clone, Debug)]

pub struct Playlist {
    pub playlist_id: Option<String>,
    #[serde(default)]
    pub playlist_name: String,
    #[serde(rename = "playlist_coverPath")]
    pub playlist_coverpath: Option<String>,
    #[serde(default)]
    pub playlist_song_count: f64,
    pub playlist_desc: Option<String>,
    pub playlist_path: Option<String>,
    pub extension: Option<String>,
    pub icon: Option<String>,
    pub library_item: Option<bool>,
}

impl std::hash::Hash for Playlist {
    #[tracing::instrument(level = "debug", skip(self, state))]
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.playlist_id.hash(state);
    }
}

impl PartialEq for Playlist {
    #[tracing::instrument(level = "debug", skip(self, other))]
    fn eq(&self, other: &Self) -> bool {
        self.playlist_id == other.playlist_id
    }
}

impl Eq for Playlist {}

impl PartialOrd for Playlist {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Playlist {
    #[tracing::instrument(level = "debug", skip(self, other))]
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.playlist_name
            .to_lowercase()
            .cmp(&other.playlist_name.to_lowercase())
    }
}

impl SearchByTerm for Playlist {
    #[tracing::instrument(level = "debug", skip(term))]
    fn search_by_term(term: Option<String>) -> Self {
        Self {
            playlist_name: term.unwrap_or_default(),
            ..Default::default()
        }
    }
}

#[derive(Deserialize, Serialize, Debug, Default, Clone)]
pub struct SearchResult {
    #[serde(deserialize_with = "deserialize_default")]
    pub songs: Vec<Song>,
    #[serde(deserialize_with = "deserialize_default")]
    pub artists: Vec<Artist>,
    #[serde(deserialize_with = "deserialize_default")]
    pub playlists: Vec<Playlist>,
    #[serde(deserialize_with = "deserialize_default")]
    pub albums: Vec<Album>,
    #[serde(deserialize_with = "deserialize_default")]
    pub genres: Vec<Genre>,
}
