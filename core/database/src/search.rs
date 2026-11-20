use songs_proto::moosync::types::{
    Album, Artist, Genre, GetSongOptions, Playlist, SearchResult, SearchableSong,
};
use tracing::{info, trace};

use crate::{entities::EntityRepository, error::DatabaseError, pool::DbPools};

#[derive(Clone)]
pub struct SearchRepository {
    entities: EntityRepository,
}

impl SearchRepository {
    #[tracing::instrument(level = "debug", skip_all)]
    pub fn new(pools: DbPools) -> Self {
        Self {
            entities: EntityRepository::new(pools),
        }
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub async fn search_all(&self, term: &str) -> Result<SearchResult, DatabaseError> {
        trace!("Searching all by term");

        let pattern = format!("%{term}%");
        let songs = self
            .entities
            .get_songs_by_options(GetSongOptions {
                song: Some(SearchableSong {
                    id: None,
                    path: Some(pattern.clone()),
                    title: Some(pattern.clone()),
                    sample_rate: None,
                    hash: None,
                    r#type: None,
                    url: None,
                    playback_url: None,
                    ..Default::default()
                }),
                artist: None,
                album: None,
                genre: None,
                playlist: None,
                inclusive: Some(false),
            })
            .await?;

        let albums = self
            .entities
            .get_albums(
                Album {
                    album_name: Some(pattern.clone()),
                    ..Default::default()
                },
                false,
            )
            .await?;

        let artists = self
            .entities
            .get_artists(
                Artist {
                    artist_name: Some(pattern.clone()),
                    ..Default::default()
                },
                false,
            )
            .await?;

        let genres = self
            .entities
            .get_genres(
                Genre {
                    genre_name: Some(pattern.clone()),
                    ..Default::default()
                },
                false,
            )
            .await?;

        let playlists = self
            .entities
            .get_playlists(
                Playlist {
                    playlist_name: pattern.clone(),
                    playlist_desc: Some(pattern.clone()),
                    playlist_path: Some(pattern),
                    ..Default::default()
                },
                false,
            )
            .await?;

        info!("Searched all by term");

        Ok(SearchResult {
            songs,
            artists,
            playlists,
            albums,
            genres,
        })
    }
}
