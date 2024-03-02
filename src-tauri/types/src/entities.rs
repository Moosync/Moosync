use diesel::{
    backend::Backend,
    deserialize::{self, FromSql, FromSqlRow},
    expression::AsExpression,
    serialize::{IsNull, ToSql},
    sql_types::Text,
    sqlite::Sqlite,
    AsChangeset, Identifiable, Insertable, Queryable,
};
use serde::{Deserialize, Serialize};

use crate::schema::{
    album_bridge, albums, analytics, artist_bridge, artists, genre_bridge, genres, playlist_bridge,
    playlists,
};

use super::{
    songs::Song,
    traits::{BridgeUtils, SearchByTerm},
};

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, FromSqlRow, AsExpression)]
#[diesel(sql_type = diesel::sql_types::Text)]
pub struct EntityInfo(pub serde_json::Value);

impl<DB> FromSql<Text, DB> for EntityInfo
where
    DB: Backend,
    String: FromSql<Text, DB>,
{
    fn from_sql(bytes: DB::RawValue<'_>) -> deserialize::Result<Self> {
        let t = <String as FromSql<Text, DB>>::from_sql(bytes)?;
        Ok(Self(serde_json::from_str(&t)?))
    }
}

impl ToSql<Text, Sqlite> for EntityInfo
where
    String: ToSql<Text, Sqlite>,
{
    fn to_sql<'b>(
        &'b self,
        out: &mut diesel::serialize::Output<'b, '_, Sqlite>,
    ) -> diesel::serialize::Result {
        let s = serde_json::to_string(&self.0)?;

        out.set_value(s);
        Ok(IsNull::No)
    }
}

#[derive(
    Deserialize, Serialize, Insertable, Default, Queryable, Identifiable, AsChangeset, Clone, Debug,
)]
#[diesel(table_name = albums)]
#[diesel(primary_key(album_id))]
pub struct QueryableAlbum {
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

impl SearchByTerm for QueryableAlbum {
    fn search_by_term(term: Option<String>) -> Self {
        Self {
            album_name: term,
            ..Default::default()
        }
    }
}

#[derive(Deserialize, Insertable, Default, Queryable, Identifiable, Clone, Debug)]
#[diesel(table_name = album_bridge)]
#[diesel(primary_key(id))]
pub struct AlbumBridge {
    pub id: Option<i32>,
    pub song: Option<String>,
    pub album: Option<String>,
}

impl BridgeUtils for AlbumBridge {
    fn insert_value(entity: String, song: String) -> Self {
        Self {
            album: Some(entity),
            song: Some(song),
            ..Default::default()
        }
    }
}

#[derive(
    Deserialize, Serialize, Insertable, Default, Queryable, Identifiable, AsChangeset, Clone, Debug,
)]
#[diesel(table_name = artists)]
#[diesel(primary_key(artist_id))]
pub struct QueryableArtist {
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

impl SearchByTerm for QueryableArtist {
    fn search_by_term(term: Option<String>) -> Self {
        Self {
            artist_name: term,
            ..Default::default()
        }
    }
}

#[derive(Deserialize, Insertable, Default, Queryable, Identifiable, Clone, Debug)]
#[diesel(table_name = artist_bridge)]
#[diesel(primary_key(id))]
pub struct ArtistBridge {
    pub id: Option<i32>,
    pub song: Option<String>,
    pub artist: Option<String>,
}

impl BridgeUtils for ArtistBridge {
    fn insert_value(entity: String, song: String) -> Self {
        Self {
            artist: Some(entity),
            song: Some(song),
            ..Default::default()
        }
    }
}

#[derive(
    Deserialize, Serialize, Insertable, Default, Queryable, Identifiable, AsChangeset, Clone, Debug,
)]
#[diesel(table_name = genres)]
#[diesel(primary_key(genre_id))]
pub struct QueryableGenre {
    pub genre_id: Option<String>,
    pub genre_name: Option<String>,
    #[serde(default)]
    pub genre_song_count: f64,
}

impl SearchByTerm for QueryableGenre {
    fn search_by_term(term: Option<String>) -> Self {
        Self {
            genre_name: term,
            ..Default::default()
        }
    }
}

#[derive(Deserialize, Insertable, Default, Queryable, Identifiable, Clone, Debug)]
#[diesel(table_name = genre_bridge)]
#[diesel(primary_key(id))]
pub struct GenreBridge {
    pub id: Option<i32>,
    pub song: Option<String>,
    pub genre: Option<String>,
}

impl BridgeUtils for GenreBridge {
    fn insert_value(entity: String, song: String) -> Self {
        Self {
            genre: Some(entity),
            song: Some(song),
            ..Default::default()
        }
    }
}

#[derive(Debug, Deserialize, Clone, Default)]
pub struct GetEntityOptions {
    pub artist: Option<QueryableArtist>,
    pub album: Option<QueryableAlbum>,
    pub genre: Option<QueryableGenre>,
    pub playlist: Option<QueryablePlaylist>,
    pub inclusive: Option<bool>,
}

#[derive(Deserialize, Insertable, Default, Queryable, Identifiable, Clone, Debug)]
#[diesel(table_name = playlist_bridge)]
#[diesel(primary_key(id))]
pub struct PlaylistBridge {
    pub id: Option<i32>,
    pub song: Option<String>,
    pub playlist: Option<String>,
}

impl BridgeUtils for PlaylistBridge {
    fn insert_value(entity: String, song: String) -> Self {
        Self {
            playlist: Some(entity),
            song: Some(song),
            ..Default::default()
        }
    }
}

#[derive(
    Deserialize, Serialize, Insertable, Default, Queryable, Identifiable, AsChangeset, Clone, Debug,
)]
#[diesel(table_name = playlists)]
#[diesel(primary_key(playlist_id))]
pub struct QueryablePlaylist {
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
}

impl SearchByTerm for QueryablePlaylist {
    fn search_by_term(term: Option<String>) -> Self {
        Self {
            playlist_name: term.unwrap_or_default(),
            ..Default::default()
        }
    }
}

#[derive(Deserialize, Serialize, Debug)]
pub struct SearchResult {
    pub songs: Vec<Song>,
    pub artists: Vec<QueryableArtist>,
    pub playlists: Vec<QueryablePlaylist>,
    pub albums: Vec<QueryableAlbum>,
    pub genres: Vec<QueryableGenre>,
}

#[derive(
    Deserialize, Serialize, Insertable, Default, Queryable, Identifiable, AsChangeset, Clone, Debug,
)]
#[diesel(table_name = analytics)]
#[diesel(primary_key(id))]
pub struct Analytics {
    pub id: Option<String>,
    pub song_id: Option<String>,
    pub play_count: Option<i32>,
    pub play_time: Option<f64>,
}
