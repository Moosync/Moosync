use crate::{
    entities::{QueryablePlaylist, SearchResult},
    errors::errors::Result,
    songs::Song,
};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct Pagination {
    pub limit: u32,
    pub offset: u32,
    pub token: Option<String>,
    pub is_first: bool,
}

impl Pagination {
    pub fn new_limit(limit: u32, offset: u32) -> Self {
        Pagination {
            limit,
            offset,
            is_first: true,
            ..Default::default()
        }
    }

    pub fn new_token(token: Option<String>) -> Self {
        Pagination {
            token,
            is_first: true,
            ..Default::default()
        }
    }

    pub fn next_page(&self) -> Self {
        Pagination {
            limit: self.limit,
            offset: self.offset + self.limit.max(1),
            token: self.token.clone(),
            is_first: false,
        }
    }

    pub fn next_page_wtoken(&self, token: Option<String>) -> Self {
        Pagination {
            limit: self.limit,
            offset: self.offset + self.limit,
            token,
            is_first: false,
        }
    }
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct ProviderStatus {
    pub key: String,
    pub name: String,
    pub user_name: Option<String>,
    pub logged_in: bool,
    pub bg_color: String,
}

#[async_trait]
pub trait GenericProvider: std::fmt::Debug + Send {
    async fn initialize(&mut self) -> Result<()>;
    fn key(&self) -> String;
    fn match_id(&self, id: String) -> bool;

    async fn login(&mut self) -> Result<()>;
    async fn authorize(&mut self, code: String) -> Result<()>;

    async fn fetch_user_playlists(
        &self,
        pagination: Pagination,
    ) -> Result<(Vec<QueryablePlaylist>, Pagination)>;
    async fn get_playlist_content(
        &self,
        playlist_id: String,
        pagination: Pagination,
    ) -> Result<(Vec<Song>, Pagination)>;
    async fn get_playback_url(&self, song: Song, player: String) -> Result<String>;

    async fn search(&self, term: String) -> Result<SearchResult>;

    async fn match_url(&self, url: String) -> Result<bool>;
    async fn playlist_from_url(&self, url: String) -> Result<QueryablePlaylist>;
}
