use std::collections::HashMap;
use std::rc::Rc;

use crate::components::cardview::{CardView, SimplifiedCardItem};
use crate::components::songlist::ShowProvidersArgs;
use crate::components::songview::SongView;
use crate::dyn_provider_songs;
use crate::store::player_store::PlayerStore;
use crate::store::provider_store::ProviderStore;
use crate::utils::common::{convert_file_src, fetch_infinite};
use crate::utils::songs::get_songs_from_indices;
use leptos::{
    component, create_effect, create_memo, create_rw_signal, create_write_slice, expect_context,
    view, IntoView, RwSignal, SignalGet, SignalUpdate, SignalWith,
};
use leptos_router::use_query_map;
use rand::seq::SliceRandom;
use types::entities::QueryableAlbum;
use types::songs::{GetSongOptions, Song};
use types::ui::song_details::{DefaultDetails, SongDetailIcons};
use wasm_bindgen_futures::spawn_local;

use crate::utils::db_utils::{get_albums_by_option, get_songs_by_option};

#[tracing::instrument(level = "trace", skip())]
#[component()]
pub fn SingleAlbum() -> impl IntoView {
    let params = use_query_map();
    let album = create_memo(move |_| {
        params.with(|params| {
            let entity = params.get("entity");
            if let Some(entity) = entity {
                let album = serde_json::from_str::<QueryableAlbum>(entity);
                if let Ok(album) = album {
                    return Some(album);
                }
            }
            None
        })
    });
    if album.get().is_none() {
        tracing::error!("Failed to parse album");
        return view! {}.into_view();
    }

    let songs = create_rw_signal(vec![]);
    let selected_songs = create_rw_signal(vec![]);

    let default_details = create_rw_signal(DefaultDetails::default());

    create_effect(move |_| {
        let album = album.get();
        if let Some(album) = album {
            default_details.update(|d| {
                d.title = album.album_name.clone();
                d.icon = album.album_coverpath_high.clone().map(convert_file_src);
            });

            get_songs_by_option(
                GetSongOptions {
                    album: Some(QueryableAlbum {
                        album_id: album.album_id,
                        ..Default::default()
                    }),
                    ..Default::default()
                },
                songs,
            );
        }
    });

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

    let icons = create_rw_signal(SongDetailIcons {
        play: Some(Rc::new(Box::new(play_songs))),
        add_to_queue: Some(Rc::new(Box::new(add_to_queue))),
        random: Some(Rc::new(Box::new(random))),
        ..Default::default()
    });

    let provider_store = expect_context::<Rc<ProviderStore>>();
    let selected_providers = create_rw_signal::<Vec<String>>(vec![]);

    let (_, filtered_songs, fetch_selected_providers) = dyn_provider_songs!(
        selected_providers,
        album,
        provider_store,
        songs,
        get_album_content
    );

    let refresh_songs = move || {};
    let fetch_next_page = move || {
        fetch_selected_providers.as_ref()();
    };

    view! {
        <SongView
            default_details=default_details
            songs=filtered_songs
            icons=icons
            selected_songs=selected_songs
            providers=ShowProvidersArgs {
                show_providers: true,
                selected_providers,
            }
            refresh_cb=refresh_songs
            fetch_next_page=fetch_next_page
        />
    }
}

#[tracing::instrument(level = "trace", skip())]
#[component()]
pub fn AllAlbums() -> impl IntoView {
    let albums = create_rw_signal(vec![]);
    get_albums_by_option(QueryableAlbum::default(), albums.write_only());

    view! {
        <div class="w-100 h-100">
            <div class="container-fluid song-container h-100 d-flex flex-column">
                <div class="row page-title no-gutters">

                    <div class="col-auto">Albums</div>
                    <div class="col align-self-center"></div>
                </div>

                <div
                    class="row no-gutters w-100 flex-grow-1"
                    style="align-items: flex-start; height: 70%"
                >

                    <CardView
                        items=albums
                        redirect_root="/main/albums"
                        card_item=move |(_, item)| {
                            let album_name = item.album_name.clone().unwrap_or_default();
                            let album_coverpath = item.album_coverpath_high.clone();
                            SimplifiedCardItem {
                                title: album_name,
                                cover: album_coverpath,
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
