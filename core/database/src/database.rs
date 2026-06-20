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

use std::{
    cmp::min,
    fmt::{Debug, Write},
    fs,
    path::PathBuf,
    vec,
};

use songs_proto::moosync::types::{
    Album, AlbumList, AllAnalytics, Artist, ArtistList, EntityResult, Genre, GenreList,
    GetEntityOptions, GetSongOptions, InnerSong, Playlist, PlaylistList, SearchResult,
    SearchableSong, Song, all_analytics::SongListenTime,
};
use tracing::{debug, info, trace, warn};
use types::{
    errors::{Result, error_helpers},
    prelude::SongsExt,
};
use uuid::Uuid;

use super::migrations::run_migrations;
use crate::utils::{
    map_row_to_album, map_row_to_artist, map_row_to_genre, map_row_to_inner_song,
    map_row_to_playlist, proto_to_db_ms, song_type_to_str,
};

#[derive(Debug, Clone)]
pub struct Database {
    pub pool: r2d2::Pool<r2d2_sqlite::SqliteConnectionManager>,
}

#[plugin_macro::generate]
impl Database {
    #[tracing::instrument(level = "debug", skip(path))]
    pub fn new(path: PathBuf) -> Self {
        debug!("Creating database handler");
        if !path.exists() {
            fs::create_dir_all(&path).expect("Failed to create dir to store database");
        }

        let db = Self {
            pool: Self::connect(path.join("songs.db")),
        };

        let mut conn = db.pool.get().expect("Failed to get connection to DB");
        run_migrations(&mut conn);
        conn.execute_batch("
            PRAGMA journal_mode = WAL;          -- better write-concurrency
            PRAGMA synchronous = NORMAL;        -- fsync only in critical moments
            PRAGMA wal_autocheckpoint = 1000;   -- write WAL changes back every 1000 pages, for an in average 1MB WAL file. May affect readers if number is increased
            PRAGMA wal_checkpoint(TRUNCATE);    -- free some space by truncating possibly massive WAL files from the last run.
            PRAGMA busy_timeout = 250;          -- sleep if the database is busy
        ").expect("Failed to set DB options");

        info!("Created DB instance");
        db
    }

    #[tracing::instrument(level = "debug", skip(path))]
    fn connect(path: PathBuf) -> r2d2::Pool<r2d2_sqlite::SqliteConnectionManager> {
        let manager = r2d2_sqlite::SqliteConnectionManager::file(path).with_init(|conn| {
            conn.trace_v2(
                rusqlite::trace::TraceEventCodes::SQLITE_TRACE_STMT,
                Some(|event| {
                    if let rusqlite::trace::TraceEvent::Stmt(_, sql) = event {
                        tracing::trace!("Executing SQL: {}", sql);
                    }
                }),
            );
            Ok(())
        });

        r2d2::Pool::builder()
            .build(manager)
            .expect("Failed to create pool.")
    }

    #[tracing::instrument(level = "debug", skip(self, conn))]
    fn insert_playlist(
        &self,
        conn: &mut rusqlite::Connection,
        playlist: Playlist,
    ) -> Result<String> {
        let id = playlist.playlist_id.as_ref().unwrap().clone();
        trace!("Inserting playlist");
        conn.execute(
            "INSERT INTO playlists (playlist_id, playlist_name, playlist_coverpath, playlist_song_count, playlist_desc, playlist_path, extension, icon, library_item)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            (
                &playlist.playlist_id,
                &playlist.playlist_name,
                &playlist.playlist_coverpath,
                &playlist.playlist_song_count,
                &playlist.playlist_desc,
                &playlist.playlist_path,
                &playlist.extension,
                &playlist.icon,
                &playlist.library_item,
            ),
        )
        .map_err(error_helpers::to_database_error)?;
        info!("Inserted playlist");
        Ok(id)
    }

    #[tracing::instrument(level = "debug", skip(self))]
    pub fn create_playlist(&self, mut playlist: Playlist) -> Result<String> {
        let mut conn = self.pool.get().unwrap();

        trace!("Sanitizing playlist");

        if playlist.playlist_id.is_none() {
            playlist.playlist_id = Some(Uuid::new_v4().to_string());
        }

        if playlist.playlist_name.is_empty() {
            playlist.playlist_name = "New playlist".to_string();
        }

        if playlist.playlist_path.is_some() {
            let fetched = self.get_playlists(
                Playlist {
                    playlist_path: playlist.playlist_path.clone(),
                    ..Default::default()
                },
                false,
                &mut conn,
            )?;
            if !fetched.is_empty() {
                return Ok(fetched[0].playlist_id.clone().unwrap());
            }
        }

        self.insert_playlist(&mut conn, playlist)
    }

    #[tracing::instrument(level = "debug", skip(self))]
    pub fn create_playlist_with_songs(&self, playlist: Playlist, songs: &[Song]) -> Result<()> {
        let playlist_id = match self.create_playlist(playlist.clone()) {
            Ok(id) => id,
            Err(e) => {
                if let Some(ref id) = playlist.playlist_id {
                    id.clone()
                } else {
                    return Err(e);
                }
            }
        };
        if songs.is_empty() {
            return Ok(());
        }
        self.add_to_playlist(&playlist_id, songs)?;
        Ok(())
    }

    #[tracing::instrument(level = "debug", skip(self))]
    pub fn add_to_playlist_bridge(&self, playlist_id: String, song_id: String) -> Result<()> {
        let conn = self.pool.get().unwrap();
        trace!("Inserting song in playlist bridge");
        conn.execute(
            "INSERT INTO playlist_bridge (playlist, song) VALUES (?1, ?2)",
            (&playlist_id, &song_id),
        )
        .map_err(error_helpers::to_database_error)?;

        trace!("Inserted song in playlist bridge");

        Ok(())
    }

    #[tracing::instrument(level = "debug", skip(self))]
    pub fn insert_songs(&self, mut songs: Vec<Song>) -> Result<Vec<Song>> {
        self.insert_songs_by_ref(&mut songs)?;
        Ok(songs)
    }

    pub fn insert_songs_by_ref(&self, songs: &mut [Song]) -> Result<()> {
        let mut conn = self.pool.get().unwrap();
        trace!("Inserting songs");

        struct DbSongRow {
            id: Option<String>,
            path: Option<String>,
            size: Option<f64>,
            inode: Option<String>,
            deviceno: Option<String>,
            title: Option<String>,
            date: Option<String>,
            year: Option<String>,
            lyrics: Option<String>,
            release_type: Option<String>,
            bitrate: Option<f64>,
            codec: Option<String>,
            container: Option<String>,
            duration: Option<i64>,
            sample_rate: Option<f64>,
            hash: Option<String>,
            r#type: String,
            url: Option<String>,
            song_cover_path_high: Option<String>,
            playback_url: Option<String>,
            song_cover_path_low: Option<String>,
            date_added: Option<i64>,
            provider_extension: Option<String>,
            icon: Option<String>,
            show_in_library: Option<bool>,
            track_no: Option<f64>,
            library_item: Option<bool>,
        }

        // 1. Gather all unique albums, artists, genres from the songs
        let mut unique_albums = std::collections::HashMap::new();
        let mut unique_artists = std::collections::HashMap::new();
        let mut unique_genres = std::collections::HashMap::new();

        for song in songs.iter() {
            if let Some(album) = &song.album {
                if let Some(ref name) = album.album_name {
                    if !name.is_empty() {
                        unique_albums
                            .entry(name.clone())
                            .or_insert_with(|| album.clone());
                    }
                }
            }
            for artist in &song.artists {
                if let Some(ref name) = artist.artist_name {
                    if !name.is_empty() {
                        unique_artists
                            .entry(name.clone())
                            .or_insert_with(|| artist.clone());
                    }
                }
            }
            for genre in &song.genre {
                if let Some(ref name) = genre.genre_name {
                    if !name.is_empty() {
                        unique_genres
                            .entry(name.clone())
                            .or_insert_with(|| genre.clone());
                    }
                }
            }
        }

        // 2. Fetch existing albums, artists, genres from the DB
        let mut album_ids = std::collections::HashMap::new();
        if !unique_albums.is_empty() {
            let names: Vec<String> = unique_albums.keys().cloned().collect();
            for chunk in names.chunks(500) {
                let placeholders = vec!["?"; chunk.len()].join(",");
                let query = format!(
                    "SELECT album_id, album_name FROM albums WHERE album_name IN ({})",
                    placeholders
                );
                let mut stmt = conn
                    .prepare(&query)
                    .map_err(error_helpers::to_database_error)?;
                let params = chunk
                    .iter()
                    .map(|n| n as &dyn rusqlite::ToSql)
                    .collect::<Vec<_>>();
                let rows = stmt
                    .query_map(&*params, |row| {
                        let id: String = row.get(0)?;
                        let name: String = row.get(1)?;
                        Ok((name, id))
                    })
                    .map_err(error_helpers::to_database_error)?;
                for r in rows {
                    let (name, id) = r.map_err(error_helpers::to_database_error)?;
                    album_ids.insert(name, id);
                }
            }
        }

        let mut artist_ids = std::collections::HashMap::new();
        if !unique_artists.is_empty() {
            let names: Vec<String> = unique_artists.keys().cloned().collect();
            for chunk in names.chunks(500) {
                let placeholders = vec!["?"; chunk.len()].join(",");
                let query = format!(
                    "SELECT artist_id, artist_name FROM artists WHERE artist_name IN ({})",
                    placeholders
                );
                let mut stmt = conn
                    .prepare(&query)
                    .map_err(error_helpers::to_database_error)?;
                let params = chunk
                    .iter()
                    .map(|n| n as &dyn rusqlite::ToSql)
                    .collect::<Vec<_>>();
                let rows = stmt
                    .query_map(&*params, |row| {
                        let id: String = row.get(0)?;
                        let name: String = row.get(1)?;
                        Ok((name, id))
                    })
                    .map_err(error_helpers::to_database_error)?;
                for r in rows {
                    let (name, id) = r.map_err(error_helpers::to_database_error)?;
                    artist_ids.insert(name, id);
                }
            }
        }

        let mut genre_ids = std::collections::HashMap::new();
        if !unique_genres.is_empty() {
            let names: Vec<String> = unique_genres.keys().cloned().collect();
            for chunk in names.chunks(500) {
                let placeholders = vec!["?"; chunk.len()].join(",");
                let query = format!(
                    "SELECT genre_id, genre_name FROM genres WHERE genre_name IN ({})",
                    placeholders
                );
                let mut stmt = conn
                    .prepare(&query)
                    .map_err(error_helpers::to_database_error)?;
                let params = chunk
                    .iter()
                    .map(|n| n as &dyn rusqlite::ToSql)
                    .collect::<Vec<_>>();
                let rows = stmt
                    .query_map(&*params, |row| {
                        let id: String = row.get(0)?;
                        let name: String = row.get(1)?;
                        Ok((name, id))
                    })
                    .map_err(error_helpers::to_database_error)?;
                for r in rows {
                    let (name, id) = r.map_err(error_helpers::to_database_error)?;
                    genre_ids.insert(name, id);
                }
            }
        }

        // 3. Collect new albums, artists, genres and insert them
        let mut new_albums = Vec::new();
        for (name, album) in unique_albums.iter() {
            if !album_ids.contains_key(name) {
                let id = Uuid::new_v4().to_string();
                album_ids.insert(name.clone(), id.clone());
                let mut alb = album.clone();
                alb.album_id = Some(id);
                new_albums.push(alb);
            }
        }
        if !new_albums.is_empty() {
            for chunk in new_albums.chunks(100) {
                let mut query = "INSERT INTO albums (album_id, album_name, album_artist, album_coverpath_high, album_song_count, year, album_coverpath_low) VALUES ".to_string();
                let mut placeholders = Vec::new();
                let mut params: Vec<&dyn rusqlite::ToSql> = Vec::new();
                for (i, alb) in chunk.iter().enumerate() {
                    let offset = i * 7;
                    placeholders.push(format!(
                        "(?{}, ?{}, ?{}, ?{}, ?{}, ?{}, ?{})",
                        offset + 1,
                        offset + 2,
                        offset + 3,
                        offset + 4,
                        offset + 5,
                        offset + 6,
                        offset + 7
                    ));
                    params.push(alb.album_id.as_ref().unwrap());
                    params.push(&alb.album_name);
                    params.push(&alb.album_artist);
                    params.push(&alb.album_coverpath_high);
                    params.push(&alb.album_song_count);
                    params.push(&alb.year);
                    params.push(&alb.album_coverpath_low);
                }
                query.push_str(&placeholders.join(", "));
                conn.execute(&query, &*params)
                    .map_err(error_helpers::to_database_error)?;
            }
        }

        let mut new_artists = Vec::new();
        for (name, artist) in unique_artists.iter() {
            if !artist_ids.contains_key(name) {
                let id = Uuid::new_v4().to_string();
                artist_ids.insert(name.clone(), id.clone());
                let mut art = artist.clone();
                art.artist_id = Some(id);
                new_artists.push(art);
            }
        }
        if !new_artists.is_empty() {
            for chunk in new_artists.chunks(100) {
                let mut query = "INSERT INTO artists (artist_id, artist_mbid, artist_name, artist_coverpath, artist_song_count, artist_extra_info, sanitized_artist_name) VALUES ".to_string();
                let mut placeholders = Vec::new();
                let mut params: Vec<&dyn rusqlite::ToSql> = Vec::new();
                for (i, art) in chunk.iter().enumerate() {
                    let offset = i * 7;
                    placeholders.push(format!(
                        "(?{}, ?{}, ?{}, ?{}, ?{}, ?{}, ?{})",
                        offset + 1,
                        offset + 2,
                        offset + 3,
                        offset + 4,
                        offset + 5,
                        offset + 6,
                        offset + 7
                    ));
                    params.push(art.artist_id.as_ref().unwrap());
                    params.push(&art.artist_mbid);
                    params.push(&art.artist_name);
                    params.push(&art.artist_coverpath);
                    params.push(&art.artist_song_count);
                    params.push(&None::<String>);
                    params.push(&art.sanitized_artist_name);
                }
                query.push_str(&placeholders.join(", "));
                conn.execute(&query, &*params)
                    .map_err(error_helpers::to_database_error)?;
            }
        }

        let mut new_genres = Vec::new();
        for (name, genre) in unique_genres.iter() {
            if !genre_ids.contains_key(name) {
                let id = Uuid::new_v4().to_string();
                genre_ids.insert(name.clone(), id.clone());
                let mut genre_obj = genre.clone();
                genre_obj.genre_id = Some(id);
                new_genres.push(genre_obj);
            }
        }
        if !new_genres.is_empty() {
            for chunk in new_genres.chunks(100) {
                let mut query =
                    "INSERT INTO genres (genre_id, genre_name, genre_song_count) VALUES "
                        .to_string();
                let mut placeholders = Vec::new();
                let mut params: Vec<&dyn rusqlite::ToSql> = Vec::new();
                for (i, genre_obj) in chunk.iter().enumerate() {
                    let offset = i * 3;
                    placeholders.push(format!(
                        "(?{}, ?{}, ?{})",
                        offset + 1,
                        offset + 2,
                        offset + 3
                    ));
                    params.push(genre_obj.genre_id.as_ref().unwrap());
                    params.push(&genre_obj.genre_name);
                    params.push(&genre_obj.genre_song_count);
                }
                query.push_str(&placeholders.join(", "));
                conn.execute(&query, &*params)
                    .map_err(error_helpers::to_database_error)?;
            }
        }

        // 4. Fetch existing song IDs for paths that already exist in the database
        let mut paths_to_query = Vec::new();
        for song in songs.iter() {
            if let Some(inner_song) = &song.song {
                if let Some(ref path) = inner_song.path {
                    if !path.is_empty() {
                        paths_to_query.push(path.clone());
                    }
                }
            }
        }

        let mut existing_song_ids = std::collections::HashMap::new();
        if !paths_to_query.is_empty() {
            for chunk in paths_to_query.chunks(500) {
                let placeholders = vec!["?"; chunk.len()].join(",");
                let query = format!(
                    "SELECT _id, path FROM allsongs WHERE path IN ({})",
                    placeholders
                );
                let mut stmt = conn
                    .prepare(&query)
                    .map_err(error_helpers::to_database_error)?;
                let params = chunk
                    .iter()
                    .map(|n| n as &dyn rusqlite::ToSql)
                    .collect::<Vec<_>>();
                let rows = stmt
                    .query_map(&*params, |row| {
                        let id: String = row.get(0)?;
                        let path: String = row.get(1)?;
                        Ok((path, id))
                    })
                    .map_err(error_helpers::to_database_error)?;
                for r in rows {
                    let (path, id) = r.map_err(error_helpers::to_database_error)?;
                    existing_song_ids.insert(path, id);
                }
            }
        }

        // 5. Update the incoming songs slice with matched or new IDs
        for song in songs.iter_mut() {
            if let Some(inner_song) = song.song.as_mut() {
                let mut resolved_id = None;
                if let Some(ref path) = inner_song.path {
                    if !path.is_empty() {
                        if let Some(id) = existing_song_ids.get(path) {
                            resolved_id = Some(id.clone());
                        }
                    }
                }
                if resolved_id.is_none() {
                    resolved_id = Some(
                        inner_song
                            .id
                            .clone()
                            .unwrap_or_else(|| Uuid::new_v4().to_string()),
                    );
                }
                inner_song.id = resolved_id;
            }
            if let Some(album) = song.album.as_mut() {
                if let Some(ref name) = album.album_name {
                    if let Some(id) = album_ids.get(name) {
                        album.album_id = Some(id.clone());
                    }
                }
            }
            for artist in song.artists.iter_mut() {
                if let Some(ref name) = artist.artist_name {
                    if let Some(id) = artist_ids.get(name) {
                        artist.artist_id = Some(id.clone());
                    }
                }
            }
            for genre in song.genre.iter_mut() {
                if let Some(ref name) = genre.genre_name {
                    if let Some(id) = genre_ids.get(name) {
                        genre.genre_id = Some(id.clone());
                    }
                }
            }
        }

        // 5. Map songs to DbSongRow and prepare bridge entries
        let mut db_song_rows = Vec::new();
        let mut album_bridges = Vec::new();
        let mut artist_bridges = Vec::new();
        let mut genre_bridges = Vec::new();

        for song in songs.iter() {
            if let Some(inner_song) = &song.song {
                let song_id = inner_song.id.clone().unwrap_or_default();
                let song_type = song_type_to_str(inner_song.r#type).to_string();

                db_song_rows.push(DbSongRow {
                    id: inner_song.id.clone(),
                    path: inner_song.path.clone(),
                    size: inner_song.size,
                    inode: inner_song.inode.clone(),
                    deviceno: inner_song.deviceno.clone(),
                    title: inner_song.title.clone(),
                    date: inner_song.date.clone(),
                    year: inner_song.year.clone(),
                    lyrics: inner_song.lyrics.clone(),
                    release_type: inner_song.release_type.clone(),
                    bitrate: inner_song.bitrate,
                    codec: inner_song.codec.clone(),
                    container: inner_song.container.clone(),
                    duration: Some(proto_to_db_ms(&inner_song.duration)),
                    sample_rate: inner_song.sample_rate,
                    hash: inner_song.hash.clone(),
                    r#type: song_type,
                    url: inner_song.url.clone(),
                    song_cover_path_high: inner_song.song_cover_path_high.clone(),
                    playback_url: inner_song.playback_url.clone(),
                    song_cover_path_low: inner_song.song_cover_path_low.clone(),
                    date_added: inner_song.date_added,
                    provider_extension: inner_song.provider_extension.clone(),
                    icon: inner_song.icon.clone(),
                    show_in_library: inner_song.show_in_library,
                    track_no: inner_song.track_no,
                    library_item: inner_song.library_item,
                });

                if let Some(album) = &song.album {
                    if let Some(ref album_id) = album.album_id {
                        album_bridges.push((song_id.clone(), album_id.clone()));
                    }
                }

                for artist in &song.artists {
                    if let Some(ref artist_id) = artist.artist_id {
                        artist_bridges.push((song_id.clone(), artist_id.clone()));
                    }
                }

                for genre in &song.genre {
                    if let Some(ref genre_id) = genre.genre_id {
                        genre_bridges.push((song_id.clone(), genre_id.clone()));
                    }
                }
            }
        }

        // 6. Execute bulk insertions within a transaction
        let mut conn = self.pool.get().unwrap();
        let tx = conn
            .transaction()
            .map_err(error_helpers::to_database_error)?;

        for chunk in db_song_rows.chunks(30) {
            let mut query = "INSERT INTO allsongs (
                _id, path, size, inode, deviceno, title, date, year, lyrics, releasetype,
                bitrate, codec, container, duration, samplerate, hash, type, url,
                song_coverpath_high, playbackurl, song_coverpath_low, date_added,
                provider_extension, icon, show_in_library, track_no, library_item
            ) VALUES "
                .to_string();
            let mut placeholders = Vec::new();
            let mut params: Vec<&dyn rusqlite::ToSql> = Vec::new();

            for (i, row) in chunk.iter().enumerate() {
                let offset = i * 27;
                placeholders.push(format!(
                    "(?{}, ?{}, ?{}, ?{}, ?{}, ?{}, ?{}, ?{}, ?{}, ?{}, ?{}, ?{}, ?{}, ?{}, ?{}, ?{}, ?{}, ?{}, ?{}, ?{}, ?{}, ?{}, ?{}, ?{}, ?{}, ?{}, ?{})",
                    offset + 1, offset + 2, offset + 3, offset + 4, offset + 5, offset + 6, offset + 7, offset + 8, offset + 9, offset + 10,
                    offset + 11, offset + 12, offset + 13, offset + 14, offset + 15, offset + 16, offset + 17, offset + 18, offset + 19, offset + 20,
                    offset + 21, offset + 22, offset + 23, offset + 24, offset + 25, offset + 26, offset + 27
                ));

                params.push(&row.id);
                params.push(&row.path);
                params.push(&row.size);
                params.push(&row.inode);
                params.push(&row.deviceno);
                params.push(&row.title);
                params.push(&row.date);
                params.push(&row.year);
                params.push(&row.lyrics);
                params.push(&row.release_type);
                params.push(&row.bitrate);
                params.push(&row.codec);
                params.push(&row.container);
                params.push(&row.duration);
                params.push(&row.sample_rate);
                params.push(&row.hash);
                params.push(&row.r#type);
                params.push(&row.url);
                params.push(&row.song_cover_path_high);
                params.push(&row.playback_url);
                params.push(&row.song_cover_path_low);
                params.push(&row.date_added);
                params.push(&row.provider_extension);
                params.push(&row.icon);
                params.push(&row.show_in_library);
                params.push(&row.track_no);
                params.push(&row.library_item);
            }

            query.push_str(&placeholders.join(", "));
            query.push_str(
                " ON CONFLICT(path) DO UPDATE SET
                size = excluded.size,
                inode = excluded.inode,
                deviceno = excluded.deviceno,
                title = excluded.title,
                date = excluded.date,
                year = excluded.year,
                lyrics = excluded.lyrics,
                releasetype = excluded.releasetype,
                bitrate = excluded.bitrate,
                codec = excluded.codec,
                container = excluded.container,
                duration = excluded.duration,
                samplerate = excluded.samplerate,
                hash = excluded.hash,
                type = excluded.type,
                url = excluded.url,
                song_coverpath_high = excluded.song_coverpath_high,
                playbackurl = excluded.playbackurl,
                song_coverpath_low = excluded.song_coverpath_low,
                date_added = excluded.date_added,
                provider_extension = excluded.provider_extension,
                icon = excluded.icon,
                show_in_library = excluded.show_in_library,
                track_no = excluded.track_no,
                library_item = excluded.library_item",
            );

            tx.execute(&query, &*params)
                .map_err(error_helpers::to_database_error)?;
        }

        for chunk in album_bridges.chunks(400) {
            let mut query = "INSERT INTO album_bridge (song, album) VALUES ".to_string();
            let mut placeholders = Vec::new();
            let mut params: Vec<&dyn rusqlite::ToSql> = Vec::new();
            for (i, (song_id, album_id)) in chunk.iter().enumerate() {
                let offset = i * 2;
                placeholders.push(format!("(?{}, ?{})", offset + 1, offset + 2));
                params.push(song_id);
                params.push(album_id);
            }
            query.push_str(&placeholders.join(", "));
            query.push_str(" ON CONFLICT DO NOTHING");
            tx.execute(&query, &*params)
                .map_err(error_helpers::to_database_error)?;
        }

        for chunk in artist_bridges.chunks(400) {
            let mut query = "INSERT INTO artist_bridge (song, artist) VALUES ".to_string();
            let mut placeholders = Vec::new();
            let mut params: Vec<&dyn rusqlite::ToSql> = Vec::new();
            for (i, (song_id, artist_id)) in chunk.iter().enumerate() {
                let offset = i * 2;
                placeholders.push(format!("(?{}, ?{})", offset + 1, offset + 2));
                params.push(song_id);
                params.push(artist_id);
            }
            query.push_str(&placeholders.join(", "));
            query.push_str(" ON CONFLICT DO NOTHING");
            tx.execute(&query, &*params)
                .map_err(error_helpers::to_database_error)?;
        }

        for chunk in genre_bridges.chunks(400) {
            let mut query = "INSERT INTO genre_bridge (song, genre) VALUES ".to_string();
            let mut placeholders = Vec::new();
            let mut params: Vec<&dyn rusqlite::ToSql> = Vec::new();
            for (i, (song_id, genre_id)) in chunk.iter().enumerate() {
                let offset = i * 2;
                placeholders.push(format!("(?{}, ?{})", offset + 1, offset + 2));
                params.push(song_id);
                params.push(genre_id);
            }
            query.push_str(&placeholders.join(", "));
            query.push_str(" ON CONFLICT DO NOTHING");
            tx.execute(&query, &*params)
                .map_err(error_helpers::to_database_error)?;
        }

        tx.commit().map_err(error_helpers::to_database_error)?;
        info!("Inserted all songs");
        Ok(())
    }

    #[tracing::instrument(level = "debug", skip(self))]
    pub fn remove_songs<T>(&self, ids: &[T]) -> Result<()>
    where
        T: AsRef<str> + rusqlite::ToSql + Debug,
    {
        trace!("Removing song");
        let mut conn = self.pool.get().unwrap();
        let tx = conn
            .transaction()
            .map_err(error_helpers::to_database_error)?;
        for id in ids {
            let s: &str = id.as_ref();
            tx.execute("DELETE FROM analytics WHERE song_id = ?1", [&s])
                .map_err(error_helpers::to_database_error)?;
            tx.execute("DELETE FROM album_bridge WHERE song = ?1", [&id])
                .map_err(error_helpers::to_database_error)?;
            tx.execute("DELETE FROM artist_bridge WHERE song = ?1", [&id])
                .map_err(error_helpers::to_database_error)?;
            tx.execute("DELETE FROM genre_bridge WHERE song = ?1", [&id])
                .map_err(error_helpers::to_database_error)?;
            tx.execute("DELETE FROM playlist_bridge WHERE song = ?1", [&id])
                .map_err(error_helpers::to_database_error)?;
            tx.execute("DELETE FROM allsongs WHERE _id = ?1", [&id])
                .map_err(error_helpers::to_database_error)?;
        }
        tx.commit().map_err(error_helpers::to_database_error)?;
        info!("Removed song");
        Ok(())
    }

    #[tracing::instrument(level = "debug", skip(self, song))]
    pub fn update_song(&self, song: &InnerSong) -> Result<()> {
        trace!("Updating song");
        if let Some(id) = song.id.as_ref() {
            let conn = self.pool.get().unwrap();
            let song_type = song_type_to_str(song.r#type);
            let params: &[&dyn rusqlite::ToSql] = &[
                &song.path,
                &song.size,
                &song.inode,
                &song.deviceno,
                &song.title,
                &song.date,
                &song.year,
                &song.lyrics,
                &song.release_type,
                &song.bitrate,
                &song.codec,
                &song.container,
                &proto_to_db_ms(&song.duration),
                &song.sample_rate,
                &song.hash,
                &song_type,
                &song.url,
                &song.song_cover_path_high,
                &song.playback_url,
                &song.song_cover_path_low,
                &song.date_added,
                &song.provider_extension,
                &song.icon,
                &song.show_in_library,
                &song.track_no,
                &song.library_item,
                id,
            ];
            conn.execute(
                "UPDATE allsongs SET
                    path = ?1, size = ?2, inode = ?3, deviceno = ?4, title = ?5, date = ?6,
                    year = ?7, lyrics = ?8, releasetype = ?9, bitrate = ?10, codec = ?11,
                    container = ?12, duration = ?13, samplerate = ?14, hash = ?15, type = ?16,
                    url = ?17, song_coverpath_high = ?18, playbackurl = ?19, song_coverpath_low = ?20,
                    date_added = ?21, provider_extension = ?22, icon = ?23, show_in_library = ?24,
                    track_no = ?25, library_item = ?26
                 WHERE _id = ?27",
                params,
            ).map_err(error_helpers::to_database_error)?;
            debug!("Updated song");
        } else {
            debug!("Song does not have an ID");
        }
        Ok(())
    }

    #[tracing::instrument(level = "debug", skip(self, conn))]
    fn get_albums(
        &self,
        options: Album,
        inclusive: bool,
        conn: &mut rusqlite::Connection,
    ) -> Result<Vec<Album>> {
        let mut query = "SELECT album_id, album_name, album_artist, album_coverpath_high, album_song_count, year, album_coverpath_low FROM albums".to_string();
        let mut clauses = Vec::new();
        let mut params = Vec::new();

        if let Some(ref v) = options.album_id {
            clauses.push("album_id = ?".to_string());
            params.push(v as &dyn rusqlite::ToSql);
        }
        if let Some(ref v) = options.album_name {
            clauses.push("album_name LIKE ?".to_string());
            params.push(v as &dyn rusqlite::ToSql);
        }

        if !clauses.is_empty() {
            let joiner = if inclusive { " AND " } else { " OR " };
            query.push_str(" WHERE ");
            query.push_str(&clauses.join(joiner));
        }

        trace!("Getting albums");
        let mut stmt = conn
            .prepare(&query)
            .map_err(error_helpers::to_database_error)?;
        let rows = stmt
            .query_map(&*params, map_row_to_album)
            .map_err(error_helpers::to_database_error)?;

        let mut fetched = Vec::new();
        for r in rows {
            fetched.push(r.map_err(error_helpers::to_database_error)?);
        }
        info!("Fetched albums");
        Ok(fetched)
    }

    #[tracing::instrument(level = "debug", skip(self, conn))]
    fn get_artists(
        &self,
        options: Artist,
        inclusive: bool,
        conn: &mut rusqlite::Connection,
    ) -> Result<Vec<Artist>> {
        let mut query = "SELECT artist_id, artist_mbid, artist_name, artist_coverpath, artist_song_count, sanitized_artist_name FROM artists".to_string();
        let mut clauses = Vec::new();
        let mut params = Vec::new();

        if let Some(ref v) = options.artist_id {
            clauses.push("artist_id = ?".to_string());
            params.push(v as &dyn rusqlite::ToSql);
        }
        if let Some(ref v) = options.artist_name {
            clauses.push("artist_name LIKE ?".to_string());
            params.push(v as &dyn rusqlite::ToSql);
        }
        if let Some(ref v) = options.artist_mbid {
            clauses.push("artist_mbid = ?".to_string());
            params.push(v as &dyn rusqlite::ToSql);
        }

        if !clauses.is_empty() {
            let joiner = if inclusive { " AND " } else { " OR " };
            query.push_str(" WHERE ");
            query.push_str(&clauses.join(joiner));
        }

        trace!("Fetching artists");
        let mut stmt = conn
            .prepare(&query)
            .map_err(error_helpers::to_database_error)?;
        let rows = stmt
            .query_map(&*params, map_row_to_artist)
            .map_err(error_helpers::to_database_error)?;

        let mut fetched = Vec::new();
        for r in rows {
            fetched.push(r.map_err(error_helpers::to_database_error)?);
        }
        info!("Fetched artists");
        Ok(fetched)
    }

    #[tracing::instrument(level = "debug", skip(self, conn))]
    fn get_genres(
        &self,
        options: Genre,
        inclusive: bool,
        conn: &mut rusqlite::Connection,
    ) -> Result<Vec<Genre>> {
        let mut query = "SELECT genre_id, genre_name, genre_song_count FROM genres".to_string();
        let mut clauses = Vec::new();
        let mut params = Vec::new();

        if let Some(ref v) = options.genre_id {
            clauses.push("genre_id = ?".to_string());
            params.push(v as &dyn rusqlite::ToSql);
        }
        if let Some(ref v) = options.genre_name {
            clauses.push("genre_name LIKE ?".to_string());
            params.push(v as &dyn rusqlite::ToSql);
        }

        if !clauses.is_empty() {
            let joiner = if inclusive { " AND " } else { " OR " };
            query.push_str(" WHERE ");
            query.push_str(&clauses.join(joiner));
        }

        trace!("Fetching genres");
        let mut stmt = conn
            .prepare(&query)
            .map_err(error_helpers::to_database_error)?;
        let rows = stmt
            .query_map(&*params, map_row_to_genre)
            .map_err(error_helpers::to_database_error)?;

        let mut fetched = Vec::new();
        for r in rows {
            fetched.push(r.map_err(error_helpers::to_database_error)?);
        }
        info!("Fetched genres");
        Ok(fetched)
    }

    #[tracing::instrument(level = "debug", skip(self, conn))]
    fn get_playlists(
        &self,
        options: Playlist,
        inclusive: bool,
        conn: &mut rusqlite::Connection,
    ) -> Result<Vec<Playlist>> {
        let mut query = "SELECT playlist_id, playlist_name, playlist_coverpath, playlist_song_count, playlist_desc, playlist_path, extension, icon, library_item FROM playlists".to_string();
        let mut clauses = Vec::new();
        let mut params = Vec::new();

        if let Some(ref v) = options.playlist_id {
            clauses.push("playlist_id = ?".to_string());
            params.push(v as &dyn rusqlite::ToSql);
        }
        if !options.playlist_name.is_empty() {
            clauses.push("playlist_name LIKE ?".to_string());
            params.push(&options.playlist_name as &dyn rusqlite::ToSql);
        }
        if let Some(ref v) = options.playlist_path {
            clauses.push("playlist_path LIKE ?".to_string());
            params.push(v as &dyn rusqlite::ToSql);
        }

        if !clauses.is_empty() {
            let joiner = if inclusive { " AND " } else { " OR " };
            query.push_str(" WHERE ");
            query.push_str(&clauses.join(joiner));
        }

        trace!("Fetching playlists");
        let mut stmt = conn
            .prepare(&query)
            .map_err(error_helpers::to_database_error)?;
        let rows = stmt
            .query_map(&*params, map_row_to_playlist)
            .map_err(error_helpers::to_database_error)?;

        let mut fetched = Vec::new();
        for r in rows {
            fetched.push(r.map_err(error_helpers::to_database_error)?);
        }
        Ok(fetched)
    }

    pub fn is_song_in_playlist(&self, playlist_id: &str, song_id: &str) -> Result<bool> {
        let conn = self.pool.get().unwrap();
        let count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM playlist_bridge WHERE playlist = ?1 AND song = ?2",
                (playlist_id, song_id),
                |row| row.get(0),
            )
            .map_err(error_helpers::to_database_error)?;
        Ok(count > 0)
    }

    #[tracing::instrument(level = "debug", skip(self))]
    pub fn get_entity_by_options(&self, options: GetEntityOptions) -> Result<EntityResult> {
        let mut conn = self.pool.get().unwrap();
        let inclusive = options.inclusive.unwrap_or_default();

        trace!("Getting entity by options");

        if let Some(album) = options.album {
            return Ok(EntityResult {
                result: Some(songs_proto::moosync::types::entity_result::Result::Albums(
                    AlbumList {
                        albums: self.get_albums(album, inclusive, &mut conn)?,
                    },
                )),
            });
        }

        if let Some(artist) = options.artist {
            return Ok(EntityResult {
                result: Some(songs_proto::moosync::types::entity_result::Result::Artists(
                    ArtistList {
                        artists: self.get_artists(artist, inclusive, &mut conn)?,
                    },
                )),
            });
        }

        if let Some(genre) = options.genre {
            return Ok(EntityResult {
                result: Some(songs_proto::moosync::types::entity_result::Result::Genres(
                    GenreList {
                        genres: self.get_genres(genre, inclusive, &mut conn)?,
                    },
                )),
            });
        }

        if let Some(playlist) = options.playlist {
            return Ok(EntityResult {
                result: Some(
                    songs_proto::moosync::types::entity_result::Result::Playlists(PlaylistList {
                        playlists: self.get_playlists(playlist, inclusive, &mut conn)?,
                    }),
                ),
            });
        }

        Ok(EntityResult { result: None })
    }

    #[tracing::instrument(level = "debug", skip(self, conn))]
    pub fn get_album_songs(
        &self,
        options: Album,
        inclusive: bool,
        conn: &mut rusqlite::Connection,
    ) -> Result<Vec<InnerSong>> {
        trace!("Fetching album songs");
        let mut query = "SELECT a._id, a.path, a.size, a.inode, a.deviceno, a.title, a.date, a.year, a.lyrics, a.releasetype,
                                a.bitrate, a.codec, a.container, a.duration, a.samplerate, a.hash, a.type, a.url,
                                a.song_coverpath_high, a.playbackurl, a.song_coverpath_low, a.date_added,
                                a.provider_extension, a.icon, a.show_in_library, a.track_no, a.library_item
                         FROM allsongs a
                         JOIN album_bridge b ON a._id = b.song
                         JOIN albums al ON b.album = al.album_id".to_string();
        let mut clauses = Vec::new();
        let mut params = Vec::new();

        if let Some(ref v) = options.album_id {
            clauses.push("al.album_id = ?".to_string());
            params.push(v as &dyn rusqlite::ToSql);
        }
        if let Some(ref v) = options.album_name {
            clauses.push("al.album_name LIKE ?".to_string());
            params.push(v as &dyn rusqlite::ToSql);
        }

        if !clauses.is_empty() {
            let joiner = if inclusive { " AND " } else { " OR " };
            query.push_str(" WHERE ");
            query.push_str(&clauses.join(joiner));
        }

        let mut stmt = conn
            .prepare(&query)
            .map_err(error_helpers::to_database_error)?;
        let rows = stmt
            .query_map(&*params, map_row_to_inner_song)
            .map_err(error_helpers::to_database_error)?;

        let mut fetched = Vec::new();
        for r in rows {
            fetched.push(r.map_err(error_helpers::to_database_error)?);
        }
        info!("Fetched album songs");
        Ok(fetched)
    }

    #[tracing::instrument(level = "debug", skip(self, conn))]
    pub fn get_artist_songs(
        &self,
        options: Artist,
        inclusive: bool,
        conn: &mut rusqlite::Connection,
    ) -> Result<Vec<InnerSong>> {
        trace!("Fetching artist songs");
        let mut query = "SELECT a._id, a.path, a.size, a.inode, a.deviceno, a.title, a.date, a.year, a.lyrics, a.releasetype,
                                a.bitrate, a.codec, a.container, a.duration, a.samplerate, a.hash, a.type, a.url,
                                a.song_coverpath_high, a.playbackurl, a.song_coverpath_low, a.date_added,
                                a.provider_extension, a.icon, a.show_in_library, a.track_no, a.library_item
                         FROM allsongs a
                         JOIN artist_bridge b ON a._id = b.song
                         JOIN artists ar ON b.artist = ar.artist_id".to_string();
        let mut clauses = Vec::new();
        let mut params = Vec::new();

        if let Some(ref v) = options.artist_id {
            clauses.push("ar.artist_id = ?".to_string());
            params.push(v as &dyn rusqlite::ToSql);
        }
        if let Some(ref v) = options.artist_name {
            clauses.push("ar.artist_name LIKE ?".to_string());
            params.push(v as &dyn rusqlite::ToSql);
        }
        if let Some(ref v) = options.artist_mbid {
            clauses.push("ar.artist_mbid = ?".to_string());
            params.push(v as &dyn rusqlite::ToSql);
        }

        if !clauses.is_empty() {
            let joiner = if inclusive { " AND " } else { " OR " };
            query.push_str(" WHERE ");
            query.push_str(&clauses.join(joiner));
        }

        let mut stmt = conn
            .prepare(&query)
            .map_err(error_helpers::to_database_error)?;
        let rows = stmt
            .query_map(&*params, map_row_to_inner_song)
            .map_err(error_helpers::to_database_error)?;

        let mut fetched = Vec::new();
        for r in rows {
            fetched.push(r.map_err(error_helpers::to_database_error)?);
        }
        info!("Fetched artist songs");
        Ok(fetched)
    }

    #[tracing::instrument(level = "debug", skip(self, conn))]
    pub fn get_genre_songs(
        &self,
        options: Genre,
        inclusive: bool,
        conn: &mut rusqlite::Connection,
    ) -> Result<Vec<InnerSong>> {
        trace!("Fetching genre songs");
        let mut query = "SELECT a._id, a.path, a.size, a.inode, a.deviceno, a.title, a.date, a.year, a.lyrics, a.releasetype,
                                a.bitrate, a.codec, a.container, a.duration, a.samplerate, a.hash, a.type, a.url,
                                a.song_coverpath_high, a.playbackurl, a.song_coverpath_low, a.date_added,
                                a.provider_extension, a.icon, a.show_in_library, a.track_no, a.library_item
                         FROM allsongs a
                         JOIN genre_bridge b ON a._id = b.song
                         JOIN genres g ON b.genre = g.genre_id".to_string();
        let mut clauses = Vec::new();
        let mut params = Vec::new();

        if let Some(ref v) = options.genre_id {
            clauses.push("g.genre_id = ?".to_string());
            params.push(v as &dyn rusqlite::ToSql);
        }
        if let Some(ref v) = options.genre_name {
            clauses.push("g.genre_name LIKE ?".to_string());
            params.push(v as &dyn rusqlite::ToSql);
        }

        if !clauses.is_empty() {
            let joiner = if inclusive { " AND " } else { " OR " };
            query.push_str(" WHERE ");
            query.push_str(&clauses.join(joiner));
        }

        let mut stmt = conn
            .prepare(&query)
            .map_err(error_helpers::to_database_error)?;
        let rows = stmt
            .query_map(&*params, map_row_to_inner_song)
            .map_err(error_helpers::to_database_error)?;

        let mut fetched = Vec::new();
        for r in rows {
            fetched.push(r.map_err(error_helpers::to_database_error)?);
        }
        info!("Fetched genre songs");
        Ok(fetched)
    }

    #[tracing::instrument(level = "debug", skip(self, conn))]
    pub fn get_playlist_songs(
        &self,
        options: Playlist,
        inclusive: bool,
        conn: &mut rusqlite::Connection,
    ) -> Result<Vec<InnerSong>> {
        trace!("Fetching playlist songs");
        let mut query = "SELECT a._id, a.path, a.size, a.inode, a.deviceno, a.title, a.date, a.year, a.lyrics, a.releasetype,
                                a.bitrate, a.codec, a.container, a.duration, a.samplerate, a.hash, a.type, a.url,
                                a.song_coverpath_high, a.playbackurl, a.song_coverpath_low, a.date_added,
                                a.provider_extension, a.icon, a.show_in_library, a.track_no, a.library_item
                         FROM allsongs a
                         JOIN playlist_bridge b ON a._id = b.song
                         JOIN playlists p ON b.playlist = p.playlist_id".to_string();
        let mut clauses = Vec::new();
        let mut params = Vec::new();

        if let Some(ref v) = options.playlist_id {
            clauses.push("p.playlist_id = ?".to_string());
            params.push(v as &dyn rusqlite::ToSql);
        }
        if !options.playlist_name.is_empty() {
            clauses.push("p.playlist_name LIKE ?".to_string());
            params.push(&options.playlist_name as &dyn rusqlite::ToSql);
        }
        if let Some(ref v) = options.playlist_path {
            clauses.push("p.playlist_path LIKE ?".to_string());
            params.push(v as &dyn rusqlite::ToSql);
        }

        if !clauses.is_empty() {
            let joiner = if inclusive { " AND " } else { " OR " };
            query.push_str(" WHERE ");
            query.push_str(&clauses.join(joiner));
        }

        let mut stmt = conn
            .prepare(&query)
            .map_err(error_helpers::to_database_error)?;
        let rows = stmt
            .query_map(&*params, map_row_to_inner_song)
            .map_err(error_helpers::to_database_error)?;

        let mut fetched = Vec::new();
        for r in rows {
            fetched.push(r.map_err(error_helpers::to_database_error)?);
        }
        info!("Fetched playlist songs");
        Ok(fetched)
    }

    #[tracing::instrument(level = "debug", skip(self))]
    pub fn get_songs_by_options(&self, options: GetSongOptions) -> Result<Vec<Song>> {
        let mut ret = vec![];
        trace!("Getting songs by options");
        let inclusive = options.inclusive.unwrap_or_default();

        let mut conn = self.pool.get().unwrap();
        let mut fetched_songs: Vec<InnerSong> = vec![];

        if let Some(song) = options.song {
            let mut query = "SELECT _id, path, size, inode, deviceno, title, date, year, lyrics, releasetype, bitrate, codec, container, duration, samplerate, hash, type, url, song_coverpath_high, playbackurl, song_coverpath_low, date_added, provider_extension, icon, show_in_library, track_no, library_item FROM allsongs".to_string();
            let mut clauses = Vec::new();
            let mut params = Vec::new();
            let song_type_str;

            if let Some(ref v) = song.id {
                clauses.push("_id = ?".to_string());
                params.push(v as &dyn rusqlite::ToSql);
            }
            if let Some(ref v) = song.path {
                clauses.push("path LIKE ?".to_string());
                params.push(v as &dyn rusqlite::ToSql);
            }
            if let Some(ref v) = song.title {
                clauses.push("title LIKE ?".to_string());
                params.push(v as &dyn rusqlite::ToSql);
            }
            if let Some(ref v) = song.sample_rate {
                clauses.push("samplerate = ?".to_string());
                params.push(v as &dyn rusqlite::ToSql);
            }
            if let Some(ref v) = song.hash {
                clauses.push("hash = ?".to_string());
                params.push(v as &dyn rusqlite::ToSql);
            }
            if let Some(v) = song.r#type {
                song_type_str = song_type_to_str(v);
                clauses.push("type = ?".to_string());
                params.push(&song_type_str as &dyn rusqlite::ToSql);
            }
            if let Some(ref v) = song.url {
                clauses.push("url LIKE ?".to_string());
                params.push(v as &dyn rusqlite::ToSql);
            }
            if let Some(ref v) = song.playback_url {
                clauses.push("playbackurl LIKE ?".to_string());
                params.push(v as &dyn rusqlite::ToSql);
            }
            if let Some(ref v) = song.provider_extension {
                clauses.push("provider_extension = ?".to_string());
                params.push(v as &dyn rusqlite::ToSql);
            }
            if let Some(ref v) = song.show_in_library {
                clauses.push("show_in_library = ?".to_string());
                params.push(v as &dyn rusqlite::ToSql);
            }

            if !clauses.is_empty() {
                let joiner = if inclusive { " AND " } else { " OR " };
                query.push_str(" WHERE ");
                query.push_str(&clauses.join(joiner));
            }

            let mut stmt = conn
                .prepare(&query)
                .map_err(error_helpers::to_database_error)?;
            let rows = stmt
                .query_map(&*params, map_row_to_inner_song)
                .map_err(error_helpers::to_database_error)?;

            let mut fetched = Vec::new();
            for r in rows {
                fetched.push(r.map_err(error_helpers::to_database_error)?);
            }
            fetched_songs = fetched;
        } else if let Some(album) = options.album {
            fetched_songs = self.get_album_songs(album, inclusive, &mut conn)?;
        } else if let Some(artist) = options.artist {
            fetched_songs = self.get_artist_songs(artist, inclusive, &mut conn)?;
        } else if let Some(genre) = options.genre {
            fetched_songs = self.get_genre_songs(genre, inclusive, &mut conn)?;
        } else if let Some(playlist) = options.playlist {
            fetched_songs = self.get_playlist_songs(playlist, inclusive, &mut conn)?;
        }

        if fetched_songs.is_empty() {
            return Ok(vec![]);
        }

        let song_ids: Vec<String> = fetched_songs.iter().filter_map(|s| s.id.clone()).collect();

        // 1. Fetch albums for all these songs in chunks of 500
        let mut albums_map: std::collections::HashMap<String, Album> =
            std::collections::HashMap::new();
        for chunk in song_ids.chunks(500) {
            let placeholders = vec!["?"; chunk.len()].join(",");
            let query = format!(
                "SELECT a.album_id, a.album_name, a.album_artist, a.album_coverpath_high, a.album_song_count, a.year, a.album_coverpath_low, b.song
                 FROM albums a
                 JOIN album_bridge b ON a.album_id = b.album
                 WHERE b.song IN ({})",
                placeholders
            );
            let mut stmt = conn
                .prepare(&query)
                .map_err(error_helpers::to_database_error)?;
            let params = chunk
                .iter()
                .map(|s| s as &dyn rusqlite::ToSql)
                .collect::<Vec<_>>();
            let rows = stmt
                .query_map(&*params, |row| {
                    let album = map_row_to_album(row)?;
                    let song_id: String = row.get(7)?;
                    Ok((song_id, album))
                })
                .map_err(error_helpers::to_database_error)?;
            for r in rows {
                let (song_id, album) = r.map_err(error_helpers::to_database_error)?;
                albums_map.insert(song_id, album);
            }
        }

        // 2. Fetch artists for all these songs in chunks of 500
        let mut artists_map: std::collections::HashMap<String, Vec<Artist>> =
            std::collections::HashMap::new();
        for chunk in song_ids.chunks(500) {
            let placeholders = vec!["?"; chunk.len()].join(",");
            let query = format!(
                "SELECT a.artist_id, a.artist_mbid, a.artist_name, a.artist_coverpath, a.artist_song_count, a.sanitized_artist_name, b.song
                 FROM artists a
                 JOIN artist_bridge b ON a.artist_id = b.artist
                 WHERE b.song IN ({})",
                placeholders
            );
            let mut stmt = conn
                .prepare(&query)
                .map_err(error_helpers::to_database_error)?;
            let params = chunk
                .iter()
                .map(|s| s as &dyn rusqlite::ToSql)
                .collect::<Vec<_>>();
            let rows = stmt
                .query_map(&*params, |row| {
                    let artist = map_row_to_artist(row)?;
                    let song_id: String = row.get(6)?;
                    Ok((song_id, artist))
                })
                .map_err(error_helpers::to_database_error)?;
            for r in rows {
                let (song_id, artist) = r.map_err(error_helpers::to_database_error)?;
                artists_map.entry(song_id).or_default().push(artist);
            }
        }

        // 3. Fetch genres for all these songs in chunks of 500
        let mut genres_map: std::collections::HashMap<String, Vec<Genre>> =
            std::collections::HashMap::new();
        for chunk in song_ids.chunks(500) {
            let placeholders = vec!["?"; chunk.len()].join(",");
            let query = format!(
                "SELECT a.genre_id, a.genre_name, a.genre_song_count, b.song
                 FROM genres a
                 JOIN genre_bridge b ON a.genre_id = b.genre
                 WHERE b.song IN ({})",
                placeholders
            );
            let mut stmt = conn
                .prepare(&query)
                .map_err(error_helpers::to_database_error)?;
            let params = chunk
                .iter()
                .map(|s| s as &dyn rusqlite::ToSql)
                .collect::<Vec<_>>();
            let rows = stmt
                .query_map(&*params, |row| {
                    let genre = map_row_to_genre(row)?;
                    let song_id: String = row.get(3)?;
                    Ok((song_id, genre))
                })
                .map_err(error_helpers::to_database_error)?;
            for r in rows {
                let (song_id, genre) = r.map_err(error_helpers::to_database_error)?;
                genres_map.entry(song_id).or_default().push(genre);
            }
        }

        for s in fetched_songs {
            let id = s.id.clone().unwrap_or_default();
            let album = albums_map.remove(&id);
            let artists = artists_map.remove(&id).unwrap_or_default();
            let genre = genres_map.remove(&id).unwrap_or_default();
            ret.push(Song {
                song: Some(s),
                album,
                artists,
                genre,
            });
        }
        Ok(ret)
    }

    #[tracing::instrument(level = "debug", skip(self))]
    pub fn search_all(&self, term: &str) -> Result<SearchResult> {
        trace!("Searching all by term");

        let term = format!("%{}%", term);
        let songs = self.get_songs_by_options(GetSongOptions {
            song: Some(SearchableSong {
                id: None,
                path: Some(term.clone()),
                title: Some(term.clone()),
                sample_rate: None,
                hash: None,
                r#type: None,
                url: None,
                playback_url: None,
                provider_extension: None,
                show_in_library: None,
            }),
            artist: None,
            album: None,
            genre: None,
            playlist: None,
            inclusive: Some(false),
        })?;

        let mut conn = self.pool.get().unwrap();
        let _albums = self.get_albums(
            Album {
                album_id: None,
                album_name: Some(term.clone()),
                album_artist: None,
                album_coverpath_high: None,
                album_song_count: 0f64,
                year: None,
                album_coverpath_low: None,
            },
            false,
            &mut conn,
        )?;

        let _artists = self.get_artists(
            Artist {
                artist_id: None,
                artist_mbid: None,
                artist_name: Some(term.clone()),
                artist_coverpath: None,
                artist_song_count: 0f64,
                sanitized_artist_name: None,
            },
            false,
            &mut conn,
        )?;

        let _genres = self.get_genres(
            Genre {
                genre_id: None,
                genre_name: Some(term.clone()),
                genre_song_count: 0f64,
            },
            false,
            &mut conn,
        )?;

        let _playlists = self.get_playlists(
            Playlist {
                playlist_id: None,
                playlist_name: term.clone(),
                playlist_coverpath: None,
                playlist_song_count: 0f64,
                playlist_desc: Some(term.clone()),
                playlist_path: Some(term.clone()),
                extension: None,
                icon: None,
                library_item: None,
                ..Default::default()
            },
            false,
            &mut conn,
        )?;

        info!("Searched all by term");

        Ok(SearchResult {
            songs,
            artists: _artists,
            playlists: _playlists,
            albums: _albums,
            genres: _genres,
        })
    }

    #[tracing::instrument(level = "debug", skip(self))]
    pub fn files_not_in_db(
        &self,
        mut file_list: Vec<(PathBuf, f64)>,
    ) -> Result<Vec<(PathBuf, f64)>> {
        let conn = self.pool.get().unwrap();

        let len = file_list.len();

        let mut ret = vec![];

        let exp_limit = 300;
        for _ in 0..len / exp_limit + 1 {
            let curr_len = min(file_list.len(), exp_limit);
            if curr_len == 0 {
                break;
            }
            let mut query = "SELECT path, size FROM allsongs".to_string();
            let mut clauses = Vec::new();

            let mut temp_paths = Vec::new();
            for _ in 0..curr_len {
                let data = file_list.pop().unwrap();
                let path_str = data.0.to_string_lossy().to_string();
                temp_paths.push((path_str, data.1));
            }

            let mut params = Vec::new();
            for i in 0..curr_len {
                clauses.push("(path = ? AND size = ?)".to_string());
                params.push(&temp_paths[i].0 as &dyn rusqlite::ToSql);
                params.push(&temp_paths[i].1 as &dyn rusqlite::ToSql);
            }

            query.push_str(" WHERE ");
            query.push_str(&clauses.join(" OR "));

            let mut stmt = conn
                .prepare(&query)
                .map_err(error_helpers::to_database_error)?;
            let rows = stmt
                .query_map(&*params, |row| {
                    let p: Option<String> = row.get(0)?;
                    let s: Option<f64> = row.get(1)?;
                    Ok((p, s))
                })
                .map_err(error_helpers::to_database_error)?;

            for r in rows {
                let (path_opt, size_opt) = r.map_err(error_helpers::to_database_error)?;
                if let (Some(p), Some(s)) = (path_opt, size_opt) {
                    ret.push((PathBuf::from(p), s));
                }
            }
        }
        Ok(ret)
    }

    #[tracing::instrument(level = "debug", skip(self))]
    pub fn add_to_playlist(&self, id: &str, songs: &[Song]) -> Result<()> {
        trace!("Adding to playlist");
        let conn = self.pool.get().unwrap();
        for s in songs {
            if s.song.is_none() {
                continue;
            }

            let sid = s.get_id().unwrap_or_default();

            if let Err(e) = conn.execute(
                "INSERT INTO playlist_bridge (playlist, song) VALUES (?1, ?2)",
                (&id, &sid),
            ) {
                warn!("Failed to add {:?} to playlist: {:?}", s, e);
            }
        }
        info!("Added to playlist");
        Ok(())
    }

    #[tracing::instrument(level = "debug", skip(self))]
    pub fn remove_from_playlist<T>(&self, id: &str, songs: &[T]) -> Result<()>
    where
        T: AsRef<str> + rusqlite::ToSql + Debug,
    {
        trace!("Removing from playlist");
        let conn = self.pool.get().unwrap();
        for sid in songs {
            conn.execute(
                "DELETE FROM playlist_bridge WHERE playlist = ?1 AND song = ?2",
                (&id, sid),
            )
            .map_err(error_helpers::to_database_error)?;
        }
        info!("Removed from playlist");
        Ok(())
    }

    #[tracing::instrument(level = "debug", skip(self))]
    pub fn remove_playlist(&self, id: &str) -> Result<()> {
        trace!("Removing playlist");
        let conn = self.pool.get().unwrap();
        conn.execute("DELETE FROM playlist_bridge WHERE playlist = ?1", [&id])
            .map_err(error_helpers::to_database_error)?;
        conn.execute("DELETE FROM playlists WHERE playlist_id = ?1", [&id])
            .map_err(error_helpers::to_database_error)?;

        info!("Removed playlist");
        Ok(())
    }

    #[tracing::instrument(level = "debug", skip(self))]
    pub fn update_album(&self, album: Album) -> Result<()> {
        trace!("Updating album");
        let conn = self.pool.get().unwrap();

        conn.execute(
            "UPDATE albums SET
                album_name = ?1, album_artist = ?2, album_coverpath_high = ?3,
                album_song_count = ?4, year = ?5, album_coverpath_low = ?6
             WHERE album_id = ?7",
            (
                &album.album_name,
                &album.album_artist,
                &album.album_coverpath_high,
                &album.album_song_count,
                &album.year,
                &album.album_coverpath_low,
                &album.album_id,
            ),
        )
        .map_err(error_helpers::to_database_error)?;

        info!("Updated album");
        Ok(())
    }

    #[tracing::instrument(level = "debug", skip(self))]
    pub fn update_artist(&self, artist: Artist) -> Result<()> {
        trace!("Updating artist");
        let conn = self.pool.get().unwrap();

        conn.execute(
            "UPDATE artists SET
                artist_mbid = ?1, artist_name = ?2, artist_coverpath = ?3,
                artist_song_count = ?4, sanitized_artist_name = ?5
             WHERE artist_id = ?6",
            (
                &artist.artist_mbid,
                &artist.artist_name,
                &artist.artist_coverpath,
                &artist.artist_song_count,
                &artist.sanitized_artist_name,
                &artist.artist_id,
            ),
        )
        .map_err(error_helpers::to_database_error)?;
        info!("Updated artist");
        Ok(())
    }

    #[tracing::instrument(level = "debug", skip(self))]
    pub fn update_playlist(&self, playlist: Playlist) -> Result<()> {
        trace!("Updating playlist");
        let conn = self.pool.get().unwrap();
        conn.execute(
            "UPDATE playlists SET
                playlist_name = ?1, playlist_coverpath = ?2, playlist_song_count = ?3,
                playlist_desc = ?4, playlist_path = ?5, extension = ?6, icon = ?7, library_item = ?8
             WHERE playlist_id = ?9",
            (
                &playlist.playlist_name,
                &playlist.playlist_coverpath,
                &playlist.playlist_song_count,
                &playlist.playlist_desc,
                &playlist.playlist_path,
                &playlist.extension,
                &playlist.icon,
                &playlist.library_item,
                &playlist.playlist_id,
            ),
        )
        .map_err(error_helpers::to_database_error)?;
        info!("Updated playlist");
        Ok(())
    }

    #[tracing::instrument(level = "debug", skip(self))]
    pub fn update_songs(&self, songs: Vec<Song>) -> Result<()> {
        trace!("Updating songs");
        let conn = self.pool.get().unwrap();

        for song in songs {
            if let Some(inner_song) = song.song {
                if let Some(album) = song.album {
                    self.update_album(album)?;
                }

                for a in song.artists {
                    self.update_artist(a)?;
                }

                let song_type = song_type_to_str(inner_song.r#type);
                let params: &[&dyn rusqlite::ToSql] = &[
                    &inner_song.path,
                    &inner_song.size,
                    &inner_song.inode,
                    &inner_song.deviceno,
                    &inner_song.title,
                    &inner_song.date,
                    &inner_song.year,
                    &inner_song.lyrics,
                    &inner_song.release_type,
                    &inner_song.bitrate,
                    &inner_song.codec,
                    &inner_song.container,
                    &proto_to_db_ms(&inner_song.duration),
                    &inner_song.sample_rate,
                    &inner_song.hash,
                    &song_type,
                    &inner_song.url,
                    &inner_song.song_cover_path_high,
                    &inner_song.playback_url,
                    &inner_song.song_cover_path_low,
                    &inner_song.date_added,
                    &inner_song.provider_extension,
                    &inner_song.icon,
                    &inner_song.show_in_library,
                    &inner_song.track_no,
                    &inner_song.library_item,
                    &inner_song.id,
                ];

                conn.execute(
                    "UPDATE allsongs SET
                        path = ?1, size = ?2, inode = ?3, deviceno = ?4, title = ?5, date = ?6,
                        year = ?7, lyrics = ?8, releasetype = ?9, bitrate = ?10, codec = ?11,
                        container = ?12, duration = ?13, samplerate = ?14, hash = ?15, type = ?16,
                        url = ?17, song_coverpath_high = ?18, playbackurl = ?19, song_coverpath_low = ?20,
                        date_added = ?21, provider_extension = ?22, icon = ?23, show_in_library = ?24,
                        track_no = ?25, library_item = ?26
                     WHERE _id = ?27",
                    params,
                ).map_err(error_helpers::to_database_error)?;
            }
        }
        info!("Updated songs");
        Ok(())
    }

    #[tracing::instrument(level = "debug", skip(self))]
    pub fn update_lyrics(&self, id: String, lyrics: String) -> Result<()> {
        trace!("Updating lyrics");
        let conn = self.pool.get().unwrap();
        conn.execute(
            "UPDATE allsongs SET lyrics = ?1 WHERE _id = ?2",
            (&lyrics, &id),
        )
        .map_err(error_helpers::to_database_error)?;
        info!("Updated lyrics");
        Ok(())
    }

    #[tracing::instrument(level = "debug", skip(self))]
    pub fn increment_play_count(&self, id: &str) -> Result<()> {
        trace!("Incrementing play count");
        let conn = self.pool.get().unwrap();
        let play_count_res: std::result::Result<Option<i32>, rusqlite::Error> = conn.query_row(
            "SELECT play_count FROM analytics WHERE song_id = ?1",
            [&id],
            |row| row.get(0),
        );

        if play_count_res.is_err() {
            conn.execute(
                "INSERT INTO analytics (id, song_id, play_count, play_time) VALUES (?1, ?2, ?3, ?4)",
                (
                    &Uuid::new_v4().to_string(),
                    &id,
                    &Some(1),
                    &Some(0f64),
                ),
            ).map_err(error_helpers::to_database_error)?;
            return Ok(());
        }

        conn.execute(
            "UPDATE analytics SET play_count = COALESCE(play_count, 0) + 1 WHERE song_id = ?1",
            [&id],
        )
        .map_err(error_helpers::to_database_error)?;

        info!("Incremented play count");
        Ok(())
    }

    #[tracing::instrument(level = "debug", skip(self))]
    pub fn increment_play_time(&self, id: &str, duration: f64) -> Result<()> {
        trace!("Incrementing play time");
        let conn = self.pool.get().unwrap();
        let play_time_res: std::result::Result<Option<f64>, rusqlite::Error> = conn.query_row(
            "SELECT play_time FROM analytics WHERE song_id = ?1",
            [&id],
            |row| row.get(0),
        );

        if play_time_res.is_err() {
            conn.execute(
                "INSERT INTO analytics (id, song_id, play_count, play_time) VALUES (?1, ?2, ?3, ?4)",
                (
                    &Uuid::new_v4().to_string(),
                    &id,
                    &Some(0),
                    &duration,
                ),
            ).map_err(error_helpers::to_database_error)?;
            info!("Added new play time");
            return Ok(());
        }

        conn.execute(
            "UPDATE analytics SET play_time = COALESCE(play_time, 0.0) + ?1 WHERE song_id = ?2",
            (duration, &id),
        )
        .map_err(error_helpers::to_database_error)?;

        info!("Incremented playtime");

        Ok(())
    }

    pub fn get_top_listened_songs(&self) -> Result<AllAnalytics> {
        let conn = self.pool.get().unwrap();
        let mut stmt = conn
            .prepare("SELECT song_id, play_time FROM analytics ORDER BY play_time DESC LIMIT 10")
            .map_err(error_helpers::to_database_error)?;

        let song_rows = stmt
            .query_map([], |row| {
                let song_id: Option<String> = row.get(0)?;
                let play_time: Option<f64> = row.get(1)?;
                Ok((song_id, play_time))
            })
            .map_err(error_helpers::to_database_error)?;

        let mut songs = Vec::new();
        for r in song_rows {
            let (s_id, time) = r.map_err(error_helpers::to_database_error)?;
            if let Some(s) = s_id {
                songs.push(SongListenTime {
                    song_id: s,
                    time: time.unwrap_or_default(),
                });
            }
        }

        let total_listen_time: Option<f64> = conn
            .query_row("SELECT SUM(play_time) FROM analytics", [], |row| row.get(0))
            .unwrap_or(Some(0.0));

        Ok(AllAnalytics {
            total_listen_time: total_listen_time.unwrap_or_default(),
            songs,
        })
    }

    #[tracing::instrument(level = "debug", skip(self))]
    pub fn export_playlist(&self, playlist_id: &str) -> Result<String> {
        let mut conn = self.pool.get().unwrap();

        let binding = self.get_playlists(
            Playlist {
                playlist_id: Some(playlist_id.to_string()),
                ..Default::default()
            },
            true,
            &mut conn,
        )?;
        let playlist = binding.first();

        if playlist.is_none() {
            return Err("Playlist not found".into());
        }

        let playlist = playlist.unwrap();

        let playlist_songs = self.get_songs_by_options(GetSongOptions {
            playlist: Some(Playlist {
                playlist_id: Some(playlist_id.to_string()),
                ..Default::default()
            }),
            ..Default::default()
        })?;

        let mut ret = format!("#EXTM3U\n#PLAYLIST:{}\n", playlist.playlist_name);

        for s in playlist_songs {
            if let Some(inner_song) = s.song {
                if let Some(path) = &inner_song.path {
                    let duration = inner_song.duration.unwrap_or_default();
                    let title = inner_song.title.unwrap_or_default();
                    let album_info = s.album.as_ref().map_or(String::new(), |album| {
                        format!("#EXTALB:{}", album.album_name.clone().unwrap_or_default())
                    });
                    let genre_info = if !s.genre.is_empty() {
                        format!(
                            "#EXTGENRE:{}",
                            s.genre
                                .iter()
                                .filter_map(|g| g.genre_name.clone())
                                .collect::<Vec<String>>()
                                .join(",")
                        )
                    } else {
                        String::new()
                    };
                    let cover_path = match inner_song.song_cover_path_high {
                        Some(cover) => format!("#EXTIMG:{}", cover),
                        None => String::new(),
                    };
                    let song_info = format!("#MOOSINF:{}", inner_song.r#type);
                    let file_path = format!("file://{}", path);

                    write!(
                        ret,
                        "#EXTINF:{},{}\n{}\n{}\n{}\n{}\n{}\n",
                        duration.seconds,
                        title,
                        album_info,
                        genre_info,
                        cover_path,
                        song_info,
                        file_path
                    )?;
                } else if let Some(url) = &inner_song.url {
                    let duration = inner_song.duration.unwrap_or_default();
                    let title = inner_song.title.unwrap_or_default();
                    let album_info = s.album.as_ref().map_or(String::new(), |album| {
                        format!("#EXTALB:{}", album.album_name.clone().unwrap_or_default())
                    });
                    let genre_info = if !s.genre.is_empty() {
                        format!(
                            "#EXTGENRE:{}",
                            s.genre
                                .iter()
                                .filter_map(|g| g.genre_name.clone())
                                .collect::<Vec<String>>()
                                .join(",")
                        )
                    } else {
                        String::new()
                    };
                    let cover_path = match inner_song.song_cover_path_high {
                        Some(cover) => format!("#EXTIMG:{}", cover),
                        None => String::new(),
                    };
                    let song_info = format!("#MOOSINF:{}", inner_song.r#type);

                    write!(
                        ret,
                        "#EXTINF:{},{}\n{}\n{}\n{}\n{}\n{}\n",
                        duration.seconds, title, album_info, genre_info, cover_path, song_info, url
                    )?;
                }
            }
        }

        Ok(ret.replace("\n\n", "\n"))
    }
}

impl types::plugin::Plugin for Database {
    fn init(
        context: &types::plugin::PluginContext,
    ) -> types::plugin::Arc<types::plugin::RwLock<Self>> {
        types::plugin::Arc::new(types::plugin::RwLock::new(Database::new(
            context.data_dir.clone(),
        )))
    }
}
