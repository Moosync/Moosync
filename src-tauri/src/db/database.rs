use std::{
    path::PathBuf,
    thread::{self, JoinHandle},
    vec,
};

use diesel::internal::table_macro::BoxedSelectStatement;
use diesel::BoxableExpression;

use diesel::{
    associations::HasTable,
    connection::SimpleConnection,
    delete, insert_into,
    query_builder::{AsQuery, Query},
    query_dsl::methods::FilterDsl,
    r2d2::{self, ConnectionManager, Pool},
    result::Error,
    select, update, BoolExpressionMethods, Connection, ExpressionMethods, QueryDsl, RunQueryDsl,
    SelectableHelper, SqliteConnection,
};
use serde_json::Value;
use tauri::{App, Manager, State};
use uuid::Uuid;

use crate::{
    db::schema::{
        self,
        album_bridge::dsl::album_bridge,
        albums::{album_id, dsl::albums},
    },
    types::{
        entities::{
            AlbumBridge, ArtistBridge, GenreBridge, GetEntityOptions, QueryableAlbum,
            QueryableArtist, QueryableGenre, QueryablePlaylist,
        },
        songs::{GetSongOptions, QueryableSong, Song},
    },
};

use super::{
    migrations::run_migrations,
    schema::{
        allsongs::{_id, dsl::allsongs},
        artist_bridge::dsl::artist_bridge,
        artists::{artist_id, dsl::artists},
        genre_bridge::dsl::genre_bridge,
        genres::{dsl::genres, genre_id},
    },
};

#[macro_export]
macro_rules! generate_command {
    ($method_name:ident, $ret:ty, $($v:ident: $t:ty),*) => {
        #[tauri::command]
        pub fn $method_name(db: State<Database>, $($v: $t),*) -> Result<$ret, String> {
            db.$method_name($($v,)*).map_err(|e| e.to_string())
        }
    };
}

macro_rules! filter_field {
    ($predicate:expr, $field:expr, $column:expr, $inclusive:expr) => {
        if let Some(val) = $field {
            if $inclusive {
                QueryDsl::or_filter($predicate, $column.eq(val))
            } else {
                QueryDsl::filter($predicate, $column.eq(val))
            }
        } else {
            $predicate
        }
    };
}

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

    pub fn connect(path: PathBuf) -> Pool<ConnectionManager<SqliteConnection>> {
        let manager = ConnectionManager::<SqliteConnection>::new(path.to_str().unwrap());
        let pool = r2d2::Pool::builder()
            .build(manager)
            .expect("Failed to create pool.");
        pool
    }

    pub fn insert_songs(&self, songs: Vec<Song>) -> Result<(), Error> {
        let mut conn = self.pool.get().unwrap();
        for song in songs {
            insert_into(allsongs)
                .values(&song.song)
                .execute(&mut conn)?;

            if song.album.is_some() {
                let album = song.album.unwrap();
                insert_into(albums).values(&album).execute(&mut conn)?;
                insert_into(album_bridge)
                    .values(&AlbumBridge {
                        id: None,
                        song: song.song._id,
                        album: album.album_id,
                    })
                    .execute(&mut conn)?;
            }
        }
        Ok(())
    }

    // TODO: Remove album
    pub fn remove_songs(&self, ids: Vec<String>) -> Result<(), Error> {
        let mut conn = self.pool.get().unwrap();
        for id in ids {
            delete(QueryDsl::filter(allsongs, _id.eq(id))).execute(&mut conn)?;
        }

        Ok(())
    }

    pub fn update_song(&self, song: QueryableSong) -> Result<(), Error> {
        update(allsongs)
            .set(&song)
            .execute(&mut self.pool.get().unwrap())?;
        Ok(())
    }

    fn get_albums(
        &self,
        options: QueryableAlbum,
        inclusive: bool,
    ) -> Result<Vec<QueryableAlbum>, Error> {
        let mut conn = self.pool.get().unwrap();
        let mut predicate = schema::albums::table.into_boxed();

        predicate = filter_field!(
            predicate,
            &options.album_id,
            schema::albums::album_id,
            inclusive
        );

        predicate = filter_field!(
            predicate,
            &options.album_name,
            schema::albums::album_name,
            inclusive
        );

        let fetched: Vec<QueryableAlbum> = predicate.load(&mut conn)?;
        Ok(fetched)
    }

    fn get_artists(
        &self,
        options: QueryableArtist,
        inclusive: bool,
    ) -> Result<Vec<QueryableArtist>, Error> {
        let mut conn = self.pool.get().unwrap();
        let mut predicate = schema::artists::table.into_boxed();

        predicate = filter_field!(
            predicate,
            &options.artist_id,
            schema::artists::artist_id,
            inclusive
        );

        predicate = filter_field!(
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

        let fetched: Vec<QueryableArtist> = predicate.load(&mut conn)?;
        Ok(fetched)
    }

    fn get_genres(
        &self,
        options: QueryableGenre,
        inclusive: bool,
    ) -> Result<Vec<QueryableGenre>, Error> {
        let mut conn = self.pool.get().unwrap();
        let mut predicate = schema::genres::table.into_boxed();

        predicate = filter_field!(
            predicate,
            &options.genre_id,
            schema::genres::genre_id,
            inclusive
        );

        predicate = filter_field!(
            predicate,
            &options.genre_name,
            schema::genres::genre_name,
            inclusive
        );

        let fetched: Vec<QueryableGenre> = predicate.load(&mut conn)?;
        Ok(fetched)
    }

    fn get_playlists(
        &self,
        options: QueryablePlaylist,
        inclusive: bool,
    ) -> Result<Vec<QueryablePlaylist>, Error> {
        let mut conn = self.pool.get().unwrap();
        let mut predicate = schema::playlists::table.into_boxed();

        predicate = filter_field!(
            predicate,
            &options.playlist_id,
            schema::playlists::playlist_id,
            inclusive
        );

        predicate = filter_field!(
            predicate,
            if options.playlist_name.is_empty() {
                None
            } else {
                Some(&options.playlist_name)
            },
            schema::playlists::playlist_name,
            inclusive
        );

        predicate = filter_field!(
            predicate,
            &options.playlist_path,
            schema::playlists::playlist_path,
            inclusive
        );

        let fetched: Vec<QueryablePlaylist> = predicate.load(&mut conn)?;
        Ok(fetched)
    }

    pub fn get_entity_by_options(&self, options: GetEntityOptions) -> Result<Value, Error> {
        let inclusive = if let Some(inclusive) = options.inclusive {
            inclusive
        } else {
            false
        };

        if options.album.is_some() {
            return Ok(
                serde_json::to_value(self.get_albums(options.album.unwrap(), inclusive)?).unwrap(),
            );
        }

        if options.artist.is_some() {
            return Ok(
                serde_json::to_value(self.get_artists(options.artist.unwrap(), inclusive)?)
                    .unwrap(),
            );
        }

        if options.genre.is_some() {
            return Ok(
                serde_json::to_value(self.get_genres(options.genre.unwrap(), inclusive)?).unwrap(),
            );
        }

        if options.playlist.is_some() {
            return Ok(serde_json::to_value(
                self.get_playlists(options.playlist.unwrap(), inclusive)?,
            )
            .unwrap());
        }

        Ok(Value::Null)
    }

    pub fn get_songs_by_options(&self, options: GetSongOptions) -> Result<Vec<Song>, Error> {
        let mut ret = vec![];
        let mut conn = self.pool.get().unwrap();

        let mut predicate = schema::allsongs::table.into_boxed();
        let inclusive = if let Some(inclusive) = options.inclusive {
            inclusive
        } else {
            false
        };

        if let Some(song) = options.song {
            predicate = filter_field!(predicate, &song._id, schema::allsongs::_id, inclusive);
            predicate = filter_field!(predicate, &song.path, schema::allsongs::path, inclusive);
            predicate = filter_field!(predicate, &song.title, schema::allsongs::title, inclusive);
            predicate = filter_field!(
                predicate,
                &song.sample_rate,
                schema::allsongs::sampleRate,
                inclusive
            );
            predicate = filter_field!(predicate, &song.hash, schema::allsongs::hash, inclusive);
            predicate = filter_field!(predicate, &song.type_, schema::allsongs::type_, inclusive);
            predicate = filter_field!(predicate, &song.url, schema::allsongs::url, inclusive);
            predicate = filter_field!(
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
        }

        let fetched_songs: Vec<QueryableSong> = predicate.load(&mut conn)?;

        for s in fetched_songs {
            let mut album: Option<QueryableAlbum> = None;
            let mut artist: Vec<QueryableArtist> = vec![];
            let mut genre: Vec<QueryableGenre> = vec![];

            let album_data =
                QueryDsl::filter(album_bridge, schema::album_bridge::song.eq(s._id.clone()))
                    .first::<AlbumBridge>(&mut conn);

            if let Ok(album_data) = album_data {
                album =
                    Some(QueryDsl::filter(albums, album_id.eq(album_data.album)).first(&mut conn)?);
            }

            let artist_data =
                QueryDsl::filter(artist_bridge, schema::artist_bridge::song.eq(s._id.clone()))
                    .first::<ArtistBridge>(&mut conn);

            if let Ok(artist_data) = artist_data {
                artist =
                    QueryDsl::filter(artists, artist_id.eq(artist_data.artist)).load(&mut conn)?;
            }

            let genre_data =
                QueryDsl::filter(genre_bridge, schema::genre_bridge::song.eq(s._id.clone()))
                    .first::<GenreBridge>(&mut conn);

            if let Ok(genre_data) = genre_data {
                genre = QueryDsl::filter(genres, genre_id.eq(genre_data.genre)).load(&mut conn)?;
            }
            ret.push(Song {
                song: s,
                album,
                artists: artist,
                genre,
            });
        }
        Ok(ret)
    }
}

pub fn get_db_state(app: &mut App) -> Database {
    let path = app.path().app_data_dir().unwrap().join("songs.db");
    println!("DB path {:?}", path);
    let db = Database::new(path);

    db
}

generate_command!(insert_songs, (), a: Vec<Song>);
generate_command!(remove_songs, (), a: Vec<String>);
generate_command!(update_song, (), a: QueryableSong);
generate_command!(get_songs_by_options, Vec<Song>, options: GetSongOptions);
generate_command!(get_entity_by_options, Value, options: GetEntityOptions);
