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

use std::collections::HashMap;

use crate::utils::common::{DefaultDetails, SongDetailIcons};
use leptos::{component, html::Div, prelude::*, view};
use leptos_use::on_click_outside;
use songs_proto::moosync::types::Song;
use web_sys::{Event, Node};

use crate::{
    components::{
        songdetails::SongDetails,
        songlist::{ShowProvidersArgs, SongList},
    },
    utils::context_menu::{SongsContextMenu, create_context_menu},
};

#[tracing::instrument(
    level = "trace",
    skip(
        songs,
        icons,
        selected_songs,
        song_update_request,
        default_details,
        refresh_cb,
        fetch_next_page
    )
)]
#[component()]
pub fn SongView(
    #[prop()] songs: impl Get<Value = Vec<Song>> + Copy + 'static + Send + Sync,
    #[prop()] icons: RwSignal<SongDetailIcons>,
    #[prop()] selected_songs: RwSignal<Vec<usize>>,
    #[prop()] refresh_cb: impl Fn() + 'static + Send + Sync,
    #[prop()] fetch_next_page: impl Fn() + 'static + Send + Sync,
    #[prop(optional)] is_loading: RwSignal<HashMap<String, bool>>,
    #[prop(optional)] song_update_request: Option<Box<dyn Fn() + Send + Sync>>,
    #[prop(optional)] default_details: RwSignal<DefaultDetails>,
    #[prop(optional, default=ShowProvidersArgs::default())] providers: ShowProvidersArgs,
    #[prop(optional, default = false)] show_mobile_default_details: bool,
) -> impl IntoView {
    let last_selected_song = RwSignal::new(None::<Song>);

    let filtered_selected = RwSignal::new(vec![]);

    Effect::new(move || {
        let selected_song = selected_songs.get().last().cloned();
        if show_mobile_default_details {
            return;
        }

        if let Some(selected_song) = selected_song {
            let all_songs = songs.get();
            tracing::debug!("selected {:?}", all_songs.get(selected_song).unwrap());
            last_selected_song.set(all_songs.get(selected_song).cloned());
        } else {
            last_selected_song.set(None);
        }
    });

    let song_details_container = NodeRef::<Div>::new();
    let song_list_container = NodeRef::<Div>::new();

    let ignore_class_list = &[
        "context-menu-root",
        "context-menu-outer",
        "context-menu-item",
        "context-menu-item-text",
        "context-menu-item-icon",
        "context-menu-right-arrow",
    ];
    let ignore_class = move |e: &Event| {
        for item in e.composed_path().iter() {
            let item: web_sys::HtmlElement = item.into();
            let class_list = item.class_list();
            if class_list.is_undefined() || class_list.is_null() {
                continue;
            }

            for ele in class_list.values().into_iter().flatten() {
                if ignore_class_list.contains(&ele.as_string().unwrap_or_default().as_str()) {
                    return true;
                }
            }
        }
        false
    };

    let _ = on_click_outside(song_details_container, move |e| {
        if ignore_class(&e) {
            return;
        }

        let target = event_target::<Node>(&e);
        let song_details_elem = song_list_container.get_untracked().unwrap();

        if !song_details_elem.contains(Some(&target)) {
            selected_songs.update(|s| s.clear());
            filtered_selected.update(|s| s.clear());
        }
    });

    let _ = on_click_outside(song_list_container, move |e| {
        if ignore_class(&e) {
            return;
        }

        let target = event_target::<Node>(&e);
        let song_details_elem = song_details_container.get_untracked().unwrap();

        if !song_details_elem.contains(Some(&target)) {
            selected_songs.update(|s| s.clear());
            filtered_selected.update(|s| s.clear());
        }
    });

    let song_context_menu = create_context_menu(SongsContextMenu::new(song_update_request));

    view! {
        <div
            class="w-100 h-100"
            on:contextmenu=move |ev| {
                ev.prevent_default();
                song_context_menu.show(ev);
            }
        >
            <div class="container-fluid song-container h-100">
                <div class="row no-gutters h-100 compact-container">
                    <div style="max-height: 100%;" class="song-details-container col-xl-3 col-4">
                        <SongDetails
                            buttons_ref=song_details_container
                            default_details=default_details
                            selected_song=last_selected_song.read_only()
                            icons=icons
                        />

                    </div>
                    <div
                        node_ref=song_list_container
                        class="col-xl-9 col-md-8 col h-100 song-list-compact"
                    >
                        <SongList
                            song_list=songs
                            selected_songs_sig=selected_songs
                            filtered_selected=filtered_selected
                            providers=providers
                            refresh_cb=refresh_cb
                            fetch_next_page=fetch_next_page
                            is_loading=is_loading
                            header_height=if show_mobile_default_details { 375 } else { 0 }
                            header=if show_mobile_default_details {
                                Some(
                                    view! {
                                        <SongDetails
                                            buttons_ref=song_details_container
                                            default_details=default_details
                                            selected_song=last_selected_song.read_only()
                                            icons=icons
                                        />
                                    }
                                        .into_any(),
                                )
                            } else {
                                None
                            }
                        />
                    </div>
                </div>
            </div>

        </div>
    }
}
