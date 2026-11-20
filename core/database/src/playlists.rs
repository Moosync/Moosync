use std::fmt::Debug;

use songs_proto::moosync::types::{GetSongOptions, Playlist, Song};
use tracing::{info, trace, warn};
use types::prelude::SongsExt;
use uuid::Uuid;

use crate::{entities::EntityRepository, error::DatabaseError, pool::DbPools};

#[derive(Clone)]
pub struct PlaylistRepository {
    pools: DbPools,
    entities: EntityRepository,
}

impl PlaylistRepository {
    #[tracing::instrument(level = "debug", skip_all)]
    pub fn new(pools: DbPools) -> Self {
        let entities = EntityRepository::new(pools.clone());
        Self { pools, entities }
    }

    #[tracing::instrument(level = "debug", skip_all)]
    async fn insert_playlist(&self, playlist: &Playlist) -> Result<String, DatabaseError> {
        let id = playlist.playlist_id.as_ref().unwrap().clone();
        let song_count = playlist.playlist_song_count as i64;
        trace!("Inserting playlist");
        sqlx::query!(
            "INSERT INTO playlists (
                playlist_id, playlist_name, playlist_coverpath, playlist_song_count,
                playlist_desc, playlist_path, extension, icon, library_item
             ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, COALESCE(?9, 1))",
            playlist.playlist_id,
            playlist.playlist_name,
            playlist.playlist_coverpath,
            song_count,
            playlist.playlist_desc,
            playlist.playlist_path,
            playlist.extension,
            playlist.icon,
            playlist.library_item,
        )
        .execute(&self.pools.write)
        .await
        .map_err(DatabaseError::Query)?;
        info!("Inserted playlist");
        Ok(id)
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub async fn create_playlist(&self, mut playlist: Playlist) -> Result<String, DatabaseError> {
        trace!("Sanitizing playlist");

        if playlist.playlist_id.is_none() {
            playlist.playlist_id = Some(Uuid::new_v4().to_string());
        }

        if playlist.playlist_name.is_empty() {
            playlist.playlist_name = "New playlist".to_string();
        }

        if playlist.playlist_path.is_some() {
            let fetched = self
                .entities
                .get_playlists(
                    Playlist {
                        playlist_path: playlist.playlist_path.clone(),
                        ..Default::default()
                    },
                    false,
                )
                .await?;
            if let Some(first) = fetched.first()
                && let Some(id) = &first.playlist_id
            {
                return Ok(id.clone());
            }
        }

        self.insert_playlist(&playlist).await
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub async fn create_playlist_with_songs(
        &self,
        playlist: Playlist,
        songs: &[Song],
    ) -> Result<(), DatabaseError> {
        let playlist_id = match self.create_playlist(playlist.clone()).await {
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
        self.add_to_playlist(&playlist_id, songs).await
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub async fn add_to_playlist_bridge(
        &self,
        playlist_id: &str,
        song_id: &str,
    ) -> Result<(), DatabaseError> {
        trace!("Inserting song in playlist bridge");
        sqlx::query!(
            "INSERT INTO playlist_bridge (playlist, song) VALUES (?1, ?2)",
            playlist_id,
            song_id,
        )
        .execute(&self.pools.write)
        .await
        .map_err(DatabaseError::Query)?;
        trace!("Inserted song in playlist bridge");
        Ok(())
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub async fn is_song_in_playlist(
        &self,
        playlist_id: &str,
        song_id: &str,
    ) -> Result<bool, DatabaseError> {
        let count = sqlx::query_scalar!(
            "SELECT COUNT(*) FROM playlist_bridge WHERE playlist = ?1 AND song = ?2",
            playlist_id,
            song_id,
        )
        .fetch_one(&self.pools.read)
        .await
        .map_err(DatabaseError::Query)?;
        Ok(count > 0)
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub async fn add_to_playlist(&self, id: &str, songs: &[Song]) -> Result<(), DatabaseError> {
        trace!("Adding to playlist");
        for s in songs {
            if s.song.is_none() {
                continue;
            }
            let sid = s.get_id().unwrap_or_default();
            if let Err(e) = sqlx::query!(
                "INSERT INTO playlist_bridge (playlist, song) VALUES (?1, ?2)",
                id,
                sid,
            )
            .execute(&self.pools.write)
            .await
            {
                warn!("Failed to add {:?} to playlist: {:?}", s, e);
            }
        }
        info!("Added to playlist");
        Ok(())
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub async fn remove_from_playlist<T: AsRef<str> + Debug>(
        &self,
        id: &str,
        songs: &[T],
    ) -> Result<(), DatabaseError> {
        trace!("Removing from playlist");
        for sid in songs {
            let song_id = sid.as_ref();
            sqlx::query!(
                "DELETE FROM playlist_bridge WHERE playlist = ?1 AND song = ?2",
                id,
                song_id,
            )
            .execute(&self.pools.write)
            .await
            .map_err(DatabaseError::Query)?;
        }
        info!("Removed from playlist");
        Ok(())
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub async fn remove_playlist(&self, id: &str) -> Result<(), DatabaseError> {
        trace!("Removing playlist");
        sqlx::query!("DELETE FROM playlists WHERE playlist_id = ?1", id)
            .execute(&self.pools.write)
            .await
            .map_err(DatabaseError::Query)?;
        info!("Removed playlist");
        Ok(())
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub async fn update_playlist(&self, playlist: &Playlist) -> Result<(), DatabaseError> {
        trace!("Updating playlist");
        let song_count = playlist.playlist_song_count as i64;
        sqlx::query!(
            "UPDATE playlists SET
                playlist_name = ?1, playlist_coverpath = ?2, playlist_song_count = ?3,
                playlist_desc = ?4, playlist_path = ?5, extension = ?6, icon = ?7,
                library_item = COALESCE(?8, library_item)
             WHERE playlist_id = ?9",
            playlist.playlist_name,
            playlist.playlist_coverpath,
            song_count,
            playlist.playlist_desc,
            playlist.playlist_path,
            playlist.extension,
            playlist.icon,
            playlist.library_item,
            playlist.playlist_id,
        )
        .execute(&self.pools.write)
        .await
        .map_err(DatabaseError::Query)?;
        info!("Updated playlist");
        Ok(())
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub async fn export_playlist(&self, playlist_id: &str) -> Result<String, DatabaseError> {
        let binding = self
            .entities
            .get_playlists(
                Playlist {
                    playlist_id: Some(playlist_id.to_string()),
                    ..Default::default()
                },
                true,
            )
            .await?;
        let playlist = binding.first().ok_or(DatabaseError::PlaylistNotFound)?;

        let playlist_songs = self
            .entities
            .get_songs_by_options(GetSongOptions {
                playlist: Some(Playlist {
                    playlist_id: Some(playlist_id.to_string()),
                    ..Default::default()
                }),
                ..Default::default()
            })
            .await?;

        let mut ret = format!("#EXTM3U\n#PLAYLIST:{}\n", playlist.playlist_name);

        for song in playlist_songs {
            if let Some(entry) = format_song_playlist_entry(&song) {
                ret.push_str(&entry);
            }
        }

        Ok(ret.replace("\n\n", "\n"))
    }
}

#[tracing::instrument(level = "debug", skip_all)]
fn format_song_playlist_entry(song: &Song) -> Option<String> {
    let inner_song = song.song.as_ref()?;
    let location = if let Some(path) = &inner_song.path {
        format!("file://{}", path)
    } else if let Some(url) = &inner_song.url {
        url.clone()
    } else {
        return None;
    };

    let duration = inner_song
        .duration
        .as_ref()
        .map(|d| d.seconds)
        .unwrap_or_default();
    let title = inner_song.title.as_deref().unwrap_or_default();
    let album_info = song
        .album
        .as_ref()
        .and_then(|a| a.album_name.as_deref())
        .map_or(String::new(), |name| format!("#EXTALB:{}", name));
    let genre_info = if !song.genre.is_empty() {
        format!(
            "#EXTGENRE:{}",
            song.genre
                .iter()
                .filter_map(|g| g.genre_name.clone())
                .collect::<Vec<String>>()
                .join(",")
        )
    } else {
        String::new()
    };
    let cover_path = inner_song
        .song_cover_path_high
        .as_ref()
        .map_or(String::new(), |cover| format!("#EXTIMG:{}", cover));
    let song_info = format!("#MOOSINF:{}", inner_song.r#type);

    Some(format!(
        "#EXTINF:{},{}\n{}\n{}\n{}\n{}\n{}\n",
        duration, title, album_info, genre_info, cover_path, song_info, location
    ))
}
