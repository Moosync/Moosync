use std::rc::Rc;

use crate::components::cardview::{CardView, SimplifiedCardItem};
use crate::components::songlist::ShowProvidersArgs;
use crate::store::player_store::PlayerStore;
use crate::store::provider_store::ProviderStore;
use crate::utils::common::fetch_infinite;
use crate::utils::db_utils::get_artists_by_option;
use crate::utils::songs::get_songs_from_indices;
use leptos::{
    component, create_effect, create_rw_signal, create_write_slice, expect_context, view, IntoView,
    RwSignal, SignalGet, SignalUpdate, SignalWith,
};
use leptos_router::use_query_map;
use types::entities::QueryableArtist;
use types::songs::GetSongOptions;
use types::ui::song_details::{DefaultDetails, SongDetailIcons};
use wasm_bindgen_futures::spawn_local;

use crate::components::songview::SongView;
use crate::utils::db_utils::get_songs_by_option;
use rand::seq::SliceRandom;

#[tracing::instrument(level = "trace", skip())]
#[component()]
pub fn SingleArtist() -> impl IntoView {
    let params = use_query_map();
    let artist_id = params.with(|params| params.get("id").cloned()).unwrap();

    let artist = create_rw_signal(vec![]);
    let default_details = create_rw_signal(DefaultDetails::default());
    get_artists_by_option(
        QueryableArtist {
            artist_id: Some(artist_id.clone()),
            ..Default::default()
        },
        artist,
    );

    create_effect(move |_| {
        let binding = artist.get();
        let artist = binding.first();
        if let Some(artist) = artist {
            default_details.update(|d| {
                d.title = artist.artist_name.clone();
                d.icon = artist.artist_coverpath.clone();
            });
        }
    });

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

    let provider_store = expect_context::<Rc<ProviderStore>>();
    let selected_providers = create_rw_signal::<Vec<String>>(vec![]);
    create_effect(move |_| {
        let selected_providers = selected_providers.get();
        let artist = artist.get();
        let artist = artist.first();
        if artist.is_none() {
            return;
        }
        let artist = artist.unwrap();
        for provider in selected_providers {
            let provider_store = provider_store.clone();
            let artist = artist.clone();
            spawn_local(async move {
                fetch_infinite!(
                    provider_store,
                    provider,
                    get_artist_content,
                    songs,
                    artist.clone()
                );
            });
        }
    });

    view! {
        <SongView
            default_details=default_details
            songs=songs
            icons=icons
            selected_songs=selected_songs
            providers=ShowProvidersArgs {
                show_providers: true,
                selected_providers,
            }
        />
    }
}

#[tracing::instrument(level = "trace", skip())]
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
                        redirect_root="/main/artists"
                        card_item=move |(_, item)| {
                            let artist_name = item.artist_name.clone().unwrap_or_default();
                            let artist_coverpath = item.artist_coverpath.clone();
                            let artist_id = item.artist_id.clone().unwrap_or_default();
                            SimplifiedCardItem {
                                title: artist_name,
                                cover: artist_coverpath,
                                id: artist_id,
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
