use std::{collections::HashMap, fmt::Debug, rc::Rc, sync::Mutex};

use leptos::{spawn_local, RwSignal, SignalGet, SignalUpdate, SignalUpdateUntracked};
use serde::Serialize;
use serde_wasm_bindgen::{from_value, to_value};
use types::entities::QueryablePlaylist;
use types::errors::errors::Result;
use types::providers::generic::ProviderStatus;
use types::songs::Song;
use wasm_bindgen::JsValue;

use crate::console_log;
use crate::utils::common::invoke;

#[derive(Debug, Default)]
pub struct ProviderStore {
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

                Ok(from_value(res).unwrap())
            }
        )*
    }
}

impl ProviderStore {
    pub fn new() -> Self {
        console_log!("Creating provider store");
        spawn_local(async move {
            console_log!("Initializing providers");
            invoke("initialize_all_providers", JsValue::undefined()).await;
        });
        let store = Self::default();

        store.statuses.update(|statuses| {
            statuses.push(RwSignal::new(ProviderStatus {
                key: "spotify".to_string(),
                name: "Spotify".to_string(),
                user_name: None,
                logged_in: false,
            }))
        });
        store
    }

    pub fn get_providers(&self) -> Vec<&str> {
        return vec!["spotify"];
    }

    pub fn get_all_statuses(&self) -> RwSignal<Vec<RwSignal<ProviderStatus>>> {
        self.statuses
    }

    pub async fn get_provider_key_by_id(&self, id: String) -> Option<String> {
        // TODO: Fetch valid key
        return Some("spotify".to_string());
    }

    pub fn get_provider_keys(&self) -> Vec<String> {
        self.get_providers()
            .into_iter()
            .map(|s| s.to_string())
            .collect()
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
                limit: u32,
                offset: u32
            },
            result_type: Vec<QueryablePlaylist>,
        },
        fetch_playlist_content {
            args: {
                playlistId: String,
                limit: u32,
                offset: u32,
            },
            result_type: Vec<Song>,
        },
    );
}
