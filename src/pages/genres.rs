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

use crate::components::cardview::{CardView, SimplifiedCardItem};
use crate::components::songview::SongView;
use crate::i18n::use_i18n;
use crate::store::player_store::PlayerStore;
use crate::utils::db_utils::get_genres_by_option;
use crate::utils::db_utils::get_songs_by_option;
use leptos::{component, prelude::*, view, IntoView};
use leptos_i18n::t;
use leptos_router::hooks::use_query_map;
use rand::seq::SliceRandom;
use std::sync::Arc;
use types::entities::QueryableGenre;
use types::songs::{GetSongOptions, Song};
use types::ui::song_details::SongDetailIcons;

#[tracing::instrument(level = "trace", skip())]
#[component()]
pub fn SingleGenre() -> impl IntoView {
    let params = use_query_map();
    let genre_id = params.with(|params| params.get("id")).unwrap();

    let songs = RwSignal::new(vec![]);
    let selected_songs = RwSignal::new(vec![]);

    get_songs_by_option(
        GetSongOptions {
            genre: Some(QueryableGenre {
                genre_id: Some(genre_id),
                ..Default::default()
            }),
            ..Default::default()
        },
        songs,
    );

    let player_store = expect_context::<RwSignal<PlayerStore>>();
    let play_songs_setter = create_write_slice(player_store, |p, song| p.play_now(song));
    let add_to_queue_setter = create_write_slice(player_store, |p, songs| p.add_to_queue(songs));

    let play_songs = move || {
        let selected_songs = selected_songs.get();
        let songs = songs.get();

        let selected_songs = if selected_songs.is_empty() {
            songs
        } else {
            selected_songs
                .into_iter()
                .map(|song_index| {
                    let song: &Song = songs.get(song_index).unwrap();
                    song.clone()
                })
                .collect()
        };

        let first_song = selected_songs.first();
        if let Some(first_song) = first_song {
            play_songs_setter.set(first_song.clone())
        }
        add_to_queue_setter.set(selected_songs[1..].to_vec());
    };

    let add_to_queue = move || {
        let selected_songs = selected_songs.get();
        let songs = songs.get();
        if selected_songs.is_empty() {
            add_to_queue_setter.set(songs.clone());
        } else {
            let selected_songs = selected_songs
                .into_iter()
                .map(|song_index| {
                    let song: &Song = songs.get(song_index).unwrap();
                    song.clone()
                })
                .collect();
            add_to_queue_setter.set(selected_songs);
        }
    };

    let random = move || {
        let songs = songs.get();
        let random_song = songs.choose(&mut rand::thread_rng()).unwrap();
        play_songs_setter.set(random_song.clone());
    };

    let icons = RwSignal::new(SongDetailIcons {
        play: Some(Arc::new(Box::new(play_songs))),
        add_to_queue: Some(Arc::new(Box::new(add_to_queue))),
        random: Some(Arc::new(Box::new(random))),
        ..Default::default()
    });

    let refresh_songs = move || {};
    let fetch_next_page = move || {};

    view! {
        <SongView
            songs=songs
            icons=icons
            selected_songs=selected_songs
            refresh_cb=refresh_songs
            fetch_next_page=fetch_next_page
        />
    }
}

#[tracing::instrument(level = "trace", skip())]
#[component()]
pub fn AllGenres() -> impl IntoView {
    let genres = RwSignal::new(vec![]);
    get_genres_by_option(QueryableGenre::default(), genres.write_only());

    let i18n = use_i18n();
    view! {
        <div class="w-100 h-100">
            <div class="container-fluid song-container h-100 d-flex flex-column">
                <div class="row page-title no-gutters">

                    <div class="col-auto">{t!(i18n, pages.genres)}</div>
                    <div class="col align-self-center"></div>
                </div>

                <div
                    class="row no-gutters w-100 flex-grow-1"
                    style="align-items: flex-start; height: 70%"
                >
                    <CardView
                        items=genres
                        key=|a| a.genre_id.clone()
                        redirect_root="/main/genre"
                        card_item=move |(_, item)| {
                            let genre_name = item.genre_name.clone().unwrap_or_default();
                            SimplifiedCardItem {
                                title: genre_name,
                                cover: None,
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
