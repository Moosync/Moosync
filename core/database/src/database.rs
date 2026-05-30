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

use std::cmp::min;
use std::fmt::Write;
use std::fs;
use std::{path::PathBuf, vec};


use tracing::{debug, info, trace, warn};
use uuid::Uuid;

use songs_proto::moosync::types::SearchResult;
use songs_proto::moosync::types::{Album, AlbumList, Artist, ArtistList, EntityResult, Genre, GenreList, GetEntityOptions, Playlist, PlaylistList};
use songs_proto::moosync::types::{AllAnalytics, SearchableSong};
use songs_proto::moosync::types::{
    GetSongOptions, InnerSong, Song, all_analytics::SongListenTime,
};
use types::errors::{Result, error_helpers};
use types::prelude::SongsExt;

use crate::utils::{
    map_row_to_album, map_row_to_artist, map_row_to_genre, map_row_to_inner_song,
    map_row_to_playlist, song_type_to_str, SearchByTerm,
};

use super::migrations::run_migrations;

use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct Database {
    pool: r2d2::Pool<r2d2_sqlite::SqliteConnectionManager>,
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
        let manager = r2d2_sqlite::SqliteConnectionManager::file(path)
            .with_init(|conn| {
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

    fn get_bridge_songs(
        &self,
        bridge_table: &str,
        id_column: &str,
        entity_id: &str,
        conn: &mut rusqlite::Connection,
    ) -> Result<Vec<InnerSong>> {
        let query = format!(
            "SELECT a._id, a.path, a.size, a.inode, a.deviceno, a.title, a.date, a.year, a.lyrics, a.releasetype,
                    a.bitrate, a.codec, a.container, a.duration, a.samplerate, a.hash, a.type, a.url,
                    a.song_coverpath_high, a.playbackurl, a.song_coverpath_low, a.date_added,
                    a.provider_extension, a.icon, a.show_in_library, a.track_no, a.library_item
             FROM allsongs a
             JOIN {} b ON a._id = b.song
             WHERE b.{} = ?1",
            bridge_table, id_column
        );
        let mut stmt = conn.prepare(&query).map_err(error_helpers::to_database_error)?;
        let rows = stmt
            .query_map([entity_id], map_row_to_inner_song)
            .map_err(error_helpers::to_database_error)?;

        let mut fetched = Vec::new();
        for r in rows {
            fetched.push(r.map_err(error_helpers::to_database_error)?);
        }
        Ok(fetched)
    }

    #[tracing::instrument(level = "debug", skip(self, conn))]
    fn insert_album(
        &self,
        conn: &mut rusqlite::Connection,
        album: Album,
    ) -> Result<String> {
        let id = Uuid::new_v4().to_string();
        trace!("Inserting album");
        conn.execute(
            "INSERT INTO albums (album_id, album_name, album_artist, album_coverpath_high, album_song_count, year, album_coverpath_low, album_extra_info)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            (
                &Some(id.clone()),
                &album.album_name,
                &album.album_artist,
                &album.album_coverpath_high,
                &album.album_song_count,
                &album.year,
                &album.album_coverpath_low,
                &None::<String>,
            ),
        )
        .map_err(error_helpers::to_database_error)?;
        info!("Inserted album");
        Ok(id)
    }

    #[tracing::instrument(level = "debug", skip(self, conn))]
    fn insert_artist(
        &self,
        conn: &mut rusqlite::Connection,
        artist: Artist,
    ) -> Result<String> {
        let id = Uuid::new_v4().to_string();
        trace!("Inserting artist");
        conn.execute(
            "INSERT INTO artists (artist_id, artist_mbid, artist_name, artist_coverpath, artist_song_count, artist_extra_info, sanitized_artist_name)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            (
                &Some(id.clone()),
                &artist.artist_mbid,
                &artist.artist_name,
                &artist.artist_coverpath,
                &artist.artist_song_count,
                &None::<String>,
                &artist.sanitized_artist_name,
            ),
        )
        .map_err(error_helpers::to_database_error)?;
        info!("Inserted artist");
        Ok(id)
    }

    #[tracing::instrument(level = "debug", skip(self, conn))]
    fn insert_genre(
        &self,
        conn: &mut rusqlite::Connection,
        genre: Genre,
    ) -> Result<String> {
        let id = Uuid::new_v4().to_string();
        trace!("Inserting genre");
        conn.execute(
            "INSERT INTO genres (genre_id, genre_name, genre_song_count) VALUES (?1, ?2, ?3)",
            (
                &Some(id.clone()),
                &genre.genre_name,
                &genre.genre_song_count,
            ),
        )
        .map_err(error_helpers::to_database_error)?;
        info!("Inserted genre");
        Ok(id)
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
        for song in songs {
            if let Some(inner_song) = song.song.as_mut() {
                if inner_song.id.is_none() {
                    inner_song.id = Some(Uuid::new_v4().to_string());
                }

                let song_type = song_type_to_str(inner_song.r#type);
                let params: &[&dyn rusqlite::ToSql] = &[
                    &inner_song.id,
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
                    &inner_song.duration,
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
                ];

                let changed = conn.execute(
                    "INSERT INTO allsongs (
                        _id, path, size, inode, deviceno, title, date, year, lyrics, releasetype,
                        bitrate, codec, container, duration, samplerate, hash, type, url,
                        song_coverpath_high, playbackurl, song_coverpath_low, date_added,
                        provider_extension, icon, show_in_library, track_no, library_item
                     ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18, ?19, ?20, ?21, ?22, ?23, ?24, ?25, ?26, ?27)
                     ON CONFLICT(path) DO UPDATE SET
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
                    params,
                ).map_err(error_helpers::to_database_error)?;

                if changed == 0 {
                    continue;
                }

                if let Some(_album) = &mut song.album {
                    let album_id_ = self
                        .get_albums(
                            Album::search_by_term(_album.album_name.clone()),
                            false,
                            &mut conn,
                        )?
                        .first()
                        .map(|v| v.album_id.clone().unwrap())
                        .unwrap_or_else(|| self.insert_album(&mut conn, _album.clone()).unwrap());

                    conn.execute(
                        "INSERT INTO album_bridge (song, album) VALUES (?1, ?2) ON CONFLICT DO NOTHING",
                        (&inner_song.id.clone().unwrap(), &album_id_),
                    ).map_err(error_helpers::to_database_error)?;

                    _album.album_id = Some(album_id_);
                }

                for mut _artist in song.artists.iter_mut() {
                    let artist_id_ = self
                        .get_artists(
                            Artist::search_by_term(_artist.artist_name.clone()),
                            false,
                            &mut conn,
                        )?
                        .first()
                        .map(|v| v.artist_id.clone().unwrap())
                        .unwrap_or_else(|| self.insert_artist(&mut conn, _artist.clone()).unwrap());

                    conn.execute(
                        "INSERT INTO artist_bridge (song, artist) VALUES (?1, ?2) ON CONFLICT DO NOTHING",
                        (&inner_song.id.clone().unwrap(), &artist_id_),
                    ).map_err(error_helpers::to_database_error)?;

                    _artist.artist_id = Some(artist_id_);
                }

                for mut _genre in song.genre.iter_mut() {
                    let genre_id_ = self
                        .get_genres(
                            Genre::search_by_term(_genre.genre_name.clone()),
                            false,
                            &mut conn,
                        )?
                        .first()
                        .map(|v| v.genre_id.clone().unwrap())
                        .unwrap_or_else(|| self.insert_genre(&mut conn, _genre.clone()).unwrap());

                    conn.execute(
                        "INSERT INTO genre_bridge (song, genre) VALUES (?1, ?2) ON CONFLICT DO NOTHING",
                        (&inner_song.id.clone().unwrap(), &genre_id_),
                    ).map_err(error_helpers::to_database_error)?;

                    _genre.genre_id = Some(genre_id_);
                }

                trace!("Inserted song, {:?}", song);
            }
        }
        info!("Inserted all songs");
        Ok(())
    }

    #[tracing::instrument(level = "debug", skip(self))]
    pub fn remove_songs(&self, ids: Vec<String>) -> Result<()> {
        trace!("Removing song");
        let mut conn = self.pool.get().unwrap();
        let tx = conn.transaction().map_err(error_helpers::to_database_error)?;
        for id in ids {
            tx.execute("DELETE FROM analytics WHERE song_id = ?1", [&id])
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
    pub fn update_song(&self, song: InnerSong) -> Result<()> {
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
                &song.duration,
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
        let mut stmt = conn.prepare(&query).map_err(error_helpers::to_database_error)?;
        let rows = stmt.query_map(&*params, map_row_to_album).map_err(error_helpers::to_database_error)?;

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
        let mut stmt = conn.prepare(&query).map_err(error_helpers::to_database_error)?;
        let rows = stmt.query_map(&*params, map_row_to_artist).map_err(error_helpers::to_database_error)?;

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
        let mut stmt = conn.prepare(&query).map_err(error_helpers::to_database_error)?;
        let rows = stmt.query_map(&*params, map_row_to_genre).map_err(error_helpers::to_database_error)?;

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
        let mut stmt = conn.prepare(&query).map_err(error_helpers::to_database_error)?;
        let rows = stmt.query_map(&*params, map_row_to_playlist).map_err(error_helpers::to_database_error)?;

        let mut fetched = Vec::new();
        for r in rows {
            fetched.push(r.map_err(error_helpers::to_database_error)?);
        }
        Ok(fetched)
    }

    pub fn is_song_in_playlist(&self, playlist_id: String, song_id: String) -> Result<bool> {
        let conn = self.pool.get().unwrap();
        let count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM playlist_bridge WHERE playlist = ?1 AND song = ?2",
            (playlist_id, song_id),
            |row| row.get(0),
        ).map_err(error_helpers::to_database_error)?;
        Ok(count > 0)
    }

    #[tracing::instrument(level = "debug", skip(self))]
    pub fn get_entity_by_options(&self, options: GetEntityOptions) -> Result<EntityResult> {
        let mut conn = self.pool.get().unwrap();
        let inclusive = options.inclusive.unwrap_or_default();

        trace!("Getting entity by options");

        if let Some(album) = options.album {
            return Ok(EntityResult {
                result: Some(songs_proto::moosync::types::entity_result::Result::Albums(AlbumList {
                    albums: self.get_albums(album, inclusive, &mut conn)?,
                })),
            });
        }

        if let Some(artist) = options.artist {
            return Ok(EntityResult {
                result: Some(songs_proto::moosync::types::entity_result::Result::Artists(ArtistList {
                    artists: self.get_artists(artist, inclusive, &mut conn)?,
                })),
            });
        }

        if let Some(genre) = options.genre {
            return Ok(EntityResult {
                result: Some(songs_proto::moosync::types::entity_result::Result::Genres(GenreList {
                    genres: self.get_genres(genre, inclusive, &mut conn)?,
                })),
            });
        }

        if let Some(playlist) = options.playlist {
            return Ok(EntityResult {
                result: Some(songs_proto::moosync::types::entity_result::Result::Playlists(PlaylistList {
                    playlists: self.get_playlists(playlist, inclusive, &mut conn)?,
                })),
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
        let binding = self.get_albums(options, inclusive, conn)?;
        let album = binding.first();
        if album.is_none() {
            return Ok(vec![]);
        }
        let album = album.unwrap();
        let album_id = album.album_id.as_deref().unwrap_or_default();
        let fetched = self.get_bridge_songs("album_bridge", "album", album_id, conn)?;
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
        let binding = self.get_artists(options, inclusive, conn)?;
        let artist = binding.first();
        if artist.is_none() {
            return Ok(vec![]);
        }
        let artist = artist.unwrap();
        let artist_id = artist.artist_id.as_deref().unwrap_or_default();
        let fetched = self.get_bridge_songs("artist_bridge", "artist", artist_id, conn)?;
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
        let binding = self.get_genres(options, inclusive, conn)?;
        let genre = binding.first();
        if genre.is_none() {
            return Ok(vec![]);
        }
        let genre = genre.unwrap();
        let genre_id = genre.genre_id.as_deref().unwrap_or_default();
        let fetched = self.get_bridge_songs("genre_bridge", "genre", genre_id, conn)?;
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
        let binding = self.get_playlists(options, inclusive, conn)?;
        trace!("Fetching playlist songs");
        let playlist = binding.first();
        if playlist.is_none() {
            return Ok(vec![]);
        }
        let playlist = playlist.unwrap();
        let playlist_id = playlist.playlist_id.as_deref().unwrap_or_default();
        let fetched = self.get_bridge_songs("playlist_bridge", "playlist", playlist_id, conn)?;
        info!("Fetched playlist songs");
        Ok(fetched)
    }

    fn get_song_from_queryable(
        &self,
        conn: &mut rusqlite::Connection,
        s: InnerSong,
    ) -> Result<Song> {
        let mut album: Option<Album> = None;
        let mut artist: Vec<Artist> = vec![];
        let mut genre: Vec<Genre> = vec![];

        let album_row = conn.query_row(
            "SELECT a.album_id, a.album_name, a.album_artist, a.album_coverpath_high, a.album_song_count, a.year, a.album_coverpath_low
             FROM albums a
             JOIN album_bridge b ON a.album_id = b.album
             WHERE b.song = ?1",
            [&s.id],
            map_row_to_album,
        );

        if let Ok(alb) = album_row {
            album = Some(alb);
        }

        let mut stmt_artists = conn.prepare(
            "SELECT a.artist_id, a.artist_mbid, a.artist_name, a.artist_coverpath, a.artist_song_count, a.sanitized_artist_name
             FROM artists a
             JOIN artist_bridge b ON a.artist_id = b.artist
             WHERE b.song = ?1"
        ).map_err(error_helpers::to_database_error)?;
        let artist_rows = stmt_artists.query_map([&s.id], map_row_to_artist).map_err(error_helpers::to_database_error)?;
        for r in artist_rows {
            artist.push(r.map_err(error_helpers::to_database_error)?);
        }

        let mut stmt_genres = conn.prepare(
            "SELECT a.genre_id, a.genre_name, a.genre_song_count
             FROM genres a
             JOIN genre_bridge b ON a.genre_id = b.genre
             WHERE b.song = ?1"
        ).map_err(error_helpers::to_database_error)?;
        let genre_rows = stmt_genres.query_map([&s.id], map_row_to_genre).map_err(error_helpers::to_database_error)?;
        for r in genre_rows {
            genre.push(r.map_err(error_helpers::to_database_error)?);
        }

        Ok(Song {
            song: Some(s),
            album,
            artists: artist,
            genre,
        })
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

            let mut stmt = conn.prepare(&query).map_err(error_helpers::to_database_error)?;
            let rows = stmt.query_map(&*params, map_row_to_inner_song).map_err(error_helpers::to_database_error)?;

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

        for s in fetched_songs {
            ret.push(self.get_song_from_queryable(&mut conn, s)?);
        }
        Ok(ret)
    }

    #[tracing::instrument(level = "debug", skip(self))]
    pub fn search_all(&self, term: String) -> Result<SearchResult> {
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

            let mut stmt = conn.prepare(&query).map_err(error_helpers::to_database_error)?;
            let rows = stmt.query_map(&*params, |row| {
                let p: Option<String> = row.get(0)?;
                let s: Option<f64> = row.get(1)?;
                Ok((p, s))
            }).map_err(error_helpers::to_database_error)?;

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
    pub fn add_to_playlist(&self, id: String, songs: Vec<Song>) -> Result<()> {
        trace!("Adding to playlist");
        let conn = self.pool.get().unwrap();
        for s in songs {
            if s.song.is_none() {
                continue;
            }

            if let Err(e) = conn.execute(
                "INSERT INTO playlist_bridge (playlist, song) VALUES (?1, ?2)",
                (&id, &s.get_id().unwrap_or_default()),
            ) {
                warn!("Failed to add {:?} to playlist: {:?}", s, e);
            }
        }
        info!("Added to playlist");
        Ok(())
    }

    #[tracing::instrument(level = "debug", skip(self))]
    pub fn remove_from_playlist(&self, id: String, songs: Vec<String>) -> Result<()> {
        trace!("Removing from playlist");
        let conn = self.pool.get().unwrap();
        for s in songs {
            conn.execute(
                "DELETE FROM playlist_bridge WHERE playlist = ?1 AND song = ?2",
                (&id, &s),
            ).map_err(error_helpers::to_database_error)?;
        }
        info!("Removed from playlist");
        Ok(())
    }

    #[tracing::instrument(level = "debug", skip(self))]
    pub fn remove_playlist(&self, id: String) -> Result<()> {
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
            )
        ).map_err(error_helpers::to_database_error)?;

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
            )
        ).map_err(error_helpers::to_database_error)?;
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
            )
        ).map_err(error_helpers::to_database_error)?;
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
                    &inner_song.duration,
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
        ).map_err(error_helpers::to_database_error)?;
        info!("Updated lyrics");
        Ok(())
    }

    #[tracing::instrument(level = "debug", skip(self))]
    pub fn increment_play_count(&self, id: String) -> Result<()> {
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
        ).map_err(error_helpers::to_database_error)?;

        info!("Incremented play count");
        Ok(())
    }

    #[tracing::instrument(level = "debug", skip(self))]
    pub fn increment_play_time(&self, id: String, duration: f64) -> Result<()> {
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
        ).map_err(error_helpers::to_database_error)?;

        info!("Incremented playtime");

        Ok(())
    }

    pub fn get_top_listened_songs(&self) -> Result<AllAnalytics> {
        let conn = self.pool.get().unwrap();
        let mut stmt = conn.prepare(
            "SELECT song_id, play_time FROM analytics ORDER BY play_time DESC LIMIT 10"
        ).map_err(error_helpers::to_database_error)?;
        
        let song_rows = stmt.query_map([], |row| {
            let song_id: Option<String> = row.get(0)?;
            let play_time: Option<f64> = row.get(1)?;
            Ok((song_id, play_time))
        }).map_err(error_helpers::to_database_error)?;

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

        let total_listen_time: Option<f64> = conn.query_row(
            "SELECT SUM(play_time) FROM analytics",
            [],
            |row| row.get(0),
        ).unwrap_or(Some(0.0));

        Ok(AllAnalytics {
            total_listen_time: total_listen_time.unwrap_or_default(),
            songs,
        })
    }

    #[tracing::instrument(level = "debug", skip(self))]
    pub fn export_playlist(&self, playlist_id: String) -> Result<String> {
        let mut conn = self.pool.get().unwrap();

        let binding = self.get_playlists(
            Playlist {
                playlist_id: Some(playlist_id.clone()),
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
                playlist_id: Some(playlist_id),
                ..Default::default()
            }),
            ..Default::default()
        })?;

        let mut ret = format!("#EXTM3U\n#PLAYLIST:{}\n", playlist.playlist_name);

        for s in playlist_songs {
            if let Some(inner_song) = s.song {
                if let Some(path) = &inner_song.path {
                    let duration = inner_song.duration.unwrap_or(0f64);
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
                        duration, title, album_info, genre_info, cover_path, song_info, file_path
                    )?;
                } else if let Some(url) = &inner_song.url {
                    let duration = inner_song.duration.unwrap_or(0f64);
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
                        duration, title, album_info, genre_info, cover_path, song_info, url
                    )?;
                }
            }
        }

        Ok(ret.replace("\n\n", "\n"))
    }
}

impl types::plugin::Plugin for Database {
    fn init(context: &types::plugin::PluginContext) -> Self {
        Database::new(context.data_dir.clone())
    }
}

