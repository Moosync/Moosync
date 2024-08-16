use rand::seq::SliceRandom;
use std::rc::Rc;

use crate::components::cardview::{CardView, SimplifiedCardItem};
use crate::components::songview::SongView;
use crate::console_log;
use crate::store::modal_store::{ModalStore, Modals};
use crate::store::player_store::PlayerStore;
use crate::utils::common::fetch_infinite;
use crate::utils::db_utils::get_songs_by_option;
use leptos::{
    component, create_rw_signal, create_write_slice, expect_context, spawn_local, use_context,
    view, IntoView, RwSignal, SignalGet, SignalUpdate, SignalWith,
};
use leptos_router::use_query_map;
use types::entities::QueryablePlaylist;
use types::songs::{GetSongOptions, Song};
use types::ui::song_details::SongDetailIcons;

use crate::store::provider_store::ProviderStore;
use crate::{icons::plus_button::PlusIcon, utils::db_utils::get_playlists_by_option};

#[component()]
pub fn SinglePlaylist() -> impl IntoView {
    let params = use_query_map();
    let playlist_id = params.with(|params| params.get("id").cloned()).unwrap();
    console_log!("In single playlists {:?}", playlist_id);

    let songs = create_rw_signal(vec![]);
    let selected_songs = create_rw_signal(vec![]);

    let provider_store = use_context::<Rc<ProviderStore>>().unwrap();

    let playlist_id_tmp = playlist_id.clone();
    spawn_local(async move {
        let provider = provider_store
            .get_provider_key_by_id(playlist_id_tmp.clone())
            .await;
        match provider {
            Ok(provider) => {
                let playlist_id = playlist_id_tmp.clone();
                fetch_infinite!(
                    provider_store,
                    provider,
                    fetch_playlist_content,
                    songs,
                    playlist_id.clone()
                );
            }
            Err(e) => console_log!("{}", e),
        }
    });

    get_songs_by_option(
        GetSongOptions {
            playlist: Some(QueryablePlaylist {
                playlist_id: Some(playlist_id),
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
pub fn AllPlaylists() -> impl IntoView {
    let playlists = create_rw_signal(vec![]);
    get_playlists_by_option(QueryablePlaylist::default(), playlists.write_only());

    let modal_manager = expect_context::<RwSignal<ModalStore>>();
    let open_new_playlist_modal = move |_| {
        modal_manager.update(|m| {
            m.set_active_modal(Modals::NewPlaylistModal);
            m.on_modal_close(move || {
                get_playlists_by_option(QueryablePlaylist::default(), playlists.write_only());
            });
        });
    };

    view! {
        <div class="w-100 h-100">
            <div class="container-fluid song-container h-100 d-flex flex-column">
                <div class="row page-title no-gutters">

                    <div class="col-auto">Playlists</div>
                    <div
                        class="col-auto button-grow playlists-plus-icon"
                        on:click=open_new_playlist_modal
                    >
                        <PlusIcon />
                    </div>

                    <div class="col align-self-center"></div>
                </div>

                <div
                    class="row no-gutters w-100 flex-grow-1"
                    style="align-items: flex-start; height: 70%"
                >
                    <CardView
                        items=playlists
                        card_item=move |(_, item)| {
                            let playlist_name = item.playlist_name.clone();
                            let playlist_coverpath = item.playlist_coverpath.clone();
                            let playlist_id = item.playlist_id.clone().unwrap_or_default();
                            let playlist_extension = item.extension.clone();
                            SimplifiedCardItem {
                                title: playlist_name,
                                cover: playlist_coverpath,
                                id: playlist_id,
                                icon: playlist_extension,
                            }
                        }
                    />
                </div>
            </div>
        </div>
    }
}
