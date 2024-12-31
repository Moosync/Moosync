use crate::components::cardview::{CardView, SimplifiedCardItem};
use crate::components::songview::SongView;
use crate::dyn_provider_songs;
use crate::modals::new_playlist_modal::PlaylistModalState;
use crate::store::modal_store::{ModalStore, Modals};
use crate::store::player_store::PlayerStore;
use crate::store::ui_store::{PlaylistSortByColumns, UiStore};
use crate::utils::common::{convert_file_src, fetch_infinite};
use crate::utils::context_menu::{PlaylistContextMenu, PlaylistItemContextMenu};
use crate::utils::db_utils::get_songs_by_option;
use crate::utils::songs::get_songs_from_indices;
use leptos::{
    component, create_effect, create_memo, create_read_slice, create_rw_signal, create_write_slice,
    expect_context, spawn_local, use_context, view, IntoView, RwSignal, SignalGet, SignalUpdate,
    SignalWith,
};
use leptos_context_menu::ContextMenu;
use leptos_router::use_query_map;
use rand::seq::SliceRandom;
use std::collections::HashMap;
use std::rc::Rc;
use types::entities::QueryablePlaylist;
use types::songs::{GetSongOptions, Song};
use types::ui::song_details::{DefaultDetails, SongDetailIcons};

use crate::store::provider_store::ProviderStore;
use crate::{icons::plus_button::PlusIcon, utils::db_utils::get_playlists_by_option};

#[tracing::instrument(level = "trace", skip())]
#[component()]
pub fn SinglePlaylist() -> impl IntoView {
    let params = use_query_map();
    let playlist = create_memo(move |_| {
        params.with(|params| {
            let entity = params.get("entity");
            if let Some(entity) = entity {
                let album = serde_json::from_str::<QueryablePlaylist>(entity);
                if let Ok(album) = album {
                    return Some(album);
                }
            }
            None
        })
    });

    let songs = create_rw_signal(vec![]);
    let selected_songs = create_rw_signal(vec![]);

    let provider_store = use_context::<Rc<ProviderStore>>().unwrap();
    let provider_store_clone = provider_store.clone();
    let selected_providers = create_rw_signal::<Vec<String>>(vec![]);

    let (_, filtered_songs, fetch_selected_providers) =
        dyn_provider_songs!(selected_providers, playlist, songs, fetch_playlist_content);

    let default_details = create_rw_signal(DefaultDetails::default());

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

    create_effect(move |_| {
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

    let icons = create_rw_signal(SongDetailIcons {
        play: Some(Rc::new(Box::new(play_songs))),
        add_to_queue: Some(Rc::new(Box::new(add_to_queue))),
        random: Some(Rc::new(Box::new(random))),
        ..Default::default()
    });

    let fetch_next_page = move || {
        fetch_selected_providers.as_ref()();
    };

    view! {
        <SongView
            default_details=default_details
            songs=filtered_songs
            icons=icons
            selected_songs=selected_songs
            refresh_cb=refresh_songs
            fetch_next_page=fetch_next_page
        />
    }
}

#[tracing::instrument(level = "trace", skip())]
#[component()]
pub fn AllPlaylists() -> impl IntoView {
    let playlists = create_rw_signal(vec![]);

    let refresh_playlist_items: Rc<Box<dyn Fn()>> = Rc::new(Box::new(move || {
        tracing::debug!("Refreshing playlists");
        get_playlists_by_option(QueryablePlaylist::default(), playlists.write_only());
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
    let playlist_context_menu = ContextMenu::new(PlaylistContextMenu {
        refresh_cb: refresh_playlist_items.clone(),
    });

    let playlist_sort = create_read_slice(ui_store, |u| u.get_playlist_sort_by());

    let sorted_playlists = create_memo(move |_| {
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

    let playlist_item_context_menu = Rc::new(ContextMenu::new(PlaylistItemContextMenu {
        playlist: None,
        refresh_cb: refresh_playlist_items,
    }));

    view! {
        <div class="w-100 h-100">
            <div
                on:contextmenu=move |ev| {
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
                                    Rc::new(
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
