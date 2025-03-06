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

use std::sync::Arc;

use futures::lock::Mutex;
use indexed_db_futures::database::Database;
use indexed_db_futures::prelude::*;
use leptos::prelude::*;
use leptos::task::spawn_local;
use serde_wasm_bindgen::from_value;
use types::entities::QueryableAlbum;
use types::entities::QueryableArtist;
use types::entities::QueryableGenre;
use types::{
    entities::{GetEntityOptions, QueryablePlaylist},
    songs::{GetSongOptions, Song},
};
use wasm_bindgen::JsValue;
use web_sys::DomException;
use web_sys::IdbTransactionMode;

#[tracing::instrument(level = "trace", skip(options, setter))]
#[cfg(not(feature = "mock"))]
pub fn get_songs_by_option(options: GetSongOptions, setter: impl Set<Value = Vec<Song>> + 'static) {
    spawn_local(async move {
        let songs = super::invoke::get_songs_by_options(options).await.unwrap();
        setter.set(songs);
    });
}

#[tracing::instrument(level = "trace", skip(options, setter))]
#[cfg(feature = "mock")]
pub fn get_songs_by_option(options: GetSongOptions, setter: impl Set<Value = Vec<Song>> + 'static) {
    use types::{entities::QueryableArtist, songs::SongType};

    let mut songs = vec![];
    for i in 0..1000 {
        let mut song = Song::default();
        song.song._id = Some(format!("song_id_{}", i));
        song.song.title = Some(format!("hello world {}", i));
        song.song.song_cover_path_low = Some("https://upload.wikimedia.org/wikipedia/commons/thumb/6/66/SMPTE_Color_Bars.svg/200px-SMPTE_Color_Bars.svg.png".to_string());
        song.song.song_cover_path_high = Some("https://upload.wikimedia.org/wikipedia/commons/thumb/6/66/SMPTE_Color_Bars.svg/200px-SMPTE_Color_Bars.svg.png".to_string());
        song.artists = Some(vec![QueryableArtist {
            artist_name: Some("Test artist".to_string()),
            ..Default::default()
        }]);
        song.song.type_ = SongType::LOCAL;
        song.song.duration = Some(96f64);
        song.song.playback_url =
            Some("https://cdn.freesound.org/previews/728/728162_462105-lq.mp3".into());
        songs.push(song);
    }

    setter.set(songs);
}

#[tracing::instrument(level = "trace", skip(options, setter))]
#[cfg(feature = "mock")]
pub fn get_playlists_by_option(
    options: QueryablePlaylist,
    setter: impl Set<Value = Vec<QueryablePlaylist>> + 'static,
) {
    let mut songs = vec![];
    for i in 0..1000 {
        let mut playlist = QueryablePlaylist::default();
        playlist.playlist_id = Some(format!("playlist_id_{}", i));
        playlist.playlist_name = format!("Playlist {}", i);
        playlist.playlist_coverpath = Some("https://upload.wikimedia.org/wikipedia/commons/thumb/6/66/SMPTE_Color_Bars.svg/200px-SMPTE_Color_Bars.svg.png".to_string());
        songs.push(playlist);
    }

    setter.set(songs);
}

#[tracing::instrument(level = "trace", skip(options, setter))]
#[cfg(feature = "mock")]
pub fn get_artists_by_option(
    options: QueryableArtist,
    setter: impl Set<Value = Vec<QueryableArtist>> + 'static,
) {
    let mut songs = vec![];
    for i in 0..1000 {
        let mut artist = QueryableArtist::default();
        artist.artist_id = Some(format!("artist_id_{}", i));
        artist.artist_name = Some(format!("Artist {}", i));
        artist.artist_coverpath = Some("https://upload.wikimedia.org/wikipedia/commons/thumb/6/66/SMPTE_Color_Bars.svg/200px-SMPTE_Color_Bars.svg.png".to_string());
        songs.push(artist);
    }

    setter.set(songs);
}

#[tracing::instrument(level = "trace", skip(options, setter))]
#[cfg(feature = "mock")]
pub fn get_albums_by_option(
    options: QueryableAlbum,
    setter: impl Set<Value = Vec<QueryableAlbum>> + 'static,
) {
    let mut songs = vec![];
    for i in 0..1000 {
        let mut album = QueryableAlbum::default();
        album.album_id = Some(format!("album_id_{}", i));
        album.album_name = Some(format!("Album {}", i));
        album.album_coverpath_high = Some("https://upload.wikimedia.org/wikipedia/commons/thumb/6/66/SMPTE_Color_Bars.svg/200px-SMPTE_Color_Bars.svg.png".to_string());
        songs.push(album);
    }

    setter.set(songs);
}

#[tracing::instrument(level = "trace", skip(options, setter))]
#[cfg(feature = "mock")]
pub fn get_genres_by_option(
    options: QueryableGenre,
    setter: impl Set<Value = Vec<QueryableGenre>> + 'static,
) {
    let mut songs = vec![];
    for i in 0..1000 {
        let mut album = QueryableGenre::default();
        album.genre_id = Some(format!("genre_id_{}", i));
        album.genre_name = Some(format!("Genre {}", i));
        songs.push(album);
    }

    setter.set(songs);
}

#[tracing::instrument(level = "trace", skip(setter))]
#[cfg(not(feature = "mock"))]
pub fn get_playlists_local<T>(setter: T)
where
    T: Set<Value = Vec<QueryablePlaylist>> + Update<Value = Vec<QueryablePlaylist>> + 'static,
{
    spawn_local(async move {
        let songs = serde_wasm_bindgen::from_value(
            super::invoke::get_entity_by_options(GetEntityOptions {
                playlist: Some(QueryablePlaylist::default()),
                ..Default::default()
            })
            .await
            .unwrap(),
        )
        .unwrap();
        setter.set(songs);
    });
}

#[tracing::instrument(level = "trace", skip(options, setter))]
#[cfg(not(feature = "mock"))]
pub fn get_playlists_by_option<T>(options: QueryablePlaylist, setter: T)
where
    T: Set<Value = Vec<QueryablePlaylist>>
        + Update<Value = Vec<QueryablePlaylist>>
        + Copy
        + 'static,
{
    use std::{collections::HashMap, sync::Arc};

    use leptos::{prelude::*, task::spawn_local};
    use types::ui::extensions::ExtensionProviderScope;

    use crate::{store::provider_store::ProviderStore, utils::common::fetch_infinite};

    let provider_store = expect_context::<Arc<ProviderStore>>();
    let next_page_tokens: RwSignal<
        HashMap<String, Arc<Mutex<types::providers::generic::Pagination>>>,
    > = RwSignal::new(HashMap::new());

    spawn_local(async move {
        let res = super::invoke::get_entity_by_options(GetEntityOptions {
            playlist: Some(options),
            ..Default::default()
        })
        .await;
        if res.is_err() {
            tracing::error!("Error getting playlists: {:?}", res);
            return;
        }
        let songs: Vec<QueryablePlaylist> = from_value(res.unwrap()).unwrap();
        setter.set(songs);

        tracing::debug!(
            "provider keys {:?}",
            provider_store.get_provider_keys(ExtensionProviderScope::Playlists)
        );
        for key in provider_store.get_provider_keys(ExtensionProviderScope::Playlists) {
            tracing::debug!("Fetching playlists from {}", key);
            spawn_local(async move {
                let mut should_fetch = true;
                while should_fetch {
                    let is_loading = RwSignal::new(HashMap::new());
                    let res = fetch_infinite!(
                        key,
                        fetch_user_playlists,
                        setter,
                        next_page_tokens,
                        is_loading,
                    );
                    match res {
                        Err(e) => {
                            tracing::error!(
                                "Failed to fetch playlist content from {}: {:?}",
                                key,
                                e
                            );
                            should_fetch = false;
                        }
                        Ok(should_fetch_inner) => should_fetch = should_fetch_inner,
                    }
                }
            });
        }
    });
}

#[tracing::instrument(level = "trace", skip(options, setter))]
#[cfg(not(feature = "mock"))]
pub fn get_artists_by_option(
    options: QueryableArtist,
    setter: impl Set<Value = Vec<QueryableArtist>> + 'static,
) {
    use leptos::task::spawn_local;

    spawn_local(async move {
        let res = super::invoke::get_entity_by_options(GetEntityOptions {
            artist: Some(options),
            ..Default::default()
        })
        .await;
        if res.is_err() {
            tracing::error!("Error getting artists: {:?}", res);
            return;
        }
        let songs: Vec<QueryableArtist> = from_value(res.unwrap()).unwrap();
        setter.set(songs);
    });
}

#[tracing::instrument(level = "trace", skip(options, setter))]
#[cfg(not(feature = "mock"))]
pub fn get_albums_by_option(
    options: QueryableAlbum,
    setter: impl Set<Value = Vec<QueryableAlbum>> + 'static,
) {
    use leptos::task::spawn_local;

    spawn_local(async move {
        let res = super::invoke::get_entity_by_options(GetEntityOptions {
            album: Some(options),
            ..Default::default()
        })
        .await;
        if res.is_err() {
            tracing::error!("Error getting albums: {:?}", res);
            return;
        }
        let songs: Vec<QueryableAlbum> = from_value(res.unwrap()).unwrap();
        setter.set(songs);
    });
}

#[tracing::instrument(level = "trace", skip(options, setter))]
#[cfg(not(feature = "mock"))]
pub fn get_genres_by_option(
    options: QueryableGenre,
    setter: impl Set<Value = Vec<QueryableGenre>> + 'static,
) {
    use leptos::task::spawn_local;

    spawn_local(async move {
        let res = super::invoke::get_entity_by_options(GetEntityOptions {
            genre: Some(options),
            ..Default::default()
        })
        .await;
        if res.is_err() {
            tracing::error!("Error getting genres: {:?}", res);
            return;
        }
        let songs: Vec<QueryableGenre> = from_value(res.unwrap()).unwrap();
        setter.set(songs);
    });
}

#[tracing::instrument(level = "trace", skip(songs, refresh_cb))]
pub fn add_songs_to_library(songs: Vec<Song>, refresh_cb: Arc<Box<dyn Fn() + Send + Sync>>) {
    spawn_local(async move {
        let res = super::invoke::insert_songs(songs).await;
        if res.is_err() {
            tracing::error!("Error adding songs: {:?}", res);
        } else {
            refresh_cb.as_ref()();
        }
    });
}

#[tracing::instrument(level = "trace", skip(songs, refresh_cb))]
pub fn remove_songs_from_library(songs: Vec<Song>, refresh_cb: Arc<Box<dyn Fn() + Send + Sync>>) {
    spawn_local(async move {
        let res = super::invoke::remove_songs(
            songs
                .iter()
                .map(|s| s.song._id.clone().unwrap_or_default())
                .collect(),
        )
        .await;
        if res.is_err() {
            tracing::error!("Error removing songs: {:?}", res);
        } else {
            refresh_cb.as_ref()();
        }
    });
}

#[tracing::instrument(level = "trace", skip(id, songs))]
pub fn add_to_playlist(id: String, songs: Vec<Song>) {
    spawn_local(async move {
        let res = super::invoke::add_to_playlist(id, songs).await;
        if res.is_err() {
            tracing::error!("Error adding to playlist: {:?}", res);
        }
    });
}

#[tracing::instrument(level = "trace", skip(playlist))]
pub fn create_playlist(playlist: QueryablePlaylist, songs: Option<Vec<Song>>) {
    spawn_local(async move {
        let res = super::invoke::create_playlist(playlist).await;
        match res {
            Err(res) => {
                tracing::error!("Failed to create playlist: {:?}", res);
            }
            Ok(playlist_id) => {
                if let Some(songs) = songs {
                    let res = super::invoke::add_to_playlist(playlist_id, songs).await;
                    if let Err(e) = res {
                        tracing::error!("Failed to add songs to playlist: {:?}", e);
                    }
                }
            }
        }
    });
}

#[tracing::instrument(level = "trace", skip(playlist, refresh_cb))]
pub fn remove_playlist(playlist: QueryablePlaylist, refresh_cb: Arc<Box<dyn Fn() + Send + Sync>>) {
    if playlist.playlist_id.is_none() {
        return;
    }

    spawn_local(async move {
        let res = super::invoke::remove_playlist(playlist.playlist_id.unwrap()).await;
        if let Err(res) = res {
            tracing::error!("Failed to remove playlist: {:?}", res);
        }
        refresh_cb.as_ref()();
    });
}

#[tracing::instrument(level = "trace", skip(playlist))]
pub fn export_playlist(playlist: QueryablePlaylist) {
    spawn_local(async move {
        let res = super::invoke::export_playlist(playlist.playlist_id.unwrap()).await;
        if let Err(res) = res {
            tracing::error!("Failed to export playlist: {:?}", res);
        }
    });
}

#[tracing::instrument(level = "trace", skip(db, store, key, value))]
pub async fn write_to_indexed_db(
    db: &Database,
    store: &str,
    key: &str,
    value: Vec<u8>,
) -> Result<(), DomException> {
    let tx = db
        .transaction(store)
        .with_mode(IdbTransactionMode::Readwrite)
        .build()
        .unwrap();
    let store = tx.object_store(store).unwrap();
    store.put(value).with_key(key).await.unwrap();
    tx.commit().await.unwrap();
    tracing::debug!("Wrote to indexed db");

    Ok(())
}

#[tracing::instrument(level = "trace", skip(db, store, key))]
pub async fn read_from_indexed_db(
    db: Database,
    store: &str,
    key: &str,
) -> Result<Option<JsValue>, DomException> {
    use indexed_db_futures::prelude::*;
    let tx = db.transaction(store).build().unwrap();
    let store = tx.object_store(store).unwrap();
    let res = store.get(key).build().unwrap().await.unwrap();
    Ok(res)
}
