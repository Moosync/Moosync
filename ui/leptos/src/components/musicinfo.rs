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

use leptos::ev::{mousedown, touchstart};
use leptos::html::Div;
use leptos::task::spawn_local;
use leptos::{IntoView, component, prelude::*, view};
use leptos_use::use_event_listener;
use leptos_virtual_scroller::VirtualScroller;
use std::sync::Arc;
use types::songs::{Song, SongType};
use types::ui::extensions::ExtensionProviderScope;
use types::ui::player_details::PlayerState;
use types::ui::song_details::SongDetailIcons;
use web_sys::{ScrollBehavior, ScrollToOptions};

use crate::components::artist_list::ArtistList;
use crate::components::audiostream::AudioStream;
use crate::components::musicbar_components::{Controls, Slider};
use crate::icons::lyrics_icon::LyricsIcon;
use crate::icons::song_default_icon::SongDefaultIcon;
use crate::modals::new_playlist_modal::PlaylistModalState;
use crate::store::modal_store::{ModalStore, Modals};
use crate::store::provider_store::ProviderStore;
use crate::store::ui_store::UiStore;
use crate::utils::common::{format_duration, get_high_img};
use crate::utils::entities::get_artist_string;
use crate::utils::invoke::get_provider_lyrics;
use crate::utils::songs::fetch_lyrics;
use crate::{
    components::{low_img::LowImg, provider_icon::ProviderIcon, songdetails::SongDetails},
    icons::{
        cross_icon::CrossIcon, prev_icon::PrevIcon, queue_icon::QueueIcon, trash_icon::TrashIcon,
    },
    store::player_store::PlayerStore,
    utils::common::get_low_img,
};

#[tracing::instrument(
    level = "trace",
    skip(
        song,
        index,
        current_song_index,
        eq_playing,
        play_now,
        remove_from_queue
    )
)]
#[component]
pub fn QueueItem<T, D, P>(
    #[prop()] song: Song,
    index: usize,
    current_song_index: T,
    eq_playing: D,
    play_now: P,
    remove_from_queue: P,
    is_mobile: bool,
) -> impl IntoView
where
    T: Get<Value = usize> + Copy + 'static + Send + Sync,
    D: Get<Value = bool> + 'static + Send + Sync,
    P: Set<Value = usize> + 'static + Send + Sync,
{
    view! {
        <div class="container-fluid item-container" class:pl-2=is_mobile class:pr-0=is_mobile>
            <div class="row item-row no-gutters">
                <LowImg
                    cover_img=get_low_img(&song)
                    play_now=move || play_now.set(index)
                    show_eq=move || index == current_song_index.get()
                    eq_playing=move || eq_playing.get()
                />
                <div class="col-lg-7 col-xl-8" class:col-5=!is_mobile class:col-7=is_mobile>
                    <div class="d-flex">
                        <div class="text-left song-title text-truncate">
                            {song.song.title.clone()}
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
                    <div class="row no-gutters w-100 flex-nowrap text-truncate">
                        <ArtistList artists=song.artists />
                    </div>

                </div>
                <div class="col-auto text-right ml-auto d-flex align-items-center">
                    <div class="ml-auto remove-button">
                        <TrashIcon on:click=move |_| remove_from_queue.set(index) />
                    </div>
                </div>
            </div>
        </div>
    }
}

#[derive(Debug, Clone, Copy)]
enum MusicInfoState {
    MusicInfo,
    Queue,
    Lyrics,
}

#[tracing::instrument(level = "debug", skip(show, node_ref))]
#[component]
pub fn MusicInfoMobile(
    #[prop()] show: Signal<bool>,
    #[prop()] node_ref: NodeRef<Div>,
) -> impl IntoView {
    let player_store = expect_context::<RwSignal<PlayerStore>>();
    let current_song = create_read_slice(player_store, move |p| p.get_current_song());
    let current_song_index = create_read_slice(player_store, |p| p.get_queue_index());

    let queue_songs = create_read_slice(player_store, move |p| p.get_queue_songs());
    let is_playing = create_read_slice(player_store, |p| {
        p.get_player_state() == PlayerState::Playing
    });

    let play_now = create_write_slice(player_store, |p, val| p.change_index(val, true));
    let remove_from_queue = create_write_slice(player_store, |p, val| p.remove_from_queue(val));

    let cover_img = Memo::new(move |_| {
        if let Some(current_song) = current_song.get() {
            return Some(get_high_img(&current_song));
        }
        None
    });
    let show_default_cover_img = RwSignal::new(cover_img.get_untracked().is_none());

    Effect::new(move || {
        show_default_cover_img.set(cover_img.get().is_none());
    });

    let song_title = Memo::new(move |_| {
        if let Some(current_song) = current_song.get() {
            return current_song.song.title;
        }
        None
    });

    let song_subtitle = Memo::new(move |_| {
        if let Some(current_song) = current_song.get() {
            return Some(get_artist_string(current_song.artists));
        }
        None
    });

    let current_time = create_read_slice(player_store, |p| {
        format_duration(p.get_current_time(), false)
    });
    let total_time = Memo::new(move |_| {
        if let Some(current_song) = current_song.get()
            && let Some(duration) = current_song.song.duration
        {
            return format_duration(duration, false);
        }
        "00:00".to_string()
    });

    let canvaz_sig = RwSignal::new(None);
    Effect::new(move || {
        let current_song = current_song.get();
        canvaz_sig.set(None);
        if let Some(current_song) = current_song
            && current_song.song.type_ == SongType::SPOTIFY
            && current_song.song.playback_url.is_some()
        {
            spawn_local(async move {
                let res = crate::utils::invoke::get_canvaz(
                    current_song.song.playback_url.unwrap().clone(),
                    false,
                )
                .await;
                if let Ok(res) = res {
                    canvaz_sig.set(res.canvases.first().map(|c| c.url.clone()));
                } else {
                    tracing::error!("Failed to get canvaz {:?}", res);
                }
            });
        }
    });

    let scroller_ref: NodeRef<Div> = NodeRef::new();
    Effect::new(move || {
        let current_song_index = current_song_index.get();
        if let Some(el) = scroller_ref.get() {
            let el_top = 95usize * current_song_index;
            let options = ScrollToOptions::new();
            options.set_behavior(ScrollBehavior::Smooth);
            options.set_top(el_top as f64);
            el.scroll_with_scroll_to_options(&options);
        }
    });

    let selected_lyrics = RwSignal::new(None::<String>);
    let provider_store = expect_context::<Arc<ProviderStore>>();
    Effect::new(move || {
        let song = current_song.get();
        let provider_store = provider_store.clone();
        spawn_local(async move {
            let lyrics = fetch_lyrics(&song).await;
            if lyrics.is_none()
                && let Some(song) = song
            {
                let valid_providers =
                    provider_store.get_provider_keys(ExtensionProviderScope::Lyrics);
                for provider in valid_providers {
                    let song = song.clone();
                    let res = get_provider_lyrics(provider, song, false).await;
                    if let Ok(res) = res {
                        selected_lyrics.set(Some(res));
                        return;
                    }
                }
            }
            selected_lyrics.set(lyrics);
        });
    });

    let show_queue = RwSignal::new(MusicInfoState::MusicInfo);

    let _ = use_event_listener(scroller_ref, touchstart, |ev| {
        ev.stop_propagation();
    });
    let _ = use_event_listener(scroller_ref, mousedown, |ev| {
        ev.stop_propagation();
    });

    view! {
        <div
            class="slider"
            class:slider-mobile=true
            class:musicinfo-open=move || show.get()
            class:musicinfo-close=move || !show.get()
            node_ref=node_ref
        >

            <div class="container-fluid pl-0 pr-0 w-100 h-100 music-info-container">
                {move || {
                    let current_song = current_song.get();
                    if current_song.is_none() {
                        return ().into_any();
                    }
                    let canvas_url = canvaz_sig.get();
                    if let Some(canvas_url) = canvas_url {
                        view! {
                            <div class="dark-overlay" style="top: 0px;"></div>
                            <video class="canvaz-vid" src=canvas_url autoplay loop muted />
                        }
                            .into_any()
                    } else {
                        ().into_any()
                    }
                }} <div class="container-fluid pl-0 pr-0 musicinfo-mobile-view-container">
                    <div class="col col-auto">
                        <AudioStream />
                    </div>

                    {move || {
                        match show_queue.get() {
                            MusicInfoState::MusicInfo => {
                                view! {
                                    <div class="container-fluid">
                                        <div class="row no-gutters">
                                            <div class="col d-flex musicinfo-mobile-cover-container">
                                                <div class="musicinfo-mobile-cover">
                                                    <div class="image-container w-100">
                                                        <div class="embed-responsive embed-responsive-1by1">
                                                            <div class="embed-responsive-item albumart">
                                                                {move || {
                                                                    let cover_img = cover_img.get();
                                                                    if !show_default_cover_img.get() {
                                                                        view! {
                                                                            <img
                                                                                src=cover_img
                                                                                on:error=move |_| { show_default_cover_img.set(true) }
                                                                            />
                                                                        }
                                                                            .into_any()
                                                                    } else {
                                                                        view! { <SongDefaultIcon /> }.into_any()
                                                                    }
                                                                }}
                                                            </div>
                                                        </div>
                                                    </div>
                                                </div>
                                            </div>
                                        </div>
                                        <div class="row no-gutters song-info-container">
                                            <div class="col song-title-details text-truncate">
                                                {song_title}
                                            </div>
                                        </div>
                                        <div class="row no-gutters song-info-container">
                                            <div class="col song-subtitle-details text-truncate">
                                                {song_subtitle}
                                            </div>
                                        </div>
                                        <div class="row no-gutters time-container">
                                            <div class="col col-auto current-time-container">
                                                {current_time}
                                            </div>
                                            <div class="col slider-container">
                                                <Slider />
                                            </div>
                                            <div class="col col-auto total-time-container">
                                                {total_time}
                                            </div>
                                        </div>
                                        <div class="row no-gutters controls-container">
                                            <div class="col">
                                                <Controls show_time=false show_fav=false />
                                            </div>
                                        </div>
                                        <div class="row no-gutters mobile-musicinfo-buttons">
                                            <div class="col">
                                                <QueueIcon
                                                    on:click=move |_| { show_queue.set(MusicInfoState::Queue) }
                                                    active=RwSignal::new(false).read_only()
                                                />
                                            </div>
                                            <div class="col">
                                                <LyricsIcon on:click=move |_| {
                                                    show_queue.set(MusicInfoState::Lyrics)
                                                } />
                                            </div>
                                        </div>
                                    </div>
                                }
                                    .into_any()
                            }
                            MusicInfoState::Queue => {
                                view! {
                                    <div class="row no-gutters pl-3">
                                        <div class="col-auto prev-button">
                                            <PrevIcon on:click=move |_| {
                                                show_queue.set(MusicInfoState::MusicInfo);
                                            } />
                                        </div>
                                    </div>
                                    <div class="row queue-container-outer">
                                        <div class="col w-100 h-100 pl-0 pr-0 queue-container">

                                            <div class="w-100 h-100">

                                                <VirtualScroller
                                                    each=queue_songs
                                                    key=|(_, s)| s.song._id.clone()
                                                    item_height=95usize
                                                    inner_el_style="width: calc(100% - 15px);"
                                                    node_ref=scroller_ref
                                                    children=move |(index, song)| {
                                                        view! {
                                                            <QueueItem
                                                                current_song_index=current_song_index
                                                                eq_playing=is_playing
                                                                song=song.clone()
                                                                index=index
                                                                play_now=play_now
                                                                remove_from_queue=remove_from_queue
                                                                is_mobile=true
                                                            />
                                                        }
                                                    }
                                                    header=Some(())
                                                />

                                            </div>
                                        </div>
                                    </div>
                                }
                                    .into_any()
                            }
                            MusicInfoState::Lyrics => {
                                view! {
                                    <div class="row no-gutters pl-3">
                                        <div class="col-auto prev-button">
                                            <PrevIcon on:click=move |_| {
                                                show_queue.set(MusicInfoState::MusicInfo);
                                            } />
                                        </div>
                                    </div>
                                    <div
                                        class="row no-gutters h-100"
                                        // Stop propagating touchstart and mousedown so we don't trigger the musicbar drag
                                        on:touchstart=|ev| {
                                            ev.stop_propagation();
                                        }
                                        on:mousedown=|ev| {
                                            ev.stop_propagation();
                                        }
                                    >
                                        <div class="col">
                                            <div class="lyrics-container">
                                                <div class="lyrics-side-decoration"></div>
                                                <div class="lyrics-background"></div>
                                                <pre>{move || selected_lyrics.get()}</pre>
                                            </div>
                                        </div>
                                    </div>
                                }
                                    .into_any()
                            }
                        }
                    }}

                </div>
            </div>
        </div>
    }
}

#[tracing::instrument(level = "debug", skip(show, node_ref))]
#[component]
pub fn MusicInfo(#[prop()] show: Signal<bool>, #[prop()] node_ref: NodeRef<Div>) -> impl IntoView {
    let player_store = expect_context::<RwSignal<PlayerStore>>();
    let current_song = create_read_slice(player_store, move |p| p.get_current_song());
    let queue_songs = create_read_slice(player_store, move |p| p.get_queue_songs());
    let current_song_index = create_read_slice(player_store, |p| p.get_queue_index());
    let is_playing = create_read_slice(player_store, |p| {
        p.get_player_state() == PlayerState::Playing
    });
    let play_now = create_write_slice(player_store, |p, val| p.change_index(val, true));
    let remove_from_queue = create_write_slice(player_store, |p, val| p.remove_from_queue(val));

    let clear_queue = create_write_slice(player_store, |p, _| p.clear_queue_except_current());
    let canvaz_sig = RwSignal::new(None);

    let get_queue = create_read_slice(player_store, |p| {
        p.get_queue()
            .song_queue
            .iter()
            .filter_map(|id| p.get_queue().data.get(id).cloned())
            .collect::<Vec<_>>()
    });

    let ui_store = expect_context::<RwSignal<UiStore>>();
    let modal_store = expect_context::<RwSignal<ModalStore>>();

    Effect::new(move || {
        let current_song = current_song.get();
        canvaz_sig.set(None);
        if let Some(current_song) = current_song
            && current_song.song.type_ == SongType::SPOTIFY
            && current_song.song.playback_url.is_some()
        {
            spawn_local(async move {
                let res = crate::utils::invoke::get_canvaz(
                    current_song.song.playback_url.unwrap().clone(),
                    false,
                )
                .await;
                if let Ok(res) = res {
                    canvaz_sig.set(res.canvases.first().map(|c| c.url.clone()));
                } else {
                    tracing::error!("Failed to get canvaz {:?}", res)
                }
            });
        }
    });

    let scroller_ref: NodeRef<Div> = NodeRef::new();
    Effect::new(move || {
        let current_song_index = current_song_index.get();
        if let Some(el) = scroller_ref.get_untracked() {
            let el_top = 95usize * current_song_index;
            let options = ScrollToOptions::new();
            options.set_behavior(ScrollBehavior::Smooth);
            options.set_top(el_top as f64);
            el.scroll_with_scroll_to_options(&options);
        }
    });

    view! {
        <div
            class="slider"
            class:musicinfo-open=move || show.get()
            class:musicinfo-close=move || !show.get()
            node_ref=node_ref
        >

            <div class="h-100 w-100">
                // Canvas
                <div class="dark-overlay" style="top: 0px;"></div>
                {move || {
                    let current_song = current_song.get();
                    if current_song.is_none() {
                        return ().into_any();
                    }
                    let canvas_url = canvaz_sig.get();
                    let high_img = current_song.map(|current_song| get_high_img(&current_song));
                    if let Some(canvas_url) = canvas_url {
                        view! { <video class="canvaz-vid" src=canvas_url autoplay loop muted /> }
                            .into_any()
                    } else {
                        view! { <img class="bg-img" src=high_img /> }.into_any()
                    }
                }}

                <div class="container-fluid w-100 h-100 music-info-container">
                    <div class="row no-gutters justify-content-end">
                        // Close button
                        <div class="col-auto">
                            <div class="cross-icon button-grow">
                                <CrossIcon on:click=move |_| {
                                    ui_store.update(|s| s.show_queue(false));
                                } />
                            </div>
                        </div>
                    </div>

                    <div class="row no-gutters justify-content-center h-100 flex-nowrap">
                        // Song details
                        <div class="col-4">
                            <div class="row no-gutters">
                                <div class="col position-relative">
                                    <AudioStream />
                                </div>
                            </div>
                            <SongDetails
                                icons=RwSignal::new(SongDetailIcons::default())
                                selected_song=current_song
                                show_lyrics=true
                            />
                        </div>
                        <div class="col-7 offset-1 right-container h-100">
                            <div class="h-100">
                                <div class="row">
                                    <div class="col-auto d-flex">
                                        <div
                                            class="rounded-btn"
                                            on:click=move |_| {
                                                modal_store
                                                    .update(|store| {
                                                        store
                                                            .set_active_modal(
                                                                Modals::NewPlaylistModal(
                                                                    PlaylistModalState::NewPlaylist,
                                                                    Some(get_queue.get()),
                                                                ),
                                                            )
                                                    });
                                            }
                                        >
                                            Save as playlist
                                        </div>
                                        <div
                                            class="rounded-btn"
                                            on:click=move |_| clear_queue.set(())
                                        >
                                            Clear
                                        </div>
                                    </div>
                                </div>
                                <div class="row queue-container-outer">
                                    <div class="col w-100 h-100 mr-4 queue-container">
                                        <div class="w-100 h-100">

                                            <VirtualScroller
                                                each=queue_songs
                                                key=|(i, s)| format!("{:?}-{}", s.song._id, i)
                                                item_height=95usize
                                                inner_el_style="width: calc(100% - 15px);"
                                                node_ref=scroller_ref
                                                children=move |(index, song)| {
                                                    view! {
                                                        <QueueItem
                                                            current_song_index=current_song_index
                                                            eq_playing=is_playing
                                                            song=song.clone()
                                                            index=index
                                                            play_now=play_now
                                                            remove_from_queue=remove_from_queue
                                                            is_mobile=false
                                                        />
                                                    }
                                                }
                                                header=Some(())
                                            />

                                        </div>
                                    </div>
                                </div>
                            </div>
                        </div>
                    </div>
                </div>
            </div>

        </div>
    }
}
