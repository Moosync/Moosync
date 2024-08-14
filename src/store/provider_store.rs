use std::collections::HashMap;
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
use crate::utils::common::{invoke, listen_event};

#[derive(Debug, Default)]
pub struct ProviderStore {
    keys: RwSignal<Vec<String>>,
    statuses: RwSignal<HashMap<String, ProviderStatus>>,
    unlisten_provider_key: Option<js_sys::Function>,
}

#[cfg(not(feature = "mock"))]
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
                ).await?;

                Ok(from_value(res)?)
            }
        )*
    }
}

#[cfg(feature = "mock")]
macro_rules! generate_async_functions {
    ($($func_name:ident {
        args: { $($arg_name:ident: $arg_type:ty),* $(,)? },
        result_type: $result_type:ty,
    }),* $(,)?) => {
        $(
            pub async fn $func_name(&self, key: String, $($arg_name: $arg_type),*) -> Result<$result_type> {
                Ok(Default::default())
            }
        )*
    }
}

impl ProviderStore {
    pub fn new() -> Self {
        console_log!("Creating provider store");
        let mut store = Self::default();

        let fetch_provider_keys = move || {
            spawn_local(async move {
                let provider_keys = invoke("get_provider_keys", JsValue::undefined()).await;
                if provider_keys.is_err() {
                    console_log!("Failed to get provider keys");
                    return;
                }
                store.keys.set(from_value(provider_keys.unwrap()).unwrap());
                console_log!("Updated provider keys {:?}", store.keys.get());
            });
        };

        store.unlisten_provider_key = Some(listen_event("providers-updated", move |_| {
            fetch_provider_keys();
        }));

        fetch_provider_keys();

        listen_event("provider-status-update", move |data: JsValue| {
            println!("Got status update {:?}", data);
            let payload = js_sys::Reflect::get(&data, &JsValue::from_str("payload")).unwrap();
            let provider_status = serde_wasm_bindgen::from_value(payload).unwrap();
            store.statuses.set(provider_status);
        });

        spawn_local(async move {
            console_log!("Initializing providers");

            #[cfg(not(feature = "mock"))]
            {
                let res = invoke("initialize_all_providers", JsValue::undefined()).await;
                if res.is_err() {
                    console_log!("Failed to initialize providers");
                }
                let status = invoke("get_all_status", JsValue::undefined())
                    .await
                    .unwrap();

                store
                    .statuses
                    .set(serde_wasm_bindgen::from_value(status).unwrap());
            }
        });

        store
    }

    pub fn get_all_statuses(&self) -> RwSignal<HashMap<String, ProviderStatus>> {
        self.statuses
    }

    pub async fn get_provider_key_by_id(&self, id: String) -> Result<String> {
        #[derive(Debug, Serialize)]
        struct Args {
            id: String,
        }
        let res = invoke("get_provider_key_by_id", to_value(&Args { id }).unwrap()).await?;
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
