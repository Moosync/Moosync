use async_trait::async_trait;
use leptos::RwSignal;
use types::{entities::QueryablePlaylist, errors::errors::Result, songs::Song, ui::providers::ProviderStatus};

#[async_trait(?Send)]
pub trait GenericProvider: std::fmt::Debug {
    async fn initialize(&mut self) -> Result<()>;
    fn key(&self) -> &str;
    fn get_status(&self) -> RwSignal<ProviderStatus>;
    fn match_id(&self, id: String) -> bool;
    
    async fn login(&mut self) -> Result<()>;
    async fn authorize(&mut self, code: String) -> Result<()>;


    async fn fetch_user_details(&self) -> Result<()>;
    async fn fetch_user_playlists(&self, limit: u32, offset: u32) -> Result<Vec<QueryablePlaylist>>;
    async fn get_playlist_content(&self, playlist_id: String, limit: u32, offset: u32) -> Result<Vec<Song>>;
}
