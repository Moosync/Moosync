use leptos::{component, create_rw_signal, view, IntoView, SignalWith};
use leptos_router::use_params_map;
use leptos_virtual_scroller::VirtualGridScroller;
use types::entities::{QueryableAlbum};
use types::songs::GetSongOptions;
use crate::components::cardview::{CardItem, SimplifiedCardItem};
use crate::components::songview::SongView;
use leptos_router::A;

use crate::utils::db_utils::{get_albums_by_option, get_songs_by_option};

#[component()]
pub fn SingleAlbum() -> impl IntoView {
    let params = use_params_map();
    let album_id = params.with(|params| params.get("id").cloned()).unwrap();

    let songs = create_rw_signal(vec![]);
    
    get_songs_by_option(
        GetSongOptions {
            album: Some(QueryableAlbum {
                album_id: Some(album_id),
                ..Default::default()
            }),
            ..Default::default()
        },
        songs,
    );

    view! { <SongView songs=songs/> }
}

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

                <div class="row no-gutters w-100 flex-grow-1" style="align-items: flex-start; height: 70%">
                    <VirtualGridScroller each=albums item_width=275 item_height=275 children=move|(_, item)| {
                        let album_id = item.album_id.clone();
                        let album_name = item.album_name.clone();
                        let album_coverpath = item.album_coverpath_high.clone();
                        view! {
                            <A href=album_id.unwrap_or_default()>
                            <CardItem item= SimplifiedCardItem { title: album_name.unwrap_or_default(), cover: album_coverpath } />
                            </A>
                        }
                    } />
                </div>
            </div>
        </div>
    }
}
