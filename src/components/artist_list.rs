use leptos::{component, view, CollectView, IntoView};
use leptos_router::{use_navigate, NavigateOptions};
use types::entities::QueryableArtist;

#[tracing::instrument(level = "trace", skip())]
#[component]
pub fn ArtistList(#[prop()] artists: Option<Vec<QueryableArtist>>) -> impl IntoView {
    let artists_list = artists.unwrap_or_default();
    artists_list
        .iter()
        .map(|a| {
            let artist_name = a.artist_name.clone().unwrap();
            let artist = a.clone();
            view! {
                <div
                    class="col-auto d-flex mr-1"
                    on:click=move |_| {
                        use_navigate()(
                            format!(
                                "/main/artists/single?entity={}",
                                serde_json::to_string(&artist).unwrap(),
                            )
                                .as_str(),
                            NavigateOptions::default(),
                        );
                    }
                >
                    <div class="text song-subtitle text-truncate" title=artist_name.clone()>
                        {artist_name}
                        ,
                    </div>
                </div>
            }
        })
        .collect_view()
}
