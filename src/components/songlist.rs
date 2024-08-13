use leptos::{
    component, create_effect, create_node_ref, create_rw_signal, create_write_slice,
    ev::{keydown, keyup},
    event_target_value,
    html::{Div, Input},
    use_context, view, window_event_listener, HtmlElement, IntoView, ReadSignal, RwSignal, Show,
    SignalGet, SignalSet, SignalUpdate,
};
use leptos_use::on_click_outside;
use leptos_virtual_scroller::VirtualScroller;
use types::songs::Song;

use crate::{
    components::{low_img::LowImg, provider_icon::ProviderIcon},
    icons::{
        add_to_queue_icon::AddToQueueIcon, ellipsis_icon::EllipsisIcon, search_icon::SearchIcon,
        sort_icon::SortIcon,
    },
    store::player_store::PlayerStore,
    utils::common::{format_duration, get_low_img},
};

#[component()]
pub fn SongListItem(
    #[prop()] song: Song,
    #[prop()] is_selected: Box<dyn Fn() -> bool>,
) -> impl IntoView {
    let player_store = use_context::<RwSignal<PlayerStore>>().unwrap();
    let play_now = create_write_slice(player_store, |store, value| store.play_now(value));

    let song_cloned = song.clone();
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
                            <ProviderIcon song=song.clone() />
                        </div>
                    </div>
                    <div class="row no-gutters flex-nowrap">
                        <div class="subtitle text-truncate">
                            <span>
                                {song
                                    .artists
                                    .unwrap_or_default()
                                    .into_iter()
                                    .map(|a| a.artist_name.unwrap_or_default())
                                    .collect::<Vec<String>>()
                                    .join(", ")}
                            </span>
                        </div>
                    </div>
                </div>

                <div class="col-auto offset-1 align-self-center ml-auto timestamp">
                    {format_duration(song.song.duration.unwrap_or(-1f64))}
                </div>

                <div class="col-auto align-self-center button-icon ml-5">
                    <AddToQueueIcon title="test".to_string() />
                </div>

                <div class="col-auto align-self-center ml-5 mr-3 py-2 ellipsis-icon">
                    <EllipsisIcon />
                </div>
            </div>
        </div>
    }
}

#[component()]
pub fn SongList(
    #[prop()] song_list: ReadSignal<Vec<Song>>,
    #[prop()] selected_songs_sig: RwSignal<Vec<usize>>,
    #[prop(default = false)] expand: bool,
    #[prop(default = false)] hide_search_bar: bool,
) -> impl IntoView {
    let is_ctrl_pressed = create_rw_signal(false);
    let is_shift_pressed = create_rw_signal(false);

    let show_searchbar = create_rw_signal(false);
    let searchbar_ref = create_node_ref();

    let filter = create_rw_signal(None::<String>);

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
    });

    window_event_listener(keyup, move |ev| {
        if ev.key() == "Shift" {
            is_shift_pressed.set(false);
        }

        if ev.key() == "Control" {
            is_ctrl_pressed.set(false);
        }
    });

    let add_to_selected = move |index: usize| {
        let is_ctrl_pressed = is_ctrl_pressed.get();
        let is_shift_pressed = is_shift_pressed.get();

        if is_shift_pressed {
            let selected_songs = selected_songs_sig.get();
            let first_selected = selected_songs.first();

            if let Some(first_selected) = first_selected.cloned() {
                let (i, j) = if first_selected < index {
                    (first_selected, index)
                } else {
                    (index, first_selected)
                };
                selected_songs_sig.set((i..=j).collect::<Vec<usize>>());
            }
            return;
        }

        if is_ctrl_pressed {
            selected_songs_sig.update(move |s| {
                s.push(index);
            });
            return;
        }

        selected_songs_sig.set(vec![index]);
    };

    let target = create_node_ref::<Div>();
    on_click_outside(target, move |_| {
        selected_songs_sig.update(|s| s.clear());
    });

    view! {
        <div class=move || {
            if !expand {
                "col-xl-9 col-8 h-100 song-list-compact"
            } else {
                "col h-100 song-list-compact"
            }
        }>
            <div class="d-flex h-100 w-100">
                <div class="container-fluid">

                    <Show
                        when=move || hide_search_bar
                        fallback=move || {
                            view! {
                                <div class="container-fluid tab-carousel">
                                    <div class="row no-gutters">
                                        <div class="col song-header-options w-100">
                                            <div class="row no-gutters align-items-center h-100">
                                                // Sort icons here
                                                <div class="col-auto ml-auto d-flex">

                                                    {move || {
                                                        if show_searchbar.get() {
                                                            view! {
                                                                <div class="searchbar-container mr-3">
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
                                                                        class="searchbar"
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
                                                    </div> <div class="align-self-center">
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
                            node_ref=target
                        >

                            <VirtualScroller
                                each=song_list
                                item_height=95usize
                                inner_el_style="width: calc(100% - 15px);"
                                children=move |(index, song)| {
                                    view! {
                                        <SongListItem
                                            on:click=move |_| add_to_selected(index)
                                            is_selected=Box::new(move || {
                                                selected_songs_sig.get().contains(&index)
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
        </div>
    }
}
