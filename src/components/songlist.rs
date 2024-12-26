use std::rc::Rc;

use itertools::Itertools;
use leptos::{
    component, create_effect, create_memo, create_node_ref, create_read_slice, create_rw_signal,
    create_write_slice,
    ev::{keydown, keyup, scroll},
    event_target, event_target_value, expect_context,
    html::{Div, Input},
    leptos_dom, use_context, view, window_event_listener, HtmlElement, IntoView, RwSignal, Show,
    SignalGet, SignalGetUntracked, SignalSet, SignalSetUntracked, SignalUpdate,
};
use leptos_context_menu::ContextMenu;
use leptos_use::use_event_listener;
use leptos_virtual_scroller::VirtualScroller;
use types::songs::Song;
use web_sys::HtmlDivElement;

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
        context_menu::{SongItemContextMenu, SortContextMenu},
        db_utils::get_playlists_local,
    },
};

#[tracing::instrument(level = "trace", skip(song, is_selected))]
#[component()]
pub fn SongListItem(
    #[prop()] song: Song,
    #[prop()] is_selected: Box<dyn Fn() -> bool>,
) -> impl IntoView {
    let player_store = use_context::<RwSignal<PlayerStore>>().unwrap();
    let play_now = create_write_slice(player_store, |store, value| store.play_now(value));
    let add_to_queue =
        create_write_slice(player_store, |store, song| store.add_to_queue(vec![song]));
    let song_cloned = song.clone();
    let song_cloned1 = song.clone();
    view! {
        <div class="container-fluid wrapper w-100 mb-3" class:selectedItem=is_selected>
            <div class="row no-gutters align-content-center w-100">
                <LowImg
                    show_eq=|| false
                    eq_playing=|| false
                    cover_img=get_low_img(&song)
                    play_now=move || play_now.set(song_cloned.clone())
                />

                <div class="col-5 align-self-center ml-2">
                    <div class="row no-gutters align-items-center">
                        <div class="col-auto d-flex">
                            <div class="title text-truncate mr-2">
                                {song.song.title.clone().unwrap_or_default()}
                            </div>
                            {move || {
                                let extension = song.song.provider_extension.clone();
                                if let Some(extension) = extension {
                                    view! { <ProviderIcon extension=extension /> }
                                } else {
                                    view! {}.into_view()
                                }
                            }}
                        </div>
                    </div>
                    <div class="row no-gutters flex-nowrap">
                        <div class="row no-gutters">
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

                <div class="col-auto align-self-center ml-5 mr-3 py-2 ellipsis-icon">
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
        fetch_next_page
    )
)]
#[component()]
/// filtered_selected is the list of song indices from the **filtered song list** after they have been filtered (search / sort)
/// selected_songs_sig is the list of song indices from the **original song list**
pub fn SongList(
    #[prop()] song_list: impl SignalGet<Value = Vec<Song>> + Copy + 'static,
    #[prop()] selected_songs_sig: RwSignal<Vec<usize>>,
    #[prop()] filtered_selected: RwSignal<Vec<usize>>,
    #[prop()] refresh_cb: impl Fn() + 'static,
    #[prop()] fetch_next_page: impl Fn() + 'static,
    #[prop(default = false)] hide_search_bar: bool,
    #[prop(optional, default = ShowProvidersArgs::default())] providers: ShowProvidersArgs,
) -> impl IntoView {
    let is_ctrl_pressed = create_rw_signal(false);
    let is_shift_pressed = create_rw_signal(false);
    let select_all = create_rw_signal(false);

    let show_searchbar = create_rw_signal(false);
    let searchbar_ref = create_node_ref();

    let provider_store = expect_context::<Rc<ProviderStore>>();

    let filter = create_rw_signal(None::<String>);

    let playlists = create_rw_signal(vec![]);
    get_playlists_local(playlists);

    let ui_store: RwSignal<UiStore> = expect_context();
    let songs_sort = create_read_slice(ui_store, |u| u.get_song_sort_by());

    let sorted_songs = create_memo(move |_| {
        let sort = songs_sort.get();
        let mut songs = song_list.get();
        match sort.sort_by {
            SongSortByColumns::Album => songs.sort_by(|a, b| a.album.cmp(&b.album)),
            SongSortByColumns::Artist => songs.sort_by(|a, b| a.artists.cmp(&b.artists)),
            SongSortByColumns::Date => songs.sort_by(|a, b| a.song.date.cmp(&b.song.date)),
            SongSortByColumns::Genre => songs.sort_by(|a, b| a.genre.cmp(&b.genre)),
            SongSortByColumns::PlayCount => {}
            SongSortByColumns::Title => songs.sort_by(|a, b| a.song.title.cmp(&b.song.title)),
        }

        songs
    });

    let filtered_songs = create_memo(move |_| {
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

    create_effect(move |_| {
        let _ = filtered_songs.get();
        filtered_selected.update(|s| s.clear());
        selected_songs_sig.update(|s| s.clear());
    });

    create_effect(move |_| {
        let show_searchbar = show_searchbar.get();
        if show_searchbar {
            if let Some(searchbar) = searchbar_ref.get() {
                (searchbar as HtmlElement<Input>)
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
        let is_ctrl_pressed = is_ctrl_pressed.get();
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

        if is_ctrl_pressed {
            selected_songs_sig.update(move |s| {
                s.push(get_actual_position(index));
            });
            filtered_selected.update(|s| {
                s.push(index);
            });
            return;
        }

        selected_songs_sig.set(vec![get_actual_position(index)]);
        filtered_selected.set(vec![index]);
    };

    create_effect(move |_| {
        let select_all_val = select_all.get();
        if select_all_val {
            select_all.set_untracked(false);

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
        refresh_cb: Rc::new(Box::new(refresh_cb)),
    };
    let song_context_menu = Rc::new(ContextMenu::new(context_menu_data));

    let sort_context_menu = Rc::new(ContextMenu::new(SortContextMenu {}));

    let should_add_to_selected = move |index: usize| {
        let selected = filtered_selected.get_untracked();
        selected.len() < 2 || !selected.contains(&index)
    };

    let scroller_ref = create_node_ref();

    let _ = use_event_listener(scroller_ref, scroll, move |ev| {
        let target = event_target::<HtmlDivElement>(&ev);
        let scroll_top = target.scroll_top() + target.offset_height();
        let height = target.scroll_height();
        if scroll_top >= height - 10 {
            fetch_next_page();
        }
    });

    view! {
        <div class="d-flex h-100 w-100">
            <div class="container-fluid">

                <Show
                    when=move || hide_search_bar
                    fallback=move || {
                        let sort_context_menu = sort_context_menu.clone();
                        view! {
                            <div class="container-fluid tab-carousel">
                                <div class="row no-gutters">
                                    <div class="col song-header-options w-100">
                                        <div class="row no-gutters align-items-center h-100 d-flex justify-content-end">
                                            // Sort icons here
                                            {if providers.show_providers {
                                                view! {
                                                    <div class="col-9 d-flex">
                                                        <TabCarousel
                                                            keys=provider_store.get_provider_keys()
                                                            selected=providers.selected_providers
                                                            single_select=false
                                                            align_left=false
                                                        />
                                                    </div>
                                                }
                                                    .into_view()
                                            } else {
                                                view! {}.into_view()
                                            }}
                                            <div class="col-auto d-flex">

                                                {move || {
                                                    if show_searchbar.get() {
                                                        view! {
                                                            <div class="songlist-searchbar-container mr-3">
                                                                <input
                                                                    ref=searchbar_ref
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
                                                            .into_view()
                                                    } else {
                                                        view! {}.into_view()
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
                                                    <SortIcon />
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
                        style="height: calc(100% - 53px) !important;"
                    >

                        <VirtualScroller
                            node_ref=scroller_ref
                            each=filtered_songs
                            item_height=95usize
                            inner_el_style="width: calc(100% - 15px);"
                            children=move |(index, song)| {
                                let song = song.clone();
                                let song_context_menu = song_context_menu.clone();
                                view! {
                                    <SongListItem
                                        on:contextmenu=move |ev| {
                                            ev.stop_propagation();
                                            if should_add_to_selected(index) {
                                                add_to_selected(index);
                                            }
                                            let mut data = song_context_menu.get_data();
                                            data.current_song = Some(song.clone());
                                            drop(data);
                                            song_context_menu.show(ev);
                                        }
                                        on:click=move |_| add_to_selected(index)
                                        is_selected=Box::new(move || {
                                            filtered_selected.get().contains(&index)
                                        })

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
