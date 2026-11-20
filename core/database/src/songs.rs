use std::{
    cmp::min,
    collections::{HashMap, HashSet},
    fmt::Debug,
    path::PathBuf,
};

use prost::Message;
use songs_proto::moosync::types::{Album, Artist, Genre, InnerSong, Lyrics, Song};
use sqlx::{QueryBuilder, Row, Sqlite, Transaction};
use tracing::{debug, info, trace};
use uuid::Uuid;

use crate::{
    entities::EntityRepository,
    error::DatabaseError,
    pool::DbPools,
    utils::{proto_to_db_ms, song_type_to_str},
};

struct DbSongInsertRow {
    id: Option<String>,
    album_id: Option<String>,
    path: Option<String>,
    size: Option<f64>,
    title: Option<String>,
    date: Option<String>,
    year: Option<String>,
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
    track_no: Option<f64>,
    lyrics: Option<Vec<u8>>,
}

#[derive(Clone)]
pub struct SongRepository {
    pools: DbPools,
    entities: EntityRepository,
}

impl SongRepository {
    #[tracing::instrument(level = "debug", skip_all)]
    pub fn new(pools: DbPools) -> Self {
        let entities = EntityRepository::new(pools.clone());
        Self { pools, entities }
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub async fn insert_songs_by_ref(&self, songs: &mut [Song]) -> Result<(), DatabaseError> {
        trace!("Inserting songs");
        let (unique_albums, unique_artists, unique_genres) = gather_unique_metadata(songs);

        let mut tx = self
            .pools
            .write
            .begin()
            .await
            .map_err(DatabaseError::Transaction)?;

        let existing_song_ids = fetch_existing_song_ids(&mut tx, songs).await?;
        let existing_ids: Vec<&String> = existing_song_ids.values().collect();
        delete_existing_bridge_entries(&mut tx, "artist_bridge", &existing_ids).await?;
        delete_existing_bridge_entries(&mut tx, "genre_bridge", &existing_ids).await?;

        let album_names: Vec<String> = unique_albums.keys().cloned().collect();
        let mut album_ids =
            fetch_existing_entities(&mut tx, "albums", "album_id", "album_name", &album_names)
                .await?;

        let artist_names: Vec<String> = unique_artists.keys().cloned().collect();
        let mut artist_ids = fetch_existing_entities(
            &mut tx,
            "artists",
            "artist_id",
            "artist_name",
            &artist_names,
        )
        .await?;

        let genre_names: Vec<String> = unique_genres.keys().cloned().collect();
        let mut genre_ids =
            fetch_existing_entities(&mut tx, "genres", "genre_id", "genre_name", &genre_names)
                .await?;

        insert_new_albums(&mut tx, &unique_albums, &mut album_ids).await?;
        insert_new_artists(&mut tx, &unique_artists, &mut artist_ids).await?;
        insert_new_genres(&mut tx, &unique_genres, &mut genre_ids).await?;

        update_song_metadata_ids(
            songs,
            &existing_song_ids,
            &album_ids,
            &artist_ids,
            &genre_ids,
        );

        let prepared = build_db_song_rows_and_bridges(songs);

        insert_bulk_db_song_rows(&mut tx, &prepared.rows).await?;
        insert_bulk_lyrics(&mut tx, &prepared.rows).await?;
        insert_bulk_bridge_entries(
            &mut tx,
            "artist_bridge",
            "song",
            "artist",
            &prepared.artist_bridges,
        )
        .await?;
        insert_bulk_bridge_entries(
            &mut tx,
            "genre_bridge",
            "song",
            "genre",
            &prepared.genre_bridges,
        )
        .await?;

        tx.commit().await.map_err(DatabaseError::Transaction)?;
        info!("Inserted all songs");
        Ok(())
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub async fn remove_songs<T: AsRef<str> + Debug>(
        &self,
        ids: &[T],
    ) -> Result<(), DatabaseError> {
        if ids.is_empty() {
            return Ok(());
        }
        trace!("Removing songs");
        for chunk in ids.chunks(500) {
            let mut qb: QueryBuilder<'_, Sqlite> =
                QueryBuilder::new("DELETE FROM allsongs WHERE _id IN (");
            let mut sep = qb.separated(", ");
            for id in chunk {
                sep.push_bind(id.as_ref());
            }
            sep.push_unseparated(")");
            qb.build()
                .execute(&self.pools.write)
                .await
                .map_err(DatabaseError::Query)?;
        }
        info!("Removed songs");
        Ok(())
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub async fn remove_songs_outside_directories(
        &self,
        scan_dirs: &[PathBuf],
    ) -> Result<(), DatabaseError> {
        if scan_dirs.is_empty() {
            return Ok(());
        }

        let rows =
            sqlx::query!("SELECT _id, path FROM allsongs WHERE path IS NOT NULL AND path != ''")
                .fetch_all(&self.pools.read)
                .await
                .map_err(DatabaseError::Query)?;

        let mut ids_to_remove = Vec::new();
        for row in rows {
            if let Some(path_str) = row.path {
                let song_path = std::path::Path::new(&path_str);
                let matches = scan_dirs.iter().any(|dir| song_path.starts_with(dir));
                if !matches {
                    ids_to_remove.push(row._id);
                }
            }
        }

        if !ids_to_remove.is_empty() {
            info!(
                "Removing {} songs outside scan directories",
                ids_to_remove.len()
            );
            self.remove_songs(&ids_to_remove).await?;
        }
        Ok(())
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub async fn update_song(&self, song: &InnerSong) -> Result<bool, DatabaseError> {
        trace!("Updating song");
        let Some(id) = song.id.as_ref() else {
            debug!("Song does not have an ID");
            return Ok(false);
        };

        let song_type = song_type_to_str(song.r#type);
        let duration_ms = proto_to_db_ms(&song.duration);

        sqlx::query!(
            "UPDATE allsongs SET
                path = ?1, size = ?2, title = ?3, date = ?4, year = ?5,
                releasetype = ?6, bitrate = ?7, codec = ?8, container = ?9,
                duration = ?10, samplerate = ?11, hash = ?12, type = ?13,
                url = ?14, song_coverpath_high = ?15, playbackurl = ?16,
                song_coverpath_low = ?17, date_added = ?18, track_no = ?19
             WHERE _id = ?20",
            song.path,
            song.size,
            song.title,
            song.date,
            song.year,
            song.release_type,
            song.bitrate,
            song.codec,
            song.container,
            duration_ms,
            song.sample_rate,
            song.hash,
            song_type,
            song.url,
            song.song_cover_path_high,
            song.playback_url,
            song.song_cover_path_low,
            song.date_added,
            song.track_no,
            id,
        )
        .execute(&self.pools.write)
        .await
        .map_err(DatabaseError::Query)?;

        if let Some(ref lyrics) = song.lyrics {
            self.update_lyrics(id, lyrics).await?;
        }

        debug!("Updated song");
        Ok(true)
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub async fn update_songs(&self, songs: &[Song]) -> Result<(), DatabaseError> {
        trace!("Updating multiple songs");
        let mut tx = self
            .pools
            .write
            .begin()
            .await
            .map_err(DatabaseError::Transaction)?;

        for song in songs {
            let Some(ref inner) = song.song else {
                continue;
            };
            let Some(ref id) = inner.id else {
                continue;
            };
            let song_type = song_type_to_str(inner.r#type);
            let duration_ms = proto_to_db_ms(&inner.duration);

            sqlx::query!(
                "UPDATE allsongs SET
                    path = ?1, size = ?2, title = ?3, date = ?4, year = ?5,
                    releasetype = ?6, bitrate = ?7, codec = ?8, container = ?9,
                    duration = ?10, samplerate = ?11, hash = ?12, type = ?13,
                    url = ?14, song_coverpath_high = ?15, playbackurl = ?16,
                    song_coverpath_low = ?17, date_added = ?18, track_no = ?19
                 WHERE _id = ?20",
                inner.path,
                inner.size,
                inner.title,
                inner.date,
                inner.year,
                inner.release_type,
                inner.bitrate,
                inner.codec,
                inner.container,
                duration_ms,
                inner.sample_rate,
                inner.hash,
                song_type,
                inner.url,
                inner.song_cover_path_high,
                inner.playback_url,
                inner.song_cover_path_low,
                inner.date_added,
                inner.track_no,
                id,
            )
            .execute(&mut *tx)
            .await
            .map_err(DatabaseError::Query)?;
        }

        tx.commit().await.map_err(DatabaseError::Transaction)?;

        for song in songs {
            if let Some(ref album) = song.album
                && album.album_id.is_some()
            {
                self.entities.update_album(album).await?;
            }
            for artist in &song.artists {
                if artist.artist_id.is_some() {
                    self.entities.update_artist(artist).await?;
                }
            }
        }
        Ok(())
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub async fn files_not_in_db(
        &self,
        mut file_list: Vec<(PathBuf, f64)>,
    ) -> Result<Vec<(PathBuf, f64)>, DatabaseError> {
        let len = file_list.len();
        let mut ret = Vec::new();
        let exp_limit = 300;

        for _ in 0..len / exp_limit + 1 {
            let curr_len = min(file_list.len(), exp_limit);
            if curr_len == 0 {
                break;
            }

            let mut temp_paths = Vec::with_capacity(curr_len);
            for _ in 0..curr_len {
                let data = file_list.pop().unwrap();
                let path_str = data.0.to_string_lossy().to_string();
                temp_paths.push((path_str, data.1));
            }

            let missing = self.find_missing_in_chunk(temp_paths).await?;
            ret.extend(missing);
        }

        Ok(ret)
    }

    #[tracing::instrument(level = "debug", skip_all)]
    async fn find_missing_in_chunk(
        &self,
        temp_paths: Vec<(String, f64)>,
    ) -> Result<Vec<(PathBuf, f64)>, DatabaseError> {
        let mut qb: QueryBuilder<'_, Sqlite> =
            QueryBuilder::new("SELECT path, size FROM allsongs WHERE ");
        for (i, (p, s)) in temp_paths.iter().enumerate() {
            if i > 0 {
                qb.push(" OR ");
            }
            qb.push("(path = ")
                .push_bind(p.clone())
                .push(" AND size = ")
                .push_bind(*s)
                .push(")");
        }

        let rows = qb
            .build()
            .fetch_all(&self.pools.read)
            .await
            .map_err(DatabaseError::Query)?;

        let mut existing = HashSet::new();
        for row in rows {
            let path: String = row.try_get(0).map_err(DatabaseError::Query)?;
            let size: f64 = row.try_get(1).map_err(DatabaseError::Query)?;
            existing.insert((path, size.to_bits()));
        }

        Ok(temp_paths
            .into_iter()
            .filter(|(p, s)| !existing.contains(&(p.clone(), s.to_bits())))
            .map(|(p, s)| (PathBuf::from(p), s))
            .collect())
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub async fn get_lyrics(&self, id: &str) -> Result<Option<Lyrics>, DatabaseError> {
        trace!("Getting lyrics for song {id}");
        let lyrics_bytes: Option<Vec<u8>> =
            sqlx::query_scalar!("SELECT lyrics FROM song_lyrics WHERE song_id = ?1", id)
                .fetch_optional(&self.pools.read)
                .await
                .map_err(DatabaseError::Query)?;

        let lyrics = match lyrics_bytes {
            Some(bytes) => match Lyrics::decode(bytes.as_slice()) {
                Ok(l) => Some(l),
                Err(e) => {
                    tracing::error!("Failed to decode lyrics protobuf: {e:?}");
                    None
                }
            },
            None => None,
        };
        info!("Fetched lyrics for song {id}");
        Ok(lyrics)
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub async fn update_lyrics(&self, id: &str, lyrics: &Lyrics) -> Result<(), DatabaseError> {
        trace!("Updating lyrics for song {id}");
        let lyrics_bytes = lyrics.encode_to_vec();
        sqlx::query!(
            "INSERT INTO song_lyrics (song_id, lyrics) VALUES (?1, ?2)
             ON CONFLICT(song_id) DO UPDATE SET lyrics = excluded.lyrics",
            id,
            lyrics_bytes,
        )
        .execute(&self.pools.write)
        .await
        .map_err(DatabaseError::Query)?;
        info!("Updated lyrics for song {id}");
        Ok(())
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub async fn update_song_cover(
        &self,
        id: &str,
        cover_high: Option<&str>,
        cover_low: Option<&str>,
    ) -> Result<(), DatabaseError> {
        trace!("Updating cover for song {id}");
        sqlx::query!(
            "UPDATE allsongs SET song_coverpath_high = ?1, song_coverpath_low = ?2 WHERE _id = ?3",
            cover_high,
            cover_low,
            id,
        )
        .execute(&self.pools.write)
        .await
        .map_err(DatabaseError::Query)?;
        info!("Updated cover for song {id}");
        Ok(())
    }
}

#[tracing::instrument(level = "debug", skip_all)]
fn gather_unique_metadata(
    songs: &[Song],
) -> (
    HashMap<String, Album>,
    HashMap<String, Artist>,
    HashMap<String, Genre>,
) {
    let mut unique_albums = HashMap::new();
    let mut unique_artists = HashMap::new();
    let mut unique_genres = HashMap::new();

    for song in songs {
        if let Some(album) = &song.album
            && let Some(ref name) = album.album_name
            && !name.is_empty()
        {
            unique_albums
                .entry(name.clone())
                .or_insert_with(|| album.clone());
        }
        for artist in &song.artists {
            if let Some(ref name) = artist.artist_name
                && !name.is_empty()
            {
                unique_artists
                    .entry(name.clone())
                    .or_insert_with(|| artist.clone());
            }
        }
        for genre in &song.genre {
            if let Some(ref name) = genre.genre_name
                && !name.is_empty()
            {
                unique_genres
                    .entry(name.clone())
                    .or_insert_with(|| genre.clone());
            }
        }
    }
    (unique_albums, unique_artists, unique_genres)
}

#[tracing::instrument(level = "debug", skip_all)]
async fn fetch_existing_song_ids(
    tx: &mut Transaction<'_, Sqlite>,
    songs: &[Song],
) -> Result<HashMap<String, String>, DatabaseError> {
    let paths: Vec<String> = songs
        .iter()
        .filter_map(|s| s.song.as_ref()?.path.clone())
        .filter(|p| !p.is_empty())
        .collect();
    let mut existing = HashMap::new();
    for chunk in paths.chunks(500) {
        let mut qb: QueryBuilder<'_, Sqlite> =
            QueryBuilder::new("SELECT path, _id FROM allsongs WHERE path IN (");
        let mut sep = qb.separated(", ");
        for p in chunk {
            sep.push_bind(p);
        }
        sep.push_unseparated(")");
        let rows = qb
            .build()
            .fetch_all(&mut **tx)
            .await
            .map_err(DatabaseError::Query)?;
        for row in rows {
            let path: String = row.try_get(0).map_err(DatabaseError::Query)?;
            let id: String = row.try_get(1).map_err(DatabaseError::Query)?;
            existing.insert(path, id);
        }
    }
    Ok(existing)
}

#[tracing::instrument(level = "debug", skip_all)]
async fn delete_existing_bridge_entries(
    tx: &mut Transaction<'_, Sqlite>,
    table: &str,
    ids: &[&String],
) -> Result<(), DatabaseError> {
    for chunk in ids.chunks(500) {
        let mut qb: QueryBuilder<'_, Sqlite> =
            QueryBuilder::new(format!("DELETE FROM {table} WHERE song IN ("));
        let mut sep = qb.separated(", ");
        for id in chunk {
            sep.push_bind(id.as_str());
        }
        sep.push_unseparated(")");
        qb.build()
            .execute(&mut **tx)
            .await
            .map_err(DatabaseError::Query)?;
    }
    Ok(())
}

#[tracing::instrument(level = "debug", skip_all)]
async fn fetch_existing_entities(
    tx: &mut Transaction<'_, Sqlite>,
    table: &str,
    id_col: &str,
    name_col: &str,
    names: &[String],
) -> Result<HashMap<String, String>, DatabaseError> {
    let mut entity_ids = HashMap::new();
    for chunk in names.chunks(500) {
        let mut qb: QueryBuilder<'_, Sqlite> = QueryBuilder::new(format!(
            "SELECT {id_col}, {name_col} FROM {table} WHERE {name_col} IN ("
        ));
        let mut sep = qb.separated(", ");
        for name in chunk {
            sep.push_bind(name);
        }
        sep.push_unseparated(")");
        let rows = qb
            .build()
            .fetch_all(&mut **tx)
            .await
            .map_err(DatabaseError::Query)?;
        for row in rows {
            let id: String = row.try_get(0).map_err(DatabaseError::Query)?;
            let name: String = row.try_get(1).map_err(DatabaseError::Query)?;
            entity_ids.insert(name, id);
        }
    }
    Ok(entity_ids)
}

#[tracing::instrument(level = "debug", skip_all)]
async fn insert_new_albums(
    tx: &mut Transaction<'_, Sqlite>,
    unique_albums: &HashMap<String, Album>,
    album_ids: &mut HashMap<String, String>,
) -> Result<(), DatabaseError> {
    let mut new_albums = Vec::new();
    for (name, album) in unique_albums {
        if !album_ids.contains_key(name) {
            let id = Uuid::new_v4().to_string();
            album_ids.insert(name.clone(), id.clone());
            let mut alb = album.clone();
            alb.album_id = Some(id);
            new_albums.push(alb);
        }
    }
    for chunk in new_albums.chunks(100) {
        let mut qb: QueryBuilder<'_, Sqlite> = QueryBuilder::new(
            "INSERT INTO albums (album_id, album_name, album_artist, album_coverpath_high, album_song_count, year, album_coverpath_low) ",
        );
        qb.push_values(chunk, |mut b, alb| {
            b.push_bind(alb.album_id.as_ref().unwrap())
                .push_bind(&alb.album_name)
                .push_bind(&alb.album_artist)
                .push_bind(&alb.album_coverpath_high)
                .push_bind(alb.album_song_count as i64)
                .push_bind(&alb.year)
                .push_bind(&alb.album_coverpath_low);
        });
        qb.build()
            .execute(&mut **tx)
            .await
            .map_err(DatabaseError::Query)?;
    }
    Ok(())
}

#[tracing::instrument(level = "debug", skip_all)]
async fn insert_new_artists(
    tx: &mut Transaction<'_, Sqlite>,
    unique_artists: &HashMap<String, Artist>,
    artist_ids: &mut HashMap<String, String>,
) -> Result<(), DatabaseError> {
    let mut new_artists = Vec::new();
    for (name, artist) in unique_artists {
        if !artist_ids.contains_key(name) {
            let id = Uuid::new_v4().to_string();
            artist_ids.insert(name.clone(), id.clone());
            let mut art = artist.clone();
            art.artist_id = Some(id);
            new_artists.push(art);
        }
    }
    for chunk in new_artists.chunks(100) {
        let mut qb: QueryBuilder<'_, Sqlite> = QueryBuilder::new(
            "INSERT INTO artists (artist_id, artist_mbid, artist_name, artist_coverpath, artist_song_count, sanitized_artist_name) ",
        );
        qb.push_values(chunk, |mut b, art| {
            b.push_bind(art.artist_id.as_ref().unwrap())
                .push_bind(&art.artist_mbid)
                .push_bind(&art.artist_name)
                .push_bind(&art.artist_coverpath)
                .push_bind(art.artist_song_count as i64)
                .push_bind(&art.sanitized_artist_name);
        });
        qb.build()
            .execute(&mut **tx)
            .await
            .map_err(DatabaseError::Query)?;
    }
    Ok(())
}

#[tracing::instrument(level = "debug", skip_all)]
async fn insert_new_genres(
    tx: &mut Transaction<'_, Sqlite>,
    unique_genres: &HashMap<String, Genre>,
    genre_ids: &mut HashMap<String, String>,
) -> Result<(), DatabaseError> {
    let mut new_genres = Vec::new();
    for (name, genre) in unique_genres {
        if !genre_ids.contains_key(name) {
            let id = Uuid::new_v4().to_string();
            genre_ids.insert(name.clone(), id.clone());
            let mut new_genre = genre.clone();
            new_genre.genre_id = Some(id);
            new_genres.push(new_genre);
        }
    }
    for chunk in new_genres.chunks(100) {
        let mut qb: QueryBuilder<'_, Sqlite> =
            QueryBuilder::new("INSERT INTO genres (genre_id, genre_name, genre_song_count) ");
        qb.push_values(chunk, |mut b, new_genre| {
            b.push_bind(new_genre.genre_id.as_ref().unwrap())
                .push_bind(&new_genre.genre_name)
                .push_bind(new_genre.genre_song_count as i64);
        });
        qb.build()
            .execute(&mut **tx)
            .await
            .map_err(DatabaseError::Query)?;
    }
    Ok(())
}

#[tracing::instrument(level = "debug", skip_all)]
fn update_song_metadata_ids(
    songs: &mut [Song],
    existing_song_ids: &HashMap<String, String>,
    album_ids: &HashMap<String, String>,
    artist_ids: &HashMap<String, String>,
    genre_ids: &HashMap<String, String>,
) {
    for song in songs.iter_mut() {
        if let Some(inner_song) = song.song.as_mut() {
            let existing = inner_song
                .path
                .as_deref()
                .filter(|p| !p.is_empty())
                .and_then(|p| existing_song_ids.get(p).cloned());
            inner_song.id = existing
                .or_else(|| inner_song.id.take())
                .or_else(|| Some(Uuid::new_v4().to_string()));
        }
        if let Some(album) = song.album.as_mut()
            && let Some(name) = &album.album_name
            && let Some(id) = album_ids.get(name)
        {
            album.album_id = Some(id.clone());
        }
        for artist in &mut song.artists {
            if let Some(name) = &artist.artist_name
                && let Some(id) = artist_ids.get(name)
            {
                artist.artist_id = Some(id.clone());
            }
        }
        for genre in &mut song.genre {
            if let Some(name) = &genre.genre_name
                && let Some(id) = genre_ids.get(name)
            {
                genre.genre_id = Some(id.clone());
            }
        }
    }
}

struct SongRowsAndBridges {
    rows: Vec<DbSongInsertRow>,
    artist_bridges: Vec<(String, String)>,
    genre_bridges: Vec<(String, String)>,
}

#[tracing::instrument(level = "debug", skip_all)]
fn build_db_song_rows_and_bridges(songs: &[Song]) -> SongRowsAndBridges {
    let mut rows = Vec::with_capacity(songs.len());
    let mut artist_bridges = Vec::new();
    let mut genre_bridges = Vec::new();

    for song in songs {
        let Some(inner) = &song.song else {
            continue;
        };
        let song_id = inner.id.clone().unwrap_or_default();
        let album_id = song.album.as_ref().and_then(|a| a.album_id.clone());

        rows.push(DbSongInsertRow {
            id: inner.id.clone(),
            album_id,
            path: inner.path.clone(),
            size: inner.size,
            title: inner.title.clone(),
            date: inner.date.clone(),
            year: inner.year.clone(),
            release_type: inner.release_type.clone(),
            bitrate: inner.bitrate,
            codec: inner.codec.clone(),
            container: inner.container.clone(),
            duration: Some(proto_to_db_ms(&inner.duration)),
            sample_rate: inner.sample_rate,
            hash: inner.hash.clone(),
            r#type: song_type_to_str(inner.r#type).to_string(),
            url: inner.url.clone(),
            song_cover_path_high: inner.song_cover_path_high.clone(),
            playback_url: inner.playback_url.clone(),
            song_cover_path_low: inner.song_cover_path_low.clone(),
            date_added: inner.date_added,
            track_no: inner.track_no,
            lyrics: inner.lyrics.as_ref().map(|l| l.encode_to_vec()),
        });

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

    SongRowsAndBridges {
        rows,
        artist_bridges,
        genre_bridges,
    }
}

#[tracing::instrument(level = "debug", skip_all)]
async fn insert_bulk_db_song_rows(
    tx: &mut Transaction<'_, Sqlite>,
    rows: &[DbSongInsertRow],
) -> Result<(), DatabaseError> {
    for chunk in rows.chunks(30) {
        let mut qb: QueryBuilder<'_, Sqlite> = QueryBuilder::new(
            "INSERT INTO allsongs (
                _id, album_id, path, size, title, date, year, releasetype,
                bitrate, codec, container, duration, samplerate, hash, type, url,
                song_coverpath_high, playbackurl, song_coverpath_low, date_added, track_no
            ) ",
        );
        qb.push_values(chunk, |mut b, r| {
            b.push_bind(&r.id)
                .push_bind(&r.album_id)
                .push_bind(&r.path)
                .push_bind(r.size)
                .push_bind(&r.title)
                .push_bind(&r.date)
                .push_bind(&r.year)
                .push_bind(&r.release_type)
                .push_bind(r.bitrate)
                .push_bind(&r.codec)
                .push_bind(&r.container)
                .push_bind(r.duration)
                .push_bind(r.sample_rate)
                .push_bind(&r.hash)
                .push_bind(&r.r#type)
                .push_bind(&r.url)
                .push_bind(&r.song_cover_path_high)
                .push_bind(&r.playback_url)
                .push_bind(&r.song_cover_path_low)
                .push_bind(r.date_added)
                .push_bind(r.track_no);
        });
        qb.push(
            " ON CONFLICT(path) DO UPDATE SET
                album_id = excluded.album_id,
                size = excluded.size,
                title = excluded.title,
                date = excluded.date,
                year = excluded.year,
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
                track_no = excluded.track_no",
        );
        qb.build()
            .execute(&mut **tx)
            .await
            .map_err(DatabaseError::Query)?;
    }
    Ok(())
}

#[tracing::instrument(level = "debug", skip_all)]
async fn insert_bulk_lyrics(
    tx: &mut Transaction<'_, Sqlite>,
    rows: &[DbSongInsertRow],
) -> Result<(), DatabaseError> {
    let lyric_rows: Vec<(&str, &[u8])> = rows
        .iter()
        .filter_map(|r| {
            let id = r.id.as_deref()?;
            let bytes = r.lyrics.as_deref()?;
            Some((id, bytes))
        })
        .collect();

    for chunk in lyric_rows.chunks(100) {
        let mut qb: QueryBuilder<'_, Sqlite> =
            QueryBuilder::new("INSERT INTO song_lyrics (song_id, lyrics) ");
        qb.push_values(chunk, |mut b, (id, bytes)| {
            b.push_bind(*id).push_bind(*bytes);
        });
        qb.push(" ON CONFLICT(song_id) DO UPDATE SET lyrics = excluded.lyrics");
        qb.build()
            .execute(&mut **tx)
            .await
            .map_err(DatabaseError::Query)?;
    }
    Ok(())
}

#[tracing::instrument(level = "debug", skip_all)]
async fn insert_bulk_bridge_entries(
    tx: &mut Transaction<'_, Sqlite>,
    table: &str,
    col1: &str,
    col2: &str,
    entries: &[(String, String)],
) -> Result<(), DatabaseError> {
    for chunk in entries.chunks(400) {
        let mut qb: QueryBuilder<'_, Sqlite> =
            QueryBuilder::new(format!("INSERT INTO {table} ({col1}, {col2}) "));
        qb.push_values(chunk, |mut b, (v1, v2)| {
            b.push_bind(v1).push_bind(v2);
        });
        qb.push(" ON CONFLICT DO NOTHING");
        qb.build()
            .execute(&mut **tx)
            .await
            .map_err(DatabaseError::Query)?;
    }
    Ok(())
}
