use std::collections::HashMap;

use prost::Message;
use songs_proto::moosync::types::{
    Album, Artist, Genre, GetSongOptions, InnerSong, Playlist, SearchableSong, Song,
};
use sqlx::{QueryBuilder, Sqlite};
use tracing::{info, trace};

use crate::{
    error::DatabaseError,
    models::{DbAlbumRow, DbArtistRow, DbGenreRow, DbInnerSongRow, DbPlaylistRow},
    pool::DbPools,
    utils::song_type_to_str,
};

const ALLSONGS_SELECT: &str = "SELECT \
    a._id AS id, a.path, a.size, a.title, a.date, a.year, a.releasetype AS release_type, \
    a.bitrate, a.codec, a.container, a.duration, a.samplerate AS sample_rate, a.hash, \
    a.type, a.url, a.song_coverpath_high AS song_cover_path_high, a.playbackurl AS playback_url, \
    a.song_coverpath_low AS song_cover_path_low, a.date_added, a.provider_extension AS extension, \
    a.track_no \
    FROM allsongs a";

#[derive(Clone)]
pub struct EntityRepository {
    pools: DbPools,
}

impl EntityRepository {
    #[tracing::instrument(level = "debug", skip_all)]
    pub fn new(pools: DbPools) -> Self { Self { pools } }

    #[tracing::instrument(level = "debug", skip_all)]
    pub async fn update_album(&self, album: &Album) -> Result<(), DatabaseError> {
        trace!("Updating album");
        let song_count = album.album_song_count as i64;
        sqlx::query!(
            "UPDATE albums SET
                album_name = ?1, album_artist = ?2, album_coverpath_high = ?3,
                album_song_count = ?4, year = ?5, album_coverpath_low = ?6
             WHERE album_id = ?7",
            album.album_name,
            album.album_artist,
            album.album_coverpath_high,
            song_count,
            album.year,
            album.album_coverpath_low,
            album.album_id,
        )
        .execute(&self.pools.write)
        .await
        .map_err(DatabaseError::Query)?;
        info!("Updated album");
        Ok(())
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub async fn update_artist(&self, artist: &Artist) -> Result<(), DatabaseError> {
        trace!("Updating artist");
        let song_count = artist.artist_song_count as i64;
        sqlx::query!(
            "UPDATE artists SET
                artist_mbid = ?1, artist_name = ?2, artist_coverpath = ?3,
                artist_song_count = ?4, sanitized_artist_name = ?5
             WHERE artist_id = ?6",
            artist.artist_mbid,
            artist.artist_name,
            artist.artist_coverpath,
            song_count,
            artist.sanitized_artist_name,
            artist.artist_id,
        )
        .execute(&self.pools.write)
        .await
        .map_err(DatabaseError::Query)?;
        info!("Updated artist");
        Ok(())
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub async fn get_albums(
        &self,
        options: Album,
        inclusive: bool,
    ) -> Result<Vec<Album>, DatabaseError> {
        let mut qb: QueryBuilder<'_, Sqlite> = QueryBuilder::new(
            "SELECT album_id, album_name, album_artist, album_coverpath_high, album_song_count, year, album_coverpath_low FROM albums",
        );
        let mut has_where = false;
        let joiner = if inclusive { " AND " } else { " OR " };

        if let Some(v) = options.album_id {
            push_cond(&mut qb, &mut has_where, joiner);
            qb.push("album_id = ").push_bind(v);
        }
        if let Some(v) = options.album_name {
            push_cond(&mut qb, &mut has_where, joiner);
            qb.push("album_name LIKE ").push_bind(v);
        }

        let rows: Vec<DbAlbumRow> = qb
            .build_query_as()
            .fetch_all(&self.pools.read)
            .await
            .map_err(DatabaseError::Query)?;
        Ok(rows.into_iter().map(Into::into).collect())
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub async fn get_artists(
        &self,
        options: Artist,
        inclusive: bool,
    ) -> Result<Vec<Artist>, DatabaseError> {
        let mut qb: QueryBuilder<'_, Sqlite> = QueryBuilder::new(
            "SELECT artist_id, artist_mbid, artist_name, artist_coverpath, artist_song_count, sanitized_artist_name FROM artists",
        );
        let mut has_where = false;
        let joiner = if inclusive { " AND " } else { " OR " };

        if let Some(v) = options.artist_id {
            push_cond(&mut qb, &mut has_where, joiner);
            qb.push("artist_id = ").push_bind(v);
        }
        if let Some(v) = options.artist_name {
            push_cond(&mut qb, &mut has_where, joiner);
            qb.push("artist_name LIKE ").push_bind(v);
        }
        if let Some(v) = options.artist_mbid {
            push_cond(&mut qb, &mut has_where, joiner);
            qb.push("artist_mbid = ").push_bind(v);
        }

        let rows: Vec<DbArtistRow> = qb
            .build_query_as()
            .fetch_all(&self.pools.read)
            .await
            .map_err(DatabaseError::Query)?;
        Ok(rows.into_iter().map(Into::into).collect())
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub async fn get_genres(
        &self,
        options: Genre,
        inclusive: bool,
    ) -> Result<Vec<Genre>, DatabaseError> {
        let mut qb: QueryBuilder<'_, Sqlite> =
            QueryBuilder::new("SELECT genre_id, genre_name, genre_song_count FROM genres");
        let mut has_where = false;
        let joiner = if inclusive { " AND " } else { " OR " };

        if let Some(v) = options.genre_id {
            push_cond(&mut qb, &mut has_where, joiner);
            qb.push("genre_id = ").push_bind(v);
        }
        if let Some(v) = options.genre_name {
            push_cond(&mut qb, &mut has_where, joiner);
            qb.push("genre_name LIKE ").push_bind(v);
        }

        let rows: Vec<DbGenreRow> = qb
            .build_query_as()
            .fetch_all(&self.pools.read)
            .await
            .map_err(DatabaseError::Query)?;
        Ok(rows.into_iter().map(Into::into).collect())
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub async fn get_playlists(
        &self,
        options: Playlist,
        inclusive: bool,
    ) -> Result<Vec<Playlist>, DatabaseError> {
        let mut qb: QueryBuilder<'_, Sqlite> = QueryBuilder::new(
            "SELECT playlist_id, playlist_name, playlist_coverpath, playlist_song_count, playlist_desc, playlist_path, extension, icon, library_item FROM playlists",
        );
        let mut has_where = false;
        let joiner = if inclusive { " AND " } else { " OR " };

        if let Some(v) = options.playlist_id {
            push_cond(&mut qb, &mut has_where, joiner);
            qb.push("playlist_id = ").push_bind(v);
        }
        if !options.playlist_name.is_empty() {
            push_cond(&mut qb, &mut has_where, joiner);
            qb.push("playlist_name LIKE ")
                .push_bind(options.playlist_name);
        }
        if let Some(v) = options.playlist_path {
            push_cond(&mut qb, &mut has_where, joiner);
            qb.push("playlist_path LIKE ").push_bind(v);
        }
        if let Some(v) = options.playlist_desc {
            push_cond(&mut qb, &mut has_where, joiner);
            qb.push("playlist_desc LIKE ").push_bind(v);
        }

        let rows: Vec<DbPlaylistRow> = qb
            .build_query_as()
            .fetch_all(&self.pools.read)
            .await
            .map_err(DatabaseError::Query)?;
        Ok(rows.into_iter().map(Into::into).collect())
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub async fn get_songs_by_options(
        &self,
        options: GetSongOptions,
    ) -> Result<Vec<Song>, DatabaseError> {
        trace!("Getting songs by options");
        let inclusive = options.inclusive.unwrap_or_default();
        let fetched_songs = self.fetch_inner_songs(&options, inclusive).await?;

        if fetched_songs.is_empty() {
            return Ok(vec![]);
        }

        let song_ids: Vec<String> = fetched_songs.iter().filter_map(|s| s.id.clone()).collect();
        let mut albums_map = self.fetch_song_albums(&song_ids).await?;
        let mut artists_map = self.fetch_song_artists(&song_ids).await?;
        let mut genres_map = self.fetch_song_genres(&song_ids).await?;

        let mut ret = Vec::with_capacity(fetched_songs.len());
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

    #[tracing::instrument(level = "debug", skip_all)]
    async fn fetch_inner_songs(
        &self,
        options: &GetSongOptions,
        inclusive: bool,
    ) -> Result<Vec<InnerSong>, DatabaseError> {
        let mut qb: QueryBuilder<'_, Sqlite> = QueryBuilder::new(ALLSONGS_SELECT);
        let joiner = if inclusive { " AND " } else { " OR " };
        let mut has_where = false;

        if let Some(ref album) = options.album {
            qb.push(" INNER JOIN albums al ON a.album_id = al.album_id");
            if let Some(ref v) = album.album_id {
                push_cond(&mut qb, &mut has_where, joiner);
                qb.push("al.album_id = ").push_bind(v.clone());
            }
            if let Some(ref v) = album.album_name {
                push_cond(&mut qb, &mut has_where, joiner);
                qb.push("al.album_name LIKE ").push_bind(v.clone());
            }
        }

        if let Some(ref artist) = options.artist {
            qb.push(" INNER JOIN artist_bridge arb ON a._id = arb.song INNER JOIN artists ar ON arb.artist = ar.artist_id");
            if let Some(ref v) = artist.artist_id {
                push_cond(&mut qb, &mut has_where, joiner);
                qb.push("ar.artist_id = ").push_bind(v.clone());
            }
            if let Some(ref v) = artist.artist_name {
                push_cond(&mut qb, &mut has_where, joiner);
                qb.push("ar.artist_name LIKE ").push_bind(v.clone());
            }
            if let Some(ref v) = artist.artist_mbid {
                push_cond(&mut qb, &mut has_where, joiner);
                qb.push("ar.artist_mbid = ").push_bind(v.clone());
            }
        }

        if let Some(ref genre) = options.genre {
            qb.push(" INNER JOIN genre_bridge gb ON a._id = gb.song INNER JOIN genres g ON gb.genre = g.genre_id");
            if let Some(ref v) = genre.genre_id {
                push_cond(&mut qb, &mut has_where, joiner);
                qb.push("g.genre_id = ").push_bind(v.clone());
            }
            if let Some(ref v) = genre.genre_name {
                push_cond(&mut qb, &mut has_where, joiner);
                qb.push("g.genre_name LIKE ").push_bind(v.clone());
            }
        }

        if let Some(ref playlist) = options.playlist {
            qb.push(" INNER JOIN playlist_bridge pb ON a._id = pb.song INNER JOIN playlists p ON pb.playlist = p.playlist_id");
            if let Some(ref v) = playlist.playlist_id {
                push_cond(&mut qb, &mut has_where, joiner);
                qb.push("p.playlist_id = ").push_bind(v.clone());
            }
            if !playlist.playlist_name.is_empty() {
                push_cond(&mut qb, &mut has_where, joiner);
                qb.push("p.playlist_name LIKE ")
                    .push_bind(playlist.playlist_name.clone());
            }
            if let Some(ref v) = playlist.playlist_path {
                push_cond(&mut qb, &mut has_where, joiner);
                qb.push("p.playlist_path LIKE ").push_bind(v.clone());
            }
        }

        if let Some(ref song) = options.song {
            apply_song_filters(&mut qb, song, &mut has_where, joiner);
        }

        if options.playlist.is_some() {
            qb.push(" ORDER BY pb.position ASC");
        }

        let rows: Vec<DbInnerSongRow> = qb
            .build_query_as()
            .fetch_all(&self.pools.read)
            .await
            .map_err(DatabaseError::Query)?;
        Ok(rows.into_iter().map(Into::into).collect())
    }

    #[tracing::instrument(level = "debug", skip_all)]
    async fn fetch_song_albums(
        &self,
        song_ids: &[String],
    ) -> Result<HashMap<String, Album>, DatabaseError> {
        #[derive(sqlx::FromRow)]
        struct SongAlbumRow {
            song_id: String,
            album_id: Option<String>,
            album_name: Option<String>,
            album_artist: Option<String>,
            album_coverpath_high: Option<String>,
            album_song_count: i64,
            year: Option<String>,
            album_coverpath_low: Option<String>,
        }

        let mut map = HashMap::new();
        for chunk in song_ids.chunks(500) {
            let mut qb: QueryBuilder<'_, Sqlite> = QueryBuilder::new(
                "SELECT a._id AS song_id, b.album_id, b.album_name, b.album_artist, \
                 b.album_coverpath_high, b.album_song_count, b.year, b.album_coverpath_low \
                 FROM allsongs a INNER JOIN albums b ON a.album_id = b.album_id WHERE a._id IN (",
            );
            let mut sep = qb.separated(", ");
            for id in chunk {
                sep.push_bind(id);
            }
            sep.push_unseparated(")");

            let rows: Vec<SongAlbumRow> = qb
                .build_query_as()
                .fetch_all(&self.pools.read)
                .await
                .map_err(DatabaseError::Query)?;
            for r in rows {
                map.insert(
                    r.song_id,
                    Album {
                        album_id: r.album_id,
                        album_name: r.album_name,
                        album_artist: r.album_artist,
                        album_coverpath_high: r.album_coverpath_high,
                        album_song_count: r.album_song_count as f64,
                        year: r.year,
                        album_coverpath_low: r.album_coverpath_low,
                        extension: None,
                    },
                );
            }
        }
        Ok(map)
    }

    #[tracing::instrument(level = "debug", skip_all)]
    async fn fetch_song_artists(
        &self,
        song_ids: &[String],
    ) -> Result<HashMap<String, Vec<Artist>>, DatabaseError> {
        #[derive(sqlx::FromRow)]
        struct SongArtistRow {
            song_id: String,
            artist_id: Option<String>,
            artist_mbid: Option<String>,
            artist_name: Option<String>,
            artist_coverpath: Option<String>,
            artist_song_count: i64,
            sanitized_artist_name: Option<String>,
        }

        let mut map: HashMap<String, Vec<Artist>> = HashMap::new();
        for chunk in song_ids.chunks(500) {
            let mut qb: QueryBuilder<'_, Sqlite> = QueryBuilder::new(
                "SELECT ab.song AS song_id, ar.artist_id, ar.artist_mbid, ar.artist_name, \
                 ar.artist_coverpath, ar.artist_song_count, ar.sanitized_artist_name \
                 FROM artist_bridge ab INNER JOIN artists ar ON ab.artist = ar.artist_id WHERE ab.song IN (",
            );
            let mut sep = qb.separated(", ");
            for id in chunk {
                sep.push_bind(id);
            }
            sep.push_unseparated(")");

            let rows: Vec<SongArtistRow> = qb
                .build_query_as()
                .fetch_all(&self.pools.read)
                .await
                .map_err(DatabaseError::Query)?;
            for r in rows {
                map.entry(r.song_id).or_default().push(Artist {
                    artist_id: r.artist_id,
                    artist_mbid: r.artist_mbid,
                    artist_name: r.artist_name,
                    artist_coverpath: r.artist_coverpath,
                    artist_song_count: r.artist_song_count as f64,
                    sanitized_artist_name: r.sanitized_artist_name,
                    extension: None,
                });
            }
        }
        Ok(map)
    }

    #[tracing::instrument(level = "debug", skip_all)]
    async fn fetch_song_genres(
        &self,
        song_ids: &[String],
    ) -> Result<HashMap<String, Vec<Genre>>, DatabaseError> {
        #[derive(sqlx::FromRow)]
        struct SongGenreRow {
            song_id: String,
            genre_id: Option<String>,
            genre_name: Option<String>,
            genre_song_count: i64,
        }

        let mut map: HashMap<String, Vec<Genre>> = HashMap::new();
        for chunk in song_ids.chunks(500) {
            let mut qb: QueryBuilder<'_, Sqlite> = QueryBuilder::new(
                "SELECT gb.song AS song_id, g.genre_id, g.genre_name, g.genre_song_count \
                 FROM genre_bridge gb INNER JOIN genres g ON gb.genre = g.genre_id WHERE gb.song IN (",
            );
            let mut sep = qb.separated(", ");
            for id in chunk {
                sep.push_bind(id);
            }
            sep.push_unseparated(")");

            let rows: Vec<SongGenreRow> = qb
                .build_query_as()
                .fetch_all(&self.pools.read)
                .await
                .map_err(DatabaseError::Query)?;
            for r in rows {
                map.entry(r.song_id).or_default().push(Genre {
                    genre_id: r.genre_id,
                    genre_name: r.genre_name,
                    genre_song_count: r.genre_song_count as f64,
                });
            }
        }
        Ok(map)
    }
}

#[tracing::instrument(level = "debug", skip_all)]
fn apply_song_filters(
    qb: &mut QueryBuilder<'_, Sqlite>,
    song: &SearchableSong,
    has_where: &mut bool,
    joiner: &str,
) {
    if let Some(ref v) = song.id {
        push_cond(qb, has_where, joiner);
        qb.push("a._id = ").push_bind(v.clone());
    }
    if let Some(ref v) = song.path {
        push_cond(qb, has_where, joiner);
        qb.push("a.path LIKE ").push_bind(v.clone());
    }
    if let Some(ref v) = song.title {
        push_cond(qb, has_where, joiner);
        qb.push("a.title LIKE ").push_bind(v.clone());
    }
    if let Some(v) = song.sample_rate {
        push_cond(qb, has_where, joiner);
        qb.push("a.samplerate = ").push_bind(v);
    }
    if let Some(ref v) = song.hash {
        push_cond(qb, has_where, joiner);
        qb.push("a.hash = ").push_bind(v.clone());
    }
    if let Some(v) = song.r#type {
        push_cond(qb, has_where, joiner);
        qb.push("a.type = ").push_bind(song_type_to_str(v));
    }
    if let Some(ref v) = song.url {
        push_cond(qb, has_where, joiner);
        qb.push("a.url = ").push_bind(v.clone());
    }
    if let Some(ref v) = song.playback_url {
        push_cond(qb, has_where, joiner);
        qb.push("a.playbackurl = ").push_bind(v.encode_to_vec());
    }
    if let Some(v) = song.show_in_library {
        push_cond(qb, has_where, joiner);
        qb.push("a.show_in_library = ").push_bind(v);
    }
}

#[tracing::instrument(level = "debug", skip_all)]
fn push_cond(qb: &mut QueryBuilder<'_, Sqlite>, has_where: &mut bool, joiner: &str) {
    if !*has_where {
        qb.push(" WHERE ");
        *has_where = true;
    } else {
        qb.push(joiner);
    }
}
