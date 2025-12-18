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

use diesel::{
    AsChangeset, Identifiable, Insertable, Queryable, Selectable,
    backend::Backend,
    deserialize::{self, FromSql, FromSqlRow},
    expression::AsExpression,
    prelude::QueryableByName,
    serialize::{IsNull, ToSql},
    sql_types::Text,
    sqlite::Sqlite,
};
use types::{
    common::BridgeUtils,
    songs::{InnerSong, SongType},
};

use crate::{
    cache_schema::cache,
    schema::{
        album_bridge, albums, allsongs, analytics, artist_bridge, artists, genre_bridge, genres,
        playlist_bridge, playlists,
    },
};

#[derive(Debug, Clone, PartialEq, Eq, FromSqlRow, AsExpression)]
#[diesel(sql_type = diesel::sql_types::Text)]
pub struct EntityInfo(pub String);

impl<DB> FromSql<Text, DB> for EntityInfo
where
    DB: Backend,
    String: FromSql<Text, DB>,
{
    #[tracing::instrument(level = "debug", skip(bytes))]
    fn from_sql(bytes: DB::RawValue<'_>) -> deserialize::Result<Self> {
        let t = <String as FromSql<Text, DB>>::from_sql(bytes)?;
        Ok(Self(serde_json::from_str(&t)?))
    }
}

impl ToSql<Text, Sqlite> for EntityInfo
where
    String: ToSql<Text, Sqlite>,
{
    #[tracing::instrument(level = "debug", skip(self, out))]
    fn to_sql<'b>(
        &'b self,
        out: &mut diesel::serialize::Output<'b, '_, Sqlite>,
    ) -> diesel::serialize::Result {
        let s = serde_json::to_string(&self.0)?;

        out.set_value(s);
        Ok(IsNull::No)
    }
}

impl From<types::entities::EntityInfo> for EntityInfo {
    #[tracing::instrument(level = "debug", skip(value))]
    fn from(value: types::entities::EntityInfo) -> Self {
        Self(value.0)
    }
}

#[derive(Default, Clone, Debug, Insertable, Queryable, Identifiable, AsChangeset)]
#[diesel(table_name = albums)]
#[diesel(primary_key(album_id))]
pub struct QueryableAlbum {
    pub album_id: Option<String>,
    pub album_name: Option<String>,
    pub album_artist: Option<String>,
    pub album_coverpath_high: Option<String>,
    pub album_song_count: f64,
    pub year: Option<String>,
    pub album_coverpath_low: Option<String>,
    pub album_extra_info: Option<EntityInfo>,
}

impl From<types::entities::Album> for QueryableAlbum {
    #[tracing::instrument(level = "debug", skip(value))]
    fn from(value: types::entities::Album) -> Self {
        Self {
            album_id: value.album_id,
            album_name: value.album_name,
            album_artist: value.album_artist,
            album_coverpath_high: value.album_coverpath_high,
            album_song_count: value.album_song_count,
            year: value.year,
            album_coverpath_low: value.album_coverpath_low,
            album_extra_info: value.album_extra_info.map(EntityInfo::from),
        }
    }
}

impl Into<types::entities::Album> for QueryableAlbum {
    #[tracing::instrument(level = "debug", skip(self))]
    fn into(self) -> types::entities::Album {
        types::entities::Album {
            album_id: self.album_id,
            album_name: self.album_name,
            album_artist: self.album_artist,
            album_coverpath_high: self.album_coverpath_high,
            album_song_count: self.album_song_count,
            year: self.year,
            album_coverpath_low: self.album_coverpath_low,
            album_extra_info: self
                .album_extra_info
                .map(|info| types::entities::EntityInfo(info.0)),
        }
    }
}

#[derive(Default, Clone, Debug, Insertable, Queryable, Identifiable, AsChangeset)]
#[diesel(table_name = album_bridge)]
#[diesel(primary_key(id))]
pub struct AlbumBridge {
    pub id: Option<i32>,
    pub song: Option<String>,
    pub album: Option<String>,
}

impl BridgeUtils for AlbumBridge {
    #[tracing::instrument(level = "debug", skip(entity, song))]
    fn insert_value(entity: String, song: String) -> Self {
        Self {
            album: Some(entity),
            song: Some(song),
            ..Default::default()
        }
    }
}

#[derive(Default, Clone, Debug, Insertable, Queryable, Identifiable, AsChangeset)]
#[diesel(table_name = artists)]
#[diesel(primary_key(artist_id))]
pub struct QueryableArtist {
    pub artist_id: Option<String>,
    pub artist_mbid: Option<String>,
    pub artist_name: Option<String>,
    pub artist_coverpath: Option<String>,
    pub artist_song_count: f64,
    pub artist_extra_info: Option<EntityInfo>,
    pub sanitized_artist_name: Option<String>,
}

impl From<types::entities::Artist> for QueryableArtist {
    #[tracing::instrument(level = "debug", skip(value))]
    fn from(value: types::entities::Artist) -> Self {
        Self {
            artist_id: value.artist_id,
            artist_mbid: value.artist_mbid,
            artist_name: value.artist_name,
            artist_coverpath: value.artist_coverpath,
            artist_song_count: value.artist_song_count,
            artist_extra_info: value.artist_extra_info.map(EntityInfo::from),
            sanitized_artist_name: value.sanitized_artist_name,
        }
    }
}

impl Into<types::entities::Artist> for QueryableArtist {
    #[tracing::instrument(level = "debug", skip(self))]
    fn into(self) -> types::entities::Artist {
        types::entities::Artist {
            artist_id: self.artist_id,
            artist_mbid: self.artist_mbid,
            artist_name: self.artist_name,
            artist_coverpath: self.artist_coverpath,
            artist_song_count: self.artist_song_count,
            artist_extra_info: self
                .artist_extra_info
                .map(|info| types::entities::EntityInfo(info.0)),
            sanitized_artist_name: self.sanitized_artist_name,
        }
    }
}

#[derive(Default, Clone, Debug, Insertable, Queryable, Identifiable, AsChangeset)]
#[diesel(table_name = artist_bridge)]
#[diesel(primary_key(id))]
pub struct ArtistBridge {
    pub id: Option<i32>,
    pub song: Option<String>,
    pub artist: Option<String>,
}

impl BridgeUtils for ArtistBridge {
    #[tracing::instrument(level = "debug", skip(entity, song))]
    fn insert_value(entity: String, song: String) -> Self {
        Self {
            artist: Some(entity),
            song: Some(song),
            ..Default::default()
        }
    }
}

#[derive(Default, Clone, Debug, Insertable, Queryable, Identifiable, AsChangeset)]
#[diesel(table_name = genres)]
#[diesel(primary_key(genre_id))]
pub struct QueryableGenre {
    pub genre_id: Option<String>,
    pub genre_name: Option<String>,
    pub genre_song_count: f64,
}

impl From<types::entities::Genre> for QueryableGenre {
    #[tracing::instrument(level = "debug", skip(value))]
    fn from(value: types::entities::Genre) -> Self {
        Self {
            genre_id: value.genre_id,
            genre_name: value.genre_name,
            genre_song_count: value.genre_song_count,
        }
    }
}

impl Into<types::entities::Genre> for QueryableGenre {
    #[tracing::instrument(level = "debug", skip(self))]
    fn into(self) -> types::entities::Genre {
        types::entities::Genre {
            genre_id: self.genre_id,
            genre_name: self.genre_name,
            genre_song_count: self.genre_song_count,
        }
    }
}

#[derive(Default, Clone, Debug, Insertable, Queryable, Identifiable, AsChangeset)]
#[diesel(table_name = genre_bridge)]
#[diesel(primary_key(id))]
pub struct GenreBridge {
    pub id: Option<i32>,
    pub song: Option<String>,
    pub genre: Option<String>,
}

impl BridgeUtils for GenreBridge {
    #[tracing::instrument(level = "debug", skip(entity, song))]
    fn insert_value(entity: String, song: String) -> Self {
        Self {
            genre: Some(entity),
            song: Some(song),
            ..Default::default()
        }
    }
}

#[derive(Default, Clone, Debug, Insertable, Queryable, Identifiable, AsChangeset)]
#[diesel(table_name = playlist_bridge)]
#[diesel(primary_key(id))]
pub struct PlaylistBridge {
    pub id: Option<i32>,
    pub song: Option<String>,
    pub playlist: Option<String>,
}

impl BridgeUtils for PlaylistBridge {
    #[tracing::instrument(level = "debug", skip(entity, song))]
    fn insert_value(entity: String, song: String) -> Self {
        Self {
            playlist: Some(entity),
            song: Some(song),
            ..Default::default()
        }
    }
}

#[derive(Default, Clone, Debug, Insertable, Queryable, Identifiable, AsChangeset)]
#[diesel(table_name = playlists)]
#[diesel(primary_key(playlist_id))]

pub struct QueryablePlaylist {
    pub playlist_id: Option<String>,
    pub playlist_name: String,
    pub playlist_coverpath: Option<String>,
    pub playlist_song_count: f64,
    pub playlist_desc: Option<String>,
    pub playlist_path: Option<String>,
    pub extension: Option<String>,
    pub icon: Option<String>,
    pub library_item: Option<bool>,
}

impl From<types::entities::Playlist> for QueryablePlaylist {
    #[tracing::instrument(level = "debug", skip(value))]
    fn from(value: types::entities::Playlist) -> Self {
        Self {
            playlist_id: value.playlist_id,
            playlist_name: value.playlist_name,
            playlist_coverpath: value.playlist_coverpath,
            playlist_song_count: value.playlist_song_count,
            playlist_desc: value.playlist_desc,
            playlist_path: value.playlist_path,
            extension: value.extension,
            icon: value.icon,
            library_item: value.library_item,
        }
    }
}

impl Into<types::entities::Playlist> for QueryablePlaylist {
    #[tracing::instrument(level = "debug", skip(self))]
    fn into(self) -> types::entities::Playlist {
        types::entities::Playlist {
            playlist_id: self.playlist_id,
            playlist_name: self.playlist_name,
            playlist_coverpath: self.playlist_coverpath,
            playlist_song_count: self.playlist_song_count,
            playlist_desc: self.playlist_desc,
            playlist_path: self.playlist_path,
            extension: self.extension,
            icon: self.icon,
            library_item: self.library_item,
        }
    }
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Copy, FromSqlRow, AsExpression)]
#[diesel(sql_type = diesel::sql_types::Text)]
pub enum QueryableSongType {
    #[default]
    LOCAL,
    URL,
    SPOTIFY,
    DASH,
    HLS,
}

impl From<SongType> for QueryableSongType {
    #[tracing::instrument(level = "debug", skip(value))]
    fn from(value: SongType) -> Self {
        match value {
            SongType::LOCAL => Self::LOCAL,
            SongType::URL => Self::URL,
            SongType::SPOTIFY => Self::SPOTIFY,
            SongType::DASH => Self::DASH,
            SongType::HLS => Self::HLS,
        }
    }
}

impl Into<SongType> for QueryableSongType {
    #[tracing::instrument(level = "debug", skip(self))]
    fn into(self) -> SongType {
        match self {
            Self::LOCAL => SongType::LOCAL,
            Self::URL => SongType::URL,
            Self::SPOTIFY => SongType::SPOTIFY,
            Self::DASH => SongType::DASH,
            Self::HLS => SongType::HLS,
        }
    }
}

impl ToSql<Text, Sqlite> for QueryableSongType
where
    String: ToSql<Text, Sqlite>,
{
    #[tracing::instrument(level = "debug", skip(self, out))]
    fn to_sql<'b>(
        &'b self,
        out: &mut diesel::serialize::Output<'b, '_, Sqlite>,
    ) -> diesel::serialize::Result {
        match self {
            Self::LOCAL => ToSql::<Text, Sqlite>::to_sql("LOCAL", out),
            Self::URL => ToSql::<Text, Sqlite>::to_sql("URL", out),
            Self::SPOTIFY => ToSql::<Text, Sqlite>::to_sql("SPOTIFY", out),
            Self::DASH => ToSql::<Text, Sqlite>::to_sql("DASH", out),
            Self::HLS => ToSql::<Text, Sqlite>::to_sql("HLS", out),
        }
    }
}

impl<DB> FromSql<Text, DB> for QueryableSongType
where
    DB: Backend,
    String: FromSql<Text, DB>,
{
    #[tracing::instrument(level = "debug", skip(bytes))]
    fn from_sql(bytes: DB::RawValue<'_>) -> deserialize::Result<Self> {
        match String::from_sql(bytes)?.as_str() {
            "LOCAL" => Ok(Self::LOCAL),
            "URL" => Ok(Self::URL),
            "SPOTIFY" => Ok(Self::SPOTIFY),
            "DASH" => Ok(Self::DASH),
            "HLS" => Ok(Self::HLS),
            _ => Ok(Self::LOCAL),
        }
    }
}

#[derive(
    Debug,
    Default,
    Clone,
    Insertable,
    Queryable,
    Identifiable,
    AsChangeset,
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
    #[diesel(column_name = "releasetype")]
    pub release_type: Option<String>,
    pub bitrate: Option<f64>,
    pub codec: Option<String>,
    pub container: Option<String>,
    pub duration: Option<f64>,
    #[diesel(column_name = "samplerate")]
    pub sample_rate: Option<f64>,
    pub hash: Option<String>,
    pub type_: QueryableSongType,
    pub url: Option<String>,
    #[diesel(column_name = "song_coverpath_high")]
    pub song_cover_path_high: Option<String>,
    #[diesel(column_name = "playbackurl")]
    pub playback_url: Option<String>,
    #[diesel(column_name = "song_coverpath_low")]
    pub song_cover_path_low: Option<String>,
    pub date_added: Option<i64>,
    pub provider_extension: Option<String>,
    pub icon: Option<String>,
    pub show_in_library: Option<bool>,
    pub track_no: Option<f64>,
    pub library_item: Option<bool>,
}

impl From<InnerSong> for QueryableSong {
    #[tracing::instrument(level = "debug", skip(value))]
    fn from(value: InnerSong) -> Self {
        Self {
            _id: value._id,
            path: value.path,
            size: value.size,
            inode: value.inode,
            deviceno: value.deviceno,
            title: value.title,
            date: value.date,
            year: value.year,
            lyrics: value.lyrics,
            release_type: value.release_type,
            bitrate: value.bitrate,
            codec: value.codec,
            container: value.container,
            duration: value.duration,
            sample_rate: value.sample_rate,
            hash: value.hash,
            type_: QueryableSongType::from(value.type_),
            url: value.url,
            song_cover_path_high: value.song_cover_path_high,
            playback_url: value.playback_url,
            song_cover_path_low: value.song_cover_path_low,
            date_added: value.date_added,
            provider_extension: value.provider_extension,
            icon: value.icon,
            show_in_library: value.show_in_library,
            track_no: value.track_no,
            library_item: value.library_item,
        }
    }
}

impl Into<InnerSong> for QueryableSong {
    #[tracing::instrument(level = "debug", skip(self))]
    fn into(self) -> InnerSong {
        InnerSong {
            _id: self._id,
            path: self.path,
            size: self.size,
            inode: self.inode,
            deviceno: self.deviceno,
            title: self.title,
            date: self.date,
            year: self.year,
            lyrics: self.lyrics,
            release_type: self.release_type,
            bitrate: self.bitrate,
            codec: self.codec,
            container: self.container,
            duration: self.duration,
            sample_rate: self.sample_rate,
            hash: self.hash,
            type_: self.type_.into(),
            url: self.url,
            song_cover_path_high: self.song_cover_path_high,
            playback_url: self.playback_url,
            song_cover_path_low: self.song_cover_path_low,
            date_added: self.date_added,
            provider_extension: self.provider_extension,
            icon: self.icon,
            show_in_library: self.show_in_library,
            track_no: self.track_no,
            library_item: self.library_item,
        }
    }
}

#[derive(Default, Clone, Debug, Insertable, Queryable, Identifiable, AsChangeset)]
#[diesel(table_name = analytics)]
#[diesel(primary_key(id))]
pub struct QueryableAnalytics {
    pub id: Option<String>,
    pub song_id: Option<String>,
    pub play_count: Option<i32>,
    pub play_time: Option<f64>,
}

#[derive(Default, Clone, Debug, Insertable, Queryable, AsChangeset)]
#[diesel(table_name = cache)]

pub struct CacheModel {
    pub id: Option<i32>,
    pub url: String,
    pub blob: Vec<u8>,
    pub expires: i64,
}
