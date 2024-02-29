use std::{fmt::Display, str::FromStr};

use crate::errors::errors::MoosyncError;
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
use uuid::Uuid;

use crate::schema::allsongs;

use super::{
    entities::{QueryableAlbum, QueryableArtist, QueryableGenre, QueryablePlaylist},
    traits::SearchByTerm,
};

#[derive(
    Debug, Default, Deserialize, Serialize, FromSqlRow, AsExpression, Clone, PartialEq, Eq,
)]
#[diesel(sql_type = diesel::sql_types::Text)]
pub enum SongType {
    #[default]
    LOCAL,
    URL,
    YOUTUBE,
    SPOTIFY,
    DASH,
    HLS,
}
impl Display for SongType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let data = match self {
            SongType::LOCAL => "LOCAL",
            SongType::URL => "URL",
            SongType::YOUTUBE => "YOUTUBE",
            SongType::SPOTIFY => "SPOTIFY",
            SongType::DASH => "DASH",
            SongType::HLS => "HLS",
        };
        write!(f, "{}", data)
    }
}

impl FromStr for SongType {
    type Err = MoosyncError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "LOCAL" => Ok(SongType::LOCAL),
            "URL" => Ok(SongType::URL),
            "YOUTUBE" => Ok(SongType::YOUTUBE),
            "SPOTIFY" => Ok(SongType::SPOTIFY),
            "DASH" => Ok(SongType::DASH),
            "HLS" => Ok(SongType::HLS),
            _ => Err(MoosyncError::String(format!("Invalid song type: {}", s))),
        }
    }
}

impl ToSql<Text, Sqlite> for SongType
where
    String: ToSql<Text, Sqlite>,
{
    fn to_sql<'b>(
        &'b self,
        out: &mut diesel::serialize::Output<'b, '_, Sqlite>,
    ) -> diesel::serialize::Result {
        match self {
            SongType::LOCAL => ToSql::<Text, Sqlite>::to_sql("LOCAL", out),
            SongType::URL => ToSql::<Text, Sqlite>::to_sql("URL", out),
            SongType::YOUTUBE => ToSql::<Text, Sqlite>::to_sql("YOUTUBE", out),
            SongType::SPOTIFY => ToSql::<Text, Sqlite>::to_sql("SPOTIFY", out),
            SongType::DASH => ToSql::<Text, Sqlite>::to_sql("DASH", out),
            SongType::HLS => ToSql::<Text, Sqlite>::to_sql("HLS", out),
        }
    }
}

impl<DB> FromSql<Text, DB> for SongType
where
    DB: Backend,
    String: FromSql<Text, DB>,
{
    fn from_sql(bytes: DB::RawValue<'_>) -> deserialize::Result<Self> {
        match String::from_sql(bytes)?.as_str() {
            "LOCAL" => Ok(SongType::LOCAL),
            "URL" => Ok(SongType::URL),
            "YOUTUBE" => Ok(SongType::YOUTUBE),
            "SPOTIFY" => Ok(SongType::SPOTIFY),
            "DASH" => Ok(SongType::DASH),
            "HLS" => Ok(SongType::HLS),
            _ => Ok(SongType::LOCAL),
        }
    }
}

#[derive(
    Debug,
    Deserialize,
    Serialize,
    Insertable,
    Default,
    Queryable,
    Identifiable,
    AsChangeset,
    Clone,
    Selectable,
    QueryableByName,
)]
#[diesel(table_name = allsongs)]
#[diesel(primary_key(_id))]
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
    #[diesel(column_name = "releasetype")]
    pub release_type: Option<String>,
    pub bitrate: Option<f64>,
    pub codec: Option<String>,
    pub container: Option<String>,
    pub duration: Option<f64>,
    #[serde(rename = "sampleRate")]
    #[diesel(column_name = "samplerate")]
    pub sample_rate: Option<f64>,
    pub hash: Option<String>,
    #[serde(rename = "type")]
    pub type_: SongType,
    pub url: Option<String>,
    #[diesel(column_name = "song_coverpath_high")]
    #[serde(rename = "song_coverPath_high")]
    pub song_cover_path_high: Option<String>,
    #[diesel(column_name = "playbackurl")]
    #[serde(rename = "playbackUrl")]
    pub playback_url: Option<String>,
    #[diesel(column_name = "song_coverpath_low")]
    #[serde(rename = "song_coverPath_low")]
    pub song_cover_path_low: Option<String>,
    pub date_added: Option<i64>,
    pub provider_extension: Option<String>,
    pub icon: Option<String>,
    pub show_in_library: Option<bool>,

    pub track_no: Option<f64>,
}

impl QueryableSong {
    pub fn empty() -> Self {
        Self {
            _id: Some(Uuid::new_v4().to_string()),
            ..Default::default()
        }
    }
}

impl SearchByTerm for QueryableSong {
    fn search_by_term(term: Option<String>) -> Self {
        let mut data = Self::empty();
        data.title = term.clone();
        data.path = term;

        data
    }
}

#[derive(Debug, Deserialize, Clone, Default)]
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

#[derive(Debug, Deserialize, Clone, Default)]
pub struct GetSongOptions {
    pub song: Option<SearchableSong>,
    pub artist: Option<QueryableArtist>,
    pub album: Option<QueryableAlbum>,
    pub genre: Option<QueryableGenre>,
    pub playlist: Option<QueryablePlaylist>,
    pub inclusive: Option<bool>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Song {
    #[serde(flatten)]
    pub song: QueryableSong,
    pub album: Option<QueryableAlbum>,
    pub artists: Option<Vec<QueryableArtist>>,
    pub genre: Option<Vec<QueryableGenre>>,
}
