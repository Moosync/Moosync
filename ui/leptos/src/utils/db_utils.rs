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
use leptos::prelude::*;
use leptos::task::spawn_local;
use serde_wasm_bindgen::from_value;
use songs_proto::moosync::types::Album;
use songs_proto::moosync::types::Artist;
use songs_proto::moosync::types::Genre;
use songs_proto::moosync::types::{GetEntityOptions, GetSongOptions, Playlist, Song};

#[tracing::instrument(level = "debug", skip(options, setter))]
pub fn get_songs_by_option(options: GetSongOptions, setter: impl Set<Value = Vec<Song>> + 'static) {
    spawn_local(async move {
        let songs = super::invoke::get_songs_by_options(options).await.unwrap();
        setter.set(songs);
    });
}

#[tracing::instrument(level = "debug", skip(setter))]
pub fn get_playlists_local<T>(setter: T)
where
    T: Set<Value = Vec<Playlist>> + Update<Value = Vec<Playlist>> + 'static,
{
    spawn_local(async move {
        let songs = serde_wasm_bindgen::from_value(
            super::invoke::get_entity_by_options(GetEntityOptions {
                playlist: Some(Playlist::default()),
                ..Default::default()
            })
            .await
            .unwrap(),
        )
        .unwrap();
        setter.set(songs);
    });
}

#[tracing::instrument(level = "debug", skip(options, setter))]
pub fn get_playlists_by_option<T>(options: Playlist, setter: T)
where
    T: Set<Value = Vec<Playlist>> + Update<Value = Vec<Playlist>> + Copy + 'static,
{
    use std::{collections::HashMap, sync::Arc};

    use extensions_proto::moosync::types::ExtensionProviderScope;
    use leptos::{prelude::*, task::spawn_local};

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
        let songs: Vec<Playlist> = from_value(res.unwrap()).unwrap();
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

#[tracing::instrument(level = "debug", skip(options, setter))]
pub fn get_artists_by_option(options: Artist, setter: impl Set<Value = Vec<Artist>> + 'static) {
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
        let songs: Vec<Artist> = from_value(res.unwrap()).unwrap();
        setter.set(songs);
    });
}

#[tracing::instrument(level = "debug", skip(options, setter))]
pub fn get_albums_by_option(options: Album, setter: impl Set<Value = Vec<Album>> + 'static) {
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
        let songs: Vec<Album> = from_value(res.unwrap()).unwrap();
        setter.set(songs);
    });
}

#[tracing::instrument(level = "debug", skip(options, setter))]
pub fn get_genres_by_option(options: Genre, setter: impl Set<Value = Vec<Genre>> + 'static) {
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
        let songs: Vec<Genre> = from_value(res.unwrap()).unwrap();
        setter.set(songs);
    });
}

#[tracing::instrument(level = "debug", skip(songs, refresh_cb))]
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

#[tracing::instrument(level = "debug", skip(songs, refresh_cb))]
pub fn remove_songs_from_library(songs: Vec<Song>, refresh_cb: Arc<Box<dyn Fn() + Send + Sync>>) {
    spawn_local(async move {
        let res = super::invoke::remove_songs(
            songs
                .into_iter()
                .map(|s| s.song.unwrap_or_default().id.unwrap_or_default())
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

#[tracing::instrument(level = "debug", skip(id, songs))]
pub fn add_to_playlist(id: String, songs: Vec<Song>) {
    spawn_local(async move {
        let res = super::invoke::add_to_playlist(id, songs).await;
        if res.is_err() {
            tracing::error!("Error adding to playlist: {:?}", res);
        }
    });
}

#[tracing::instrument(level = "debug", skip(cb))]
pub fn create_playlist_and(
    playlist: Playlist,
    songs: Option<Vec<Song>>,
    cb: Arc<Box<dyn Fn() + Send + Sync>>,
) {
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

        cb.as_ref()();
    });
}

#[tracing::instrument(level = "debug", skip(playlist, refresh_cb))]
pub fn remove_playlist(playlist: Playlist, refresh_cb: Arc<Box<dyn Fn() + Send + Sync>>) {
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

#[tracing::instrument(level = "debug", skip(playlist))]
pub fn export_playlist(playlist: Playlist) {
    spawn_local(async move {
        let res = super::invoke::export_playlist(playlist.playlist_id.unwrap()).await;
        if let Err(res) = res {
            tracing::error!("Failed to export playlist: {:?}", res);
        }
    });
}
