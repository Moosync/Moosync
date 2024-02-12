use diesel::{AsChangeset, Identifiable, Insertable, Queryable};
use serde::{Deserialize, Serialize};

use crate::db::schema::{
    album_bridge, albums, artist_bridge, artists, genre_bridge, genres, playlist_bridge, playlists,
};

#[derive(
    Deserialize, Serialize, Insertable, Default, Queryable, Identifiable, AsChangeset, Clone, Debug,
)]
#[diesel(table_name = albums)]
#[diesel(primary_key(album_id))]
pub struct QueryableAlbum {
    pub album_id: Option<String>,
    pub album_name: Option<String>,
    pub album_artist: Option<String>,
    pub album_coverPath_high: Option<String>,
    #[serde(default)]
    pub album_song_count: f64,
    pub year: Option<String>,
    pub album_coverPath_low: Option<String>,
    pub album_extra_info: Option<String>,
}

#[derive(Deserialize, Insertable, Default, Queryable, Identifiable, Clone, Debug)]
#[diesel(table_name = album_bridge)]
#[diesel(primary_key(id))]
pub struct AlbumBridge {
    pub id: Option<i32>,
    pub song: Option<String>,
    pub album: Option<String>,
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
    pub artist_coverPath: Option<String>,
    #[serde(default)]
    pub artist_song_count: f64,
    pub artist_extra_info: Option<String>,
    pub sanitized_artist_name: Option<String>,
}

#[derive(Deserialize, Insertable, Default, Queryable, Identifiable, Clone, Debug)]
#[diesel(table_name = artist_bridge)]
#[diesel(primary_key(id))]
pub struct ArtistBridge {
    pub id: Option<i32>,
    pub song: Option<String>,
    pub artist: Option<String>,
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

#[derive(Deserialize, Insertable, Default, Queryable, Identifiable, Clone, Debug)]
#[diesel(table_name = genre_bridge)]
#[diesel(primary_key(id))]
pub struct GenreBridge {
    pub id: Option<i32>,
    pub song: Option<String>,
    pub genre: Option<String>,
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

#[derive(
    Deserialize, Serialize, Insertable, Default, Queryable, Identifiable, AsChangeset, Clone, Debug,
)]
#[diesel(table_name = playlists)]
#[diesel(primary_key(playlist_id))]
pub struct QueryablePlaylist {
    pub playlist_id: Option<String>,
    #[serde(default)]
    pub playlist_name: String,
    pub playlist_coverPath: Option<String>,
    #[serde(default)]
    pub playlist_song_count: f64,
    pub playlist_desc: Option<String>,
    pub playlist_path: Option<String>,
    pub extension: Option<String>,
    pub icon: Option<String>,
}
