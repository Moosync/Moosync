use diesel::{
    backend::Backend,
    deserialize::{self, FromSql, FromSqlRow},
    expression::AsExpression,
    serialize::ToSql,
    sql_types::Text,
    sqlite::Sqlite,
    AsChangeset, Identifiable, Insertable, Queryable, Selectable,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::db::schema::allsongs;

use super::entities::{QueryableAlbum, QueryableArtist, QueryableGenre, QueryablePlaylist};

#[derive(Debug, Default, Deserialize, Serialize, FromSqlRow, AsExpression, Clone)]
#[sql_type = "diesel::sql_types::Text"]
pub enum SongType {
    #[default]
    LOCAL,
    URL,
    YOUTUBE,
    SPOTIFY,
    DASH,
    HLS,
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
    #[diesel(column_name = "releaseType")]
    pub release_type: Option<String>,
    pub bitrate: Option<f64>,
    pub codec: Option<String>,
    pub container: Option<String>,
    pub duration: Option<f64>,
    #[diesel(column_name = "sampleRate")]
    pub sample_rate: Option<f64>,
    pub hash: Option<String>,
    pub type_: SongType,
    pub url: Option<String>,
    #[diesel(column_name = "song_coverPath_high")]
    pub song_cover_path_high: Option<String>,
    #[diesel(column_name = "song_coverPath_low")]
    pub song_cover_path_low: Option<String>,
    #[diesel(column_name = "playbackUrl")]
    pub playback_url: Option<String>,
    pub date_added: Option<String>,
    pub provider_extension: Option<String>,
    pub icon: Option<String>,
    pub show_in_library: Option<bool>,

    pub track_no: Option<f64>,
}

impl QueryableSong {
    pub fn empty() -> Self {
        let mut song: Self = Default::default();
        song._id = Some(Uuid::new_v4().to_string());
        song
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
    pub artists: Vec<QueryableArtist>,
    pub genre: Vec<QueryableGenre>,
}
