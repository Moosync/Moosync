use leptos::{component, create_rw_signal, view, IntoView};
use leptos_virtual_scroller::VirtualGridScroller;
use types::entities::{QueryableArtist};
use crate::components::cardview::{CardItem, SimplifiedCardItem};

use crate::utils::db_utils::get_artists_by_option;

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

                <div class="row no-gutters w-100 flex-grow-1" style="align-items: flex-start; height: 70%">
                    <VirtualGridScroller each=artists item_width=275 item_height=275 children=move|(_, item)| {
                        view! {
                            <CardItem item= SimplifiedCardItem { title: item.artist_name.clone().unwrap_or_default(), cover: item.artist_coverpath.clone() } />
                        }
                    } />
                </div>
            </div>
        </div>
    }
}
