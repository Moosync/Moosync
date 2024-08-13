use std::rc::Rc;

use crate::components::cardview::{CardView, SimplifiedCardItem};
use crate::store::player_store::PlayerStore;
use crate::utils::db_utils::get_artists_by_option;
use leptos::{
    component, create_rw_signal, create_write_slice, expect_context, view, IntoView, RwSignal,
    SignalGet, SignalWith,
};
use leptos_router::use_query_map;
use types::entities::QueryableArtist;
use types::songs::{GetSongOptions, Song};
use types::ui::song_details::SongDetailIcons;

use crate::components::songview::SongView;
use crate::utils::db_utils::get_songs_by_option;
use rand::seq::SliceRandom;

#[component()]
pub fn SingleArtist() -> impl IntoView {
    let params = use_query_map();
    let artist_id = params.with(|params| params.get("id").cloned()).unwrap();

    let songs = create_rw_signal(vec![]);
    let selected_songs = create_rw_signal(vec![]);

    get_songs_by_option(
        GetSongOptions {
            artist: Some(QueryableArtist {
                artist_id: Some(artist_id),
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

    let icons = create_rw_signal(SongDetailIcons {
        play: Some(Rc::new(Box::new(play_songs))),
        add_to_queue: Some(Rc::new(Box::new(add_to_queue))),
        random: Some(Rc::new(Box::new(random))),
        ..Default::default()
    });

    view! { <SongView songs=songs icons=icons selected_songs=selected_songs /> }
}

#[component()]
pub fn AllArtists() -> impl IntoView {
    let artists = create_rw_signal(vec![]);
    get_artists_by_option(QueryableArtist::default(), artists.write_only());

    view! {
        <div class="w-100 h-100">
            <div class="container-fluid song-container h-100 d-flex flex-column">
                <div class="row page-title no-gutters">

                    <div class="col-auto">Artists</div>
                    <div class="col align-self-center"></div>
                </div>

                <div
                    class="row no-gutters w-100 flex-grow-1"
                    style="align-items: flex-start; height: 70%"
                >
                    <CardView
                        items=artists
                        card_item=move |(_, item)| {
                            let artist_name = item.artist_name.clone().unwrap_or_default();
                            let artist_coverpath = item.artist_coverpath.clone();
                            let artist_id = item.artist_id.clone().unwrap_or_default();
                            SimplifiedCardItem {
                                title: artist_name,
                                cover: artist_coverpath,
                                id: artist_id,
                            }
                        }
                    />
                </div>
            </div>
        </div>
    }
}
