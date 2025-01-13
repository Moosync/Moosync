use leptos::{component, prelude::*, view, IntoView};
use leptos_router::{hooks::use_navigate, NavigateOptions};
use types::entities::QueryableArtist;

use crate::store::ui_store::UiStore;

#[tracing::instrument(level = "trace", skip())]
#[component]
pub fn ArtistList(#[prop()] artists: Option<Vec<QueryableArtist>>) -> impl IntoView {
    let ui_store = expect_context::<RwSignal<UiStore>>();
    let is_mobile = create_read_slice(ui_store, |u| u.get_is_mobile()).get_untracked();

    let artists_list = artists.unwrap_or_default();
    artists_list
        .iter()
        .enumerate()
        .map(|(pos, a)| {
            let artist_name = a.artist_name.clone().unwrap();
            let artist = a.clone();
            view! {
                <div
                    class="col-auto d-flex mr-1"
                    on:click=move |_| {
                        if !is_mobile {
                            use_navigate()(
                                format!(
                                    "/main/artists/single?entity={}",
                                    url_escape::encode_component(
                                        &serde_json::to_string(&artist).unwrap(),
                                    ),
                                )
                                    .as_str(),
                                NavigateOptions::default(),
                            );
                        }
                    }
                >
                    <div class="text song-subtitle text-truncate" title=artist_name.clone()>
                        {artist_name.clone()}
                        {if pos == artists_list.len() - 1 { "" } else { "," }}
                    </div>
                </div>
            }
        })
        .collect_view()
}
