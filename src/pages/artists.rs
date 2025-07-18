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

use std::sync::Arc;

use crate::components::cardview::{CardView, SimplifiedCardItem};
use crate::components::songlist::ShowProvidersArgs;
use crate::dyn_provider_songs;
use crate::i18n::use_i18n;
use crate::store::player_store::PlayerStore;
use crate::store::ui_store::UiStore;
use crate::utils::common::{convert_file_src, fetch_infinite};
use crate::utils::db_utils::get_artists_by_option;
use crate::utils::songs::get_songs_from_indices;
use leptos::{component, prelude::*, view, IntoView};
use leptos_i18n::t;
use leptos_router::hooks::use_query_map;
use std::collections::HashMap;
use types::entities::QueryableArtist;
use types::songs::{GetSongOptions, Song};
use types::ui::extensions::ExtensionProviderScope;
use types::ui::song_details::{DefaultDetails, SongDetailIcons};
use uuid::Uuid;
use wasm_bindgen_futures::spawn_local;

use crate::components::songview::SongView;
use crate::utils::db_utils::get_songs_by_option;
use rand::seq::IndexedRandom;

#[tracing::instrument(level = "debug", skip())]
#[component()]
pub fn SingleArtist() -> impl IntoView {
    let params = use_query_map();
    let artist = Memo::new(move |_| {
        params.with(|params| {
            let entity = params.get("entity");
            tracing::info!("Got entity {:?}", entity);
            if let Some(entity) = entity {
                let album = serde_json::from_str::<QueryableArtist>(&entity);
                match album {
                    Ok(album) => return Some(album),
                    Err(e) => tracing::error!("Failed to parse artist: {:?}", e),
                }
            }
            None
        })
    });
    if artist.get().is_none() {
        tracing::error!("Failed to parse artist");
        return ().into_any();
    }

    tracing::debug!("Parsed artist {:?}", artist.get());

    let songs = RwSignal::new(vec![]);
    let selected_songs = RwSignal::new(vec![]);

    let default_details = RwSignal::new(DefaultDetails::default());

    Effect::new(move || {
        let artist = artist.get();
        if let Some(artist) = artist {
            default_details.update(|d| {
                d.title = artist.artist_name.clone();
                d.icon = artist.artist_coverpath.clone().map(convert_file_src);
            });

            get_songs_by_option(
                GetSongOptions {
                    artist: Some(QueryableArtist {
                        artist_id: artist.artist_id,
                        ..Default::default()
                    }),
                    ..Default::default()
                },
                songs,
            );
        }
    });

    let selected_providers = RwSignal::<Vec<String>>::new(vec![]);

    let (_, filtered_songs, fetch_selected_providers, is_loading) =
        dyn_provider_songs!(selected_providers, artist, songs, get_artist_content);

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
        let random_song = songs.choose(&mut rand::rng()).unwrap();
        play_songs_setter.set(random_song.clone());
    };

    let icons = RwSignal::new(SongDetailIcons {
        play: Some(Arc::new(Box::new(play_songs))),
        add_to_queue: Some(Arc::new(Box::new(add_to_queue))),
        random: Some(Arc::new(Box::new(random))),
        ..Default::default()
    });

    let refresh_songs = move || {};
    let fetch_next_page = move || {
        fetch_selected_providers.as_ref()();
    };

    let is_mobile =
        create_read_slice(expect_context::<RwSignal<UiStore>>(), |u| u.get_is_mobile()).get();

    view! {
        <SongView
            default_details=default_details
            songs=filtered_songs
            icons=icons
            selected_songs=selected_songs
            providers=ShowProvidersArgs {
                show_providers: true,
                selected_providers,
                scope: Some(ExtensionProviderScope::ArtistSongs),
            }
            refresh_cb=refresh_songs
            fetch_next_page=fetch_next_page
            show_mobile_default_details=is_mobile
            is_loading=is_loading
        />
    }
    .into_any()
}

#[tracing::instrument(level = "debug", skip())]
#[component()]
pub fn AllArtists() -> impl IntoView {
    let artists = RwSignal::new(vec![]);
    get_artists_by_option(QueryableArtist::default(), artists.write_only());

    let i18n = use_i18n();
    view! {
        <div class="w-100 h-100">
            <div class="container-fluid song-container h-100 d-flex flex-column">
                <div class="row page-title no-gutters">

                    <div class="col-auto">{t!(i18n, pages.artists)}</div>
                    <div class="col align-self-center"></div>
                </div>

                <div
                    class="row no-gutters w-100 flex-grow-1"
                    style="align-items: flex-start; height: 70%"
                >
                    <CardView
                        items=artists
                        key=|a| a.artist_id.clone().unwrap_or(Uuid::new_v4().to_string())
                        redirect_root="/main/artists"
                        card_item=move |(_, item)| {
                            let artist_name = item.artist_name.clone().unwrap_or_default();
                            let artist_coverpath = item.artist_coverpath.clone();
                            SimplifiedCardItem {
                                title: artist_name,
                                cover: artist_coverpath,
                                id: item.clone(),
                                icon: None,
                                context_menu: None,
                            }
                        }
                    />
                </div>
            </div>
        </div>
    }
}
