use crate::components::cardview::{CardItem, SimplifiedCardItem};
use crate::components::songview::SongView;
use crate::utils::db_utils::get_genres_by_option;
use crate::utils::db_utils::{get_albums_by_option, get_songs_by_option};
use leptos::{component, create_rw_signal, view, IntoView, SignalWith};
use leptos_router::use_params_map;
use leptos_router::A;
use leptos_virtual_scroller::VirtualGridScroller;
use types::entities::QueryableGenre;
use types::songs::GetSongOptions;

#[component()]
pub fn SingleGenre() -> impl IntoView {
    let params = use_params_map();
    let genre_id = params.with(|params| params.get("id").cloned()).unwrap();

    let songs = create_rw_signal(vec![]);

    get_songs_by_option(
        GetSongOptions {
            genre: Some(QueryableGenre {
                genre_id: Some(genre_id),
                ..Default::default()
            }),
            ..Default::default()
        },
        songs,
    );

    view! { <SongView songs=songs/> }
}

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

                <div
                    class="row no-gutters w-100 flex-grow-1"
                    style="align-items: flex-start; height: 70%"
                >
                    <VirtualGridScroller
                        each=genres
                        item_width=275
                        item_height=275
                        children=move |(_, item)| {
                            let genre_name = item.genre_name.clone().unwrap_or_default();
                            let genre_id = item.genre_id.clone().unwrap_or_default();
                            view! {
                                <A href=genre_id>
                                    <CardItem item=SimplifiedCardItem {
                                        title: genre_name,
                                        cover: None,
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
