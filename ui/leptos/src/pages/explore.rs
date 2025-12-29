// Moosync
// Copyright (C) 2024, 2025 Moosync <support@moosync.app>
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program. If not, see <http://www.gnu.org/licenses/>.

use std::sync::Arc;

use leptos::{component, prelude::*, view, IntoView};
use leptos_i18n::t;
use types::{
    songs::{GetSongOptions, SearchableSong, Song},
    ui::{extensions::ExtensionProviderScope, song_details::SongDetailIcons},
};
use wasm_bindgen_futures::spawn_local;

use crate::{
    components::{
        cardview::{CardView, SimplifiedCardItem},
        songdetails::SongDetails,
        songlist::SongListItem,
    },
    i18n::use_i18n,
    icons::play_hover_icon::PlayHoverIcon,
    store::{player_store::PlayerStore, provider_store::ProviderStore, ui_store::UiStore},
    utils::{
        common::{format_duration, get_high_img},
        context_menu::{create_context_menu, SongItemContextMenu},
        db_utils::get_playlists_local,
        invoke::{
            get_provider_key_by_id, get_song_from_id, get_songs_by_options, get_suggestions,
            get_top_listened_songs,
        },
    },
};

struct AllAnalyticsParsed {
    total_listen_time: f64,
    songs: Vec<(Song, f64)>,
}

#[tracing::instrument(level = "debug", skip())]
#[component]
pub fn Explore() -> impl IntoView {
    let provider_store: Arc<ProviderStore> = expect_context();
    let provider_keys = provider_store.get_provider_keys(ExtensionProviderScope::Recommendations);
    let suggestion_items = RwSignal::new(vec![]);
    let analytics = RwSignal::<Option<AllAnalyticsParsed>>::new(None);
    spawn_local(async move {
        if let Ok(a) = get_top_listened_songs().await {
            tracing::debug!("Got analytics {:?}, {:?}", a, a.songs.len());
            analytics.set(Some(AllAnalyticsParsed {
                total_listen_time: a.total_listen_time,
                songs: vec![],
            }));
            for (song_id, time) in a.songs {
                let provider = get_provider_key_by_id(song_id.clone()).await;
                if let Ok(provider) = provider {
                    let song = get_song_from_id(provider.clone(), song_id, false).await;
                    match song {
                        Ok(song) => {
                            analytics.update(|a| {
                                if let Some(a) = a.as_mut() {
                                    a.songs.push((song, time))
                                }
                            });
                        }
                        Err(e) => tracing::error!("Failed to fetch song from {} {:?}", provider, e),
                    }
                } else {
                    let song = get_songs_by_options(GetSongOptions {
                        song: Some(SearchableSong {
                            _id: Some(song_id),
                            ..Default::default()
                        }),
                        ..Default::default()
                    })
                    .await;
                    if let Ok(song) = song {
                        if let Some(song) = song.first() {
                            analytics.update(|a| {
                                if let Some(a) = a.as_mut() {
                                    a.songs.push((song.clone(), time))
                                }
                            });
                        }
                    }
                }
            }
        }
    });

    spawn_local(async move {
        for key in provider_keys {
            let suggestions = get_suggestions(key, false).await;
            if let Ok(suggestions) = suggestions {
                suggestion_items.update(|s| s.extend(suggestions));
            }
        }
    });
    let total_time = create_read_slice(analytics, |a| a.as_ref().map(|a| a.total_listen_time));
    let first_song = create_read_slice(analytics, |a| {
        a.as_ref().and_then(|a| a.songs.first().cloned())
    });
    let first_four_songs = create_read_slice(analytics, |a| {
        a.as_ref().and_then(|a| {
            a.songs
                .get(1.min(a.songs.len())..5.min(a.songs.len()).max(1.min(a.songs.len())))
                .map(|s| s.to_vec())
        })
    });
    let second_four_songs = create_read_slice(analytics, |a| {
        a.as_ref().and_then(|a| {
            a.songs
                .get(5.min(a.songs.len())..9.min(a.songs.len()).max(5.min(a.songs.len())))
                .map(|s| s.to_vec())
        })
    });

    let player_store: RwSignal<PlayerStore> = expect_context();
    let play_now = create_write_slice(player_store, |p, s| p.play_now(s));

    let playlists = RwSignal::new(vec![]);
    get_playlists_local(playlists);

    let refresh_songs = move || {};

    let context_menu_data = SongItemContextMenu {
        current_song: None,
        song_list: suggestion_items.read_only(),
        selected_songs: RwSignal::new(vec![]),
        playlists,
        refresh_cb: Arc::new(Box::new(refresh_songs)),
    };
    let song_context_menu = create_context_menu(context_menu_data);

    let i18n = use_i18n();

    let is_mobile =
        create_read_slice(expect_context::<RwSignal<UiStore>>(), |u| u.get_is_mobile()).get();
    view! {
        <div class="w-100 h-100">
            <div class="container-fluid song-container h-100 d-flex flex-column">

                <div class="row page-title no-gutters">

                    <div class="col-auto">{t!(i18n, pages.explore)}</div>
                    <div class="col align-self-center"></div>
                </div>

                <div class="row no-gutters">
                    <div class="col total-listen-time">
                        <span class="d-inline">
                            {"You listened to "}
                            <span class="total-listen-time-item">
                                {move || format_duration(
                                    total_time.get().unwrap_or_default(),
                                    true,
                                )}
                            </span> {" Hours of music"}
                        </span>
                    </div>
                </div>

                {move || {
                    if !is_mobile {
                        view! {
                            <div class="row no-gutters analytics">
                                <div class="col">
                                    <div class="row no-gutters">

                                        {move || {
                                            if let Some((first_song, time)) = first_song.get() {
                                                view! {
                                                    <div class="col big-song">
                                                        <SongDetails
                                                            selected_song=RwSignal::new(Some(first_song.clone()))
                                                            icons=RwSignal::new(SongDetailIcons::default())
                                                        />
                                                        <div
                                                            class="play-button-song-list card-overlay-background d-flex justify-content-center explore-big-song-play"
                                                            on:click=move |_| { play_now.set(first_song.clone()) }
                                                        >
                                                            <PlayHoverIcon />
                                                        </div>
                                                    </div>
                                                    <div class="col-auto">
                                                        <div class="played-for">"Played for"</div>
                                                        <div class="d-flex">
                                                            <span class="playtime big-playtime">
                                                                {format_duration(time, true)}
                                                            </span>
                                                        </div>
                                                    </div>
                                                }
                                                    .into_any()
                                            } else {
                                                ().into_any()
                                            }
                                        }}

                                    </div>
                                </div>
                                <div class="col-3 small-song-first">
                                    <For
                                        each=move || first_four_songs.get().unwrap_or_default()
                                        key=move |(s, _)| s.song._id.clone()
                                        children=move |(s, time)| {
                                            view! {
                                                <SongListItem
                                                    song=s
                                                    is_selected=Box::new(move || false)
                                                    on_context_menu=move |_| {}
                                                    on_click=move |_| {}
                                                    show_background=false
                                                    custom_duration=format_duration(time, true)
                                                />
                                            }
                                        }
                                    />

                                </div>
                                <div class="col-3 small-song-second">
                                    <For
                                        each=move || second_four_songs.get().unwrap_or_default()
                                        key=move |(s, _)| s.song._id.clone()
                                        children=move |(s, time)| {
                                            view! {
                                                <SongListItem
                                                    song=s
                                                    is_selected=Box::new(move || false)
                                                    on_context_menu=move |_| {}
                                                    on_click=move |_| {}
                                                    show_background=false
                                                    custom_duration=format_duration(time, true)
                                                />
                                            }
                                        }
                                    />
                                </div>
                            </div>
                        }
                            .into_any()
                    } else {
                        ().into_any()
                    }
                }}

                <div
                    class="row no-gutters w-100 flex-grow-1"
                    style="align-items: flex-start; height: 100%"
                >

                    <CardView
                        items=suggestion_items
                        key=|a| a.song._id.clone()
                        songs_view=true
                        on_click=Box::new(move |item| { play_now.set(item) })
                        card_item=move |(_, item)| {
                            let song_context_menu = song_context_menu.clone();
                            SimplifiedCardItem {
                                title: item.song.title.clone().unwrap_or_default(),
                                cover: Some(get_high_img(item)),
                                id: item.clone(),
                                icon: item.song.provider_extension.clone(),
                                context_menu: Some(
                                    Arc::new(
                                        Box::new(move |ev, song| {
                                            ev.stop_propagation();
                                            let mut data = song_context_menu.get_data();
                                            data.current_song = Some(song.clone());
                                            drop(data);
                                            song_context_menu.show(ev);
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
