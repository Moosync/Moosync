use futures::lock::Mutex;
use std::{collections::HashMap, sync::Arc};

use macros::{generate_command, generate_command_async};
use tauri::{AppHandle, State};
use types::{
    entities::QueryablePlaylist,
    errors::errors::Result,
    providers::generic::{GenericProvider, ProviderStatus},
    songs::Song,
};

use super::spotify::SpotifyProvider;

macro_rules! generate_wrapper {
    ($($func_name:ident {
        args: { $($param_name:ident: $param_type:ty),* $(,)? },
        result_type: $result_type:ty,
        method_name: $method_name:ident,
    }),* $(,)?) => {
        $(
            pub async fn $func_name(&self, key: String, $($param_name: $param_type),*) -> Result<$result_type> {
                let provider = self.provider_store.get(&key);
                if let Some(provider) = provider {
                    println!("calling provider {} - {}", key, stringify!($method_name));
                    let provider = provider.lock().await;
                    let res = provider.$method_name($($param_name),*).await;
                    return Ok(res?);
                }

                Err(format!("Provider ({}) not found", key).into())
            }
        )*
    }
}

macro_rules! generate_wrapper_mut {
    ($($func_name:ident {
        args: { $($param_name:ident: $param_type:ty),* $(,)? },
        result_type: $result_type:ty,
        method_name: $method_name:ident,
    }),* $(,)?) => {
        $(
            pub async fn $func_name(&self, key: String, $($param_name: $param_type),*) -> Result<$result_type> {
                let provider = self.provider_store.get(&key);
                if let Some(provider) = provider {
                    println!("calling provider {} - {}", key, stringify!($provider_func));
                    let mut provider = provider.lock().await;
                    let res = provider.$method_name($($param_name),*).await;
                    return Ok(res?);
                }

                Err(format!("Provider ({}) not found", key).into())
            }
        )*
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

    pub async fn get_provider_key_by_id(&self, id: String) -> Result<String> {
        for (key, provider) in &self.provider_store {
            let provider = provider.lock().await;
            if provider.match_id(id.clone()) {
                return Ok(key.clone());
            }
        }
        Err(format!("Provider for id {} not found", id).into())
    }

    pub fn get_provider_keys(&self) -> Result<Vec<String>> {
        Ok(self.provider_store.keys().cloned().collect())
    }

    generate_wrapper_mut!(
        provider_login {
            args: {},
            result_type: (),
            method_name: login,
        },
        provider_authorize {
            args: { code: String },
            result_type: (),
            method_name: authorize,
        }
    );

    generate_wrapper!(
        fetch_user_details {
            args: {},
            result_type: ProviderStatus,
            method_name: fetch_user_details,
        },
        fetch_user_playlists {
            args: {
                limit: u32,
                offset: u32,
            },
            result_type: Vec<QueryablePlaylist>,
            method_name: fetch_user_playlists,
        },
        fetch_playlist_content {
            args: {
                playlist_id: String,
                limit: u32,
                offset: u32,
            },
            result_type: Vec<Song>,
            method_name: get_playlist_content,
        },
    );
}

pub fn get_provider_handler_state(app: AppHandle) -> ProviderHandler {
    ProviderHandler::new(app)
}

generate_command!(get_provider_keys, ProviderHandler, Vec<String>,);
generate_command_async!(initialize_all_providers, ProviderHandler, (),);
generate_command_async!(provider_login, ProviderHandler, (), key: String);
generate_command_async!(provider_authorize, ProviderHandler, (), key: String, code: String);
generate_command_async!(get_provider_key_by_id, ProviderHandler, String, id: String);
generate_command_async!(fetch_user_details, ProviderHandler, ProviderStatus, key: String);
generate_command_async!(fetch_user_playlists, ProviderHandler, Vec<QueryablePlaylist>, key: String, limit: u32, offset: u32);
generate_command_async!(fetch_playlist_content, ProviderHandler, Vec<Song>, key: String, playlist_id: String, limit: u32, offset: u32);
