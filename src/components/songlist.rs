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

use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};

use itertools::Itertools;
use leptos::{
    component,
    ev::{keydown, keyup, scroll},
    html::Div,
    prelude::*,
    view, IntoView,
};
use leptos_use::use_event_listener;
use leptos_virtual_scroller::VirtualScroller;
use types::songs::Song;
use web_sys::{HtmlDivElement, HtmlInputElement, MouseEvent};

use crate::{
    components::{artist_list::ArtistList, low_img::LowImg, provider_icon::ProviderIcon},
    icons::{
        add_to_queue_icon::AddToQueueIcon, ellipsis_icon::EllipsisIcon, search_icon::SearchIcon,
        sort_icon::SortIcon,
    },
    pages::search::TabCarousel,
    store::{
        player_store::PlayerStore,
        provider_store::ProviderStore,
        ui_store::{SongSortByColumns, UiStore},
    },
    utils::{
        common::{format_duration, get_low_img},
        context_menu::{create_context_menu, SongItemContextMenu, SortContextMenu},
        db_utils::get_playlists_local,
    },
};

#[tracing::instrument(level = "trace", skip(song, is_selected, on_context_menu, on_click))]
#[component()]
pub fn SongListItem(
    #[prop()] song: Song,
    #[prop()] is_selected: Box<dyn Fn() -> bool + Send + Sync>,
    #[prop()] on_context_menu: impl Fn((MouseEvent, bool)) + 'static + Send + Sync,
    #[prop()] on_click: impl Fn(MouseEvent) + 'static + Send + Sync,
) -> impl IntoView {
    let player_store = use_context::<RwSignal<PlayerStore>>().unwrap();
    let play_now = create_write_slice(player_store, |store, value| store.play_now(value));
    let add_to_queue =
        create_write_slice(player_store, |store, song| store.add_to_queue(vec![song]));
    let song_cloned = song.clone();
    let song_cloned1 = song.clone();

    let on_context_menu = Arc::new(Box::new(on_context_menu));
    let on_context_menu_cl = on_context_menu.clone();
    view! {
        <div
            class="container-fluid wrapper w-100 mb-3"
            class:selectedItem=is_selected
            on:click=on_click
            on:contextmenu=move |ev| on_context_menu.as_ref()((ev, false))
        >
            <div class="row no-gutters align-content-center w-100">
                <LowImg
                    show_eq=|| false
                    eq_playing=|| false
                    cover_img=get_low_img(&song)
                    play_now=move || play_now.set(song_cloned.clone())
                />

                <div class="col-8 col-md-5 align-self-center ml-2">
                    <div class="row no-gutters align-items-center">
                        <div class="col-auto d-flex">
                            <div class="title text-truncate mr-2">
                                {song.song.title.clone().unwrap_or_default()}
                            </div>
                            {move || {
                                let extension = song.song.provider_extension.clone();
                                if let Some(extension) = extension {
                                    view! { <ProviderIcon extension=extension /> }.into_any()
                                } else {
                                    ().into_any()
                                }
                            }}
                        </div>
                    </div>
                    <div class="row no-gutters flex-nowrap">
                        <div class="row no-gutters w-100 flex-nowrap text-truncate">
                            <ArtistList artists=song.artists />
                        </div>
                    </div>
                </div>

                <div class="col-auto offset-1 align-self-center ml-auto timestamp">
                    {format_duration(song.song.duration.unwrap_or(-1f64))}
                </div>

                <div class="col-auto align-self-center button-icon ml-5">
                    <AddToQueueIcon
                        title="Add to queue".into()
                        on:click=move |_| {
                            add_to_queue.set(song_cloned1.clone());
                        }
                    />
                </div>

                <div
                    class="col-auto align-self-center ml-5 mr-3 py-2 ellipsis-icon"
                    on:click=move |ev| {
                        tracing::info!("got click");
                        ev.stop_propagation();
                        ev.prevent_default();
                        on_context_menu_cl.as_ref()((ev, true))
                    }
                >
                    <EllipsisIcon />
                </div>
            </div>
        </div>
    }
}

#[derive(Debug, Default, Copy, Clone)]
pub struct ShowProvidersArgs {
    pub show_providers: bool,
    pub selected_providers: RwSignal<Vec<String>>,
}

#[tracing::instrument(
    level = "trace",
    skip(
        song_list,
        selected_songs_sig,
        filtered_selected,
        hide_search_bar,
        refresh_cb,
        fetch_next_page,
        root_ref,
        scroller_ref,
        header
    )
)]
#[component()]
/// filtered_selected is the list of song indices from the **filtered song list** after they have been filtered (search / sort)
/// selected_songs_sig is the list of song indices from the **original song list**
pub fn SongList<I>(
    #[prop()] song_list: impl Get<Value = Vec<Song>> + Copy + 'static + Send + Sync,
    #[prop()] selected_songs_sig: RwSignal<Vec<usize>>,
    #[prop()] filtered_selected: RwSignal<Vec<usize>>,
    #[prop()] refresh_cb: impl Fn() + 'static + Send + Sync,
    #[prop()] fetch_next_page: impl Fn() + 'static,
    #[prop(default = false)] hide_search_bar: bool,
    #[prop(optional, default = ShowProvidersArgs::default())] providers: ShowProvidersArgs,
    #[prop(optional)] root_ref: Option<NodeRef<Div>>,
    #[prop(optional)] scroller_ref: Option<NodeRef<Div>>,
    #[prop(optional)] header_height: usize,
    #[prop(optional, default = true)] enable_sort: bool,
    #[prop()] header: I,
) -> impl IntoView
where
    I: IntoView,
{
    let is_ctrl_pressed = RwSignal::new(false);
    let is_shift_pressed = RwSignal::new(false);
    let select_all = RwSignal::new(false);

    let show_searchbar = RwSignal::new(false);
    let searchbar_ref = NodeRef::new();

    let provider_store = expect_context::<Arc<ProviderStore>>();

    let ui_store: RwSignal<UiStore> = expect_context();
    let is_mobile = create_read_slice(ui_store, |u| u.get_is_mobile()).get();

    let filter = RwSignal::new(None::<String>);

    let playlists = RwSignal::new(vec![]);
    get_playlists_local(playlists);

    let songs_sort = create_read_slice(ui_store, |u| u.get_song_sort_by());
    let sort_icon_rotated = create_read_slice(ui_store, |u| u.get_song_sort_by().asc);

    let player_store = use_context::<RwSignal<PlayerStore>>().unwrap();
    let play_now = create_write_slice(player_store, |store, value| store.play_now(value));

    let sorted_songs = Memo::new(move |_| {
        let mut songs = song_list.get();
        if enable_sort {
            let sort = songs_sort.get();
            match sort.sort_by {
                SongSortByColumns::Album => songs.sort_by(|a, b| a.album.cmp(&b.album)),
                SongSortByColumns::Artist => songs.sort_by(|a, b| a.artists.cmp(&b.artists)),
                SongSortByColumns::Date => songs.sort_by(|a, b| a.song.date.cmp(&b.song.date)),
                SongSortByColumns::Genre => songs.sort_by(|a, b| a.genre.cmp(&b.genre)),
                SongSortByColumns::PlayCount => {}
                SongSortByColumns::Title => songs.sort_by(|a, b| {
                    let title_a = a.song.title.as_ref().map(|t| t.to_lowercase());
                    let title_b = b.song.title.as_ref().map(|t| t.to_lowercase());
                    title_a.cmp(&title_b)
                }),
            }

            if !sort.asc {
                songs.reverse();
            }
        }

        songs
    });

    let filtered_songs = Memo::new(move |_| {
        let filter = filter.get();
        if filter.is_none() {
            return sorted_songs.get();
        }
        let binding = filter.unwrap();
        let binding = binding.to_lowercase();
        let filter = binding.as_str();

        sorted_songs
            .get()
            .into_iter()
            .filter(|s| {
                if let Some(title) = &s.song.title {
                    title.to_lowercase().contains(filter)
                } else {
                    false
                }
            })
            .collect()
    });

    Effect::new(move || {
        let _ = filtered_songs.get();
        filtered_selected.update(|s| s.clear());
        selected_songs_sig.update(|s| s.clear());
    });

    Effect::new(move || {
        let show_searchbar = show_searchbar.get();
        if show_searchbar {
            if let Some(searchbar) = searchbar_ref.get() {
                (searchbar as HtmlInputElement)
                    .focus()
                    .expect("Could not focus on searchbar");
            }
        }
    });

    window_event_listener(keydown, move |ev| {
        if ev.shift_key() {
            is_shift_pressed.set(true);
        }

        if ev.ctrl_key() {
            is_ctrl_pressed.set(true);
        }

        if ev.code() == "KeyA" && is_ctrl_pressed.get_untracked() {
            select_all.set(true);
        }
    });

    window_event_listener(keyup, move |ev| {
        if ev.key() == "Shift" {
            is_shift_pressed.set(false);
        }

        if ev.key() == "Control" {
            is_ctrl_pressed.set(false);
        }
    });

    let get_actual_position = move |filtered_index: usize| {
        let filtered_songs = filtered_songs.get();
        tracing::debug!("Filtered index {}", filtered_index);
        let filtered_song = filtered_songs.get(filtered_index).unwrap();
        song_list
            .get()
            .iter()
            .position(|s| s == filtered_song)
            .unwrap()
    };

    let add_to_selected = move |index: usize| {
        let is_ctrl_pressed_val = is_ctrl_pressed.get();
        let is_shift_pressed = is_shift_pressed.get();

        if is_shift_pressed {
            let selected_songs = filtered_selected.get();
            let first_selected = selected_songs.first();

            if let Some(&first_selected) = first_selected {
                let (i, j) = if first_selected < index {
                    (first_selected, index)
                } else {
                    (index, first_selected)
                };

                tracing::debug!("First selected {}, index {}", i, j);

                let mut ret = vec![];
                for k in i..=j {
                    ret.push(get_actual_position(k));
                }

                filtered_selected.set((i..=j).collect());
                selected_songs_sig.set(ret);
            }
            return;
        }

        if is_ctrl_pressed_val {
            let is_removing = AtomicBool::new(false);
            selected_songs_sig.update(|s| {
                let actual_pos = get_actual_position(index);
                if let Some((i, _)) = s.iter().find_position(|pos| **pos == actual_pos) {
                    s.remove(i);
                    is_removing.store(true, Ordering::Relaxed);
                } else {
                    s.push(actual_pos);
                }
            });
            filtered_selected.update(|s| {
                if is_removing.load(Ordering::Relaxed) {
                    if let Some((i, _)) = s.iter().find_position(|i| **i == index) {
                        s.remove(i);
                    }
                } else {
                    s.push(index);
                }
            });

            if is_ctrl_pressed_val && is_mobile && selected_songs_sig.get_untracked().is_empty() {
                is_ctrl_pressed.set(false);
            }

            return;
        }

        selected_songs_sig.set(vec![get_actual_position(index)]);
        filtered_selected.set(vec![index]);
    };

    Effect::new(move || {
        let select_all_val = select_all.get();
        if select_all_val {
            select_all.set(false);

            let song_space = (0..song_list.get().len()).collect_vec();

            selected_songs_sig.set(song_space.clone());
            filtered_selected.set(song_space);
        }
    });

    let context_menu_data = SongItemContextMenu {
        current_song: None,
        song_list,
        selected_songs: selected_songs_sig,
        playlists,
        refresh_cb: Arc::new(Box::new(refresh_cb)),
    };
    let song_context_menu = create_context_menu(context_menu_data);

    let sort_context_menu = create_context_menu(SortContextMenu {});

    let should_add_to_selected = move |index: usize| {
        let selected = filtered_selected.get_untracked();
        selected.len() < 2 || !selected.contains(&index)
    };

    let root_ref = if root_ref.is_none() {
        NodeRef::new()
    } else {
        root_ref.unwrap()
    };

    let scroller_ref = if scroller_ref.is_none() {
        NodeRef::new()
    } else {
        scroller_ref.unwrap()
    };

    let _ = use_event_listener(scroller_ref, scroll, move |ev| {
        let target = event_target::<HtmlDivElement>(&ev);
        let scroll_top = target.scroll_top() + target.offset_height();
        let height = target.scroll_height();
        if scroll_top >= height - 10 {
            fetch_next_page();
        }
    });

    view! {
        <div class="d-flex h-100 w-100" node_ref=root_ref>
            <div class="container-fluid">

                <Show
                    when=move || hide_search_bar
                    fallback=move || {
                        let sort_context_menu = sort_context_menu.clone();
                        view! {
                            <div
                                class="container-fluid tab-carousel"
                                class:tab-carousel-show-mobile=is_mobile && providers.show_providers
                            >
                                <div class="row no-gutters">
                                    <div class="col song-header-options w-100">

                                        <div class="row no-gutters align-items-center h-100 d-flex justify-content-end">
                                            // Sort icons here
                                            {if providers.show_providers {
                                                view! {
                                                    <div class="col-auto d-flex">
                                                        <TabCarousel
                                                            keys=provider_store.get_provider_keys()
                                                            selected=providers.selected_providers
                                                            single_select=false
                                                            align_left=false
                                                        />
                                                    </div>
                                                }
                                                    .into_any()
                                            } else {
                                                ().into_any()
                                            }}
                                            <div class="col-auto d-flex">

                                                {move || {
                                                    if show_searchbar.get() {
                                                        view! {
                                                            <div class="songlist-searchbar-container mr-3">
                                                                <input
                                                                    node_ref=searchbar_ref
                                                                    on:input=move |ev| {
                                                                        let text = event_target_value(&ev);
                                                                        if text.is_empty() {
                                                                            filter.set(None);
                                                                        } else {
                                                                            filter.set(Some(text));
                                                                        }
                                                                    }

                                                                    type="text"
                                                                    class="songlist-searchbar"
                                                                    placeholder="search"
                                                                />
                                                            </div>
                                                        }
                                                            .into_any()
                                                    } else {
                                                        ().into_any()
                                                    }
                                                }}
                                                <div
                                                    class="mr-3 align-self-center"
                                                    on:click=move |_| show_searchbar.set(!show_searchbar.get())
                                                >
                                                    <SearchIcon accent=false />
                                                </div>
                                                <div
                                                    class="align-self-center"
                                                    on:click=move |e| { sort_context_menu.show(e) }
                                                >
                                                    <SortIcon rotate=sort_icon_rotated />
                                                </div>
                                            </div>
                                        </div>
                                    </div>
                                </div>
                            </div>
                        }
                    }
                >
                    <div></div>
                </Show>

                <div class="row no-gutters h-100">
                    <div
                        class="scroller w-100 full-height"
                        style=move || {
                            if !is_mobile {
                                "height: calc(100% - 53px) !important;"
                            } else {
                                "height: 100% !important"
                            }
                        }
                    >

                        <VirtualScroller
                            node_ref=scroller_ref
                            each=filtered_songs
                            key=|s| s.song._id.clone()
                            item_height=95usize
                            inner_el_style="width: calc(100% - 15px);"
                            header=header
                            header_height=header_height
                            children=move |(index, song)| {
                                let song_cl = song.clone();
                                let song_cl1 = song.clone();
                                let song_context_menu = song_context_menu.clone();
                                view! {
                                    <SongListItem
                                        on_click=move |_| {
                                            if is_mobile && !is_ctrl_pressed.get_untracked() {
                                                play_now.set(song_cl1.clone());
                                            } else {
                                                is_ctrl_pressed.set(true);
                                                add_to_selected(index);
                                            }
                                        }
                                        is_selected=Box::new(move || {
                                            filtered_selected.get().contains(&index)
                                        })
                                        on_context_menu=move |(ev, is_button): (MouseEvent, bool)| {
                                            ev.prevent_default();
                                            ev.stop_propagation();
                                            if is_mobile && !is_button {
                                                is_ctrl_pressed.set(true);
                                                add_to_selected(index);
                                            } else {
                                                if should_add_to_selected(index) {
                                                    add_to_selected(index);
                                                }
                                                let mut data = song_context_menu.get_data();
                                                data.current_song = Some(song_cl.clone());
                                                drop(data);
                                                song_context_menu.show(ev);
                                            }
                                        }

                                        song=song.clone()
                                    />
                                }
                            }
                        />

                    </div>
                </div>
            </div>
        </div>
    }
}
