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

// Moosync
// Copyright (C) 2025 Moosync
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
// along with this program. If not, see <http://www.gnu.org/licenses/>.

use crate::components::cardview::{CardView, SimplifiedCardItem};
use crate::components::songview::SongView;
use crate::dyn_provider_songs;
use crate::modals::new_playlist_modal::PlaylistModalState;
use crate::store::modal_store::{ModalStore, Modals};
use crate::store::player_store::PlayerStore;
use crate::store::ui_store::{PlaylistSortByColumns, UiStore};
use crate::utils::common::{convert_file_src, fetch_infinite};
use crate::utils::context_menu::{
    create_context_menu, PlaylistContextMenu, PlaylistItemContextMenu,
};
use crate::utils::db_utils::get_songs_by_option;
use crate::utils::songs::get_songs_from_indices;
use leptos::task::spawn_local;
use leptos::{component, prelude::*, view, IntoView};
use leptos_router::hooks::use_query_map;
use rand::seq::SliceRandom;
use std::collections::HashMap;
use std::sync::Arc;
use types::entities::QueryablePlaylist;
use types::songs::{GetSongOptions, Song};
use types::ui::song_details::{DefaultDetails, SongDetailIcons};

use crate::store::provider_store::ProviderStore;
use crate::{icons::plus_button::PlusIcon, utils::db_utils::get_playlists_by_option};

#[tracing::instrument(level = "trace", skip())]
#[component()]
pub fn SinglePlaylist() -> impl IntoView {
    let params = use_query_map();
    let playlist = Memo::new(move |_| {
        params.with(|params| {
            let entity = params.get("entity");
            if let Some(entity) = entity {
                let album = serde_json::from_str::<QueryablePlaylist>(&entity);
                if let Ok(album) = album {
                    return Some(album);
                }
            }
            None
        })
    });

    let songs = RwSignal::new(vec![]);
    let selected_songs = RwSignal::new(vec![]);

    let provider_store = use_context::<Arc<ProviderStore>>().unwrap();
    let provider_store_clone = provider_store.clone();
    let selected_providers = RwSignal::<Vec<String>>::new(vec![]);

    let (_, filtered_songs, fetch_selected_providers) =
        dyn_provider_songs!(selected_providers, playlist, songs, fetch_playlist_content);

    let default_details = RwSignal::new(DefaultDetails::default());

    let refresh_songs = move || {
        tracing::debug!("Refreshing song list");
        let playlist = playlist.get();
        if let Some(playlist) = playlist {
            get_songs_by_option(
                GetSongOptions {
                    playlist: Some(QueryablePlaylist {
                        playlist_id: Some(playlist.playlist_id.unwrap()),
                        ..Default::default()
                    }),
                    ..Default::default()
                },
                songs,
            );
        }
    };

    Effect::new(move || {
        let playlist = playlist.get();
        if let Some(playlist) = playlist {
            default_details.update(|d| {
                d.title = Some(playlist.playlist_name.clone());
                d.icon = playlist.playlist_coverpath.clone().map(convert_file_src);
            });

            let playlist_id = playlist.playlist_id.clone().unwrap();
            let provider_store = provider_store_clone.clone();
            spawn_local(async move {
                let provider = provider_store
                    .get_provider_key_by_id(playlist_id.clone())
                    .await;
                match provider {
                    Ok(provider) => {
                        selected_providers.update(|s| s.push(provider));
                    }
                    Err(e) => tracing::error!("{}", e),
                }
            });

            refresh_songs();
        }
    });

    let player_store = expect_context::<RwSignal<PlayerStore>>();
    let play_songs_setter = create_write_slice(player_store, |p, song| p.play_now(song));
    let play_songs_multiple_setter =
        create_write_slice(player_store, |p, songs| p.play_now_multiple(songs));
    let add_to_queue_setter = create_write_slice(player_store, |p, songs| p.add_to_queue(songs));

    let play_songs = move || {
        let selected_songs = if selected_songs.get().is_empty() {
            filtered_songs.get()
        } else {
            get_songs_from_indices(&filtered_songs, selected_songs)
        };

        play_songs_multiple_setter.set(selected_songs);
    };

    let add_to_queue = move || {
        if selected_songs.get().is_empty() {
            add_to_queue_setter.set(filtered_songs.get());
        } else {
            add_to_queue_setter.set(get_songs_from_indices(&filtered_songs, selected_songs));
        }
    };

    let random = move || {
        let songs = filtered_songs.get();
        let random_song = songs.choose(&mut rand::thread_rng()).unwrap();
        play_songs_setter.set(random_song.clone());
    };

    let icons = RwSignal::new(SongDetailIcons {
        play: Some(Arc::new(Box::new(play_songs))),
        add_to_queue: Some(Arc::new(Box::new(add_to_queue))),
        random: Some(Arc::new(Box::new(random))),
        ..Default::default()
    });

    let fetch_next_page = move || {
        fetch_selected_providers.as_ref()();
    };

    let is_mobile =
        create_read_slice(expect_context::<RwSignal<UiStore>>(), |u| u.get_is_mobile()).get();

    view! {
        <SongView
            default_details=default_details
            songs=filtered_songs
            icons=icons
            selected_songs=selected_songs
            refresh_cb=refresh_songs
            fetch_next_page=fetch_next_page
            show_mobile_default_details=is_mobile
        />
    }
}

#[tracing::instrument(level = "trace", skip())]
#[component()]
pub fn AllPlaylists() -> impl IntoView {
    let playlists = RwSignal::new(vec![]);

    let owner = Owner::new();
    let refresh_playlist_items: Arc<Box<dyn Fn() + Send + Sync>> = Arc::new(Box::new(move || {
        tracing::debug!("Refreshing playlists");
        owner.with(|| {
            get_playlists_by_option(QueryablePlaylist::default(), playlists.write_only());
        });
    }));

    refresh_playlist_items.as_ref()();

    let modal_manager = expect_context::<RwSignal<ModalStore>>();
    let refresh_item_clone = refresh_playlist_items.clone();
    let open_new_playlist_modal = move |_| {
        modal_manager.update(|m| {
            m.set_active_modal(Modals::NewPlaylistModal(PlaylistModalState::None, None));
            let refresh_item_clone = refresh_item_clone.clone();
            m.on_modal_close(move || {
                refresh_item_clone.as_ref()();
            });
        });
    };

    let ui_store = expect_context::<RwSignal<UiStore>>();
    let playlist_context_menu = create_context_menu(PlaylistContextMenu {
        refresh_cb: refresh_playlist_items.clone(),
    });

    let playlist_sort = create_read_slice(ui_store, |u| u.get_playlist_sort_by());

    let sorted_playlists = Memo::new(move |_| {
        let mut playlists = playlists.get();
        let sort = playlist_sort.get();
        match sort.sort_by {
            PlaylistSortByColumns::Title => {
                playlists.sort_by(|a, b| a.playlist_name.cmp(&b.playlist_name))
            }
            PlaylistSortByColumns::Provider => {
                playlists.sort_by(|a, b| a.extension.cmp(&b.extension))
            }
        }
        playlists
    });

    let playlist_item_context_menu = create_context_menu(PlaylistItemContextMenu {
        playlist: None,
        refresh_cb: refresh_playlist_items,
    });

    view! {
        <div class="w-100 h-100">
            <div
                on:contextmenu=move |ev| {
                    ev.prevent_default();
                    playlist_context_menu.show(ev);
                }
                class="container-fluid song-container h-100 d-flex flex-column"
            >
                <div class="row page-title no-gutters">

                    <div class="col-auto">Playlists</div>
                    <div
                        class="col-auto button-grow playlists-plus-icon"
                        on:click=open_new_playlist_modal
                    >
                        <PlusIcon />
                    </div>

                    <div class="col align-self-center"></div>
                </div>

                <div
                    class="row no-gutters w-100 flex-grow-1"
                    style="align-items: flex-start; height: 70%"
                >
                    <CardView
                        items=sorted_playlists
                        key=|a| a.playlist_id.clone()
                        redirect_root="/main/playlists"
                        card_item=move |(_, item)| {
                            let playlist_name = item.playlist_name.clone();
                            let playlist_coverpath = item.playlist_coverpath.clone();
                            let playlist_extension = item.extension.clone();
                            let playlist_item_context_menu = playlist_item_context_menu.clone();
                            SimplifiedCardItem {
                                title: playlist_name,
                                cover: playlist_coverpath,
                                id: item.clone(),
                                icon: playlist_extension,
                                context_menu: Some(
                                    Arc::new(
                                        Box::new(move |ev, playlist| {
                                            ev.prevent_default();
                                            ev.stop_propagation();
                                            let mut data = playlist_item_context_menu.get_data();
                                            data.playlist = Some(playlist);
                                            drop(data);
                                            playlist_item_context_menu.show(ev);
                                        }),
                                    ),
                                ),
                            }
                        }
                    />
                </div>
            </div>
        </div>
    }
}
