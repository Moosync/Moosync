use std::{
    collections::HashSet,
    error::Error,
    path::{Path, PathBuf},
    sync::Arc,
};

use async_trait::async_trait;
use extensions::ExtensionGroupExt;
use extensions_proto::moosync::types::{
    ExtensionProviderScope, RequestedCoverArtRequest, requested_cover_art_request::Item,
};
use futures::StreamExt;
use player::PlayerHandler;
use songs_proto::moosync::types::{
    Album, Artist, EntityResult, GetEntityOptions, GetSongOptions, SearchableSong, Song,
    entity_result,
};
use tokio::sync::Mutex;
use tracing::{Instrument, debug, error};
use types::{image::download_and_save_image, prelude::SongsExt};

use super::Hook;
use crate::StateManager;

#[derive(Debug, Clone)]
pub enum CoverEntity {
    Song(Box<Song>),
    Album(Album),
    Artist(Artist),
}

impl CoverEntity {
    #[tracing::instrument(level = "debug", skip_all)]
    pub fn id(&self) -> Option<String> {
        match self {
            CoverEntity::Song(song) => song.get_id().map(|id| id.into_owned()),
            CoverEntity::Album(album) => album.album_id.clone(),
            CoverEntity::Artist(artist) => artist.artist_id.clone(),
        }
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub fn has_cover(&self) -> bool {
        match self {
            CoverEntity::Song(song) => song
                .song
                .as_ref()
                .and_then(|s| s.song_cover_path_high.as_ref())
                .is_some_and(|h| !h.is_empty()),
            CoverEntity::Album(album) => album
                .album_coverpath_high
                .as_ref()
                .is_some_and(|h| !h.is_empty()),
            CoverEntity::Artist(artist) => artist
                .artist_coverpath
                .as_ref()
                .is_some_and(|h| !h.is_empty()),
        }
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub fn dedupe_key(&self) -> Option<String> {
        let prefix = match self {
            CoverEntity::Song(_) => "song",
            CoverEntity::Album(_) => "album",
            CoverEntity::Artist(_) => "artist",
        };
        self.id().map(|id| format!("{prefix}:{id}"))
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub fn to_item(&self) -> Item {
        match self {
            CoverEntity::Song(song) => Item::Song((**song).clone()),
            CoverEntity::Album(album) => Item::Album(album.clone()),
            CoverEntity::Artist(artist) => Item::Artist(artist.clone()),
        }
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub fn save_cover(&self, state_manager: &StateManager, high: &Path, low: &Path) {
        let high_str = high.to_str().unwrap_or_default();
        let low_str = low.to_str().unwrap_or_default();
        let db = state_manager.get_database();

        match self {
            CoverEntity::Song(_) => {
                if let Some(id) = self.id()
                    && let Err(e) = db.update_song_cover(&id, Some(high_str), Some(low_str))
                {
                    error!("Failed to update song cover in database for {id}: {e:?}");
                }
            }
            CoverEntity::Album(album) => {
                let mut updated_album = album.clone();
                updated_album.album_coverpath_high = Some(high_str.to_string());
                updated_album.album_coverpath_low = Some(low_str.to_string());
                if let Err(e) = db.update_album(updated_album) {
                    error!("Failed to update album cover in database: {e:?}");
                }
            }
            CoverEntity::Artist(artist) => {
                let mut updated_artist = artist.clone();
                updated_artist.artist_coverpath = Some(high_str.to_string());
                if let Err(e) = db.update_artist(updated_artist) {
                    error!("Failed to update artist cover in database: {e:?}");
                }
            }
        }
    }
}

#[derive(Clone)]
pub struct CoverArtHook {
    pub(crate) in_flight: Arc<Mutex<HashSet<String>>>,
    is_resolving_all: Arc<Mutex<bool>>,
}

impl Default for CoverArtHook {
    #[tracing::instrument(level = "debug", skip_all)]
    fn default() -> Self { Self::new() }
}

impl CoverArtHook {
    #[tracing::instrument(level = "debug", skip_all)]
    pub fn new() -> Self {
        Self {
            in_flight: Arc::new(Mutex::new(HashSet::new())),
            is_resolving_all: Arc::new(Mutex::new(false)),
        }
    }

    #[tracing::instrument(level = "debug", skip_all)]
    async fn fetch_and_save(
        &self,
        state_manager: &StateManager,
        item: Item,
    ) -> Option<(PathBuf, PathBuf)> {
        let ext_handler = state_manager.get_extension_handler();
        let cover_extensions = ext_handler
            .get_extensions_with_scope(ExtensionProviderScope::CoverArt)
            .await;
        if cover_extensions.is_empty() {
            return None;
        }

        let cache_dir = state_manager.get_cache_dir().join("image_cache");
        let mut stream = cover_extensions.with(|ext| {
            let item_clone = item.clone();
            async move {
                ext.get_cover_art(RequestedCoverArtRequest {
                    item: Some(item_clone),
                })
                .await
            }
            .in_current_span()
        });

        while let Some(res) = stream.next().await {
            match res {
                Ok(resp) if !resp.cover_urls.is_empty() => {
                    if let Some(paths) = download_and_save_image(&resp.cover_urls, &cache_dir).await
                    {
                        return Some(paths);
                    }
                }
                Ok(_) => {}
                Err(e) => {
                    error!("Extension get_cover_art error: {e:?}");
                }
            }
        }

        None
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub async fn resolve_cover(&self, state_manager: &StateManager, entity: CoverEntity) {
        if entity.has_cover() {
            return;
        }

        let Some(key) = entity.dedupe_key() else {
            return;
        };

        let inserted = {
            let mut in_flight = self.in_flight.lock().await;
            in_flight.insert(key.clone())
        };
        if !inserted {
            return;
        }

        debug!("Resolving missing cover art for {key}");

        let paths = self.fetch_and_save(state_manager, entity.to_item()).await;
        if let Some((high, low)) = paths {
            entity.save_cover(state_manager, &high, &low);
        }

        let mut in_flight = self.in_flight.lock().await;
        in_flight.remove(&key);
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub async fn check_and_resolve_song_cover(
        &self,
        state_manager: &StateManager,
        song: &Song,
        _prioritized: bool,
    ) {
        self.resolve_cover(state_manager, CoverEntity::Song(Box::new(song.clone())))
            .await;
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub async fn check_and_resolve_album_cover(&self, state_manager: &StateManager, album: &Album) {
        self.resolve_cover(state_manager, CoverEntity::Album(album.clone()))
            .await;
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub async fn check_and_resolve_artist_cover(
        &self,
        state_manager: &StateManager,
        artist: &Artist,
    ) {
        self.resolve_cover(state_manager, CoverEntity::Artist(artist.clone()))
            .await;
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub async fn resolve_all_missing(&self, state_manager: &StateManager) {
        let Ok(mut guard) = self.is_resolving_all.try_lock() else {
            debug!("Full cover art scan already in progress");
            return;
        };
        *guard = true;

        let ext_handler = state_manager.get_extension_handler();
        let cover_extensions = ext_handler
            .get_extensions_with_scope(ExtensionProviderScope::CoverArt)
            .await;
        if cover_extensions.is_empty() {
            return;
        }

        let db = state_manager.get_database();

        match db.get_songs_by_options(GetSongOptions {
            song: Some(SearchableSong::default()),
            ..Default::default()
        }) {
            Ok(songs) => {
                for song in songs {
                    let entity = CoverEntity::Song(Box::new(song));
                    if !entity.has_cover() {
                        self.resolve_cover(state_manager, entity).await;
                    }
                }
            }
            Err(e) => {
                error!("Failed to fetch songs for cover art resolution: {e:?}");
            }
        }

        match db.get_entity_by_options(GetEntityOptions {
            album: Some(Album::default()),
            ..Default::default()
        }) {
            Ok(EntityResult {
                result: Some(entity_result::Result::Albums(list)),
            }) => {
                for album in list.albums {
                    let entity = CoverEntity::Album(album);
                    if !entity.has_cover() {
                        self.resolve_cover(state_manager, entity).await;
                    }
                }
            }
            Ok(_) => {}
            Err(e) => {
                error!("Failed to fetch albums for cover art resolution: {e:?}");
            }
        }

        match db.get_entity_by_options(GetEntityOptions {
            artist: Some(Artist::default()),
            ..Default::default()
        }) {
            Ok(EntityResult {
                result: Some(entity_result::Result::Artists(list)),
            }) => {
                for artist in list.artists {
                    let entity = CoverEntity::Artist(artist);
                    if !entity.has_cover() {
                        self.resolve_cover(state_manager, entity).await;
                    }
                }
            }
            Ok(_) => {}
            Err(e) => {
                error!("Failed to fetch artists for cover art resolution: {e:?}");
            }
        }
    }
}

#[async_trait]
impl Hook for CoverArtHook {
    #[tracing::instrument(level = "debug", skip_all)]
    async fn on_startup(
        &self,
        state_manager: &StateManager,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        let player_handler = state_manager.plugins.get::<PlayerHandler>();
        let extension_handler = state_manager.get_extension_handler();

        let hook_for_song = self.clone();
        let sm_for_song = state_manager.clone();
        let _cancel_song = player_handler.on_song_changed.listen(move |song_opt| {
            if let Some(song) = song_opt {
                let hook = hook_for_song.clone();
                let sm = sm_for_song.clone();
                tokio::spawn(
                    async move {
                        hook.resolve_cover(&sm, CoverEntity::Song(Box::new(song)))
                            .await;
                    }
                    .in_current_span(),
                );
            }
        });

        let hook_for_ext = self.clone();
        let sm_for_ext = state_manager.clone();
        let _cancel_ext = extension_handler.on_extensions_updated.listen(move |_| {
            let hook = hook_for_ext.clone();
            let sm = sm_for_ext.clone();
            tokio::spawn(
                async move {
                    hook.resolve_all_missing(&sm).await;
                }
                .in_current_span(),
            );
        });

        let hook_startup = self.clone();
        let sm_startup = state_manager.clone();
        tokio::spawn(
            async move {
                hook_startup.resolve_all_missing(&sm_startup).await;
            }
            .in_current_span(),
        );

        Ok(())
    }
}
