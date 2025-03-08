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

use leptos::task::spawn_local;
use leptos::{component, prelude::*, view, IntoView};
use leptos_i18n::t;
use types::entities::QueryablePlaylist;
use types::songs::Song;
use types::ui::extensions::ExtensionProviderScope;

use crate::i18n::use_i18n;
use crate::icons::{
    import_playlist_icon::ImportPlaylistIcon, new_playlist_icon::NewPlaylistIcon,
    song_default_icon::SongDefaultIcon,
};
use crate::store::modal_store::ModalStore;
use crate::utils::db_utils::create_playlist;
use crate::utils::invoke::{match_url, playlist_from_url};
use crate::{modals::common::GenericModal, store::provider_store::ProviderStore};

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum PlaylistModalState {
    None,
    NewPlaylist,
    ImportPlaylist,
}

#[tracing::instrument(level = "debug", skip())]
#[component]
pub fn NewPlaylistModal(
    #[prop()] initial_state: PlaylistModalState,
    #[prop()] songs: Option<Vec<Song>>,
) -> impl IntoView {
    let state = RwSignal::new(initial_state);

    let playlist = RwSignal::new(None::<QueryablePlaylist>);
    let import_url = RwSignal::new(String::new());
    let provider_store: Arc<ProviderStore> = expect_context();
    Effect::new(move || {
        let import_url = import_url.get().clone();
        if import_url.is_empty() {
            playlist.set(None);
            return;
        }
        let provider_store = provider_store.clone();
        spawn_local(async move {
            let import_url = import_url.clone();
            for key in provider_store.get_provider_keys(ExtensionProviderScope::PlaylistFromUrl) {
                if let Ok(matched) = match_url(key.clone(), import_url.clone()).await {
                    if matched {
                        let imported = playlist_from_url(key.clone(), import_url.clone()).await;
                        if let Ok(imported) = imported {
                            playlist.set(Some(imported));
                        }
                    }
                }
            }
        });
    });

    let modal_store: RwSignal<ModalStore> = expect_context();
    let close_modal = move || modal_store.update(|m| m.clear_active_modal());
    let songs = RwSignal::new(songs);

    let create_new_playlist = move |_| {
        let playlist = playlist.get();
        let songs = songs.get_untracked();

        if playlist.is_none() {
            return;
        }

        let playlist = playlist.unwrap();
        if playlist.playlist_name.is_empty() {
            return;
        }

        create_playlist(playlist, songs);
        close_modal();
    };

    let i18n = use_i18n();

    view! {
        <GenericModal size=move || {
            {
                match state.get() {
                    PlaylistModalState::None => "modal-md",
                    PlaylistModalState::NewPlaylist => "modal-lg",
                    PlaylistModalState::ImportPlaylist => "modal-lg",
                }
            }
                .into()
        }>
            {move || match state.get() {
                PlaylistModalState::None => {
                    view! {
                        <div class="container">
                            <div class="row h-100">
                                <div
                                    class="col d-flex"
                                    on:click=move |_| state.set(PlaylistModalState::NewPlaylist)
                                >
                                    <div class="row item-box align-self-center">
                                        <div class="col-auto d-flex playlist-modal-item-container w-100">
                                            <div class="row w-100">
                                                <div class="col d-flex justify-content-center w-100">
                                                    <div class="item-icon">
                                                        <NewPlaylistIcon />
                                                    </div>
                                                </div>
                                            </div>
                                            <div class="row">
                                                <div class="col d-flex justify-content-center item-title">
                                                    {t!(i18n, contextMenu.playlist.newPlaylist)}
                                                </div>
                                            </div>
                                        </div>
                                    </div>
                                </div>
                                <div
                                    class="col d-flex"
                                    on:click=move |_| state.set(PlaylistModalState::ImportPlaylist)
                                >
                                    <div class="row item-box align-self-center">
                                        <div class="col-auto d-flex playlist-modal-item-container w-100">
                                            <div class="row w-100">
                                                <div class="col d-flex justify-content-center w-100">
                                                    <div class="item-icon">
                                                        <ImportPlaylistIcon />
                                                    </div>
                                                </div>
                                            </div>
                                            <div class="row">
                                                <div class="col d-flex justify-content-center item-title">
                                                    {t!(i18n, contextMenu.playlist.addFromURL)}
                                                </div>
                                            </div>
                                        </div>
                                    </div>
                                </div>
                            </div>
                        </div>
                    }
                        .into_any()
                }
                PlaylistModalState::NewPlaylist => {
                    view! {
                        <div class="modal-content-container">
                            <div class="container-fluid p-0">
                                <div class="row no-gutters d-flex" no-gutters="">
                                    <div class="playlist-cover">
                                        <SongDefaultIcon />
                                    </div>
                                    <div class="col playlist-details">
                                        <div class="d-flex">
                                            <input
                                                class="form-control playlist-title"
                                                id="playlist-title"
                                                placeholder="Playlist Name..."
                                                prop:value=move || {
                                                    playlist.get().unwrap_or_default().playlist_name
                                                }
                                                maxlength="20"
                                                on:input=move |e| {
                                                    playlist
                                                        .update(|p| {
                                                            if let Some(p) = p {
                                                                p.playlist_name = event_target_value(&e)
                                                            } else {
                                                                *p = Some(QueryablePlaylist {
                                                                    playlist_name: event_target_value(&e),
                                                                    ..Default::default()
                                                                })
                                                            }
                                                        })
                                                }
                                            />
                                        </div>
                                        <p class="songs-count">0 Songs</p>
                                    </div>
                                </div>
                                <div class="row no-gutters" no-gutters="">
                                    <textarea
                                        class="form-control playlist-desc"
                                        id="playlist-desc"
                                        placeholder="Description..."
                                        wrap="soft"
                                        style="resize: none; overflow-y: scroll; height: 72px;"
                                        prop:value=move || {
                                            playlist.get().unwrap_or_default().playlist_desc
                                        }
                                        on:input=move |e| {
                                            playlist
                                                .update(|p| {
                                                    if let Some(p) = p {
                                                        p.playlist_name = event_target_value(&e)
                                                    } else {
                                                        *p = Some(QueryablePlaylist {
                                                            playlist_name: event_target_value(&e),
                                                            ..Default::default()
                                                        })
                                                    }
                                                })
                                        }
                                    ></textarea>
                                </div>
                            </div>
                            <button
                                class="btn btn-secondary close-button ml-3"
                                on:click=move |_| close_modal()
                            >
                                Close
                            </button>
                            <button
                                class="btn btn-secondary create-button ml-3"
                                on:click=create_new_playlist
                                class:disabled=move || {
                                    let playlist = playlist.get();
                                    playlist.is_none() || playlist.unwrap().playlist_name.is_empty()
                                }
                            >
                                {t!(i18n, playlists.new_playlist.create)}
                            </button>
                        </div>
                    }
                        .into_any()
                }
                PlaylistModalState::ImportPlaylist => {
                    view! {
                        // let mut playlist = playlist.get().unwrap_or_default();
                        // playlist.playlist_name = event_target_value(&e);
                        // *p = Some(playlist);
                        <div class="modal-content-container">
                            <div class="container-fluid p-0">
                                <div class="row no-gutters d-flex">
                                    <div class="col-auto playlist-url-cover">
                                        {move || {
                                            if let Some(playlist) = playlist.get() {
                                                if let Some(cover) = playlist.playlist_coverpath {
                                                    return view! { <img class="h-100 w-100" src=cover /> }
                                                        .into_any();
                                                }
                                            }
                                            view! { <SongDefaultIcon /> }.into_any()
                                        }}
                                    </div>
                                    <div class="col-9">
                                        <div
                                            class="row no-gutters playlist-url-details"
                                            no-gutters=""
                                        >
                                            <div class="col-12 w-100">
                                                <div class="row w-100">
                                                    <div class="playlist-title text-truncate deactivated">
                                                        {move || {
                                                            let playlist = playlist.get();
                                                            if let Some(playlist) = playlist {
                                                                playlist.playlist_name.into_any()
                                                            } else {
                                                                t!(i18n, contextMenu.playlist.newPlaylist).into_any()
                                                            }
                                                        }}
                                                    </div>
                                                </div>
                                                <div class="row w-100">
                                                    <div class="playlist-subtitle text-truncate deactivated"></div>
                                                </div>
                                            </div>
                                        </div>
                                        <div class="row no-gutters" no-gutters="">
                                            <div class="col-12">
                                                <div class="container-fluid path-container w-100 input-group">
                                                    <div
                                                        class="row no-gutters import-playlist-background w-100 mt-2 d-flex"
                                                        no-gutters=""
                                                    >
                                                        <div
                                                            class="col-auto align-self-center h-100 ml-3 mr-3"
                                                            cols="auto"
                                                            align-self="center"
                                                        >
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
                                                        <div
                                                            class="col-auto align-self-center flex-grow-1 justify-content-start"
                                                            align-self="center"
                                                        >
                                                            <input
                                                                class="form-control ext-input"
                                                                id="ext-input"
                                                                type="text"
                                                                prop:value=import_url
                                                                on:input=move |e| import_url.set(event_target_value(&e))
                                                                placeholder=move || {
                                                                    i18n.get_keys()
                                                                        .playlists()
                                                                        .url()
                                                                        .input_hint()
                                                                        .build_string()
                                                                }
                                                            />
                                                        </div>
                                                    </div>
                                                </div>
                                            </div>
                                        </div>
                                    </div>
                                </div>
                            </div>
                            <button
                                class="btn btn-secondary close-button ml-3"
                                on:click=move |_| close_modal()
                            >
                                Close
                            </button>
                            <button
                                class:disabled=move || {
                                    let playlist = playlist.get();
                                    playlist.is_none()
                                }
                                on:click=create_new_playlist
                                class="btn btn-secondary create-button ml-3"
                            >
                                {t!(i18n, playlists.new_playlist.create)}
                            </button>
                        </div>
                    }
                        .into_any()
                }
            }}
        </GenericModal>
    }
}
