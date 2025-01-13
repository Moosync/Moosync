use leptos::{component, prelude::*, view, IntoView};

use crate::{
    icons::{
        animated_equalizer_icon::AnimatedEqualizerIcon, play_hover_icon::PlayHoverIcon,
        song_default_icon::SongDefaultIcon,
    },
    utils::common::convert_file_src,
};

#[tracing::instrument(
    level = "trace",
    skip(cover_img, show_play_button, show_eq, eq_playing, play_now)
)]
#[component]
pub fn LowImg<T, D, E>(
    #[prop()] cover_img: String,
    #[prop(default = true)] show_play_button: bool,
    #[prop()] show_eq: D,
    #[prop()] eq_playing: E,
    #[prop()] play_now: T,
) -> impl IntoView
where
    T: Fn() + 'static + Send + Sync,
    D: (Fn() -> bool) + 'static + Send + Sync,
    E: (Fn() -> bool) + 'static + Send + Sync,
{
    let show_default_cover_img = RwSignal::new(cover_img.is_empty());
    view! {
        <div class="col-auto img-container h-100 d-flex justify-content-start">
            <div class="img-container justify-content-around ms-auto coverimg align-self-center">
                {move || {
                    if !show_default_cover_img.get() {
                        view! {
                            <img
                                // class="fade-in-image"
                                src=convert_file_src(cover_img.clone())
                                on:error=move |_| { show_default_cover_img.set(true) }
                            />
                        }
                            .into_any()
                    } else {
                        view! {
                            // class="fade-in-image"
                            // class="fade-in-image"
                            // class="fade-in-image"
                            // class="fade-in-image"
                            // class="fade-in-image"
                            // class="fade-in-image"
                            // class="fade-in-image"
                            <SongDefaultIcon />
                        }
                            .into_any()
                    }
                }}
                {if show_play_button {
                    view! {
                        <div
                            class="play-button-song-list d-flex justify-content-center"
                            on:click=move |_| play_now()
                        >
                            <PlayHoverIcon />
                        </div>
                    }
                        .into_any()
                } else {
                    ().into_any()
                }}
                {move || {
                    if show_eq() {
                        view! {
                            <div class="equalizer-bg d-flex justify-content-center">
                                <AnimatedEqualizerIcon playing=eq_playing() />
                            </div>
                        }
                            .into_any()
                    } else {
                        ().into_any()
                    }
                }}

            </div>
        </div>
    }
}
