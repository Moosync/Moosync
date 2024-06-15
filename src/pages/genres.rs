use leptos::{component, create_rw_signal, view, IntoView};
use leptos_virtual_scroller::VirtualGridScroller;
use types::entities::{QueryableGenre};
use crate::components::cardview::{CardItem, SimplifiedCardItem};

use crate::utils::db_utils::{get_genres_by_option};

#[component()]
pub fn AllGenres() -> impl IntoView { 
    let genres = create_rw_signal(vec![]);
    get_genres_by_option(QueryableGenre::default(), genres.write_only());

    view! {
        <div class="w-100 h-100">
            <div class="container-fluid song-container h-100 d-flex flex-column">
                <div class="row page-title no-gutters">

                    <div class="col-auto">Albums</div>
                    <div class="col align-self-center"></div>
                </div>

                <div class="row no-gutters w-100 flex-grow-1" style="align-items: flex-start; height: 70%">
                    <VirtualGridScroller each=genres item_width=275 item_height=275 children=move|(_, item)| {
                        view! {
                            <CardItem item= SimplifiedCardItem { title: item.genre_name.clone().unwrap_or_default(), cover: None } />
                        }
                    } />
                </div>
            </div>
        </div>
    }
}
