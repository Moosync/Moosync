use std::collections::HashMap;
use std::fmt::Debug;

use leptos::{
    create_read_slice, expect_context, spawn_local, RwSignal, SignalGet, SignalSet, SignalUpdate,
};
use serde::Serialize;
use serde_wasm_bindgen::{from_value, to_value};
use types::entities::{QueryableAlbum, QueryableArtist, QueryablePlaylist, SearchResult};
use types::errors::Result;
use types::providers::generic::{Pagination, ProviderStatus};
use types::songs::Song;
use wasm_bindgen::JsValue;

use crate::players::librespot::LibrespotPlayer;
use crate::store::modal_store::{ModalStore, Modals};
use crate::utils::common::{invoke, listen_event};
use crate::utils::invoke::{
    get_all_status, get_provider_key_by_id, get_provider_keys, initialize_all_providers,
};

#[derive(Debug, Default)]
pub struct ProviderStore {
    keys: RwSignal<Vec<String>>,
    statuses: RwSignal<Vec<ProviderStatus>>,
    unlisten_provider_key: Option<js_sys::Function>,
    pub is_initialized: RwSignal<bool>,
}

#[cfg(not(feature = "mock"))]
macro_rules! generate_async_functions {
    ($($func_name:ident {
        args: { $($arg_name:ident: $arg_type:ty),* $(,)? },
        result_type: $result_type:ty,
    }),* $(,)?) => {
        $(
            #[tracing::instrument(level = "trace", skip(self))]
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
            #[tracing::instrument(level = "trace", skip(self))]
            pub async fn $func_name(&self, key: String, $($arg_name: $arg_type),*) -> Result<$result_type> {
                Ok(Default::default())
            }
        )*
    }
}

impl ProviderStore {
    #[tracing::instrument(level = "trace", skip())]
    pub fn new() -> Self {
        tracing::debug!("Creating provider store");
        let mut store = Self::default();
        store.is_initialized.set(false);

        let fetch_provider_keys = move || {
            spawn_local(async move {
                let provider_keys = get_provider_keys().await;
                if provider_keys.is_err() {
                    tracing::debug!("Failed to get provider keys");
                    return;
                }
                store.keys.set(provider_keys.unwrap());
                tracing::debug!("Updated provider keys {:?}", store.keys.get());
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
            tracing::debug!("Got status update {:?}", provider_status);
            store
                .statuses
                .set(provider_status.values().cloned().collect());

            let modal_store: RwSignal<ModalStore> = expect_context();
            let get_active_modal = create_read_slice(modal_store, |m| m.get_active_modal());
            if let Some(Modals::LoginModal(_, _, _)) = get_active_modal.get() {
                modal_store.update(|m| m.clear_active_modal());
            }

            if let Some(spotify) = provider_status.get("spotify") {
                if spotify.user_name.is_some() {
                    LibrespotPlayer::set_initialized(true);
                }
            }
        });

        spawn_local(async move {
            tracing::debug!("Initializing providers");

            #[cfg(not(feature = "mock"))]
            {
                let res = initialize_all_providers().await;
                if res.is_err() {
                    tracing::error!("Failed to initialize providers");
                }
                let statuses = get_all_status().await.unwrap();

                store.statuses.set(statuses.values().cloned().collect());
                store.is_initialized.set(true);
            }
        });

        store
    }

    #[tracing::instrument(level = "trace", skip(self))]
    pub fn get_all_statuses(&self) -> RwSignal<Vec<ProviderStatus>> {
        self.statuses
    }

    #[tracing::instrument(level = "trace", skip(self, id))]
    pub async fn get_provider_key_by_id(&self, id: String) -> Result<String> {
        get_provider_key_by_id(id).await
    }

    #[tracing::instrument(level = "trace", skip(self))]
    pub fn get_provider_keys(&self) -> Vec<String> {
        self.keys.get()
    }

    generate_async_functions!(
        provider_login {
            args: {
                account_id: String
            },
            result_type: String,
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
        get_album_content {
            args: {
                album: QueryableAlbum,
                pagination: Pagination
            },
            result_type: (Vec<Song>, Pagination),
        },
        get_artist_content {
            args: {
                artist: QueryableArtist,
                pagination: Pagination
            },
            result_type: (Vec<Song>, Pagination),
        },
    );
}
