use leptos_context_menu::{ContextMenu, ContextMenuData, ContextMenuItemInner};
use rand::seq::SliceRandom;
use std::rc::Rc;

use crate::components::cardview::{CardView, SimplifiedCardItem};
use crate::components::songview::SongView;
use crate::console_log;
use crate::store::modal_store::{ModalStore, Modals};
use crate::store::player_store::PlayerStore;
use crate::store::ui_store::{PlaylistSortByColumns, UiStore};
use crate::utils::common::fetch_infinite;
use crate::utils::db_utils::{
    create_playlist, export_playlist, get_songs_by_option, remove_playlist,
};
use crate::utils::entities::get_playlist_sort_cx_items;
use crate::utils::songs::get_songs_from_indices;
use leptos::{
    component, create_memo, create_read_slice, create_rw_signal, create_write_slice,
    expect_context, spawn_local, use_context, view, IntoView, RwSignal, SignalGet, SignalUpdate,
    SignalWith,
};
use leptos_router::use_query_map;
use types::entities::QueryablePlaylist;
use types::songs::GetSongOptions;
use types::ui::song_details::SongDetailIcons;

use crate::store::provider_store::ProviderStore;
use crate::{icons::plus_button::PlusIcon, utils::db_utils::get_playlists_by_option};

struct PlaylistContextMenu {}

impl PlaylistContextMenu {
    fn open_import_from_url_modal(&self) {
        let modal_store: RwSignal<ModalStore> = expect_context();
        modal_store.update(|modal_store| {
            modal_store.set_active_modal(Modals::NewPlaylistModal);
        });
    }
}

impl ContextMenuData<Self> for PlaylistContextMenu {
    fn get_menu_items(&self) -> leptos_context_menu::ContextMenuItems<Self> {
        vec![
            ContextMenuItemInner::new_with_handler(
                "Import from Url".into(),
                |_, cx| cx.open_import_from_url_modal(),
                None,
            ),
            ContextMenuItemInner::new("Sort by".into(), Some(get_playlist_sort_cx_items())),
        ]
    }
}

struct PlaylistItemContextMenu {
    playlist: Option<QueryablePlaylist>,
}

impl PlaylistItemContextMenu {
    fn add_to_library(&self) {
        if let Some(playlist) = &self.playlist {
            create_playlist(playlist.clone());
        }
    }

    fn remove_from_library(&self) {
        if let Some(playlist) = &self.playlist {
            remove_playlist(playlist.clone());
        }
    }

    fn export_playlist(&self) {
        if let Some(playlist) = &self.playlist {
            export_playlist(playlist.clone());
        }
    }
}

impl ContextMenuData<Self> for PlaylistItemContextMenu {
    fn get_menu_items(&self) -> leptos_context_menu::ContextMenuItems<Self> {
        if let Some(playlist) = &self.playlist {
            if let Some(library_item) = playlist.library_item {
                if library_item {
                    return vec![
                        ContextMenuItemInner::new_with_handler(
                            "Remove from library".into(),
                            |_, cx| cx.remove_from_library(),
                            None,
                        ),
                        ContextMenuItemInner::new_with_handler(
                            "Export playlist".into(),
                            |_, cx| cx.export_playlist(),
                            None,
                        ),
                    ];
                }
            }

            return vec![ContextMenuItemInner::new_with_handler(
                "Add to library".into(),
                |_, cx| cx.add_to_library(),
                None,
            )];
        }
        vec![]
    }
}

#[component()]
pub fn SinglePlaylist() -> impl IntoView {
    let params = use_query_map();
    let playlist_id = params.with(|params| params.get("id").cloned()).unwrap();
    console_log!("In single playlists {:?}", playlist_id);

    let songs = create_rw_signal(vec![]);
    let selected_songs = create_rw_signal(vec![]);

    let provider_store = use_context::<Rc<ProviderStore>>().unwrap();

    let playlist_id_tmp = playlist_id.clone();
    spawn_local(async move {
        let provider = provider_store
            .get_provider_key_by_id(playlist_id_tmp.clone())
            .await;
        match provider {
            Ok(provider) => {
                let playlist_id = playlist_id_tmp.clone();
                fetch_infinite!(
                    provider_store,
                    provider,
                    fetch_playlist_content,
                    songs,
                    playlist_id.clone()
                );
            }
            Err(e) => console_log!("{}", e),
        }
    });

    get_songs_by_option(
        GetSongOptions {
            playlist: Some(QueryablePlaylist {
                playlist_id: Some(playlist_id),
                ..Default::default()
            }),
            ..Default::default()
        },
        songs,
    );

    let player_store = expect_context::<RwSignal<PlayerStore>>();
    let play_songs_setter = create_write_slice(player_store, |p, song| p.play_now(song));
    let play_songs_multiple_setter =
        create_write_slice(player_store, |p, songs| p.play_now_multiple(songs));
    let add_to_queue_setter = create_write_slice(player_store, |p, songs| p.add_to_queue(songs));

    let play_songs = move || {
        let selected_songs = if selected_songs.get().is_empty() {
            songs.get()
        } else {
            get_songs_from_indices(songs, selected_songs)
        };

        play_songs_multiple_setter.set(selected_songs);
    };

    let add_to_queue = move || {
        if selected_songs.get().is_empty() {
            add_to_queue_setter.set(songs.get());
        } else {
            add_to_queue_setter.set(get_songs_from_indices(songs, selected_songs));
        }
    };

    let random = move || {
        let songs = songs.get();
        let random_song = songs.choose(&mut rand::thread_rng()).unwrap();
        play_songs_setter.set(random_song.clone());
    };

    let icons = create_rw_signal(SongDetailIcons {
        play: Some(Rc::new(Box::new(play_songs))),
        add_to_queue: Some(Rc::new(Box::new(add_to_queue))),
        random: Some(Rc::new(Box::new(random))),
        ..Default::default()
    });

    view! { <SongView songs=songs icons=icons selected_songs=selected_songs /> }
}

#[component()]
pub fn AllPlaylists() -> impl IntoView {
    let playlists = create_rw_signal(vec![]);
    get_playlists_by_option(QueryablePlaylist::default(), playlists.write_only());

    let modal_manager = expect_context::<RwSignal<ModalStore>>();
    let open_new_playlist_modal = move |_| {
        modal_manager.update(|m| {
            m.set_active_modal(Modals::NewPlaylistModal);
            m.on_modal_close(move || {
                get_playlists_by_option(QueryablePlaylist::default(), playlists.write_only());
            });
        });
    };

    let ui_store = expect_context::<RwSignal<UiStore>>();
    let playlist_context_menu = ContextMenu::new(PlaylistContextMenu {});

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

    let playlist_item_context_menu =
        Rc::new(ContextMenu::new(PlaylistItemContextMenu { playlist: None }));

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
                        card_item=move |(_, item)| {
                            let playlist_name = item.playlist_name.clone();
                            let playlist_coverpath = item.playlist_coverpath.clone();
                            let playlist_id = item.playlist_id.clone().unwrap_or_default();
                            let playlist_extension = item.extension.clone();
                            let playlist_item_context_menu = playlist_item_context_menu.clone();
                            SimplifiedCardItem {
                                title: playlist_name,
                                cover: playlist_coverpath,
                                id: playlist_id,
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
