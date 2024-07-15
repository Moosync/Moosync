use futures::lock::Mutex;
use std::{collections::HashMap, sync::Arc};

use macros::generate_command_async;
use tauri::{AppHandle, State};
use types::{
    entities::QueryablePlaylist,
    errors::errors::Result,
    providers::generic::{GenericProvider, ProviderStatus},
};

use super::spotify::SpotifyProvider;

macro_rules! generate_wrapper {
    (
        $func_name:ident,
        $provider_func:ident,
        ($($param_name:ident: $param_type:ty),*),
        $return_type:ty
    ) => {
        pub async fn $func_name(&self, key: String, $($param_name: $param_type),*) -> Result<$return_type> {
            let provider = self.provider_store.get(&key);
            if let Some(provider) = provider {
                let provider = provider.lock().await;
                println!("calling provider {} - {}", key, stringify!($provider_func));
                let res = provider.$provider_func($($param_name),*).await?;
                return Ok(res);
            }

            Err(format!("Provider ({}) not found", key).into())
        }
    }
}

macro_rules! generate_wrapper_mut {
    (
        $func_name:ident,
        $provider_func:ident,
        ($($param_name:ident: $param_type:ty),*),
        $return_type:ty
    ) => {
        pub async fn $func_name(&self, key: String, $($param_name: $param_type),*) -> Result<$return_type> {
            let provider = self.provider_store.get(&key);
            if let Some(provider) = provider {
                let mut provider = provider.lock().await;
                let res = provider.$provider_func($($param_name),*).await?;
                return Ok(res);
            }

            Err(format!("Provider ({}) not found", key).into())
        }
    }
}

#[derive(Debug, Default)]
pub struct ProviderHandler {
    provider_store: HashMap<String, Arc<Mutex<dyn GenericProvider>>>,
}

impl ProviderHandler {
    pub fn new(app: AppHandle) -> Self {
        let mut store = Self::default();

        let spotify_provider = SpotifyProvider::new(app);
        store.provider_store.insert(
            spotify_provider.key().into(),
            Arc::new(Mutex::new(spotify_provider)),
        );
        store
    }

    pub async fn initialize_all_providers(&self) -> Result<()> {
        for (_, provider) in &self.provider_store {
            let mut provider = provider.lock().await;
            provider.initialize().await?;
        }
        Ok(())
    }

    generate_wrapper_mut!(provider_login, login, (), ());
    generate_wrapper_mut!(provider_authorize, authorize, (code: String), ());

    generate_wrapper!(fetch_user_details, fetch_user_details, (), ProviderStatus);

    generate_wrapper!(
        fetch_user_playlists,
        fetch_user_playlists,
        (limit: u32, offset: u32),
        Vec<QueryablePlaylist>
    );
}

pub fn get_provider_handler_state(app: AppHandle) -> ProviderHandler {
    ProviderHandler::new(app)
}

generate_command_async!(initialize_all_providers, ProviderHandler, (),);
generate_command_async!(provider_login, ProviderHandler, (), key: String);
generate_command_async!(provider_authorize, ProviderHandler, (), key: String, code: String);
generate_command_async!(fetch_user_details, ProviderHandler, ProviderStatus, key: String);
generate_command_async!(fetch_user_playlists, ProviderHandler, Vec<QueryablePlaylist>, key: String, limit: u32, offset: u32);
