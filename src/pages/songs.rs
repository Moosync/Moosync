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

use leptos::{component, prelude::*, view, IntoView};
use rand::seq::SliceRandom;
use std::sync::Arc;
use types::songs::GetSongOptions;
use types::ui::song_details::SongDetailIcons;

use crate::store::player_store::PlayerStore;
use crate::utils::db_utils::get_songs_by_option;

use crate::components::songview::SongView;
use crate::utils::songs::get_songs_from_indices;

#[tracing::instrument(level = "trace", skip())]
#[component()]
pub fn AllSongs() -> impl IntoView {
    let songs = RwSignal::new(vec![]);
    let selected_songs = RwSignal::new(vec![]);
    let refresh_songs = move || {
        tracing::debug!("Calling refresh cb");
        get_songs_by_option(
            GetSongOptions {
                song: Some(Default::default()),
                ..Default::default()
            },
            songs,
        );
    };

    let player_store = expect_context::<RwSignal<PlayerStore>>();
    let play_songs_setter = create_write_slice(player_store, |p, song| p.play_now(song));
    let play_songs_multiple_setter =
        create_write_slice(player_store, |p, songs| p.play_now_multiple(songs));

    let add_to_queue_setter = create_write_slice(player_store, |p, songs| p.add_to_queue(songs));

    let play_songs = move || {
        let selected_songs = if selected_songs.get().is_empty() {
            songs.get()
        } else {
            get_songs_from_indices(&songs, selected_songs)
        };

        play_songs_multiple_setter.set(selected_songs);
    };

    let add_to_queue = move || {
        if selected_songs.get().is_empty() {
            add_to_queue_setter.set(songs.get());
        } else {
            add_to_queue_setter.set(get_songs_from_indices(&songs, selected_songs));
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

    refresh_songs();
    let fetch_next_page = move || {};

    view! {
        <SongView
            icons=icons
            songs=songs
            selected_songs=selected_songs
            song_update_request=Box::new(move || {
                get_songs_by_option(
                    GetSongOptions {
                        song: Some(Default::default()),
                        ..Default::default()
                    },
                    songs,
                );
            })
            refresh_cb=refresh_songs
            fetch_next_page=fetch_next_page
        />
    }
}
