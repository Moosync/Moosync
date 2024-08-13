
use leptos::{component, create_rw_signal, view, IntoView, SignalGet, SignalSet};
use serde::Serialize;
use types::{
    extensions::PackageNameArgs,
    songs::{Song, SongType},
};
use wasm_bindgen_futures::spawn_local;

use crate::{
    icons::{spotify_icon::SpotifyIcon, youtube_icon::YoutubeIcon},
    utils::common::invoke,
};

#[component]
pub fn ProviderIcon(#[prop()] song: Song) -> impl IntoView {
    let provider_icon = create_rw_signal(String::new());
    spawn_local(async move {
        if let Some(extension) = song.song.provider_extension {
            #[derive(Serialize)]
            struct ExtensionIconArgs {
                args: PackageNameArgs,
            }
            let res = invoke(
                "get_extension_icon",
                serde_wasm_bindgen::to_value(&ExtensionIconArgs {
                    args: PackageNameArgs {
                        package_name: extension,
                    },
                })
                .unwrap(),
            )
            .await;

            provider_icon.set(res.unwrap().as_string().unwrap());
        }
    });
    view! {
        <div class="d-flex provider-icon">
            {move || {
                if song.song.type_ == SongType::YOUTUBE {
                    view! { <YoutubeIcon /> }
                } else if song.song.type_ == SongType::SPOTIFY {
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
