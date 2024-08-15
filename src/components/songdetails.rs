use leptos::{
    component, create_effect, create_rw_signal, view, CollectView, IntoView, RwSignal, SignalGet,
    SignalSet,
};
use types::{songs::Song, ui::song_details::SongDetailIcons};

use crate::{
    icons::{
        add_to_library_icon::AddToLibraryIcon, add_to_queue_icon::AddToQueueIcon,
        plain_play_icon::PlainPlayIcon, random_icon::RandomIcon,
        song_default_icon::SongDefaultIcon,
    },
    utils::common::{format_duration, get_high_img},
};

#[component()]
pub fn SongDetails<T>(
    #[prop()] selected_song: T,
    #[prop()] icons: RwSignal<SongDetailIcons>,
) -> impl IntoView
where
    T: SignalGet<Value = Option<Song>> + 'static,
{
    let selected_title = create_rw_signal(None::<String>);
    let selected_artists = create_rw_signal(None::<String>);
    let selected_duration = create_rw_signal(None::<String>);
    let selected_cover_path = create_rw_signal("".to_string());

    let show_default_cover_img = create_rw_signal(true);

    create_effect(move |_| {
        let selected_song = selected_song.get();

        selected_title.set(
            selected_song
                .clone()
                .map(|s| s.song.clone().title.unwrap_or_default()),
        );
        selected_artists.set(Some(
            selected_song
                .as_ref()
                .map(|s| s.clone().artists.unwrap_or_default())
                .unwrap_or_default()
                .iter()
                .map(|a| a.artist_name.clone().unwrap_or_default())
                .collect::<Vec<String>>()
                .join(", "),
        ));

        selected_duration.set(
            selected_song
                .clone()
                .map(|s| format_duration(s.song.duration.unwrap_or(-1f64))),
        );

        if let Some(selected_song) = selected_song {
            selected_cover_path.set(get_high_img(&selected_song));
            show_default_cover_img.set(false);
        } else {
            selected_cover_path.set("".to_string());
            show_default_cover_img.set(true);
        }
    });

    view! {
        <div class="container-fluid scrollable" style="max-height: 100%;">
            <div class="row no-gutters">
                <div class="col position-relative">

                    <div class="image-container w-100">
                        <div class="embed-responsive embed-responsive-1by1">
                            <div class="embed-responsive-item albumart">

                                {move || {
                                    let cover_path = selected_cover_path.get();
                                    if !show_default_cover_img.get() {
                                        view! {
                                            <img
                                                src=cover_path
                                                on:error=move |_| { show_default_cover_img.set(true) }
                                            />
                                        }
                                            .into_view()
                                    } else {
                                        view! {
                                            <SongDefaultIcon class="fade-in-image".to_string() />
                                        }
                                            .into_view()
                                    }
                                }}

                            </div>
                        </div>
                    </div>

                    <div class="song-info-container">
                        <div class="row d-flex">
                            <div class="col song-title-details text-truncate">
                                {move || selected_title.get()}

                            </div>
                        </div>

                        <div class="song-subtitle-details text-truncate">
                            {move || selected_artists.get()}

                        </div>

                        <div class="song-timestamp-details">

                            {move || selected_duration.get()}

                        </div>
                    </div>
                </div>
            </div>

            <div class="row no-gutters flex-fill mt-2">
                <div class="col">
                    <div class="button-group d-flex">

                        {move || {
                            let title = selected_title.get();
                            let icons = icons.get();
                            let mut icons_ret = vec![];
                            if let Some(play_cb) = icons.play {
                                icons_ret
                                    .push(
                                        view! {
                                            <PlainPlayIcon
                                                title=title.clone().unwrap_or_default()
                                                on:click=move |_| play_cb()
                                            />
                                        },
                                    );
                            }
                            if let Some(add_to_queue_cb) = icons.add_to_queue {
                                icons_ret
                                    .push(
                                        view! {
                                            <AddToQueueIcon
                                                title=title.clone().unwrap_or_default()
                                                on:click=move |_| add_to_queue_cb()
                                            />
                                        },
                                    );
                            }
                            if let Some(add_to_library_cb) = icons.add_to_library {
                                icons_ret
                                    .push(
                                        view! {
                                            <AddToLibraryIcon
                                                title=title.unwrap_or_default()
                                                on:click=move |_| add_to_library_cb()
                                            />
                                        },
                                    );
                            }
                            if let Some(random_cb) = icons.random {
                                icons_ret
                                    .push(view! { <RandomIcon on:click=move |_| random_cb() /> });
                            }
                            icons_ret.collect_view()
                        }}
                    </div>
                </div>
            </div>

        </div>
    }
}
