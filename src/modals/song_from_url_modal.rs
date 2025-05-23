// Moosync
// Copyright (C) 2024, 2025  Moosync <support@moosync.app>
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <http://www.gnu.org/licenses/>.

use std::sync::Arc;
use std::time::Duration;

use crate::modals::common::GenericModal;
use crate::store::provider_store::ProviderStore;
use crate::utils::common::get_high_img;
use crate::utils::db_utils::add_songs_to_library;
use crate::utils::invoke::{match_url, song_from_url};
use crate::{icons::song_default_icon::SongDefaultIcon, store::modal_store::ModalStore};
use leptos::task::spawn_local;
use leptos::{component, prelude::*, view, IntoView};
use types::songs::Song;
use types::ui::extensions::ExtensionProviderScope;

#[tracing::instrument(level = "debug", skip())]
#[component]
pub fn SongFromUrlModal() -> impl IntoView {
    let modal_store: RwSignal<ModalStore> = expect_context();
    let provider_store: Arc<ProviderStore> = expect_context();
    let close_modal = move || modal_store.update(|m| m.clear_active_modal());

    let imported_song = RwSignal::new(None::<Song>);
    let song_url = RwSignal::new("".to_string());

    Effect::new(move || {
        let song_url = song_url.get();
        if song_url.is_empty() {
            imported_song.set(None);
            return;
        }

        let provider_store = provider_store.clone();
        let provider_keys = provider_store.get_provider_keys(ExtensionProviderScope::SongFromUrl);
        spawn_local(async move {
            for key in provider_keys {
                let matched = match_url(key.clone(), song_url.clone(), false).await;
                if let Ok(matched) = matched {
                    if matched {
                        let imported = song_from_url(key, song_url, false).await;
                        if let Ok(song) = imported {
                            imported_song.set(Some(song));
                        }
                        return;
                    }
                }
            }
        });
    });

    let create_new_song = move |_| {
        let song = imported_song.get();
        if song.is_none() {
            return;
        }

        // TODO: Update all songs after importing song
        let refresh_songs = move || {};
        add_songs_to_library(vec![song.unwrap()], Arc::new(Box::new(refresh_songs)));
        set_timeout(
            move || {
                modal_store.update(|m| m.clear_active_modal());
            },
            Duration::from_millis(200),
        );
    };

    view! {
        <GenericModal size=move || "modal-lg".into()>
            <div class="modal-content-container">
                <div class="container-fluid p-0">
                    <div class="row no-gutters d-flex">
                        <div class="col-auto playlist-url-cover">
                            {move || {
                                if let Some(song) = imported_song.get() {
                                    if song.song.song_cover_path_high.is_some() {
                                        return view! {
                                            <img class="h-100 w-100" src=get_high_img(&song) />
                                        }
                                            .into_any();
                                    }
                                }
                                view! { <SongDefaultIcon /> }.into_any()
                            }}
                        </div>
                        <div class="col-9">
                            <div class="row no-gutters playlist-url-details">
                                <div class="col w-100">
                                    <div class="row w-100">
                                        <input
                                            class="playlist-title text-truncate deactivated"
                                            type="text"
                                            disabled=true
                                            placeholder="New Song"
                                            prop:value=move || {
                                                imported_song
                                                    .get()
                                                    .map(|s| s.song.title.clone())
                                                    .unwrap_or_default()
                                            }
                                        />
                                    </div>
                                    <div class="row w-100">
                                        {move || {
                                            imported_song
                                                .get()
                                                .map(|s| {
                                                    s.artists
                                                        .unwrap_or_default()
                                                        .iter()
                                                        .map(|a| a.artist_name.clone().unwrap_or_default())
                                                        .collect::<Vec<String>>()
                                                })
                                                .unwrap_or_default()
                                        }}
                                    </div>
                                </div>
                            </div>
                            <div class="row no-gutters" no-gutters="">
                                <div class="col-12">
                                    <div class="container-fluid path-container w-100 input-group">
                                        <div class="row no-gutters import-playlist-background w-100 mt-2 d-flex">
                                            <div class="col-auto align-self-center h-100 ml-3 mr-3">
                                                <svg

                                                    width="16"
                                                    height="16"
                                                    viewBox="0 0 16 16"
                                                    fill="none"
                                                    xmlns="http://www.w3.org/2000/svg"
                                                >
                                                    <path
                                                        d="M9.20145 6.79953C8.56434 6.16269 7.7004 5.80493 6.79958 5.80493C5.89876 5.80493 5.03481 6.16269 4.3977 6.79953L1.99505 9.2014C1.35793 9.83852 1 10.7026 1 11.6037C1 12.5047 1.35793 13.3688 1.99505 14.0059C2.63217 14.6431 3.49629 15.001 4.39731 15.001C5.29834 15.001 6.16246 14.6431 6.79958 14.0059L8.00052 12.805"
                                                        stroke="white"
                                                        stroke-width="1.55562"
                                                        stroke-linecap="round"
                                                        stroke-linejoin="round"
                                                    ></path>
                                                    <path
                                                        d="M6.79883 9.20145C7.43594 9.8383 8.29988 10.1961 9.2007 10.1961C10.1015 10.1961 10.9655 9.8383 11.6026 9.20145L14.0052 6.79958C14.6424 6.16246 15.0003 5.29834 15.0003 4.39731C15.0003 3.49629 14.6424 2.63217 14.0052 1.99505C13.3681 1.35793 12.504 1 11.603 1C10.7019 1 9.83782 1.35793 9.2007 1.99505L7.99977 3.19599"
                                                        stroke="white"
                                                        stroke-width="1.55562"
                                                        stroke-linecap="round"
                                                        stroke-linejoin="round"
                                                    ></path>
                                                </svg>
                                            </div>
                                            <div class="col-auto align-self-center flex-grow-1 justify-content-start">
                                                <input

                                                    class="form-control ext-input"
                                                    id="ext-input"
                                                    type="text"
                                                    prop:value=move || song_url.get()
                                                    on:input=move |e| song_url.set(event_target_value(&e))
                                                    placeholder="Enter URL Here.."
                                                />
                                            </div>
                                            <div

                                                class="col-auto mr-4"
                                                cols="auto"
                                            ></div>
                                        </div>
                                    </div>
                                </div>
                            </div>
                        </div>
                    </div>
                </div>
                <button class="btn btn-secondary close-button ml-3" on:click=move |_| close_modal()>
                    Close
                </button>
                <button
                    class="btn btn-secondary create-button ml-3"
                    on:click=create_new_song
                    class:disabled=move || { imported_song.get().is_none() }
                >
                    Create
                </button>
            </div>
        </GenericModal>
    }
}
