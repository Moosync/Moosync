use async_trait::async_trait;
use leptos::RwSignal;
use types::{errors::errors::Result, ui::providers::ProviderStatus};

#[async_trait(?Send)]
pub trait GenericProvider: std::fmt::Debug {
    async fn initialize(&mut self) -> Result<()>;
    fn key(&self) -> &str;
    fn get_status(&self) -> RwSignal<ProviderStatus>;
    async fn login(&mut self) -> Result<()>;
    async fn authorize(&mut self, code: String) -> Result<()>;

    async fn fetch_user_details(&self) -> Result<()>;
}
