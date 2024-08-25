use leptos::{
    component, create_rw_signal, create_write_slice, expect_context, view, IntoView, RwSignal,
    SignalGet,
};
use rand::seq::SliceRandom;
use std::rc::Rc;
use types::songs::GetSongOptions;
use types::ui::song_details::SongDetailIcons;

use crate::console_log;
use crate::store::player_store::PlayerStore;
use crate::utils::db_utils::get_songs_by_option;

use crate::components::songview::SongView;
use crate::utils::songs::get_songs_from_indices;

#[component()]
pub fn AllSongs() -> impl IntoView {
    let songs = create_rw_signal(vec![]);
    let selected_songs = create_rw_signal(vec![]);
    get_songs_by_option(
        GetSongOptions {
            song: Some(Default::default()),
            ..Default::default()
        },
        songs,
    );

    let player_store = expect_context::<RwSignal<PlayerStore>>();
    let play_songs_setter = create_write_slice(player_store, |p, song| p.play_now(song));
    let play_songs_multiple_setter =
        create_write_slice(player_store, |p, songs| p.play_now_multiple(songs));

    let add_to_queue_setter = create_write_slice(player_store, |p, songs| p.add_to_queue(songs));

    let play_songs = move || {
        let selected_songs = if selected_songs.get().is_empty() {
            songs.get()
        } else {
            get_songs_from_indices(songs, selected_songs)
        };

        play_songs_multiple_setter.set(selected_songs);
    };

    let add_to_queue = move || {
        if selected_songs.get().is_empty() {
            add_to_queue_setter.set(songs.get());
        } else {
            add_to_queue_setter.set(get_songs_from_indices(songs, selected_songs));
        }
    };

    let random = move || {
        let songs = songs.get();
        let random_song = songs.choose(&mut rand::thread_rng()).unwrap();
        play_songs_setter.set(random_song.clone());
    };

    let icons = create_rw_signal(SongDetailIcons {
        play: Some(Rc::new(Box::new(play_songs))),
        add_to_queue: Some(Rc::new(Box::new(add_to_queue))),
        random: Some(Rc::new(Box::new(random))),
        ..Default::default()
    });

    view! {
        <SongView
            icons=icons
            songs=songs
            selected_songs=selected_songs
            song_update_request=Box::new(move || {
                console_log!("Song update request");
                get_songs_by_option(
                    GetSongOptions {
                        song: Some(Default::default()),
                        ..Default::default()
                    },
                    songs,
                );
            })
        />
    }
}
