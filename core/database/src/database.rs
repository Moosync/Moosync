use std::{fmt::Debug, fs, path::PathBuf, sync::Arc};

use async_trait::async_trait;
use cache::{CacheBackend, CacheError, DiskCacheFilter, DiskCacheRecord, ExtensionCacheStats};
use songs_proto::moosync::types::{
    Album, AllAnalytics, Artist, Genre, GetSongOptions, InnerSong, Lyrics, Playlist, SearchResult,
    Song,
};
use tracing::{debug, info};
use types::{
    plugin::{Plugin, PluginContext},
    subscription::SubscriberList,
};

use crate::{
    analytics::AnalyticsRepository, cache::CacheRepository, entities::EntityRepository,
    error::DatabaseError, playlists::PlaylistRepository, pool::create_pools,
    search::SearchRepository, songs::SongRepository,
};

#[derive(Clone)]
pub struct Database {
    songs: SongRepository,
    entities: EntityRepository,
    playlists: PlaylistRepository,
    search: SearchRepository,
    analytics: AnalyticsRepository,
    cache: CacheRepository,
    pub on_songs_added: SubscriberList<Vec<Song>>,
    pub on_song_updated: SubscriberList<Song>,
    pub on_album_updated: SubscriberList<Album>,
    pub on_artist_updated: SubscriberList<Artist>,
    pub on_playlist_updated: SubscriberList<Playlist>,
}

#[plugin_macro::generate]
impl Database {
    #[tracing::instrument(level = "debug", skip_all)]
    pub async fn new(path: PathBuf) -> Self {
        debug!("Creating database handler");
        if !path.exists() {
            fs::create_dir_all(&path).expect("Failed to create dir to store database");
        }

        let db_path = path.join("songs.db");
        let pools = create_pools(&db_path)
            .await
            .expect("Failed to initialize database pools");

        info!("Created DB instance");
        Self {
            songs: SongRepository::new(pools.clone()),
            entities: EntityRepository::new(pools.clone()),
            playlists: PlaylistRepository::new(pools.clone()),
            search: SearchRepository::new(pools.clone()),
            analytics: AnalyticsRepository::new(pools.clone()),
            cache: CacheRepository::new(pools),
            on_songs_added: SubscriberList::new(),
            on_song_updated: SubscriberList::new(),
            on_album_updated: SubscriberList::new(),
            on_artist_updated: SubscriberList::new(),
            on_playlist_updated: SubscriberList::new(),
        }
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub async fn create_playlist(&self, playlist: Playlist) -> Result<String, DatabaseError> {
        self.playlists.create_playlist(playlist).await
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub async fn create_playlist_with_songs(
        &self,
        playlist: Playlist,
        songs: &[Song],
    ) -> Result<(), DatabaseError> {
        self.playlists
            .create_playlist_with_songs(playlist, songs)
            .await
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub async fn add_to_playlist_bridge(
        &self,
        playlist_id: String,
        song_id: String,
    ) -> Result<(), DatabaseError> {
        self.playlists
            .add_to_playlist_bridge(&playlist_id, &song_id)
            .await
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub async fn insert_songs(&self, songs: Vec<Song>) -> Result<Vec<Song>, DatabaseError> {
        let mut songs = songs;
        self.insert_songs_by_ref(&mut songs).await?;
        Ok(songs)
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub async fn insert_songs_by_ref(&self, songs: &mut [Song]) -> Result<(), DatabaseError> {
        self.songs.insert_songs_by_ref(songs).await?;
        if !songs.is_empty() {
            self.on_songs_added.emit(songs.to_vec());
        }
        Ok(())
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub async fn remove_songs<T>(&self, ids: &[T]) -> Result<(), DatabaseError>
    where
        T: AsRef<str> + Debug + Sync,
    {
        self.songs.remove_songs(ids).await
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub async fn remove_songs_outside_directories(
        &self,
        scan_dirs: &[PathBuf],
    ) -> Result<(), DatabaseError> {
        self.songs.remove_songs_outside_directories(scan_dirs).await
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub async fn update_song(&self, song: &InnerSong) -> Result<(), DatabaseError> {
        if self.songs.update_song(song).await? {
            self.emit_song_updated(Song {
                song: Some(song.clone()),
                ..Default::default()
            });
        }
        Ok(())
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub async fn is_song_in_playlist(
        &self,
        playlist_id: &str,
        song_id: &str,
    ) -> Result<bool, DatabaseError> {
        self.playlists
            .is_song_in_playlist(playlist_id, song_id)
            .await
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub async fn get_albums(
        &self,
        options: Album,
        inclusive: bool,
    ) -> Result<Vec<Album>, DatabaseError> {
        self.entities.get_albums(options, inclusive).await
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub async fn get_artists(
        &self,
        options: Artist,
        inclusive: bool,
    ) -> Result<Vec<Artist>, DatabaseError> {
        self.entities.get_artists(options, inclusive).await
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub async fn get_genres(
        &self,
        options: Genre,
        inclusive: bool,
    ) -> Result<Vec<Genre>, DatabaseError> {
        self.entities.get_genres(options, inclusive).await
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub async fn get_playlists(
        &self,
        options: Playlist,
        inclusive: bool,
    ) -> Result<Vec<Playlist>, DatabaseError> {
        self.entities.get_playlists(options, inclusive).await
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub async fn get_album_songs(&self, album_id: &str) -> Result<Vec<Song>, DatabaseError> {
        self.get_songs_by_options(GetSongOptions {
            album: Some(Album {
                album_id: Some(album_id.to_string()),
                ..Default::default()
            }),
            ..Default::default()
        })
        .await
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub async fn get_artist_songs(&self, artist_id: &str) -> Result<Vec<Song>, DatabaseError> {
        self.get_songs_by_options(GetSongOptions {
            artist: Some(Artist {
                artist_id: Some(artist_id.to_string()),
                ..Default::default()
            }),
            ..Default::default()
        })
        .await
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub async fn get_genre_songs(&self, genre_id: &str) -> Result<Vec<Song>, DatabaseError> {
        self.get_songs_by_options(GetSongOptions {
            genre: Some(Genre {
                genre_id: Some(genre_id.to_string()),
                ..Default::default()
            }),
            ..Default::default()
        })
        .await
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub async fn get_playlist_songs(&self, playlist_id: &str) -> Result<Vec<Song>, DatabaseError> {
        self.get_songs_by_options(GetSongOptions {
            playlist: Some(Playlist {
                playlist_id: Some(playlist_id.to_string()),
                ..Default::default()
            }),
            ..Default::default()
        })
        .await
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub async fn get_songs_by_options(
        &self,
        options: GetSongOptions,
    ) -> Result<Vec<Song>, DatabaseError> {
        self.entities.get_songs_by_options(options).await
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub async fn search_all(&self, term: &str) -> Result<SearchResult, DatabaseError> {
        self.search.search_all(term).await
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub async fn files_not_in_db(
        &self,
        file_list: Vec<(PathBuf, f64)>,
    ) -> Result<Vec<(PathBuf, f64)>, DatabaseError> {
        self.songs.files_not_in_db(file_list).await
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub async fn add_to_playlist(&self, id: &str, songs: &[Song]) -> Result<(), DatabaseError> {
        self.playlists.add_to_playlist(id, songs).await
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub async fn remove_from_playlist<T>(&self, id: &str, songs: &[T]) -> Result<(), DatabaseError>
    where
        T: AsRef<str> + Debug + Sync,
    {
        self.playlists.remove_from_playlist(id, songs).await
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub async fn remove_playlist(&self, id: &str) -> Result<(), DatabaseError> {
        self.playlists.remove_playlist(id).await
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub async fn update_album(&self, album: Album) -> Result<(), DatabaseError> {
        self.entities.update_album(&album).await?;
        self.on_album_updated.emit(album);
        Ok(())
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub async fn update_artist(&self, artist: Artist) -> Result<(), DatabaseError> {
        self.entities.update_artist(&artist).await?;
        self.on_artist_updated.emit(artist);
        Ok(())
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub async fn update_playlist(&self, playlist: Playlist) -> Result<(), DatabaseError> {
        self.playlists.update_playlist(&playlist).await?;
        self.on_playlist_updated.emit(playlist);
        Ok(())
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub async fn update_songs(&self, songs: Vec<Song>) -> Result<(), DatabaseError> {
        self.songs.update_songs(&songs).await?;
        for song in songs {
            self.emit_song_updated(song);
        }
        Ok(())
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub async fn get_lyrics(&self, id: &str) -> Result<Option<Lyrics>, DatabaseError> {
        self.songs.get_lyrics(id).await
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub async fn update_lyrics(&self, id: &str, lyrics: &Lyrics) -> Result<(), DatabaseError> {
        self.songs.update_lyrics(id, lyrics).await?;
        Ok(())
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub async fn update_song_cover(
        &self,
        id: &str,
        cover_high: Option<&str>,
        cover_low: Option<&str>,
    ) -> Result<(), DatabaseError> {
        self.songs
            .update_song_cover(id, cover_high, cover_low)
            .await?;
        self.emit_song_updated(Song {
            song: Some(InnerSong {
                id: Some(id.to_string()),
                song_cover_path_high: cover_high.map(ToString::to_string),
                song_cover_path_low: cover_low.map(ToString::to_string),
                ..Default::default()
            }),
            ..Default::default()
        });
        Ok(())
    }

    #[tracing::instrument(level = "debug", skip_all)]
    fn emit_song_updated(&self, song: Song) { self.on_song_updated.emit(song); }

    #[tracing::instrument(level = "debug", skip_all)]
    pub async fn increment_play_count(&self, id: &str) -> Result<(), DatabaseError> {
        self.analytics.increment_play_count(id).await
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub async fn increment_play_time(&self, id: &str, duration: f64) -> Result<(), DatabaseError> {
        self.analytics.increment_play_time(id, duration).await
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub async fn get_top_listened_songs(&self) -> Result<AllAnalytics, DatabaseError> {
        self.analytics.get_top_listened_songs().await
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub async fn export_playlist(&self, playlist_id: &str) -> Result<String, DatabaseError> {
        self.playlists.export_playlist(playlist_id).await
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub async fn get_extension_cache(
        &self,
        package_name: &str,
        key: &str,
    ) -> Result<Option<Vec<u8>>, DatabaseError> {
        self.cache.get_extension_cache(package_name, key).await
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub async fn set_extension_cache(
        &self,
        package_name: &str,
        key: &str,
        data: &[u8],
        ttl_seconds: u64,
    ) -> Result<(), DatabaseError> {
        self.cache
            .set_extension_cache(package_name, key, data, ttl_seconds)
            .await
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub async fn remove_extension_cache(&self, package_name: &str) -> Result<(), DatabaseError> {
        self.cache.remove_extension_cache(package_name).await
    }
}

#[async_trait]
impl CacheBackend for Database {
    #[tracing::instrument(level = "debug", skip_all)]
    async fn upsert_disk_cache(&self, entry: DiskCacheRecord) -> Result<(), CacheError> {
        self.cache
            .upsert_disk_cache(entry)
            .await
            .map_err(|err| CacheError::Database(err.to_string()))
    }

    #[tracing::instrument(level = "debug", skip_all)]
    async fn remove_disk_cache(&self, filter: DiskCacheFilter) -> Result<(), CacheError> {
        self.cache
            .remove_disk_cache(filter)
            .await
            .map_err(|err| CacheError::Database(err.to_string()))
    }

    #[tracing::instrument(level = "debug", skip_all)]
    async fn get_disk_cache_entries(&self) -> Result<Vec<DiskCacheRecord>, CacheError> {
        self.cache
            .get_disk_cache_entries()
            .await
            .map_err(|err| CacheError::Database(err.to_string()))
    }

    #[tracing::instrument(level = "debug", skip_all)]
    async fn get_extension_cache_stats(&self) -> Result<Vec<ExtensionCacheStats>, CacheError> {
        self.cache
            .get_extension_cache_stats()
            .await
            .map_err(|err| CacheError::Database(err.to_string()))
    }

    #[tracing::instrument(level = "debug", skip_all)]
    async fn remove_extension_cache(&self, package_name: &str) -> Result<(), CacheError> {
        self.cache
            .remove_extension_cache(package_name)
            .await
            .map_err(|err| CacheError::Database(err.to_string()))
    }

    #[tracing::instrument(level = "debug", skip_all)]
    async fn clear_expired_extension_cache(&self) -> Result<u64, CacheError> {
        self.cache
            .clear_expired_extension_cache()
            .await
            .map_err(|err| CacheError::Database(err.to_string()))
    }

    #[tracing::instrument(level = "debug", skip_all)]
    async fn clear_all_extension_cache(&self) -> Result<(), CacheError> {
        self.cache
            .clear_all_extension_cache()
            .await
            .map_err(|err| CacheError::Database(err.to_string()))
    }

    #[tracing::instrument(level = "debug", skip_all)]
    async fn get_orphan_lyrics_stats(&self) -> Result<(u64, u64), CacheError> {
        self.cache
            .get_orphan_lyrics_stats()
            .await
            .map_err(|err| CacheError::Database(err.to_string()))
    }

    #[tracing::instrument(level = "debug", skip_all)]
    async fn clear_orphan_lyrics(&self) -> Result<u64, CacheError> {
        self.cache
            .clear_orphan_lyrics()
            .await
            .map_err(|err| CacheError::Database(err.to_string()))
    }
}

impl Plugin for Database {
    #[tracing::instrument(level = "debug", skip_all)]
    fn init(context: &PluginContext) -> Arc<Self> {
        let data_dir = context.data_dir.clone();
        std::thread::spawn(move || {
            tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .expect("Failed to create Tokio runtime for Database init")
                .block_on(Database::new(data_dir))
        })
        .join()
        .expect("Database init thread panicked")
        .into()
    }
}
