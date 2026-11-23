use prost::Message;
use songs_proto::moosync::types::{Album, Artist, Genre, InnerSong, PlaybackUrl, Playlist};

use crate::utils::{db_ms_to_proto, song_type_from_str};

#[derive(Debug, sqlx::FromRow)]
pub struct DbAlbumRow {
    pub album_id: Option<String>,
    pub album_name: Option<String>,
    pub album_artist: Option<String>,
    pub album_coverpath_high: Option<String>,
    pub album_song_count: i64,
    pub year: Option<String>,
    pub album_coverpath_low: Option<String>,
}

impl From<DbAlbumRow> for Album {
    #[tracing::instrument(level = "debug", skip_all)]
    fn from(row: DbAlbumRow) -> Self {
        Self {
            album_id: row.album_id,
            album_name: row.album_name,
            album_artist: row.album_artist,
            album_coverpath_high: row.album_coverpath_high,
            album_song_count: row.album_song_count as f64,
            year: row.year,
            album_coverpath_low: row.album_coverpath_low,
            extension: None,
        }
    }
}

#[derive(Debug, sqlx::FromRow)]
pub struct DbArtistRow {
    pub artist_id: Option<String>,
    pub artist_mbid: Option<String>,
    pub artist_name: Option<String>,
    pub artist_coverpath: Option<String>,
    pub artist_song_count: i64,
    pub sanitized_artist_name: Option<String>,
}

impl From<DbArtistRow> for Artist {
    #[tracing::instrument(level = "debug", skip_all)]
    fn from(row: DbArtistRow) -> Self {
        Self {
            artist_id: row.artist_id,
            artist_mbid: row.artist_mbid,
            artist_name: row.artist_name,
            artist_coverpath: row.artist_coverpath,
            artist_song_count: row.artist_song_count as f64,
            sanitized_artist_name: row.sanitized_artist_name,
            extension: None,
        }
    }
}

#[derive(Debug, sqlx::FromRow)]
pub struct DbGenreRow {
    pub genre_id: Option<String>,
    pub genre_name: Option<String>,
    pub genre_song_count: i64,
}

impl From<DbGenreRow> for Genre {
    #[tracing::instrument(level = "debug", skip_all)]
    fn from(row: DbGenreRow) -> Self {
        Self {
            genre_id: row.genre_id,
            genre_name: row.genre_name,
            genre_song_count: row.genre_song_count as f64,
        }
    }
}

#[derive(Debug, sqlx::FromRow)]
pub struct DbPlaylistRow {
    pub playlist_id: Option<String>,
    pub playlist_name: String,
    pub playlist_coverpath: Option<String>,
    pub playlist_song_count: i64,
    pub playlist_desc: Option<String>,
    pub playlist_path: Option<String>,
    pub extension: Option<String>,
    pub icon: Option<String>,
    pub library_item: Option<bool>,
}

impl From<DbPlaylistRow> for Playlist {
    #[tracing::instrument(level = "debug", skip_all)]
    fn from(row: DbPlaylistRow) -> Self {
        Self {
            playlist_id: row.playlist_id,
            playlist_name: row.playlist_name,
            playlist_coverpath: row.playlist_coverpath,
            playlist_song_count: row.playlist_song_count as f64,
            playlist_desc: row.playlist_desc,
            playlist_path: row.playlist_path,
            extension: row.extension,
            icon: row.icon,
            library_item: row.library_item,
        }
    }
}

#[derive(Debug, sqlx::FromRow)]
pub struct DbInnerSongRow {
    pub id: Option<String>,
    pub path: Option<String>,
    pub size: Option<f64>,
    pub title: Option<String>,
    pub date: Option<String>,
    pub year: Option<String>,
    pub release_type: Option<String>,
    pub bitrate: Option<f64>,
    pub codec: Option<String>,
    pub container: Option<String>,
    pub duration: Option<i64>,
    pub sample_rate: Option<f64>,
    pub hash: Option<String>,
    pub r#type: String,
    pub url: Option<String>,
    pub song_cover_path_high: Option<String>,
    pub playback_url: Option<Vec<u8>>,
    pub song_cover_path_low: Option<String>,
    pub date_added: Option<i64>,
    pub extension: Option<String>,
    pub track_no: Option<f64>,
}

impl From<DbInnerSongRow> for InnerSong {
    #[tracing::instrument(level = "debug", skip_all)]
    fn from(row: DbInnerSongRow) -> Self {
        Self {
            id: row.id,
            path: row.path,
            size: row.size,
            title: row.title,
            date: row.date,
            year: row.year,
            lyrics: None,
            release_type: row.release_type,
            bitrate: row.bitrate,
            codec: row.codec,
            container: row.container,
            duration: Some(db_ms_to_proto(row.duration.unwrap_or_default())),
            sample_rate: row.sample_rate,
            hash: row.hash,
            r#type: song_type_from_str(&row.r#type),
            url: row.url,
            song_cover_path_high: row.song_cover_path_high,
            playback_url: row
                .playback_url
                .and_then(|bytes| PlaybackUrl::decode(bytes.as_slice()).ok()),
            song_cover_path_low: row.song_cover_path_low,
            date_added: row.date_added,
            extension: row.extension,
            track_no: row.track_no,
        }
    }
}
