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

use crate::{
    entities::{QueryableAlbum, QueryableArtist, QueryablePlaylist, SearchResult},
    errors::Result,
    songs::Song,
    ui::extensions::{ContextMenuReturnType, ExtensionProviderScope},
};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct Pagination {
    pub limit: u32,
    pub offset: u32,
    pub token: Option<String>,
    pub is_first: bool,
    pub is_valid: bool,
}

impl Pagination {
    #[tracing::instrument(level = "trace", skip(limit, offset))]
    pub fn new_limit(limit: u32, offset: u32) -> Self {
        Pagination {
            limit,
            offset,
            is_first: true,
            is_valid: true,
            ..Default::default()
        }
    }

    #[tracing::instrument(level = "trace", skip(token))]
    pub fn new_token(token: Option<String>) -> Self {
        Pagination {
            token,
            is_first: true,
            is_valid: true,
            ..Default::default()
        }
    }

    #[tracing::instrument(level = "trace", skip(self))]
    pub fn next_page(&self) -> Self {
        Pagination {
            limit: self.limit,
            offset: self.offset + self.limit.max(1),
            token: self.token.clone(),
            is_first: false,
            is_valid: true,
        }
    }

    #[tracing::instrument(level = "trace", skip(self, token))]
    pub fn next_page_wtoken(&self, token: Option<String>) -> Self {
        Pagination {
            limit: self.limit,
            offset: self.offset + self.limit,
            token,
            is_first: false,
            is_valid: true,
        }
    }

    pub fn invalidate(&mut self) {
        self.is_valid = false;
    }
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct ProviderStatus {
    pub key: String,
    pub name: String,
    pub user_name: Option<String>,
    pub logged_in: bool,
    pub bg_color: String,
    pub account_id: String,
    pub scopes: Vec<ExtensionProviderScope>,
}

#[async_trait]
pub trait GenericProvider: std::fmt::Debug + Send {
    async fn initialize(&mut self) -> Result<()>;
    async fn get_provider_scopes(&self) -> Result<Vec<ExtensionProviderScope>>;
    fn key(&self) -> String;
    fn match_id(&self, id: String) -> bool;

    async fn login(&mut self, account_id: String) -> Result<String>;
    async fn signout(&mut self, account_id: String) -> Result<()>;
    async fn requested_account_status(&mut self) -> Result<()>;

    async fn authorize(&mut self, code: String) -> Result<()>;

    async fn fetch_user_playlists(
        &self,
        pagination: Pagination,
    ) -> Result<(Vec<QueryablePlaylist>, Pagination)>;
    async fn get_playlist_content(
        &self,
        playlist: QueryablePlaylist,
        pagination: Pagination,
    ) -> Result<(Vec<Song>, Pagination)>;
    async fn get_playback_url(&self, song: Song, player: String) -> Result<String>;

    async fn search(&self, term: String) -> Result<SearchResult>;

    async fn match_url(&self, url: String) -> Result<bool>;
    async fn playlist_from_url(&self, url: String) -> Result<QueryablePlaylist>;
    async fn song_from_url(&self, url: String) -> Result<Song>;
    async fn get_suggestions(&self) -> Result<Vec<Song>>;

    async fn get_album_content(
        &self,
        album: QueryableAlbum,
        pagination: Pagination,
    ) -> Result<(Vec<Song>, Pagination)>;
    async fn get_artist_content(
        &self,
        artist: QueryableArtist,
        pagination: Pagination,
    ) -> Result<(Vec<Song>, Pagination)>;

    async fn get_lyrics(&self, song: Song) -> Result<String>;
    async fn get_song_context_menu(&self, songs: Vec<Song>) -> Result<Vec<ContextMenuReturnType>>;
    async fn get_playlist_context_menu(
        &self,
        playlist: QueryablePlaylist,
    ) -> Result<Vec<ContextMenuReturnType>>;
    async fn trigger_context_menu_action(&self, action_id: String) -> Result<()>;
}
