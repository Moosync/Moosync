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

use std::{collections::HashMap, sync::Arc};

use crate::{
    components::cardview::{CardView, SimplifiedCardItem},
    utils::invoke::{provider_search, search_all},
};
use colors_transform::{Color, Rgb};
use leptos::{component, ev::wheel, html::Div, prelude::*, view, IntoView, Params};
use leptos_router::{hooks::use_query, params::Params};
use leptos_use::{use_event_listener, use_resize_observer};
use types::ui::extensions::ExtensionProviderScope;
use wasm_bindgen_futures::spawn_local;
use web_sys::window;

use crate::{
    components::songlist::SongList,
    icons::{next_icon::NextIcon, prev_icon::PrevIcon},
    store::provider_store::ProviderStore,
};

#[derive(Params, PartialEq)]
struct SearchQuery {
    q: Option<String>,
}

#[tracing::instrument(level = "trace", skip(keys, selected, single_select))]
#[component()]
pub fn TabCarousel(
    #[prop()] keys: Vec<String>,
    #[prop()] selected: RwSignal<Vec<String>>,
    #[prop()] single_select: bool,
    #[prop(optional, default = true)] align_left: bool,
) -> impl IntoView {
    let provider_container = NodeRef::<Div>::new();

    let container_size = RwSignal::new(0f64);

    let show_next_icon = RwSignal::new(false);
    let show_prev_icon = RwSignal::new(false);

    let gradient_style = RwSignal::new("".to_string());

    let scroll_left = RwSignal::new(0);

    use_resize_observer(provider_container, move |entries, _| {
        request_animation_frame(move || {
            let rect = entries[0].content_rect();
            container_size.set(rect.width());
        });
    });

    // I have no idea what the fuck is this supposed to do.... but it works
    // Designed this like an year ago and never added comments to it
    let _ = use_event_listener(provider_container, wheel, move |ev| {
        ev.stop_propagation();
        ev.prevent_default();

        let provider_container = provider_container.get().unwrap();

        let scroll_left_prev = provider_container.scroll_left();
        if ev.delta_y() > 0f64 {
            provider_container.set_scroll_left(scroll_left_prev + 20);
        } else {
            provider_container.set_scroll_left(scroll_left_prev - 20);
        }

        scroll_left.set(provider_container.scroll_left());
    });

    Effect::new(move || {
        let provider_container = provider_container.get();
        if let Some(provider_container) = provider_container {
            let scroll_width = provider_container.scroll_width();
            let scroll_left = scroll_left.get();
            let container_size = container_size.get() as i32;

            show_next_icon.set((scroll_left + container_size) < scroll_width);
            show_prev_icon.set(scroll_left > 0);

            let gradient_left = if show_prev_icon.get() { 10 } else { 0 };

            let gradient_right = if show_next_icon.get() { 90 } else { 100 };

            let primary_color = window()
                .unwrap()
                .get_computed_style(&provider_container)
                .unwrap()
                .unwrap()
                .get_property_value("--primary")
                .unwrap();

            let rgb_color = if primary_color.starts_with("#") {
                Rgb::from_hex_str(primary_color.as_str())
            } else {
                primary_color.parse::<Rgb>()
            };

            if let Ok(rgb_color) = rgb_color {
                let rgba_string = format!(
                    "rgba({}, {}, {}, 0)",
                    rgb_color.get_red(),
                    rgb_color.get_green(),
                    rgb_color.get_blue()
                );

                gradient_style.set(format!(
                    "background: linear-gradient(90deg, var(--primary) 0% , {} {}%, {} {}%, var(--primary) 100%);", rgba_string,
                    gradient_left, rgba_string, gradient_right
                ));
            }
        }
    });

    let provider_store = expect_context::<Arc<ProviderStore>>();

    view! {
        <div class="container-fluid">
            <div class="row no-gutters">
                <div class="col song-header-options w-100">
                    <div class="row no-gutters align-items-center h-100">

                        <Show
                            when=move || { !show_prev_icon.get() }
                            fallback=|| {
                                view! {
                                    <div class="col-auto mr-3 h-100 d-flex align-items-center">
                                        <PrevIcon />
                                    </div>
                                }
                            }
                        >
                            <div></div>
                        </Show>

                        <div class="col provider-outer-container">
                            <div class="gradient-overlay" style=move || gradient_style.get()></div>

                            <div
                                node_ref=provider_container
                                class="provider-container d-flex"
                                class:justify-content-end=move || !align_left
                            >

                                <For
                                    each=move || keys.clone()
                                    key=|key| key.clone()
                                    children=move |key| {
                                        let key_tmp = key.clone();
                                        let key_tmp1 = key.clone();
                                        view! {
                                            <div
                                                class="h-100 item-checkbox-col mr-2"
                                                on:click=move |_| {
                                                    if selected.get().contains(&key_tmp) {
                                                        selected.update(|s| s.retain(|x| x != &key_tmp));
                                                    } else if !single_select {
                                                        selected.update(|s| s.push(key_tmp.clone()));
                                                    } else {
                                                        selected.set(vec![key_tmp.clone()]);
                                                    }
                                                }
                                            >
                                                <div
                                                    class="h-100 d-flex item-checkbox-container"
                                                    style=move || {
                                                        if selected.get().contains(&key_tmp1) {
                                                            "background: var(--textSecondary);"
                                                        } else {
                                                            "background: var(--secondary);"
                                                        }
                                                    }
                                                >
                                                    <span class="align-self-center provider-title">
                                                        {if let Some(provider) = provider_store
                                                            .get_provider_name_by_key(key.clone())
                                                        {
                                                            provider.name
                                                        } else {
                                                            key
                                                        }}
                                                    </span>
                                                </div>
                                            </div>
                                        }
                                    }
                                />

                            </div>
                        </div>

                        <Show
                            when=move || { !show_next_icon.get() }
                            fallback=|| {
                                view! {
                                    <div class="col-auto ml-3 mr-3 h-100 d-flex align-items-center">
                                        <NextIcon />
                                    </div>
                                }
                            }
                        >
                            <div></div>
                        </Show>
                    </div>
                </div>
            </div>
        </div>
    }
}

#[tracing::instrument(level = "trace", skip())]
#[component()]
pub fn Search() -> impl IntoView {
    let query = use_query::<SearchQuery>();
    let term = move || {
        query.with(|query| {
            query
                .as_ref()
                .map(|query| query.q.clone())
                .unwrap_or_default()
        })
    };

    let search_results = RwSignal::new(HashMap::new());

    let provider_store = expect_context::<Arc<ProviderStore>>();
    let mut keys = provider_store.get_provider_keys(ExtensionProviderScope::Search);
    keys.insert(0, "Local".into());

    let selected_provider = RwSignal::new(vec![]);
    let selected_category = RwSignal::new(vec![]);

    let category_keys = vec![
        "Songs".to_string(),
        "Albums".to_string(),
        "Artists".to_string(),
        "Playlists".to_string(),
    ];

    if let Some(first_provider) = keys.first() {
        selected_provider.set(vec![first_provider.clone()]);
    }
    selected_category.set(vec![category_keys.first().unwrap().clone()]);

    let keys_clone = keys.clone();
    let is_loading = RwSignal::new(HashMap::new());
    Effect::new(move || {
        let search_term = term();
        if let Some(search_term) = search_term {
            if search_term.is_empty() {
                return;
            }
            tracing::debug!("Searching for: {}", search_term);

            let keys = keys_clone.clone();
            spawn_local(async move {
                for key in keys {
                    let key_cl = key.clone();
                    is_loading.update(move |map| {
                        map.insert(key_cl, true);
                    });
                    let res = if key == "Local" {
                        search_all(search_term.clone()).await
                    } else {
                        provider_search(key.clone(), search_term.clone()).await
                    };
                    match res {
                        Ok(res) => {
                            search_results.update(|map| {
                                map.insert(key.clone(), res);
                            });
                        }
                        Err(err) => {
                            tracing::error!(
                                "Error searching for {} ({}): {:?}",
                                search_term,
                                key,
                                err
                            );
                        }
                    }
                    is_loading.update(move |map| {
                        map.remove(&key);
                    });
                }
            });
        }
    });

    let refresh_songs = move || {};
    let fetch_next_page = move || {};

    view! {
        <div class="w-100 h-100">
            <div class="container-fluid h-100 d-flex flex-column">

                <TabCarousel keys=keys.clone() selected=selected_provider single_select=true />
                <TabCarousel
                    keys=category_keys.clone()
                    selected=selected_category
                    single_select=true
                />

                <div class="container-fluid mt-3 search-song-list-container">
                    <div class="row no-gutters h-100" style="position: relative;">
                        {move || {
                            view! {
                                <Show
                                    when=move || {
                                        is_loading
                                            .with(move |map| {
                                                let binding = selected_provider.get();
                                                let active_provider = binding.first();
                                                if let Some(active_provider) = active_provider {
                                                    *map.get(active_provider).unwrap_or(&false)
                                                } else {
                                                    false
                                                }
                                            })
                                    }
                                    fallback=move || ()
                                >
                                    <div class="spinner-container">
                                        <div class="spinner-border overlay-spinner"></div>
                                    </div>
                                </Show>
                                {move || {
                                    let search_results = search_results.get();
                                    let binding = selected_provider.get();
                                    let active_provider = binding.first();
                                    if active_provider.is_none() {
                                        return ().into_any();
                                    }
                                    let active_provider = active_provider.unwrap();
                                    if let Some(res) = search_results.get(active_provider) {
                                        let binding = selected_category.get();
                                        let active_category = binding.first();
                                        if active_category.is_none() {
                                            return ().into_any();
                                        }
                                        let active_category = active_category.unwrap();
                                        return match active_category.as_str() {
                                            "Songs" => {
                                                view! {
                                                    <div class="col h-100 song-list-compact">
                                                        <SongList
                                                            hide_search_bar=true
                                                            song_list=RwSignal::new(res.songs.clone()).read_only()
                                                            selected_songs_sig=RwSignal::new(vec![])
                                                            filtered_selected=RwSignal::new(vec![])
                                                            refresh_cb=refresh_songs
                                                            fetch_next_page=fetch_next_page
                                                            header=()
                                                            enable_sort=false
                                                        />
                                                    </div>
                                                }
                                                    .into_any()
                                            }
                                            "Albums" => {
                                                view! {
                                                    <CardView
                                                        items=RwSignal::new(res.albums.clone())
                                                        key=|a| a.album_id.clone()
                                                        redirect_root="/main/albums"
                                                        card_item=move |(_, item)| {
                                                            SimplifiedCardItem {
                                                                title: item.album_name.clone().unwrap_or_default(),
                                                                cover: item.album_coverpath_high.clone(),
                                                                id: item.clone(),
                                                                icon: None,
                                                                context_menu: None,
                                                            }
                                                        }
                                                    />
                                                }
                                                    .into_any()
                                            }
                                            "Artists" => {
                                                view! {
                                                    <CardView
                                                        items=RwSignal::new(res.artists.clone())
                                                        key=|a| a.artist_id.clone()
                                                        redirect_root="/main/artists"
                                                        card_item=move |(_, item)| {
                                                            SimplifiedCardItem {
                                                                title: item.artist_name.clone().unwrap_or_default(),
                                                                cover: item.artist_coverpath.clone(),
                                                                id: item.clone(),
                                                                icon: None,
                                                                context_menu: None,
                                                            }
                                                        }
                                                    />
                                                }
                                                    .into_any()
                                            }
                                            "Playlists" => {
                                                view! {
                                                    <CardView
                                                        items=RwSignal::new(res.playlists.clone())
                                                        key=|a| a.playlist_id.clone()
                                                        redirect_root="/main/playlists/"
                                                        card_item=move |(_, item)| {
                                                            SimplifiedCardItem {
                                                                title: item.playlist_name.clone(),
                                                                cover: item.playlist_coverpath.clone(),
                                                                id: item.clone(),
                                                                icon: None,
                                                                context_menu: None,
                                                            }
                                                        }
                                                    />
                                                }
                                                    .into_any()
                                            }
                                            _ => ().into_any(),
                                        };
                                    }
                                    ().into_any()
                                }}
                            }
                                .into_any()
                        }}
                    </div>
                </div>
            </div>
        </div>
    }
}
