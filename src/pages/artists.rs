use crate::components::cardview::{CardItem, SimplifiedCardItem};
use crate::utils::db_utils::get_artists_by_option;
use leptos::{component, create_rw_signal, view, IntoView, SignalWith};
use leptos_router::use_params_map;
use leptos_router::A;
use leptos_virtual_scroller::VirtualGridScroller;
use types::entities::QueryableArtist;
use types::songs::GetSongOptions;

use crate::components::songview::SongView;
use crate::utils::db_utils::get_songs_by_option;

#[component()]
pub fn SingleArtist() -> impl IntoView {
    let params = use_params_map();
    let artist_id = params.with(|params| params.get("id").cloned()).unwrap();

    let songs = create_rw_signal(vec![]);

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

    view! { <SongView songs=songs/> }
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
                    <VirtualGridScroller
                        each=artists
                        item_width=275
                        item_height=275
                        children=move |(_, item)| {
                            let artist_name = item.artist_name.clone().unwrap_or_default();
                            let artist_coverpath = item.artist_coverpath.clone();
                            let artist_id = item.artist_id.clone().unwrap_or_default();
                            view! {
                                <A href=artist_id>
                                    <CardItem item=SimplifiedCardItem {
                                        title: artist_name,
                                        cover: artist_coverpath,
                                    }/>
                                </A>
                            }
                        }
                    />
                </div>
            </div>
        </div>
    }
}
