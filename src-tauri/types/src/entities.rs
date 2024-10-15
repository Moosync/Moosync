use bitcode::{Decode, Encode};
#[cfg(feature = "core")]
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

#[cfg(feature = "core")]
use crate::schema::{
    album_bridge, albums, analytics, artist_bridge, artists, genre_bridge, genres, playlist_bridge,
    playlists,
};

use super::{
    common::{deserialize_default, BridgeUtils, SearchByTerm},
    songs::Song,
};

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Encode, Decode)]
#[cfg_attr(feature = "core", derive(FromSqlRow, AsExpression))]
#[cfg_attr(feature = "core", diesel(sql_type = diesel::sql_types::Text))]
pub struct EntityInfo(pub String);

#[cfg(feature = "core")]
impl<DB> FromSql<Text, DB> for EntityInfo
where
    DB: Backend,
    String: FromSql<Text, DB>,
{
    #[tracing::instrument(level = "trace", skip(bytes))]
    fn from_sql(bytes: DB::RawValue<'_>) -> deserialize::Result<Self> {
        let t = <String as FromSql<Text, DB>>::from_sql(bytes)?;
        Ok(Self(serde_json::from_str(&t)?))
    }
}

#[cfg(feature = "core")]
impl ToSql<Text, Sqlite> for EntityInfo
where
    String: ToSql<Text, Sqlite>,
{
    #[tracing::instrument(level = "trace", skip(self, out))]
    fn to_sql<'b>(
        &'b self,
        out: &mut diesel::serialize::Output<'b, '_, Sqlite>,
    ) -> diesel::serialize::Result {
        let s = serde_json::to_string(&self.0)?;

        out.set_value(s);
        Ok(IsNull::No)
    }
}

#[derive(Deserialize, Serialize, Default, Clone, Debug, Encode, Decode)]
#[cfg_attr(
    feature = "core",
    derive(Insertable, Queryable, Identifiable, AsChangeset,)
)]
#[cfg_attr(feature = "core", diesel(table_name = albums))]
#[cfg_attr(feature = "core", diesel(primary_key(album_id)))]
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

impl std::hash::Hash for QueryableAlbum {
    #[tracing::instrument(level = "trace", skip(self, state))]
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.album_id.hash(state);
    }
}

impl PartialEq for QueryableAlbum {
    #[tracing::instrument(level = "trace", skip(self, other))]
    fn eq(&self, other: &Self) -> bool {
        self.album_id == other.album_id
    }
}

impl Eq for QueryableAlbum {}

impl Ord for QueryableAlbum {
    #[tracing::instrument(level = "trace", skip(self, other))]
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.album_name
            .as_ref()
            .unwrap_or(&String::new())
            .cmp(other.album_name.as_ref().unwrap_or(&String::new()))
    }
}

impl PartialOrd for QueryableAlbum {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl SearchByTerm for QueryableAlbum {
    #[tracing::instrument(level = "trace", skip(term))]
    fn search_by_term(term: Option<String>) -> Self {
        Self {
            album_name: term,
            ..Default::default()
        }
    }
}

#[derive(Deserialize, Serialize, Default, Clone, Debug)]
#[cfg_attr(
    feature = "core",
    derive(Insertable, Queryable, Identifiable, AsChangeset,)
)]
#[cfg_attr(feature = "core", diesel(table_name = album_bridge))]
#[cfg_attr(feature = "core", diesel(primary_key(id)))]
pub struct AlbumBridge {
    pub id: Option<i32>,
    pub song: Option<String>,
    pub album: Option<String>,
}

impl BridgeUtils for AlbumBridge {
    #[tracing::instrument(level = "trace", skip(entity, song))]
    fn insert_value(entity: String, song: String) -> Self {
        Self {
            album: Some(entity),
            song: Some(song),
            ..Default::default()
        }
    }
}

#[derive(Deserialize, Serialize, Default, Clone, Debug, Encode, Decode)]
#[cfg_attr(
    feature = "core",
    derive(Insertable, Queryable, Identifiable, AsChangeset)
)]
#[cfg_attr(feature = "core", diesel(table_name = artists))]
#[cfg_attr(feature = "core", diesel(primary_key(artist_id)))]
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

impl std::hash::Hash for QueryableArtist {
    #[tracing::instrument(level = "trace", skip(self, state))]
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.artist_id.hash(state);
    }
}

impl PartialEq for QueryableArtist {
    #[tracing::instrument(level = "trace", skip(self, other))]
    fn eq(&self, other: &Self) -> bool {
        self.artist_id == other.artist_id
    }
}

impl Eq for QueryableArtist {}

impl PartialOrd for QueryableArtist {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for QueryableArtist {
    #[tracing::instrument(level = "trace", skip(self, other))]
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.artist_name
            .as_ref()
            .unwrap_or(&String::new())
            .cmp(other.artist_name.as_ref().unwrap_or(&String::new()))
    }
}

impl SearchByTerm for QueryableArtist {
    #[tracing::instrument(level = "trace", skip(term))]
    fn search_by_term(term: Option<String>) -> Self {
        Self {
            artist_name: term,
            ..Default::default()
        }
    }
}

#[derive(Deserialize, Serialize, Default, Clone, Debug)]
#[cfg_attr(
    feature = "core",
    derive(Insertable, Queryable, Identifiable, AsChangeset,)
)]
#[cfg_attr(feature = "core", diesel(table_name = artist_bridge))]
#[cfg_attr(feature = "core", diesel(primary_key(id)))]
pub struct ArtistBridge {
    pub id: Option<i32>,
    pub song: Option<String>,
    pub artist: Option<String>,
}

impl BridgeUtils for ArtistBridge {
    #[tracing::instrument(level = "trace", skip(entity, song))]
    fn insert_value(entity: String, song: String) -> Self {
        Self {
            artist: Some(entity),
            song: Some(song),
            ..Default::default()
        }
    }
}

#[derive(Deserialize, Serialize, Default, Clone, Debug, Encode, Decode)]
#[cfg_attr(
    feature = "core",
    derive(Insertable, Queryable, Identifiable, AsChangeset,)
)]
#[cfg_attr(feature = "core", diesel(table_name = genres))]
#[cfg_attr(feature = "core", diesel(primary_key(genre_id)))]
pub struct QueryableGenre {
    pub genre_id: Option<String>,
    pub genre_name: Option<String>,
    #[serde(default)]
    pub genre_song_count: f64,
}

impl std::hash::Hash for QueryableGenre {
    #[tracing::instrument(level = "trace", skip(self, state))]
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.genre_id.hash(state);
    }
}

impl PartialEq for QueryableGenre {
    #[tracing::instrument(level = "trace", skip(self, other))]
    fn eq(&self, other: &Self) -> bool {
        self.genre_id == other.genre_id
    }
}

impl Eq for QueryableGenre {}

impl PartialOrd for QueryableGenre {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for QueryableGenre {
    #[tracing::instrument(level = "trace", skip(self, other))]
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.genre_name
            .as_ref()
            .unwrap_or(&String::new())
            .cmp(other.genre_name.as_ref().unwrap_or(&String::new()))
    }
}

impl SearchByTerm for QueryableGenre {
    #[tracing::instrument(level = "trace", skip(term))]
    fn search_by_term(term: Option<String>) -> Self {
        Self {
            genre_name: term,
            ..Default::default()
        }
    }
}

#[derive(Deserialize, Serialize, Default, Clone, Debug)]
#[cfg_attr(
    feature = "core",
    derive(Insertable, Queryable, Identifiable, AsChangeset,)
)]
#[cfg_attr(feature = "core", diesel(table_name = genre_bridge))]
#[cfg_attr(feature = "core", diesel(primary_key(id)))]
pub struct GenreBridge {
    pub id: Option<i32>,
    pub song: Option<String>,
    pub genre: Option<String>,
}

impl BridgeUtils for GenreBridge {
    #[tracing::instrument(level = "trace", skip(entity, song))]
    fn insert_value(entity: String, song: String) -> Self {
        Self {
            genre: Some(entity),
            song: Some(song),
            ..Default::default()
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct GetEntityOptions {
    pub artist: Option<QueryableArtist>,
    pub album: Option<QueryableAlbum>,
    pub genre: Option<QueryableGenre>,
    pub playlist: Option<QueryablePlaylist>,
    pub inclusive: Option<bool>,
}

#[derive(Deserialize, Serialize, Default, Clone, Debug)]
#[cfg_attr(
    feature = "core",
    derive(Insertable, Queryable, Identifiable, AsChangeset,)
)]
#[cfg_attr(feature = "core", diesel(table_name = playlist_bridge))]
#[cfg_attr(feature = "core", diesel(primary_key(id)))]
pub struct PlaylistBridge {
    pub id: Option<i32>,
    pub song: Option<String>,
    pub playlist: Option<String>,
}

impl BridgeUtils for PlaylistBridge {
    #[tracing::instrument(level = "trace", skip(entity, song))]
    fn insert_value(entity: String, song: String) -> Self {
        Self {
            playlist: Some(entity),
            song: Some(song),
            ..Default::default()
        }
    }
}

#[derive(Deserialize, Serialize, Default, Clone, Debug)]
#[cfg_attr(
    feature = "core",
    derive(Insertable, Queryable, Identifiable, AsChangeset,)
)]
#[cfg_attr(feature = "core", diesel(table_name = playlists))]
#[cfg_attr(feature = "core", diesel(primary_key(playlist_id)))]

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
    pub library_item: Option<bool>,
}

impl std::hash::Hash for QueryablePlaylist {
    #[tracing::instrument(level = "trace", skip(self, state))]
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.playlist_id.hash(state);
    }
}

impl PartialEq for QueryablePlaylist {
    #[tracing::instrument(level = "trace", skip(self, other))]
    fn eq(&self, other: &Self) -> bool {
        self.playlist_id == other.playlist_id
    }
}

impl Eq for QueryablePlaylist {}

impl PartialOrd for QueryablePlaylist {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for QueryablePlaylist {
    #[tracing::instrument(level = "trace", skip(self, other))]
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.playlist_name.cmp(&other.playlist_name)
    }
}

impl SearchByTerm for QueryablePlaylist {
    #[tracing::instrument(level = "trace", skip(term))]
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
    pub artists: Vec<QueryableArtist>,
    #[serde(deserialize_with = "deserialize_default")]
    pub playlists: Vec<QueryablePlaylist>,
    #[serde(deserialize_with = "deserialize_default")]
    pub albums: Vec<QueryableAlbum>,
    #[serde(deserialize_with = "deserialize_default")]
    pub genres: Vec<QueryableGenre>,
}

#[derive(Deserialize, Serialize, Default, Clone, Debug)]
#[cfg_attr(
    feature = "core",
    derive(Insertable, Queryable, Identifiable, AsChangeset,)
)]
#[cfg_attr(feature = "core", diesel(table_name = analytics))]
#[cfg_attr(feature = "core", diesel(primary_key(id)))]
pub struct Analytics {
    pub id: Option<String>,
    pub song_id: Option<String>,
    pub play_count: Option<i32>,
    pub play_time: Option<f64>,
}
