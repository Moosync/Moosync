use std::rc::Rc;

use leptos::{
    component, create_action, create_effect, create_rw_signal, create_write_slice, ev::Event, event_target_value, expect_context, spawn_local, view, CollectView, IntoView, RwSignal, SignalGet, SignalSet, SignalUpdate
};
use leptos_virtual_scroller::VirtualScroller;
use types::{
    entities::{QueryableAlbum, QueryableArtist, QueryableGenre, QueryablePlaylist},
    songs::{GetSongOptions, SearchableSong, Song},
};

use crate::{
    components::low_img::LowImg,
    console_log,
    icons::{
        next_icon::NextIcon, person_icon::PersonIcon, prev_icon::PrevIcon, search_icon::SearchIcon,
    },
    store::{modal_store::{ModalStore, Modals}, player_store::PlayerStore, provider_store::ProviderStore},
    utils::{common::get_low_img, db_utils::get_songs_by_option},
};

enum InputFocus {
    Focus,
    Blur,
}

#[component]
pub fn SearchResultItem(song: Song) -> impl IntoView {
    let player_store = expect_context::<RwSignal<PlayerStore>>();
    let play_now = create_write_slice(player_store, |p, val| p.play_now(val));

    let song_cloned = song.clone();
    view! {
        <div class="container-fluid single-result-container single-result">
            <div class="row justify-content-around">
                <LowImg
                    cover_img=get_low_img(&song)
                    play_now=move || play_now.set(song_cloned.clone())
                />

                <div class="col text-container text-truncate my-auto">
                    <div class="song-title text-truncate">{song.song.title.clone()}</div>
                    <div class="song-subtitle text-truncate">
                        {song
                            .artists
                            .clone()
                            .unwrap()
                            .iter()
                            .filter_map(|a| a.artist_name.clone())
                            .collect::<Vec<String>>()
                            .join(", ")}
                    </div>
                </div>
            </div>
        </div>
    }
}

#[component]
pub fn Accounts() -> impl IntoView {
    let show_accounts_popover = create_rw_signal(false);
    let provider_store = expect_context::<Rc<ProviderStore>>();

    let statuses = provider_store.get_all_statuses();

    let modal_store = expect_context::<RwSignal<ModalStore>>();
    let show_login_modal = move |key: String| {
        modal_store.update(|m| {
            m.set_active_modal(Modals::LoginModal(key))
        })
    };
    
    view! {
        <div>
            <PersonIcon on:click=move |_| show_accounts_popover.set(!show_accounts_popover.get())/>

            {move || {
                if show_accounts_popover.get() {
                    view! {
                        <div class="accounts-popover custom-popover">
                            <div class="buttons">
                                {move || {
                                    let statuses = statuses.get();
                                    console_log!("Rerendering accounts");
                                    let mut ret = vec![];
                                    for s in statuses {
                                        let status = s.get();
                                        ret.push(
                                            view! {
                                                <div on:click=move |_| show_login_modal(status.key.clone()) >
                                                    {status.name.clone()} -  {status.user_name.clone()}
                                                </div>
                                            },
                                        );
                                    }
                                    ret.collect_view()
                                }}

                            </div>
                        </div>
                    }
                        .into_view()
                } else {
                    view! {}.into_view()
                }
            }}

        </div>
    }
}

#[component]
pub fn TopBar() -> impl IntoView {
    let show_searchbar = create_rw_signal(false);
    let results = create_rw_signal(vec![]);
    let handle_input_focus = move |focus: InputFocus| match focus {
        InputFocus::Focus => {
            show_searchbar.set(!results.get().is_empty());
        }
        InputFocus::Blur => show_searchbar.set(false),
    };

    create_effect(move |_| {
        let results = results.get();
        if !results.is_empty() {
            show_searchbar.set(true);
        }
    });

    let handle_text_change = move |ev: Event| {
        let text = event_target_value(&ev);
        if text.is_empty() {
            return;
        }
        let value = format!("%{}%", text);
        get_songs_by_option(
            GetSongOptions {
                song: Some(SearchableSong {
                    title: Some(value.clone()),
                    path: Some(value.clone()),
                    ..Default::default()
                }),
                album: Some(QueryableAlbum {
                    album_name: Some(value.clone()),
                    ..Default::default()
                }),
                artist: Some(QueryableArtist {
                    artist_name: Some(value.clone()),
                    ..Default::default()
                }),
                genre: Some(QueryableGenre {
                    genre_name: Some(value.clone()),
                    ..Default::default()
                }),
                playlist: Some(QueryablePlaylist {
                    playlist_name: value,
                    ..Default::default()
                }),
                ..Default::default()
            },
            results,
        )
    };

    view! {
        <div class="topbar-container align-items-center topbar is-open">
            <div class="container-fluid d-flex">
                <div class="row justify-content-start flex-grow-1">
                    <div class="col-auto my-auto">
                        // Prev next buttons
                        <div class="row justify-content-between">
                            <div class="col-6">
                                <PrevIcon/>
                            </div>
                            <div class="col-6">
                                <NextIcon/>
                            </div>
                        </div>
                    </div>
                    // searchbar
                    <div class="col">
                        <div class="h-100 d-flex align-items-center search-container">
                            <div
                                class="w-100 searchbar-container"
                                class:full-border=move || !show_searchbar.get()
                                class:half-border=move || show_searchbar.get()
                            >
                                <div class="search-icon">
                                    <SearchIcon accent=true/>
                                </div>
                                <input
                                    class="form-control searchbar"
                                    placeholder="Search..."

                                    on:blur=move |_| handle_input_focus(InputFocus::Blur)
                                    on:focus=move |_| handle_input_focus(InputFocus::Focus)
                                    on:input=handle_text_change
                                />
                            </div>

                            <div
                                class="search-results d-flex"
                                class:search-invisible=move || !show_searchbar.get()
                            >
                                <div class="w-100">
                                    <VirtualScroller
                                        each=results
                                        item_height=95usize
                                        children=move |(_, song)| {
                                            view! { <SearchResultItem song=song.clone()/> }
                                        }
                                    />

                                </div>
                            </div>
                        </div>
                    </div>

                    // Extra buttons
                    <div class="col-auto pr-5 ml-auto my-auto icons-bar d-flex">
                        <div class="row flex-grow-1">
                            <div class="col-auto">
                                <Accounts/>
                            </div>
                        </div>
                    </div>
                </div>
            </div>
        </div>
    }
}
