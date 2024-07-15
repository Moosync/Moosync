use crate::{entities::QueryablePlaylist, errors::errors::Result, songs::Song};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct ProviderStatus {
    pub key: String,
    pub name: String,
    pub user_name: Option<String>,
    pub logged_in: bool,
}

impl ProviderStatus {
    pub fn with_name(name: &str) -> Self {
        Self {
            key: name.to_ascii_lowercase(),
            name: name.to_string(),
            user_name: None,
            logged_in: false,
        }
    }
}

#[async_trait]
pub trait GenericProvider: std::fmt::Debug + Send {
    async fn initialize(&mut self) -> Result<()>;
    fn key(&self) -> &str;
    fn match_id(&self, id: String) -> bool;

    async fn login(&mut self) -> Result<()>;
    async fn authorize(&mut self, code: String) -> Result<()>;

    async fn fetch_user_details(&self) -> Result<ProviderStatus>;
    async fn fetch_user_playlists(&self, limit: u32, offset: u32)
        -> Result<Vec<QueryablePlaylist>>;
    async fn get_playlist_content(
        &self,
        playlist_id: String,
        limit: u32,
        offset: u32,
    ) -> Result<Vec<Song>>;
}
