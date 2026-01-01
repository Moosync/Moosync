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

use futures::{
    StreamExt,
    channel::mpsc::{UnboundedReceiver, UnboundedSender, unbounded},
};
use serde_json::Value;
use std::{collections::HashMap, sync::Arc};

use crate::macros::{generate_command_async, generate_command_async_cached};
use database::cache::CacheHolder;
use tauri::{AppHandle, Emitter, State};
use tokio::sync::{Mutex, RwLock};
use types::{
    entities::{Album, Artist, Playlist, SearchResult},
    errors::{MoosyncError, Result, error_helpers},
    providers::generic::{GenericProvider, Pagination, ProviderStatus},
    songs::Song,
    ui::extensions::{ContextMenuReturnType, ExtensionExtraEvent},
};

use crate::{extensions::get_extension_handler, providers::extension::ExtensionProvider};

macro_rules! get_provider {
    ($self:ident, $key:ident) => {{
        let guard = $self.provider_store.read().await;
        if !guard.contains_key(&$key) {
            return Err(MoosyncError::from(format!("Provider ({}) not found", $key)).into());
        }
        tokio::sync::RwLockReadGuard::map(guard, |store| store.get(&$key).unwrap())
    }};
}

#[derive(Debug)]
pub struct ProviderHandler {
    provider_store: Arc<RwLock<HashMap<String, Box<dyn GenericProvider>>>>,
    app_handle: AppHandle,
    status_tx: UnboundedSender<ProviderStatus>,
    provider_status: Arc<Mutex<HashMap<String, ProviderStatus>>>,
}

impl ProviderHandler {
    #[tracing::instrument(level = "debug", skip(app))]
    pub fn new(app: AppHandle) -> Self {
        let (status_tx, status_rx) = unbounded();
        let store = Self {
            app_handle: app.clone(),
            provider_store: Default::default(),
            status_tx,
            provider_status: Default::default(),
        };
        store.listen_status_changes(status_rx);
        store
    }

    pub async fn request_account_status(&self, key: &str) -> Result<()> {
        let provider_store = self.provider_store.read().await;
        if let Some(provider) = provider_store.get(key) {
            tracing::debug!("Requesting account status from {}", key);
            provider.requested_account_status().await?;
        }

        Err("Provider not found".into())
    }

    #[tracing::instrument(level = "debug", skip(self, status_rx))]
    pub fn listen_status_changes(&self, status_rx: UnboundedReceiver<ProviderStatus>) {
        let status_rx = Arc::new(Mutex::new(status_rx));
        let provider_status = self.provider_status.clone();
        let app_handle = self.app_handle.clone();
        tauri::async_runtime::spawn(async move {
            let status_rx = status_rx.clone();
            let mut status_rx = status_rx.lock().await;
            let provider_status = provider_status.clone();
            let app_handle = app_handle.clone();

            while let Some(status) = status_rx.next().await {
                tracing::debug!("Got provider status update {:?}", status);
                let mut provider_status = provider_status.lock().await;
                provider_status.insert(status.key.clone(), status);
                let res = app_handle.emit("provider-status-update", provider_status.clone());
                if let Err(e) = res {
                    tracing::error!("Error emitting status update: {:?}", e);
                }
                tracing::debug!(provider_status = ?provider_status, "Emitted status update");
            }
        });
    }

    #[tracing::instrument(level = "debug", skip(self))]
    pub async fn discover_provider_extensions(&self) -> Result<()> {
        let ext_handler = get_extension_handler(&self.app_handle);
        let extensions_res = ext_handler.get_installed_extensions().await?;
        for extension in extensions_res {
            if !extension.active {
                continue;
            }
            let provides = ext_handler
                .get_provider_scopes(extension.package_name.clone().into())
                .await;
            tracing::info!(
                "Got provider scopes from {} {:?}",
                extension.package_name,
                provides
            );
            if let Ok(provides) = provides {
                tracing::info!(
                    "Inserting extension provider {:?} {:?}",
                    extension,
                    provides,
                );

                let provider = Box::new(ExtensionProvider::new(
                    extension.clone(),
                    provides,
                    self.app_handle.clone(),
                    self.status_tx.clone(),
                ));
                let key = {
                    let mut provider_store = self.provider_store.write().await;
                    let key = provider.key();
                    provider_store.insert(key.clone(), provider);
                    tracing::info!("provider_store: {:?}", provider_store.keys());
                    key
                };
                self.initialize_provider(key).await;
            }

            self.app_handle
                .emit("providers-updated", Value::Null)
                .map_err(error_helpers::to_extension_error)?;
        }
        Ok(())
    }

    #[tracing::instrument(level = "debug", skip(self, key))]
    pub async fn initialize_provider(&self, key: String) {
        let provider_store = self.provider_store.read().await;
        let provider = provider_store.get(&key);
        if let Some(provider) = provider
            && let Err(e) = provider.initialize().await
        {
            tracing::error!("Error initializing provider {}: {:?}", provider.key(), e);
        }
    }

    #[tracing::instrument(level = "debug", skip(self, id))]
    pub async fn get_provider_key_by_id(&self, id: String) -> Result<String> {
        let provider_store = self.provider_store.read().await;
        for (key, provider) in provider_store.iter() {
            if provider.match_id(id.clone()) {
                return Ok(key.clone());
            }
        }
        Err(format!("Provider for id {id} not found").into())
    }

    #[tracing::instrument(level = "debug", skip(self))]
    pub async fn get_provider_keys(&self) -> Result<Vec<String>> {
        let provider_store = self.provider_store.read().await;
        Ok(provider_store.keys().cloned().collect())
    }

    #[tracing::instrument(level = "debug", skip(self))]
    pub async fn get_all_status(&self) -> Result<HashMap<String, ProviderStatus>> {
        Ok(self.provider_status.lock().await.clone())
    }

    #[tracing::instrument(level = "debug", skip(self))]
    pub async fn handle_extra_event(&self, key: String, event: ExtensionExtraEvent) -> Result<()> {
        let provider_keys = if key.is_empty() {
            self.get_provider_keys().await?
        } else {
            vec![key]
        };

        let provider_store = self.provider_store.read().await;
        for provider_key in provider_keys {
            let provider = provider_store.get(&provider_key);
            if let Some(provider) = provider
                && let Err(e) = provider.handle_extra_event(event.clone()).await
            {
                tracing::error!("Provider failed to handle event {:?}: {}", event, e);
            }
        }

        Ok(())
    }

    #[tracing::instrument(level = "debug", skip(self))]
    pub async fn provider_login(&self, key: String, account_id: String) -> Result<String> {
        let provider = get_provider!(self, key);
        provider.login(account_id).await
    }

    #[tracing::instrument(level = "debug", skip(self))]
    pub async fn provider_signout(&self, key: String, account_id: String) -> Result<()> {
        let provider = get_provider!(self, key);
        provider.signout(account_id).await
    }

    #[tracing::instrument(level = "debug", skip(self))]
    pub async fn provider_authorize(&self, key: String, code: String) -> Result<()> {
        let provider = get_provider!(self, key);
        provider.authorize(code).await
    }

    #[tracing::instrument(level = "debug", skip(self))]
    pub async fn fetch_user_playlists(
        &self,
        key: String,
        pagination: Pagination,
    ) -> Result<(Vec<Playlist>, Pagination)> {
        let provider = get_provider!(self, key);
        provider.fetch_user_playlists(pagination).await
    }

    #[tracing::instrument(level = "debug", skip(self))]
    pub async fn fetch_playlist_content(
        &self,
        key: String,
        playlist: Playlist,
        pagination: Pagination,
    ) -> Result<(Vec<Song>, Pagination)> {
        let provider = get_provider!(self, key);
        provider.get_playlist_content(playlist, pagination).await
    }

    #[tracing::instrument(level = "debug", skip(self))]
    pub async fn fetch_playback_url(
        &self,
        key: String,
        song: Song,
        player: String,
    ) -> Result<String> {
        let provider = get_provider!(self, key);
        provider.get_playback_url(song, player).await
    }

    #[tracing::instrument(level = "debug", skip(self))]
    pub async fn provider_search(&self, key: String, term: String) -> Result<SearchResult> {
        let provider = get_provider!(self, key);
        provider.search(term).await
    }

    #[tracing::instrument(level = "debug", skip(self))]
    pub async fn playlist_from_url(&self, key: String, url: String) -> Result<Playlist> {
        let provider = get_provider!(self, key);
        provider.playlist_from_url(url).await
    }

    #[tracing::instrument(level = "debug", skip(self))]
    pub async fn song_from_url(&self, key: String, url: String) -> Result<Song> {
        let provider = get_provider!(self, key);
        provider.song_from_url(url).await
    }

    #[tracing::instrument(level = "debug", skip(self))]
    pub async fn match_url(&self, key: String, url: String) -> Result<bool> {
        let provider = get_provider!(self, key);
        provider.match_url(url).await
    }

    #[tracing::instrument(level = "debug", skip(self))]
    pub async fn get_suggestions(&self, key: String) -> Result<Vec<Song>> {
        let provider = get_provider!(self, key);
        provider.get_suggestions().await
    }

    #[tracing::instrument(level = "debug", skip(self))]
    pub async fn get_album_content(
        &self,
        key: String,
        album: Album,
        pagination: Pagination,
    ) -> Result<(Vec<Song>, Pagination)> {
        let provider = get_provider!(self, key);
        provider.get_album_content(album, pagination).await
    }

    #[tracing::instrument(level = "debug", skip(self))]
    pub async fn get_artist_content(
        &self,
        key: String,
        artist: Artist,
        pagination: Pagination,
    ) -> Result<(Vec<Song>, Pagination)> {
        let provider = get_provider!(self, key);
        provider.get_artist_content(artist, pagination).await
    }

    #[tracing::instrument(level = "debug", skip(self))]
    pub async fn get_provider_lyrics(&self, key: String, song: Song) -> Result<String> {
        let provider = get_provider!(self, key);
        provider.get_lyrics(song).await
    }

    #[tracing::instrument(level = "debug", skip(self))]
    pub async fn get_song_context_menu(
        &self,
        key: String,
        songs: Vec<Song>,
    ) -> Result<Vec<ContextMenuReturnType>> {
        let provider = get_provider!(self, key);
        provider.get_song_context_menu(songs).await
    }

    #[tracing::instrument(level = "debug", skip(self))]
    pub async fn get_playlist_context_menu(
        &self,
        key: String,
        playlist: Playlist,
    ) -> Result<Vec<ContextMenuReturnType>> {
        let provider = get_provider!(self, key);
        provider.get_playlist_context_menu(playlist).await
    }

    #[tracing::instrument(level = "debug", skip(self))]
    pub async fn trigger_context_menu_action(&self, key: String, action: String) -> Result<()> {
        let provider = get_provider!(self, key);
        provider.trigger_context_menu_action(action).await
    }

    #[tracing::instrument(level = "debug", skip(self))]
    pub async fn get_song_from_id(&self, key: String, id: String) -> Result<Song> {
        let provider = get_provider!(self, key);
        provider.song_from_id(id).await
    }
}

#[tracing::instrument(level = "debug", skip(app))]
pub fn get_provider_handler_state(app: AppHandle) -> ProviderHandler {
    ProviderHandler::new(app)
}

generate_command_async!(get_provider_keys, ProviderHandler, Vec<String>,);
generate_command_async!(provider_login, ProviderHandler, String, key: String, account_id: String);
generate_command_async!(provider_signout, ProviderHandler, (), key: String, account_id: String);
generate_command_async!(provider_authorize, ProviderHandler, (), key: String, code: String);
generate_command_async!(get_provider_key_by_id, ProviderHandler, String, id: String);
generate_command_async_cached!(fetch_user_playlists, ProviderHandler, (Vec<Playlist>, Pagination), key: String, pagination: Pagination);
generate_command_async_cached!(fetch_playlist_content, ProviderHandler, (Vec<Song>, Pagination), key: String, playlist: Playlist, pagination: Pagination);
generate_command_async_cached!(fetch_playback_url, ProviderHandler, String, key: String, song: Song, player: String);
generate_command_async_cached!(provider_search, ProviderHandler, SearchResult, key: String, term: String);
generate_command_async!(get_all_status, ProviderHandler, HashMap<String, ProviderStatus>, );
generate_command_async_cached!(playlist_from_url, ProviderHandler, Playlist, key: String, url: String);
generate_command_async_cached!(song_from_url, ProviderHandler, Song, key: String, url: String);
generate_command_async_cached!(get_song_from_id, ProviderHandler, Song, key: String, id: String);
generate_command_async_cached!(match_url, ProviderHandler, bool, key: String, url: String);
generate_command_async_cached!(get_suggestions, ProviderHandler, Vec<Song>, key: String);
generate_command_async_cached!(get_artist_content, ProviderHandler, (Vec<Song>, Pagination), key: String, artist: Artist, pagination: Pagination);
generate_command_async_cached!(get_album_content, ProviderHandler, (Vec<Song>, Pagination), key: String, album: Album, pagination: Pagination);
generate_command_async_cached!(get_provider_lyrics, ProviderHandler, String, key: String, song: Song);
generate_command_async!(get_song_context_menu, ProviderHandler, Vec<ContextMenuReturnType>, key: String, songs: Vec<Song>);
generate_command_async!(get_playlist_context_menu, ProviderHandler, Vec<ContextMenuReturnType>, key: String, playlist: Playlist);
generate_command_async!(trigger_context_menu_action, ProviderHandler, (), key: String, action: String);
generate_command_async!(handle_extra_event, ProviderHandler, (), key: String, event: ExtensionExtraEvent);
