use std::fmt::Debug;

use leptos::{spawn_local, RwSignal, SignalGet, SignalSet, SignalUpdate};
use serde::Serialize;
use serde_wasm_bindgen::{from_value, to_value};
use types::entities::{QueryablePlaylist, SearchResult};
use types::errors::errors::Result;
use types::providers::generic::{Pagination, ProviderStatus};
use types::songs::Song;
use wasm_bindgen::JsValue;

use crate::console_log;
use crate::utils::common::invoke;

#[derive(Debug, Default)]
pub struct ProviderStore {
    keys: RwSignal<Vec<String>>,
    statuses: RwSignal<Vec<RwSignal<ProviderStatus>>>,
}

macro_rules! generate_async_functions {
    ($($func_name:ident {
        args: { $($arg_name:ident: $arg_type:ty),* $(,)? },
        result_type: $result_type:ty,
    }),* $(,)?) => {
        $(
            pub async fn $func_name(&self, key: String, $($arg_name: $arg_type),*) -> Result<$result_type> {
                #[derive(Debug, Serialize)]
                struct Args {
                    key: String,
                    $($arg_name: $arg_type),*
                }
                let args = Args {
                    key,
                    $($arg_name),*
                };
                let res = invoke(
                    stringify!($func_name),
                    to_value(&args).unwrap(),
                ).await;

                Ok(from_value(res)?)
            }
        )*
    }
}

impl ProviderStore {
    pub fn new() -> Self {
        console_log!("Creating provider store");
        let store = Self::default();
        spawn_local(async move {
            console_log!("Initializing providers");
            invoke("initialize_all_providers", JsValue::undefined()).await;
            let provider_keys = invoke("get_provider_keys", JsValue::undefined()).await;
            store.keys.set(from_value(provider_keys).unwrap());
        });

        store.statuses.update(|statuses| {
            statuses.push(RwSignal::new(ProviderStatus {
                key: "spotify".to_string(),
                name: "Spotify".to_string(),
                user_name: None,
                logged_in: false,
            }));

            statuses.push(RwSignal::new(ProviderStatus {
                key: "youtube".to_string(),
                name: "Youtube".to_string(),
                user_name: None,
                logged_in: false,
            }));
        });
        store
    }

    pub fn get_all_statuses(&self) -> RwSignal<Vec<RwSignal<ProviderStatus>>> {
        self.statuses
    }

    pub async fn get_provider_key_by_id(&self, id: String) -> Result<String> {
        // TODO: Fetch valid key
        #[derive(Debug, Serialize)]
        struct Args {
            id: String,
        }
        let res = invoke("get_provider_key_by_id", to_value(&Args { id }).unwrap()).await;
        Ok(from_value(res)?)
    }

    pub fn get_provider_keys(&self) -> Vec<String> {
        self.keys.get()
    }

    generate_async_functions!(
        provider_login {
            args: {

            },
            result_type: (),
        },
        provider_authorize {
          args: {
              code: String
          },
          result_type: (),
        },
        fetch_user_playlists {
            args: {
                pagination: Pagination
            },
            result_type: (Vec<QueryablePlaylist>, Pagination),
        },
        fetch_playlist_content {
            args: {
                playlistId: String,
                pagination: Pagination
            },
            result_type: (Vec<Song>, Pagination),
        },
        fetch_playback_url {
            args: {
                song: Song,
                player: String
            },
            result_type: String,
        },
        provider_search {
            args: {
                term: String
            },
            result_type: SearchResult,
        }
    );
}
