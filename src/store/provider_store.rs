use std::collections::HashMap;
use std::fmt::Debug;

use leptos::{
    create_read_slice, expect_context, spawn_local, RwSignal, SignalGet, SignalSet, SignalUpdate,
};
use serde::Serialize;
use serde_wasm_bindgen::{from_value, to_value};
use types::entities::{QueryablePlaylist, SearchResult};
use types::errors::Result;
use types::providers::generic::{Pagination, ProviderStatus};
use types::songs::Song;
use wasm_bindgen::JsValue;

use crate::console_log;
use crate::store::modal_store::{ModalStore, Modals};
use crate::utils::common::{invoke, listen_event};

#[derive(Debug, Default)]
pub struct ProviderStore {
    keys: RwSignal<Vec<String>>,
    statuses: RwSignal<Vec<ProviderStatus>>,
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
                #[serde(rename_all = "camelCase")]
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
            let payload = js_sys::Reflect::get(&data, &JsValue::from_str("payload")).unwrap();
            let provider_status: HashMap<String, ProviderStatus> =
                serde_wasm_bindgen::from_value(payload).unwrap();
            console_log!("Got status update {:?}", provider_status);
            store
                .statuses
                .set(provider_status.values().cloned().collect());

            let modal_store: RwSignal<ModalStore> = expect_context();
            let get_active_modal = create_read_slice(modal_store, |m| m.get_active_modal());
            if let Some(Modals::LoginModal(_, _, _)) = get_active_modal.get() {
                modal_store.update(|m| m.clear_active_modal());
            }
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

                let statuses: HashMap<String, ProviderStatus> =
                    serde_wasm_bindgen::from_value(status).unwrap();
                store.statuses.set(statuses.values().cloned().collect());
            }
        });

        store
    }

    pub fn get_all_statuses(&self) -> RwSignal<Vec<ProviderStatus>> {
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
                account_id: String
            },
            result_type: (),
        },
        provider_signout {
            args: {
                account_id: String
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
                playlist_id: String,
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
        },
        playlist_from_url {
            args: {
                url: String
            },
            result_type: QueryablePlaylist,
        },
        song_from_url {
            args: {
                url: String
            },
            result_type: Song,
        },
        match_url {
            args: {
                url: String
            },
            result_type: bool,
        },
        get_suggestions {
            args: {

            },
            result_type: Vec<Song>,
        },
    );
}
