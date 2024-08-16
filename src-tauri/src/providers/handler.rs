use futures::{
    channel::mpsc::{unbounded, UnboundedReceiver, UnboundedSender},
    future::join_all,
    lock::Mutex,
    StreamExt,
};
use serde_json::Value;
use std::{collections::HashMap, sync::Arc};

use database::cache::CacheHolder;
use macros::{generate_command_async, generate_command_async_cached};
use tauri::{
    async_runtime::{self, block_on},
    AppHandle, Emitter, Manager, State,
};
use types::{
    entities::{QueryablePlaylist, SearchResult},
    errors::errors::{MoosyncError, Result},
    providers::generic::{GenericProvider, Pagination, ProviderStatus},
    songs::Song,
};

use crate::{extensions::get_extension_handler, providers::extension::ExtensionProvider};

use super::{spotify::SpotifyProvider, youtube::YoutubeProvider};

macro_rules! generate_wrapper {
    ($($func_name:ident {
        args: { $($param_name:ident: $param_type:ty),* $(,)? },
        result_type: $result_type:ty,
        method_name: $method_name:ident,
    }),* $(,)?) => {
        $(
            pub async fn $func_name(&self, key: String, $($param_name: $param_type),*) -> Result<$result_type> {
                let mut provider_key = key;
                loop {
                    let provider_store = self.provider_store.lock().await;
                    let provider = provider_store.get(&provider_key);
                    if let Some(provider) = provider {
                        let provider = provider.lock().await;
                        println!("calling provider {} - {}", provider_key, stringify!($method_name));
                        let res = provider.$method_name($($param_name.clone()),*).await;
                        match res {
                            Ok(result) => return Ok(result),
                            Err(MoosyncError::SwitchProviders(e)) => provider_key = e,
                            Err(err) => return Err(err),
                        }
                    }

                    return Err(format!("Provider ({}) not found", provider_key).into());
                }
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
                let mut provider_key = key;
                loop {
                    let provider_store = self.provider_store.lock().await;
                    let provider = provider_store.get(&provider_key);
                    if let Some(provider) = provider {
                        println!("calling provider {} - {}", provider_key, stringify!($method_name));
                        let mut provider = provider.lock().await;
                        let res = provider.$method_name($($param_name.clone()),*).await;
                        match res {
                            Ok(result) => return Ok(result),
                            Err(MoosyncError::SwitchProviders(e)) => provider_key = e,
                            Err(err) => return Err(err),
                        }
                    }

                    return Err(format!("Provider ({}) not found", provider_key).into());
                }
            }
        )*
    }
}

#[derive(Debug)]
pub struct ProviderHandler {
    provider_store: Mutex<HashMap<String, Arc<Mutex<dyn GenericProvider>>>>,
    app_handle: AppHandle,
    status_tx: UnboundedSender<ProviderStatus>,
    provider_status: Arc<Mutex<HashMap<String, ProviderStatus>>>,
}

impl ProviderHandler {
    pub fn new(app: AppHandle) -> Self {
        let (status_tx, status_rx) = unbounded();
        let store = Self {
            app_handle: app.clone(),
            provider_store: Default::default(),
            status_tx,
            provider_status: Default::default(),
        };
        store.listen_status_changes(status_rx);

        let mut provider_store = block_on(store.provider_store.lock());

        let spotify_provider = SpotifyProvider::new(app.clone(), store.status_tx.clone());
        provider_store.insert(
            spotify_provider.key(),
            Arc::new(Mutex::new(spotify_provider)),
        );

        let youtube_provider: YoutubeProvider = YoutubeProvider::new(app, store.status_tx.clone());
        provider_store.insert(
            youtube_provider.key(),
            Arc::new(Mutex::new(youtube_provider)),
        );
        drop(provider_store);

        store
    }

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
                println!("Got provider status update {:?}", status);
                let mut provider_status = provider_status.lock().await;
                provider_status.insert(status.key.clone(), status);
                let res = app_handle.emit("provider-status-update", provider_status.clone());
                if let Err(e) = res {
                    println!("Error emitting status update: {:?}", e);
                }
                println!("Emitted status update");
            }
        });
    }

    pub async fn discover_provider_extensions(&self) -> Result<()> {
        let ext_handler = get_extension_handler(&self.app_handle);
        let extensions_res = ext_handler.get_installed_extensions().await?;
        for ext_runner in extensions_res.values() {
            for extension in ext_runner {
                let provides = ext_handler
                    .get_provider_scopes(extension.package_name.clone().into())
                    .await;
                println!("Got provider scopes from {}", extension.package_name);
                if let Ok(provides) = provides {
                    if provides.is_empty() {
                        continue;
                    }

                    println!(
                        "Inserting extension provider {:?} {:?}",
                        extension, provides
                    );

                    let mut provider = ExtensionProvider::new(
                        extension.clone(),
                        provides,
                        self.app_handle.clone(),
                        self.status_tx.clone(),
                    );
                    let mut provider_store = self.provider_store.lock().await;
                    provider_store.remove(provider.key().as_str());
                    provider_store.insert(provider.key(), Arc::new(Mutex::new(provider.clone())));

                    println!("provider_store: {:?}", provider_store);
                    async_runtime::spawn(async move {
                        let res = provider.initialize().await;
                        if let Err(err) = res {
                            println!(
                                "Error initializing extension provider {}: {:?}",
                                provider.key(),
                                err
                            );
                        }
                    });
                }
            }
            self.app_handle.emit("providers-updated", Value::Null)?;
        }
        Ok(())
    }

    pub async fn initialize_all_providers(&self) -> Result<()> {
        let mut fut = vec![];
        let provider_store = self.provider_store.lock().await;
        for (_, provider) in provider_store.iter() {
            let provider = provider.clone();
            fut.push(Box::pin(async move {
                let mut provider = provider.lock().await;
                println!("Initializing {}", provider.key());
                provider.initialize().await;
                println!("Initialized {}", provider.key());
            }));
        }
        join_all(fut).await;
        Ok(())
    }

    pub async fn get_provider_key_by_id(&self, id: String) -> Result<String> {
        let provider_store = self.provider_store.lock().await;
        for (key, provider) in provider_store.iter() {
            let provider = provider.lock().await;
            if provider.match_id(id.clone()) {
                return Ok(key.clone());
            }
        }
        Err(format!("Provider for id {} not found", id).into())
    }

    pub async fn get_provider_keys(&self) -> Result<Vec<String>> {
        let provider_store = self.provider_store.lock().await;
        Ok(provider_store.keys().cloned().collect())
    }

    pub async fn get_all_status(&self) -> Result<HashMap<String, ProviderStatus>> {
        Ok(self.provider_status.lock().await.clone())
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
        fetch_user_playlists {
            args: {
                pagination: Pagination
            },
            result_type: (Vec<QueryablePlaylist>, Pagination),
            method_name: fetch_user_playlists,
        },
        fetch_playlist_content {
            args: {
                playlist_id: String,
                pagination: Pagination
            },
            result_type: (Vec<Song>, Pagination),
            method_name: get_playlist_content,
        },
        fetch_playback_url {
            args: {
                song: Song,
                player: String
            },
            result_type: String,
            method_name: get_playback_url,
        },
        provider_search {
            args: {
                term: String
            },
            result_type: SearchResult,
            method_name: search,
        },
        playlist_from_url {
            args: {
                url: String
            },
            result_type: QueryablePlaylist,
            method_name: playlist_from_url,
        },
        match_url {
            args: {
                url: String
            },
            result_type: bool,
            method_name: match_url,
        }
    );
}

pub fn get_provider_handler_state(app: AppHandle) -> ProviderHandler {
    ProviderHandler::new(app)
}

generate_command_async!(get_provider_keys, ProviderHandler, Vec<String>,);
generate_command_async!(initialize_all_providers, ProviderHandler, (),);
generate_command_async!(provider_login, ProviderHandler, (), key: String);
generate_command_async!(provider_authorize, ProviderHandler, (), key: String, code: String);
generate_command_async!(get_provider_key_by_id, ProviderHandler, String, id: String);
generate_command_async_cached!(fetch_user_playlists, ProviderHandler, (Vec<QueryablePlaylist>, Pagination), key: String, pagination: Pagination);
generate_command_async_cached!(fetch_playlist_content, ProviderHandler, (Vec<Song>, Pagination), key: String, playlist_id: String, pagination: Pagination);
generate_command_async_cached!(fetch_playback_url, ProviderHandler, String, key: String, song: Song, player: String);
generate_command_async_cached!(provider_search, ProviderHandler, SearchResult, key: String, term: String);
generate_command_async!(get_all_status, ProviderHandler, HashMap<String, ProviderStatus>, );
generate_command_async_cached!(playlist_from_url, ProviderHandler, QueryablePlaylist, key: String, url: String);
generate_command_async_cached!(match_url, ProviderHandler, bool, key: String, url: String);
