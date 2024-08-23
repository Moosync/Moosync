use leptos::{component, create_rw_signal, view, IntoView, SignalGet, SignalSet};
use serde::Serialize;
use types::{
    extensions::PackageNameArgs,
    songs::{Song, SongType},
};
use wasm_bindgen_futures::spawn_local;

use crate::{
    console_log,
    icons::{spotify_icon::SpotifyIcon, youtube_icon::YoutubeIcon},
    utils::common::invoke,
};

#[component]
pub fn ProviderIcon(#[prop()] extension: String) -> impl IntoView {
    let provider_icon = create_rw_signal(String::new());
    let extension_clone = extension.clone();
    spawn_local(async move {
        if !extension_clone.is_empty()
            && extension_clone != "youtube"
            && extension_clone != "spotify"
        {
            #[derive(Serialize)]
            struct ExtensionIconArgs {
                args: PackageNameArgs,
            }
            let res = invoke(
                "get_extension_icon",
                serde_wasm_bindgen::to_value(&ExtensionIconArgs {
                    args: PackageNameArgs {
                        package_name: extension_clone,
                    },
                })
                .unwrap(),
            )
            .await;

            if let Ok(res) = res {
                provider_icon.set(res.as_string().unwrap());
            } else {
                console_log!("Failed to get provider icon {:?}", res);
            }
        }
    });
    view! {
        <div class="d-flex provider-icon">
            {move || {
                let extension = extension.as_str();
                if extension == "youtube" {
                    view! { <YoutubeIcon /> }
                } else if extension == "spotify" {
                    view! { <SpotifyIcon /> }
                } else {
                    view! {
                        <img
                            style:display=if provider_icon.get().is_empty() {
                                "none"
                            } else {
                                "block"
                            }
                            referrerPolicy="no-referrer"
                            src=move || provider_icon.get()
                        />
                    }
                        .into_view()
                }
            }}
        </div>
    }
}
