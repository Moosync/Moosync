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
#[cfg(feature = "core")]
use diesel::{
    backend::Backend,
    deserialize::{self, FromSql, FromSqlRow, QueryableByName},
    expression::AsExpression,
    serialize::ToSql,
    sql_types::Text,
    sqlite::Sqlite,
    AsChangeset, Identifiable, Insertable, Queryable, Selectable,
};
use serde::{Deserialize, Serialize};

#[cfg(feature = "core")]
use crate::schema::allsongs;

use super::{
    common::{deserialize_default, SearchByTerm},
    entities::{QueryableAlbum, QueryableArtist, QueryableGenre, QueryablePlaylist},
};

#[derive(Debug, Default, Deserialize, Serialize, Clone, PartialEq, Eq, Copy, Encode, Decode)]
#[cfg_attr(feature = "core", derive(FromSqlRow, AsExpression))]
#[cfg_attr(feature = "core", diesel(sql_type = diesel::sql_types::Text))]
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
            SongType::LOCAL => "LOCAL",
            SongType::URL => "URL",
            SongType::SPOTIFY => "SPOTIFY",
            SongType::DASH => "DASH",
            SongType::HLS => "HLS",
        };
        write!(f, "{}", data)
    }
}

impl FromStr for SongType {
    type Err = MoosyncError;

    #[tracing::instrument(level = "debug", skip(s))]
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "LOCAL" => Ok(SongType::LOCAL),
            "URL" => Ok(SongType::URL),
            "SPOTIFY" => Ok(SongType::SPOTIFY),
            "DASH" => Ok(SongType::DASH),
            "HLS" => Ok(SongType::HLS),
            _ => Err(MoosyncError::String(format!("Invalid song type: {}", s))),
        }
    }
}

#[cfg(feature = "core")]
impl ToSql<Text, Sqlite> for SongType
where
    String: ToSql<Text, Sqlite>,
{
    #[tracing::instrument(level = "debug", skip(self, out))]
    fn to_sql<'b>(
        &'b self,
        out: &mut diesel::serialize::Output<'b, '_, Sqlite>,
    ) -> diesel::serialize::Result {
        match self {
            SongType::LOCAL => ToSql::<Text, Sqlite>::to_sql("LOCAL", out),
            SongType::URL => ToSql::<Text, Sqlite>::to_sql("URL", out),
            SongType::SPOTIFY => ToSql::<Text, Sqlite>::to_sql("SPOTIFY", out),
            SongType::DASH => ToSql::<Text, Sqlite>::to_sql("DASH", out),
            SongType::HLS => ToSql::<Text, Sqlite>::to_sql("HLS", out),
        }
    }
}

#[cfg(feature = "core")]
impl<DB> FromSql<Text, DB> for SongType
where
    DB: Backend,
    String: FromSql<Text, DB>,
{
    #[tracing::instrument(level = "debug", skip(bytes))]
    fn from_sql(bytes: DB::RawValue<'_>) -> deserialize::Result<Self> {
        match String::from_sql(bytes)?.as_str() {
            "LOCAL" => Ok(SongType::LOCAL),
            "URL" => Ok(SongType::URL),
            "SPOTIFY" => Ok(SongType::SPOTIFY),
            "DASH" => Ok(SongType::DASH),
            "HLS" => Ok(SongType::HLS),
            _ => Ok(SongType::LOCAL),
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Default, Clone, Encode, Decode)]
#[cfg_attr(
    feature = "core",
    derive(
        Insertable,
        Queryable,
        Identifiable,
        AsChangeset,
        Selectable,
        QueryableByName
    )
)]
#[cfg_attr(feature = "core", diesel(table_name = allsongs))]
#[cfg_attr(feature = "core", diesel(primary_key(_id)))]
pub struct QueryableSong {
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
    #[cfg_attr(feature = "core", diesel(column_name = "releasetype"))]
    pub release_type: Option<String>,
    pub bitrate: Option<f64>,
    pub codec: Option<String>,
    pub container: Option<String>,
    pub duration: Option<f64>,
    #[serde(rename = "sampleRate")]
    #[cfg_attr(feature = "core", diesel(column_name = "samplerate"))]
    pub sample_rate: Option<f64>,
    pub hash: Option<String>,
    #[serde(rename = "type")]
    pub type_: SongType,
    pub url: Option<String>,
    #[cfg_attr(feature = "core", diesel(column_name = "song_coverpath_high"))]
    #[serde(rename = "song_coverPath_high")]
    pub song_cover_path_high: Option<String>,
    #[cfg_attr(feature = "core", diesel(column_name = "playbackurl"))]
    #[serde(rename = "playbackUrl")]
    pub playback_url: Option<String>,
    #[cfg_attr(feature = "core", diesel(column_name = "song_coverpath_low"))]
    #[serde(rename = "song_coverPath_low")]
    pub song_cover_path_low: Option<String>,
    pub date_added: Option<i64>,
    pub provider_extension: Option<String>,
    pub icon: Option<String>,
    pub show_in_library: Option<bool>,
    pub track_no: Option<f64>,
    pub library_item: Option<bool>,
}

impl std::hash::Hash for QueryableSong {
    #[tracing::instrument(level = "debug", skip(self, state))]
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self._id.hash(state);
    }
}

impl PartialEq for QueryableSong {
    #[tracing::instrument(level = "debug", skip(self, other))]
    fn eq(&self, other: &Self) -> bool {
        self._id == other._id
    }
}

impl Eq for QueryableSong {}

#[cfg(any(feature = "core", feature = "ui"))]
impl SearchByTerm for QueryableSong {
    #[tracing::instrument(level = "debug", skip(term))]
    fn search_by_term(term: Option<String>) -> Self {
        let mut data = Self::default();
        data.title.clone_from(&term);
        data.path = term;

        data
    }
}

#[derive(Debug, Deserialize, Clone, Default, Serialize)]
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

#[derive(Debug, Deserialize, Clone, Serialize, Default)]
pub struct GetSongOptions {
    pub song: Option<SearchableSong>,
    pub artist: Option<QueryableArtist>,
    pub album: Option<QueryableAlbum>,
    pub genre: Option<QueryableGenre>,
    pub playlist: Option<QueryablePlaylist>,
    pub inclusive: Option<bool>,
}

#[derive(Default, Deserialize, Serialize, Clone, PartialEq, Eq, Encode, Decode)]
pub struct Song {
    #[serde(flatten)]
    pub song: QueryableSong,
    #[serde(default, deserialize_with = "deserialize_default")]
    pub album: Option<QueryableAlbum>,
    #[serde(default, deserialize_with = "deserialize_default")]
    pub artists: Option<Vec<QueryableArtist>>,
    #[serde(default, deserialize_with = "deserialize_default")]
    pub genre: Option<Vec<QueryableGenre>>,
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
        let song_id = self.song._id.as_deref().unwrap_or("No ID");

        write!(f, "{} - {} ({})", artist_names, title, song_id)
    }
}

impl std::hash::Hash for Song {
    #[tracing::instrument(level = "debug", skip(self, state))]
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.song._id.hash(state);
    }
}

#[derive(Debug, Default, Deserialize, Serialize, Clone)]
pub struct AllAnalytics {
    pub total_listen_time: f64,
    pub songs: Vec<(String, f64)>,
}
