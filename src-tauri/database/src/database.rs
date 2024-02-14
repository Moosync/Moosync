use std::{path::PathBuf, vec};

use diesel::{
    connection::SimpleConnection,
    delete, insert_into,
    r2d2::{self, ConnectionManager, Pool, PooledConnection},
    update, Connection, ExpressionMethods, QueryDsl, RunQueryDsl, SqliteConnection,
};
use diesel::{TextExpressionMethods};
use macros::{filter_field, filter_field_like};
use serde_json::Value;

use crate::types::songs::SearchableSong;
use crate::{
    schema::{
        self,
        album_bridge::dsl::album_bridge,
        albums::{album_id, dsl::albums},
    },
    types::{
        entities::{
            AlbumBridge, ArtistBridge, GenreBridge, GetEntityOptions, QueryableAlbum,
            QueryableArtist, QueryableGenre, QueryablePlaylist, SearchResult,
        },
        songs::{GetSongOptions, QueryableSong, Song},
    },
};
use types::errors::errors::{MoosyncError, Result};

use super::{
    migrations::run_migrations,
    schema::{
        allsongs::{_id, dsl::allsongs},
        artist_bridge::dsl::artist_bridge,
        artists::{artist_id, dsl::artists},
        genre_bridge::dsl::genre_bridge,
        genres::{dsl::genres, genre_id},
        playlist_bridge::dsl::playlist_bridge,
    },
};

#[derive(Debug, Clone)]
pub struct Database {
    pool: Pool<ConnectionManager<SqliteConnection>>,
}

impl Database {
    pub fn new(path: PathBuf) -> Self {
        let db = Self {
            pool: Self::connect(path),
        };

        run_migrations(&mut db.pool.get().expect("Failed to get connection to DB"));
        db.pool.get().unwrap().batch_execute("
            PRAGMA journal_mode = WAL;          -- better write-concurrency
            PRAGMA synchronous = NORMAL;        -- fsync only in critical moments
            PRAGMA wal_autocheckpoint = 1000;   -- write WAL changes back every 1000 pages, for an in average 1MB WAL file. May affect readers if number is increased
            PRAGMA wal_checkpoint(TRUNCATE);    -- free some space by truncating possibly massive WAL files from the last run.
            PRAGMA busy_timeout = 250;          -- sleep if the database is busy
        ").expect("Failed to set DB options");
        db
    }

    fn connect(path: PathBuf) -> Pool<ConnectionManager<SqliteConnection>> {
        let manager = ConnectionManager::<SqliteConnection>::new(path.to_str().unwrap());
        let pool = r2d2::Pool::builder()
            .build(manager)
            .expect("Failed to create pool.");
        pool
    }

    pub fn insert_songs(&self, songs: Vec<Song>) -> Result<()> {
        let mut conn = self.pool.get().unwrap();
        for song in songs {
            insert_into(allsongs)
                .values(&song.song)
                .execute(&mut conn)?;

            if song.album.is_some() {
                let album_ = song.album.unwrap();
                insert_into(albums).values(&album_).execute(&mut conn)?;
                insert_into(album_bridge)
                    .values(&AlbumBridge {
                        id: None,
                        song: song.song._id,
                        album: album_.album_id,
                    })
                    .execute(&mut conn)?;
            }
        }
        Ok(())
    }

    // TODO: Remove album
    pub fn remove_songs(&self, ids: Vec<String>) -> Result<()> {
        self.pool
            .get()
            .unwrap()
            .transaction::<(), MoosyncError, _>(|conn| {
                for id in ids {
                    delete(QueryDsl::filter(allsongs, _id.eq(id.clone()))).execute(conn)?;
                    delete(QueryDsl::filter(
                        album_bridge,
                        schema::album_bridge::song.eq(id.clone()),
                    ))
                    .execute(conn)?;
                    delete(QueryDsl::filter(
                        artist_bridge,
                        schema::artist_bridge::song.eq(id.clone()),
                    ))
                    .execute(conn)?;
                    delete(QueryDsl::filter(
                        genre_bridge,
                        schema::genre_bridge::song.eq(id.clone()),
                    ))
                    .execute(conn)?;
                    delete(QueryDsl::filter(
                        playlist_bridge,
                        schema::playlist_bridge::song.eq(id.clone()),
                    ))
                    .execute(conn)?;
                }
                Ok(())
            })?;

        Ok(())
    }

    pub fn update_song(&self, song: QueryableSong) -> Result<()> {
        update(allsongs)
            .set(&song)
            .execute(&mut self.pool.get().unwrap())?;
        Ok(())
    }

    fn get_albums(
        &self,
        options: QueryableAlbum,
        inclusive: bool,
        conn: &mut PooledConnection<ConnectionManager<SqliteConnection>>,
    ) -> Result<Vec<QueryableAlbum>> {
        let mut predicate = schema::albums::table.into_boxed();

        predicate = filter_field!(
            predicate,
            &options.album_id,
            schema::albums::album_id,
            inclusive
        );

        predicate = filter_field_like!(
            predicate,
            &options.album_name,
            schema::albums::album_name,
            inclusive
        );

        let fetched: Vec<QueryableAlbum> = predicate.load(conn)?;
        Ok(fetched)
    }

    fn get_artists(
        &self,
        options: QueryableArtist,
        inclusive: bool,
        conn: &mut PooledConnection<ConnectionManager<SqliteConnection>>,
    ) -> Result<Vec<QueryableArtist>> {
        let mut predicate = schema::artists::table.into_boxed();

        predicate = filter_field!(
            predicate,
            &options.artist_id,
            schema::artists::artist_id,
            inclusive
        );

        predicate = filter_field_like!(
            predicate,
            &options.artist_name,
            schema::artists::artist_name,
            inclusive
        );

        predicate = filter_field!(
            predicate,
            &options.artist_mbid,
            schema::artists::artist_mbid,
            inclusive
        );

        let fetched: Vec<QueryableArtist> = predicate.load(conn)?;
        Ok(fetched)
    }

    fn get_genres(
        &self,
        options: QueryableGenre,
        inclusive: bool,
        conn: &mut PooledConnection<ConnectionManager<SqliteConnection>>,
    ) -> Result<Vec<QueryableGenre>> {
        let mut predicate = schema::genres::table.into_boxed();

        predicate = filter_field!(
            predicate,
            &options.genre_id,
            schema::genres::genre_id,
            inclusive
        );

        predicate = filter_field_like!(
            predicate,
            &options.genre_name,
            schema::genres::genre_name,
            inclusive
        );

        let fetched: Vec<QueryableGenre> = predicate.load(conn)?;
        Ok(fetched)
    }

    fn get_playlists(
        &self,
        options: QueryablePlaylist,
        inclusive: bool,
        conn: &mut PooledConnection<ConnectionManager<SqliteConnection>>,
    ) -> Result<Vec<QueryablePlaylist>> {
        let mut predicate = schema::playlists::table.into_boxed();

        predicate = filter_field!(
            predicate,
            &options.playlist_id,
            schema::playlists::playlist_id,
            inclusive
        );

        predicate = filter_field_like!(
            predicate,
            if options.playlist_name.is_empty() {
                None
            } else {
                Some(&options.playlist_name)
            },
            schema::playlists::playlist_name,
            inclusive
        );

        predicate = filter_field_like!(
            predicate,
            &options.playlist_path,
            schema::playlists::playlist_path,
            inclusive
        );

        let fetched: Vec<QueryablePlaylist> = predicate.load(conn)?;
        Ok(fetched)
    }

    pub fn get_entity_by_options(&self, options: GetEntityOptions) -> Result<Value> {
        let mut conn = self.pool.get().unwrap();
        let inclusive = if let Some(inclusive) = options.inclusive {
            inclusive
        } else {
            false
        };

        if options.album.is_some() {
            return Ok(serde_json::to_value(self.get_albums(
                options.album.unwrap(),
                inclusive,
                &mut conn,
            )?)
            .unwrap());
        }

        if options.artist.is_some() {
            return Ok(serde_json::to_value(self.get_artists(
                options.artist.unwrap(),
                inclusive,
                &mut conn,
            )?)
            .unwrap());
        }

        if options.genre.is_some() {
            return Ok(serde_json::to_value(self.get_genres(
                options.genre.unwrap(),
                inclusive,
                &mut conn,
            )?)
            .unwrap());
        }

        if options.playlist.is_some() {
            return Ok(serde_json::to_value(self.get_playlists(
                options.playlist.unwrap(),
                inclusive,
                &mut conn,
            )?)
            .unwrap());
        }

        Ok(Value::Null)
    }

    pub fn get_album_songs(
        &self,
        options: QueryableAlbum,
        inclusive: bool,
        conn: &mut PooledConnection<ConnectionManager<SqliteConnection>>,
    ) -> Result<Vec<QueryableSong>> {
        let binding = self.get_albums(options, inclusive, conn)?;
        let album = binding.first();
        if album.is_none() {
            return Ok(vec![]);
        }

        let album = album.unwrap();
        let album_data: Vec<AlbumBridge> = QueryDsl::filter(
            album_bridge,
            schema::album_bridge::album.eq(album.album_id.clone()),
        )
        .load(conn)?;

        let songs: Vec<QueryableSong> = QueryDsl::filter(
            allsongs,
            _id.eq_any(album_data.iter().map(|v| v.song.clone())),
        )
        .load(conn)?;

        Ok(songs)
    }

    pub fn get_artist_songs(
        &self,
        options: QueryableArtist,
        inclusive: bool,
        conn: &mut PooledConnection<ConnectionManager<SqliteConnection>>,
    ) -> Result<Vec<QueryableSong>> {
        let binding = self.get_artists(options, inclusive, conn)?;
        let artist = binding.first();
        if artist.is_none() {
            return Ok(vec![]);
        }

        let artist = artist.unwrap();
        let artist_data: Vec<AlbumBridge> = QueryDsl::filter(
            artist_bridge,
            schema::artist_bridge::artist.eq(artist.artist_id.clone()),
        )
        .load(conn)?;

        let songs: Vec<QueryableSong> = QueryDsl::filter(
            allsongs,
            _id.eq_any(artist_data.iter().map(|v| v.song.clone())),
        )
        .load(conn)?;

        Ok(songs)
    }

    pub fn get_genre_songs(
        &self,
        options: QueryableGenre,
        inclusive: bool,
        conn: &mut PooledConnection<ConnectionManager<SqliteConnection>>,
    ) -> Result<Vec<QueryableSong>> {
        let binding = self.get_genres(options, inclusive, conn)?;
        let genre = binding.first();
        if genre.is_none() {
            return Ok(vec![]);
        }

        let genre = genre.unwrap();
        let genre_data: Vec<AlbumBridge> = QueryDsl::filter(
            genre_bridge,
            schema::genre_bridge::genre.eq(genre.genre_id.clone()),
        )
        .load(conn)?;

        let songs: Vec<QueryableSong> = QueryDsl::filter(
            allsongs,
            _id.eq_any(genre_data.iter().map(|v| v.song.clone())),
        )
        .load(conn)?;

        Ok(songs)
    }

    pub fn get_playlist_songs(
        &self,
        options: QueryablePlaylist,
        inclusive: bool,
        conn: &mut PooledConnection<ConnectionManager<SqliteConnection>>,
    ) -> Result<Vec<QueryableSong>> {
        let binding = self.get_playlists(options, inclusive, conn)?;
        let playlist = binding.first();
        if playlist.is_none() {
            return Ok(vec![]);
        }

        let playlist = playlist.unwrap();
        let playlist_data: Vec<AlbumBridge> = QueryDsl::filter(
            playlist_bridge,
            schema::playlist_bridge::playlist.eq(playlist.playlist_id.clone()),
        )
        .load(conn)?;

        let songs: Vec<QueryableSong> = QueryDsl::filter(
            allsongs,
            _id.eq_any(playlist_data.iter().map(|v| v.song.clone())),
        )
        .load(conn)?;

        Ok(songs)
    }

    pub fn get_songs_by_options(&self, options: GetSongOptions) -> Result<Vec<Song>> {
        let mut ret = vec![];
        let inclusive = if let Some(inclusive) = options.inclusive {
            inclusive
        } else {
            false
        };

        self.pool.get().unwrap().transaction(|conn| {
            let mut fetched_songs: Vec<QueryableSong> = vec![];

            if let Some(song) = options.song {
                let mut predicate = schema::allsongs::table.into_boxed();
                predicate = filter_field!(predicate, &song._id, schema::allsongs::_id, inclusive);
                predicate =
                    filter_field_like!(predicate, &song.path, schema::allsongs::path, inclusive);
                predicate =
                    filter_field_like!(predicate, &song.title, schema::allsongs::title, inclusive);
                predicate = filter_field!(
                    predicate,
                    &song.sample_rate,
                    schema::allsongs::sampleRate,
                    inclusive
                );
                predicate = filter_field!(predicate, &song.hash, schema::allsongs::hash, inclusive);
                predicate =
                    filter_field!(predicate, &song.type_, schema::allsongs::type_, inclusive);
                predicate =
                    filter_field_like!(predicate, &song.url, schema::allsongs::url, inclusive);
                predicate = filter_field_like!(
                    predicate,
                    &song.playback_url,
                    schema::allsongs::playbackUrl,
                    inclusive
                );
                predicate = filter_field!(
                    predicate,
                    &song.provider_extension,
                    schema::allsongs::provider_extension,
                    inclusive
                );
                predicate = filter_field!(
                    predicate,
                    &song.show_in_library,
                    schema::allsongs::show_in_library,
                    inclusive
                );

                fetched_songs = predicate.load(conn)?;
            } else if let Some(album) = options.album {
                fetched_songs = self.get_album_songs(album, inclusive, conn)?;
            } else if let Some(artist) = options.artist {
                fetched_songs = self.get_artist_songs(artist, inclusive, conn)?;
            } else if let Some(genre) = options.genre {
                fetched_songs = self.get_genre_songs(genre, inclusive, conn)?;
            } else if let Some(playlist) = options.playlist {
                fetched_songs = self.get_playlist_songs(playlist, inclusive, conn)?;
            }

            for s in fetched_songs {
                let mut album: Option<QueryableAlbum> = None;
                let mut artist: Vec<QueryableArtist> = vec![];
                let mut genre: Vec<QueryableGenre> = vec![];

                let album_data =
                    QueryDsl::filter(album_bridge, schema::album_bridge::song.eq(s._id.clone()))
                        .first::<AlbumBridge>(conn);

                if let Ok(album_data) = album_data {
                    album =
                        Some(QueryDsl::filter(albums, album_id.eq(album_data.album)).first(conn)?);
                }

                let artist_data =
                    QueryDsl::filter(artist_bridge, schema::artist_bridge::song.eq(s._id.clone()))
                        .first::<ArtistBridge>(conn);

                if let Ok(artist_data) = artist_data {
                    artist =
                        QueryDsl::filter(artists, artist_id.eq(artist_data.artist)).load(conn)?;
                }

                let genre_data =
                    QueryDsl::filter(genre_bridge, schema::genre_bridge::song.eq(s._id.clone()))
                        .first::<GenreBridge>(conn);

                if let Ok(genre_data) = genre_data {
                    genre = QueryDsl::filter(genres, genre_id.eq(genre_data.genre)).load(conn)?;
                }
                ret.push(Song {
                    song: s,
                    album,
                    artists: artist,
                    genre,
                });
            }
            Ok(ret)
        })
    }

    pub fn search_all(&self, term: String) -> Result<SearchResult> {
        let songs = self.get_songs_by_options(GetSongOptions {
            song: Some(SearchableSong {
                _id: None,
                path: Some(term.clone()),
                title: Some(term.clone()),
                sample_rate: None,
                hash: None,
                type_: None,
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

        println!("Got songs: {:?}", songs);

        let mut conn = self.pool.get().unwrap();
        let _albums = self.get_albums(
            QueryableAlbum {
                album_id: None,
                album_name: Some(term.clone()),
                album_artist: None,
                album_coverPath_high: None,
                album_song_count: 0f64,
                year: None,
                album_coverPath_low: None,
                album_extra_info: None,
            },
            false,
            &mut conn,
        )?;

        let _artists = self.get_artists(
            QueryableArtist {
                artist_id: None,
                artist_mbid: None,
                artist_name: Some(term.clone()),
                artist_coverPath: None,
                artist_song_count: 0f64,
                artist_extra_info: None,
                sanitized_artist_name: None,
            },
            false,
            &mut conn,
        )?;

        let _genres = self.get_genres(
            QueryableGenre {
                genre_id: None,
                genre_name: Some(term.clone()),
                genre_song_count: 0f64,
            },
            false,
            &mut conn,
        )?;

        let playlists = self.get_playlists(
            QueryablePlaylist {
                playlist_id: None,
                playlist_name: term.clone(),
                playlist_coverPath: None,
                playlist_song_count: 0f64,
                playlist_desc: Some(term.clone()),
                playlist_path: Some(term.clone()),
                extension: None,
                icon: None,
            },
            false,
            &mut conn,
        )?;

        Ok(SearchResult {
            songs,
            artists: _artists,
            playlists,
            albums: _albums,
            genres: _genres,
        })
    }
}
