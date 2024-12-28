use std::cmp::min;

use std::fmt::Write;
use std::str::FromStr;
use std::{path::PathBuf, vec};

use diesel::{
    connection::SimpleConnection,
    delete, insert_into,
    r2d2::{self, ConnectionManager, Pool, PooledConnection},
    update, Connection, ExpressionMethods, QueryDsl, RunQueryDsl, SqliteConnection,
};
use diesel::{BoolExpressionMethods, Insertable, TextExpressionMethods};
use diesel_logger::LoggingConnection;
use macros::{filter_field, filter_field_like};
use serde_json::Value;
use tracing::{debug, info, trace};
use uuid::Uuid;

use types::common::{BridgeUtils, SearchByTerm};
use types::entities::{Analytics, EntityInfo, PlaylistBridge, SearchResult};
use types::errors::{MoosyncError, Result};
use types::schema::analytics::dsl::analytics;
use types::schema::playlists::dsl::playlists;
use types::songs::SearchableSong;
use types::{
    schema::{
        self,
        album_bridge::dsl::album_bridge,
        albums::{album_id, dsl::albums},
        allsongs::{_id, dsl::allsongs, path as song_path},
        artist_bridge::dsl::artist_bridge,
        artists::{artist_id, dsl::artists},
        genre_bridge::dsl::genre_bridge,
        genres::{dsl::genres, genre_id},
        playlist_bridge::dsl::playlist_bridge,
    },
    {
        entities::{
            AlbumBridge, ArtistBridge, GenreBridge, GetEntityOptions, QueryableAlbum,
            QueryableArtist, QueryableGenre, QueryablePlaylist,
        },
        songs::{GetSongOptions, QueryableSong, Song},
    },
};

use super::migrations::run_migrations;

#[derive(Debug, Clone)]
pub struct Database {
    pool: Pool<ConnectionManager<LoggingConnection<SqliteConnection>>>,
}

impl Database {
    #[tracing::instrument(level = "trace", skip(path))]
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

        info!("Created DB instance");
        db
    }

    #[tracing::instrument(level = "trace", skip(path))]
    fn connect(path: PathBuf) -> Pool<ConnectionManager<LoggingConnection<SqliteConnection>>> {
        let manager =
            ConnectionManager::<LoggingConnection<SqliteConnection>>::new(path.to_str().unwrap());

        r2d2::Pool::builder()
            .build(manager)
            .expect("Failed to create pool.")
    }

    #[tracing::instrument(level = "trace", skip(self, conn))]
    fn insert_album(
        &self,
        conn: &mut PooledConnection<ConnectionManager<LoggingConnection<SqliteConnection>>>,
        _album: QueryableAlbum,
    ) -> Result<String> {
        let mut cloned = _album.clone();
        cloned.album_id = Some(Uuid::new_v4().to_string());

        trace!("Inserting album");
        insert_into(albums).values(&cloned).execute(conn)?;
        info!("Inserted album");
        Ok(cloned.album_id.unwrap())
    }

    #[tracing::instrument(level = "trace", skip(self, conn))]
    fn insert_artist(
        &self,
        conn: &mut PooledConnection<ConnectionManager<LoggingConnection<SqliteConnection>>>,
        _artist: QueryableArtist,
    ) -> Result<String> {
        let mut cloned = _artist.clone();
        cloned.artist_id = Some(Uuid::new_v4().to_string());
        trace!("Inserting artist");
        insert_into(artists).values(&cloned).execute(conn)?;
        info!("Inserted artist");
        Ok(cloned.artist_id.unwrap())
    }

    #[tracing::instrument(level = "trace", skip(self, conn))]
    fn insert_genre(
        &self,
        conn: &mut PooledConnection<ConnectionManager<LoggingConnection<SqliteConnection>>>,
        _genre: QueryableGenre,
    ) -> Result<String> {
        let mut cloned = _genre.clone();
        cloned.genre_id = Some(Uuid::new_v4().to_string());
        trace!("Inserting genre");
        insert_into(genres).values(&cloned).execute(conn)?;
        info!("Inserted genre");
        Ok(cloned.genre_id.unwrap())
    }

    #[tracing::instrument(level = "trace", skip(self, conn))]
    fn insert_playlist(
        &self,
        conn: &mut PooledConnection<ConnectionManager<LoggingConnection<SqliteConnection>>>,
        _playlist: QueryablePlaylist,
    ) -> Result<String> {
        let cloned = _playlist.clone();
        trace!("Inserting playlist");
        insert_into(playlists).values(&cloned).execute(conn)?;
        info!("Inserted playlist");
        Ok(cloned.playlist_id.unwrap())
    }

    #[tracing::instrument(level = "trace", skip(self))]
    pub fn create_playlist(&self, playlist: QueryablePlaylist) -> Result<String> {
        let mut conn = self.pool.get().unwrap();

        trace!("Sanitizing playlist");

        let mut playlist = playlist.clone();
        if playlist.playlist_id.is_none() {
            playlist.playlist_id = Some(Uuid::new_v4().to_string());
        }

        if playlist.playlist_name.is_empty() {
            playlist.playlist_name = "New playlist".to_string();
        }

        if playlist.playlist_path.is_some() {
            let fetched = self.get_playlists(
                QueryablePlaylist {
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

    #[tracing::instrument(level = "trace", skip(self))]
    pub fn add_to_playlist_bridge(&self, playlist_id: String, song_id: String) -> Result<()> {
        let mut conn = self.pool.get().unwrap();
        trace!("Inserting song in playlist bridge");
        insert_into(playlist_bridge)
            .values(PlaylistBridge::insert_value(playlist_id, song_id))
            .execute(&mut conn)?;

        trace!("Inserted song in playlist bridge");

        Ok(())
    }

    #[tracing::instrument(level = "trace", skip(self))]
    pub fn insert_songs(&self, songs: Vec<Song>) -> Result<Vec<Song>> {
        let mut ret = vec![];
        let mut conn = self.pool.get().unwrap();
        trace!("Inserting songs");
        for mut song in songs {
            if song.song._id.is_none() {
                song.song._id = Some(Uuid::new_v4().to_string());
            }

            let changed = insert_into(allsongs)
                .values(&song.song)
                .on_conflict(song_path)
                .do_update()
                .set(&song.song)
                .execute(&mut conn)?;

            if changed == 0 {
                ret.push(song);
                continue;
            }

            if let Some(_album) = &mut song.album {
                let album_id_ = self
                    .get_albums(
                        QueryableAlbum::search_by_term(_album.album_name.clone()),
                        false,
                        &mut conn,
                    )?
                    .first()
                    .map(|v| v.album_id.clone().unwrap())
                    .unwrap_or_else(|| self.insert_album(&mut conn, _album.clone()).unwrap());

                AlbumBridge::insert_value(album_id_.clone(), song.song._id.clone().unwrap())
                    .insert_into(album_bridge)
                    .on_conflict_do_nothing()
                    .execute(&mut conn)?;

                _album.album_id = Some(album_id_);
            }

            if let Some(_artists) = &mut song.artists {
                for mut _artist in _artists {
                    let artist_id_ = self
                        .get_artists(
                            QueryableArtist::search_by_term(_artist.artist_name.clone()),
                            false,
                            &mut conn,
                        )?
                        .first()
                        .map(|v| v.artist_id.clone().unwrap())
                        .unwrap_or_else(|| self.insert_artist(&mut conn, _artist.clone()).unwrap());

                    ArtistBridge::insert_value(artist_id_.clone(), song.song._id.clone().unwrap())
                        .insert_into(artist_bridge)
                        .on_conflict_do_nothing()
                        .execute(&mut conn)?;

                    _artist.artist_id = Some(artist_id_);
                }
            }

            if let Some(_genres) = &mut song.genre {
                for mut _genre in _genres {
                    let genre_id_ = self
                        .get_genres(
                            QueryableGenre::search_by_term(_genre.genre_name.clone()),
                            false,
                            &mut conn,
                        )?
                        .first()
                        .map(|v| v.genre_id.clone().unwrap())
                        .unwrap_or_else(|| self.insert_genre(&mut conn, _genre.clone()).unwrap());

                    GenreBridge::insert_value(genre_id_.clone(), song.song._id.clone().unwrap())
                        .insert_into(genre_bridge)
                        .on_conflict_do_nothing()
                        .execute(&mut conn)?;

                    _genre.genre_id = Some(genre_id_);
                }
            }

            trace!("Inserted song, {:?}", song);
            ret.push(song);
        }
        info!("Inserted all songs");
        Ok(ret)
    }

    // TODO: Remove album
    #[tracing::instrument(level = "trace", skip(self))]
    pub fn remove_songs(&self, ids: Vec<String>) -> Result<()> {
        trace!("Removing song");
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

        info!("Removed song");

        Ok(())
    }

    #[tracing::instrument(level = "trace", skip(self, song))]
    pub fn update_song(&self, song: QueryableSong) -> Result<()> {
        trace!("Updating song");
        if let Some(id) = song._id.as_ref() {
            update(allsongs.filter(schema::allsongs::_id.eq(id.clone())))
                .set(&song)
                .execute(&mut self.pool.get().unwrap())?;
            debug!("Updated song");
        } else {
            debug!("Song does not have an ID");
        }
        Ok(())
    }

    #[tracing::instrument(level = "trace", skip(self, conn))]
    fn get_albums(
        &self,
        options: QueryableAlbum,
        inclusive: bool,
        conn: &mut PooledConnection<ConnectionManager<LoggingConnection<SqliteConnection>>>,
    ) -> Result<Vec<QueryableAlbum>> {
        let mut predicate = schema::albums::table.into_boxed();

        trace!("Getting albums");
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
        info!("Fetched albums");
        Ok(fetched)
    }

    #[tracing::instrument(level = "trace", skip(self, conn))]
    fn get_artists(
        &self,
        options: QueryableArtist,
        inclusive: bool,
        conn: &mut PooledConnection<ConnectionManager<LoggingConnection<SqliteConnection>>>,
    ) -> Result<Vec<QueryableArtist>> {
        let mut predicate = schema::artists::table.into_boxed();

        trace!("Fetching artists");
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
        info!("Fetched artists");
        Ok(fetched)
    }

    #[tracing::instrument(level = "trace", skip(self, conn))]
    fn get_genres(
        &self,
        options: QueryableGenre,
        inclusive: bool,
        conn: &mut PooledConnection<ConnectionManager<LoggingConnection<SqliteConnection>>>,
    ) -> Result<Vec<QueryableGenre>> {
        let mut predicate = schema::genres::table.into_boxed();

        trace!("Fetching genres");
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
        info!("Fetched genres");
        Ok(fetched)
    }

    #[tracing::instrument(level = "trace", skip(self, conn))]
    fn get_playlists(
        &self,
        options: QueryablePlaylist,
        inclusive: bool,
        conn: &mut PooledConnection<ConnectionManager<LoggingConnection<SqliteConnection>>>,
    ) -> Result<Vec<QueryablePlaylist>> {
        let mut predicate = schema::playlists::table.into_boxed();

        trace!("Fetching playlists");
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

    pub fn is_song_in_playlist(&self, playlist_id: String, song_id: String) -> Result<bool> {
        let mut conn = self.pool.get().unwrap();
        let res: Vec<i64> = schema::playlist_bridge::table
            .filter(
                schema::playlist_bridge::playlist
                    .eq(playlist_id)
                    .and(schema::playlist_bridge::song.eq(song_id)),
            )
            .count()
            .load(&mut conn)?;
        if let Some(res) = res.first() {
            return Ok(*res > 0);
        }
        Ok(false)
    }

    #[tracing::instrument(level = "trace", skip(self))]
    pub fn get_entity_by_options(&self, options: GetEntityOptions) -> Result<Value> {
        let mut conn = self.pool.get().unwrap();
        let inclusive = options.inclusive.unwrap_or_default();

        trace!("Getting entity by options");

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

    #[tracing::instrument(level = "trace", skip(self, conn))]
    pub fn get_album_songs(
        &self,
        options: QueryableAlbum,
        inclusive: bool,
        conn: &mut PooledConnection<ConnectionManager<LoggingConnection<SqliteConnection>>>,
    ) -> Result<Vec<QueryableSong>> {
        trace!("Fetching album songs");
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

        info!("Fetched album songs");
        Ok(songs)
    }

    #[tracing::instrument(level = "trace", skip(self, conn))]
    pub fn get_artist_songs(
        &self,
        options: QueryableArtist,
        inclusive: bool,
        conn: &mut PooledConnection<ConnectionManager<LoggingConnection<SqliteConnection>>>,
    ) -> Result<Vec<QueryableSong>> {
        trace!("Fetching artist songs");
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
        info!("Fetched artist songs");

        Ok(songs)
    }

    #[tracing::instrument(level = "trace", skip(self, conn))]
    pub fn get_genre_songs(
        &self,
        options: QueryableGenre,
        inclusive: bool,
        conn: &mut PooledConnection<ConnectionManager<LoggingConnection<SqliteConnection>>>,
    ) -> Result<Vec<QueryableSong>> {
        trace!("Fetching genre songs");
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

        info!("Fetched genre songs");
        Ok(songs)
    }

    #[tracing::instrument(level = "trace", skip(self, conn))]
    pub fn get_playlist_songs(
        &self,
        options: QueryablePlaylist,
        inclusive: bool,
        conn: &mut PooledConnection<ConnectionManager<LoggingConnection<SqliteConnection>>>,
    ) -> Result<Vec<QueryableSong>> {
        let binding = self.get_playlists(options, inclusive, conn)?;
        trace!("Fetching playlist songs");
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
        info!("Fetched playlist songs");

        Ok(songs)
    }

    #[tracing::instrument(level = "trace", skip(self))]
    pub fn get_songs_by_options(&self, options: GetSongOptions) -> Result<Vec<Song>> {
        let mut ret = vec![];
        trace!("Getting songs by options");
        let inclusive = options.inclusive.unwrap_or_default();

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
                    schema::allsongs::samplerate,
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
                    schema::allsongs::playbackurl,
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
                    artists: Some(artist),
                    genre: Some(genre),
                });
            }
            Ok(ret)
        })
    }

    #[tracing::instrument(level = "trace", skip(self))]
    pub fn search_all(&self, term: String) -> Result<SearchResult> {
        trace!("Searching all by term");
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

        let mut conn = self.pool.get().unwrap();
        let _albums = self.get_albums(
            QueryableAlbum {
                album_id: None,
                album_name: Some(term.clone()),
                album_artist: None,
                album_coverpath_high: None,
                album_song_count: 0f64,
                year: None,
                album_coverpath_low: None,
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
                artist_coverpath: None,
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

        let _playlists = self.get_playlists(
            QueryablePlaylist {
                playlist_id: None,
                playlist_name: term.clone(),
                playlist_coverpath: None,
                playlist_song_count: 0f64,
                playlist_desc: Some(term.clone()),
                playlist_path: Some(term.clone()),
                extension: None,
                icon: None,
                library_item: None,
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

    #[tracing::instrument(level = "trace", skip(self))]
    pub fn files_not_in_db(&self, file_list: Vec<(PathBuf, f64)>) -> Result<Vec<(PathBuf, f64)>> {
        let mut conn = self.pool.get().unwrap();

        let mut file_list_copy = file_list.clone();
        let len = file_list.len();

        let mut ret = vec![];

        let exp_limit = 998;
        for _ in 0..len / exp_limit + 1 {
            let curr_len = min(len, exp_limit);
            let mut query =
                QueryDsl::select(allsongs, (schema::allsongs::path, schema::allsongs::size))
                    .into_boxed();
            for _ in 0..curr_len {
                let data = file_list_copy.pop().unwrap();
                let predicate = schema::allsongs::path
                    .eq(data.0.to_string_lossy().to_string())
                    .and(schema::allsongs::size.eq(data.1));
                query = query.or_filter(predicate);
            }

            let mut res = query
                .load::<(Option<String>, Option<f64>)>(&mut conn)?
                .iter()
                .map(|v| {
                    (
                        PathBuf::from_str(v.0.clone().unwrap().as_str()).unwrap(),
                        v.1.unwrap(),
                    )
                })
                .collect::<Vec<_>>();
            ret.append(&mut res);
        }
        Ok(ret)
    }

    #[tracing::instrument(level = "trace", skip(self))]
    pub fn add_to_playlist(&self, id: String, songs: Vec<Song>) -> Result<()> {
        trace!("Adding to playlist");
        let mut songs = songs.clone();
        songs.iter_mut().for_each(|v| {
            v.song.show_in_library = Some(false);
        });
        let res = self.insert_songs(songs.clone());
        if let Err(e) = res {
            // Lets hope it only fails due to unique value constrains
            tracing::warn!(
                "Failed to insert songs in DB, maybe they already exist: {:?}",
                e
            );
        }
        self.pool
            .get()
            .unwrap()
            .transaction::<(), MoosyncError, _>(|conn| {
                for s in songs {
                    insert_into(playlist_bridge)
                        .values((
                            schema::playlist_bridge::playlist.eq(id.clone()),
                            schema::playlist_bridge::song.eq(s.song._id.clone()),
                        ))
                        .execute(conn)?;
                }
                Ok(())
            })?;
        info!("Added to playlist");
        Ok(())
    }

    #[tracing::instrument(level = "trace", skip(self))]
    pub fn remove_from_playlist(&self, id: String, songs: Vec<String>) -> Result<()> {
        trace!("Removing from playlist");
        self.pool
            .get()
            .unwrap()
            .transaction::<(), MoosyncError, _>(|conn| {
                for s in songs {
                    delete(playlist_bridge)
                        .filter(schema::playlist_bridge::playlist.eq(id.clone()))
                        .filter(schema::playlist_bridge::song.eq(s.clone()))
                        .execute(conn)?;
                }
                Ok(())
            })?;
        info!("Removed from playlist");
        Ok(())
    }

    #[tracing::instrument(level = "trace", skip(self))]
    pub fn remove_playlist(&self, id: String) -> Result<()> {
        trace!("Removing playlist");
        let mut conn = self.pool.get().unwrap();
        delete(playlist_bridge)
            .filter(schema::playlist_bridge::playlist.eq(id.clone()))
            .execute(&mut conn)?;
        delete(playlists)
            .filter(schema::playlists::playlist_id.eq(id.clone()))
            .execute(&mut conn)?;

        info!("Removed playlist");
        Ok(())
    }

    #[tracing::instrument(level = "trace", skip(self, old_info, new_info))]
    fn merge_extra_info(
        &self,
        old_info: Option<EntityInfo>,
        new_info: Option<EntityInfo>,
    ) -> Option<EntityInfo> {
        if old_info.is_none() && new_info.is_none() {
            return None;
        }

        if old_info.is_none() {
            return new_info;
        }

        if new_info.is_none() {
            return old_info;
        }

        let mut res = old_info.clone().unwrap();
        let mut a: Value = serde_json::from_str(res.0.as_str()).unwrap();
        let b: Value = serde_json::from_str(new_info.unwrap().0.as_str()).unwrap();
        merge(&mut a, b);
        res.0 = serde_json::to_string(&a).unwrap();
        Some(res)
    }

    #[tracing::instrument(level = "trace", skip(self))]
    pub fn update_album(&self, album: QueryableAlbum) -> Result<()> {
        trace!("Updating album");
        let mut conn = self.pool.get().unwrap();
        let parsed_album = album.clone();

        let existing_album_info = self
            .get_albums(
                QueryableAlbum {
                    album_id: album.album_id.clone(),
                    ..Default::default()
                },
                false,
                &mut conn,
            )?
            .first()
            .and_then(|a| a.album_extra_info.clone());

        self.merge_extra_info(existing_album_info, parsed_album.album_extra_info);

        update(albums)
            .filter(schema::albums::album_id.eq(album.album_id.clone()))
            .set(album)
            .execute(&mut conn)?;

        info!("Updated album");
        Ok(())
    }

    #[tracing::instrument(level = "trace", skip(self))]
    pub fn update_artist(&self, artist: QueryableArtist) -> Result<()> {
        trace!("Updating artist");
        let mut conn = self.pool.get().unwrap();
        let parsed_artist = artist.clone();

        let existing_artist_info = self
            .get_artists(
                QueryableArtist {
                    artist_id: artist.artist_id.clone(),
                    ..Default::default()
                },
                false,
                &mut conn,
            )?
            .first()
            .and_then(|a| a.artist_extra_info.clone());

        self.merge_extra_info(existing_artist_info, parsed_artist.artist_extra_info);

        update(artists)
            .filter(schema::artists::artist_id.eq(artist.artist_id.clone()))
            .set(artist)
            .execute(&mut conn)?;
        info!("Updated artist");
        Ok(())
    }

    #[tracing::instrument(level = "trace", skip(self))]
    pub fn update_playlist(&self, playlist: QueryablePlaylist) -> Result<()> {
        trace!("Updating playlist");
        let mut conn = self.pool.get().unwrap();
        update(playlists)
            .filter(schema::playlists::playlist_id.eq(playlist.playlist_id.clone()))
            .set(playlist)
            .execute(&mut conn)?;
        info!("Updated playlist");
        Ok(())
    }

    #[tracing::instrument(level = "trace", skip(self))]
    pub fn update_songs(&self, songs: Vec<Song>) -> Result<()> {
        trace!("Updating songs");
        let mut conn = self.pool.get().unwrap();

        for song in songs {
            if let Some(album) = song.album {
                self.update_album(album)?;
            }

            if let Some(artist) = song.artists {
                for a in artist {
                    self.update_artist(a)?;
                }
            }
            update(allsongs)
                .filter(schema::allsongs::_id.eq(song.song._id.clone()))
                .set(song.song)
                .execute(&mut conn)?;
        }
        info!("Updated songs");
        Ok(())
    }

    #[tracing::instrument(level = "trace", skip(self))]
    pub fn update_lyrics(&self, id: String, lyrics: String) -> Result<()> {
        trace!("Updating lyrics");
        let mut conn = self.pool.get().unwrap();
        update(allsongs)
            .filter(schema::allsongs::_id.eq(id))
            .set(schema::allsongs::lyrics.eq(lyrics))
            .execute(&mut conn)?;
        info!("Updated lyrics");
        Ok(())
    }

    #[tracing::instrument(level = "trace", skip(self))]
    pub fn increment_play_count(&self, id: String) -> Result<()> {
        trace!("Incrementing play count");
        let mut conn = self.pool.get().unwrap();
        let play_count: Option<i32> = QueryDsl::select(analytics, schema::analytics::play_count)
            .filter(schema::analytics::song_id.eq(id.clone()))
            .first(&mut conn)?;

        if play_count.is_none() {
            insert_into(analytics)
                .values(Analytics {
                    id: Some(Uuid::new_v4().to_string()),
                    song_id: Some(id),
                    play_count: Some(1),
                    play_time: Some(0f64),
                })
                .execute(&mut conn)?;
            return Ok(());
        }

        update(analytics)
            .filter(schema::analytics::song_id.eq(id))
            .set(schema::analytics::play_count.eq(schema::analytics::play_count + 1))
            .execute(&mut conn)?;

        info!("Incremented play count");
        Ok(())
    }

    #[tracing::instrument(level = "trace", skip(self))]
    pub fn increment_play_time(&self, id: String, duration: f64) -> Result<()> {
        trace!("Incrementing play time");
        let mut conn = self.pool.get().unwrap();
        let play_time: Option<f64> = QueryDsl::select(analytics, schema::analytics::play_time)
            .filter(schema::analytics::song_id.eq(id.clone()))
            .first(&mut conn)?;

        if play_time.is_none() {
            insert_into(analytics)
                .values(Analytics {
                    id: Some(Uuid::new_v4().to_string()),
                    song_id: Some(id),
                    play_count: Some(0),
                    play_time: Some(duration),
                })
                .execute(&mut conn)?;
            info!("Added new play time");
            return Ok(());
        }

        update(analytics)
            .filter(schema::analytics::song_id.eq(id))
            .set(schema::analytics::play_time.eq(schema::analytics::play_time + duration))
            .execute(&mut conn)?;

        info!("Incremented playtime");

        Ok(())
    }

    #[tracing::instrument(level = "trace", skip(self))]
    pub fn export_playlist(&self, playlist_id: String) -> Result<String> {
        let mut conn = self.pool.get().unwrap();

        let binding = self.get_playlists(
            QueryablePlaylist {
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
            playlist: Some(QueryablePlaylist {
                playlist_id: Some(playlist_id),
                ..Default::default()
            }),
            ..Default::default()
        })?;

        let mut ret = format!("#EXTM3U\n#PLAYLIST:{}\n", playlist.playlist_name);

        for s in playlist_songs {
            if let Some(path) = &s.song.path {
                let duration = s.song.duration.unwrap_or(0f64);
                let title = s.song.title.unwrap_or_default();
                let album_info = s.album.as_ref().map_or(String::new(), |album| {
                    format!("#EXTALB:{}", album.album_name.clone().unwrap_or_default())
                });
                let genre_info = if let Some(genre) = &s.genre {
                    if !genre.is_empty() {
                        format!(
                            "#EXTGENRE:{}",
                            genre
                                .iter()
                                .filter_map(|g| g.genre_name.clone())
                                .collect::<Vec<String>>()
                                .join(",")
                        )
                    } else {
                        String::new()
                    }
                } else {
                    String::new()
                };
                let cover_path = match s.song.song_cover_path_high {
                    Some(cover) => format!("#EXTIMG:{}", cover),
                    None => String::new(),
                };
                let song_info = format!("#MOOSINF:{}", s.song.type_);
                let file_path = format!("file://{}", path);

                write!(
                    ret,
                    "#EXTINF:{},{}\n{}\n{}\n{}\n{}\n{}\n",
                    duration, title, album_info, genre_info, cover_path, song_info, file_path
                )?;
            } else if let Some(url) = &s.song.url {
                let duration = s.song.duration.unwrap_or(0f64);
                let title = s.song.title.unwrap_or_default();
                let album_info = s.album.as_ref().map_or(String::new(), |album| {
                    format!("#EXTALB:{}", album.album_name.clone().unwrap_or_default())
                });
                let genre_info = if let Some(genre) = &s.genre {
                    if !genre.is_empty() {
                        format!(
                            "#EXTGENRE:{}",
                            genre
                                .iter()
                                .filter_map(|g| g.genre_name.clone())
                                .collect::<Vec<String>>()
                                .join(",")
                        )
                    } else {
                        String::new()
                    }
                } else {
                    String::new()
                };
                let cover_path = match s.song.song_cover_path_high {
                    Some(cover) => format!("#EXTIMG:{}", cover),
                    None => String::new(),
                };
                let song_info = format!("#MOOSINF:{}", s.song.type_);

                write!(
                    ret,
                    "#EXTINF:{},{}\n{}\n{}\n{}\n{}\n{}\n",
                    duration, title, album_info, genre_info, cover_path, song_info, url
                )?;
            }
        }

        Ok(ret.replace("\n\n", "\n"))
    }
}

#[tracing::instrument(level = "trace", skip())]
fn merge(a: &mut Value, b: Value) {
    if let Value::Object(a) = a {
        if let Value::Object(b) = b {
            for (k, v) in b {
                if v.is_null() {
                    a.remove(&k);
                } else {
                    merge(a.entry(k).or_insert(Value::Null), v);
                }
            }

            return;
        }
    }

    *a = b;
}
