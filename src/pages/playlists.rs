use std::rc::Rc;

use leptos::{component, create_rw_signal, expect_context, spawn_local, view, IntoView, SignalUpdate};
use leptos_virtual_scroller::VirtualGridScroller;
use types::entities::QueryablePlaylist;
use crate::components::cardview::{CardItem, SimplifiedCardItem};

use crate::store::provider_store::ProviderStore;
use crate::{icons::plus_button::PlusIcon, utils::db_utils::get_playlists_by_option};

#[component()]
pub fn AllPlaylists() -> impl IntoView {
    
    let playlists = create_rw_signal(vec![]);
    get_playlists_by_option(QueryablePlaylist::default(), playlists.write_only());

    let provider_store = expect_context::<Rc<ProviderStore>>();
    for key in provider_store.get_provider_keys() {
        let playlistWriteSignal = playlists.write_only();
        let provider = provider_store.get_provider_by_key(key).unwrap().clone();
        spawn_local(async move {
            let provider = provider.lock().unwrap();

            let mut offset = 0;
            loop {
                let res = provider.fetch_user_playlists(50, offset).await;
                if res.is_err() {
                    break;
                }

                let mut res = res.unwrap();
                let len = res.len() as u32;

                if len == 0 {
                    break;
                }

                offset += len;

                playlistWriteSignal.update(|playlists| {
                    playlists.append(&mut res);
                });
            }
        });
    }

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

                <div class="row no-gutters w-100 flex-grow-1" style="align-items: flex-start; height: 70%">
                    <VirtualGridScroller each=playlists item_width=275 item_height=275 children=move|(_, item)| {
                        view! {
                            <CardItem item= SimplifiedCardItem { title: item.playlist_name.clone(), cover: item.playlist_coverpath.clone() } />
                        }
                    } />
                </div>
            </div>
        </div>
    }
}
