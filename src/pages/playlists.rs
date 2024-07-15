use std::rc::Rc;

use crate::components::cardview::{CardItem, SimplifiedCardItem};
use crate::components::songview::SongView;
use crate::console_log;
use crate::utils::common::fetch_infinite;
use crate::utils::db_utils::get_songs_by_option;
use leptos::{
    component, create_rw_signal, expect_context, spawn_local, use_context, view, IntoView,
    SignalUpdate, SignalWith,
};
use leptos_router::{use_params_map, A};
use leptos_virtual_scroller::VirtualGridScroller;
use types::entities::QueryablePlaylist;
use types::songs::GetSongOptions;

use crate::store::provider_store::ProviderStore;
use crate::{icons::plus_button::PlusIcon, utils::db_utils::get_playlists_by_option};

#[component()]
pub fn SinglePlaylist() -> impl IntoView {
    let params = use_params_map();
    let playlist_id = params.with(|params| params.get("id").cloned()).unwrap();

    let songs = create_rw_signal(vec![]);

    let provider_store = use_context::<Rc<ProviderStore>>().unwrap();

    let playlist_id_tmp = playlist_id.clone();
    spawn_local(async move {
        let provider = provider_store
            .get_provider_key_by_id(playlist_id_tmp.clone())
            .await;
        if let Some(provider) = provider {
            let playlist_id = playlist_id_tmp.clone();
            fetch_infinite!(
                provider_store,
                provider,
                get_playlist_content,
                songs,
                playlist_id.clone()
            );
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

    view! { <SongView songs=songs/> }
}

#[component()]
pub fn AllPlaylists() -> impl IntoView {
    let playlists = create_rw_signal(vec![]);
    get_playlists_by_option(QueryablePlaylist::default(), playlists.write_only());

    let provider_store = expect_context::<Rc<ProviderStore>>();
    spawn_local(async move {
        for key in provider_store.get_provider_keys() {
            let playlist_write_signal = playlists.write_only();
            fetch_infinite!(
                provider_store,
                key,
                fetch_user_playlists,
                playlist_write_signal,
            );
        }
    });

    view! {
        <div class="w-100 h-100">
            <div class="container-fluid song-container h-100 d-flex flex-column">
                <div class="row page-title no-gutters">

                    <div class="col-auto">Playlists</div>
                    <div class="col-auto button-grow playlists-plus-icon">
                        <PlusIcon/>
                    </div>

                    <div class="col align-self-center"></div>
                </div>

                <div
                    class="row no-gutters w-100 flex-grow-1"
                    style="align-items: flex-start; height: 70%"
                >
                    <VirtualGridScroller
                        each=playlists
                        item_width=275
                        item_height=275
                        children=move |(_, item)| {
                            let playlist_name = item.playlist_name.clone();
                            let playlist_coverpath = item.playlist_coverpath.clone();
                            let playlist_id = item.playlist_id.clone().unwrap_or_default();
                            view! {
                                <A href=playlist_id>
                                    <CardItem item=SimplifiedCardItem {
                                        title: playlist_name,
                                        cover: playlist_coverpath,
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
