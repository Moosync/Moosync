use crate::components::cardview::{CardView, SimplifiedCardItem};
use crate::components::songview::SongView;
use crate::utils::db_utils::get_genres_by_option;
use crate::utils::db_utils::get_songs_by_option;
use leptos::{component, create_rw_signal, view, IntoView, SignalWith};
use leptos_router::use_query_map;
use types::entities::QueryableGenre;
use types::songs::GetSongOptions;

#[component()]
pub fn SingleGenre() -> impl IntoView {
    let params = use_query_map();
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

    view! { <SongView songs=songs /> }
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
                    <CardView
                        items=genres
                        card_item=move |(_, item)| {
                            let genre_name = item.genre_name.clone().unwrap_or_default();
                            let genre_id = item.genre_id.clone().unwrap_or_default();
                            SimplifiedCardItem {
                                title: genre_name,
                                cover: None,
                                id: genre_id,
                            }
                        }
                    />
                </div>
            </div>
        </div>
    }
}
