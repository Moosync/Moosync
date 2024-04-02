use leptos::*;
use leptos::{component, view, IntoView, RwSignal, SignalSet};
use types::entities::QueryableArtist;
use types::ui::player_details::PlayerState;

use crate::components::audiostream::AudioStream;
use crate::components::low_img::LowImg;
use crate::components::musicinfo::MusicInfo;
use crate::icons::expand_icon::ExpandIcon;
use crate::icons::fav_icon::FavIcon;
use crate::icons::next_track_icon::NextTrackIcon;
use crate::icons::play_icon::PlayIcon;
use crate::icons::prev_track_icon::PrevTrackIcon;
use crate::icons::repeat_icon::RepeatIcon;
use crate::icons::shuffle_icon::ShuffleIcon;
use crate::icons::volume_icon::VolumeIcon;
use crate::store::player_store::PlayerStore;
use crate::utils::common::{format_duration, get_low_img};

#[component]
fn Details() -> impl IntoView {
    let player_store = use_context::<RwSignal<PlayerStore>>().unwrap();

    let current_song = create_read_slice(player_store, |player_store| {
        player_store.current_song.clone()
    });

    let title = create_rw_signal("-".to_string());
    let artists_list = create_rw_signal::<Vec<QueryableArtist>>(vec![]);
    let cover_img = create_rw_signal("".to_string());

    create_effect(move |_| {
        let current_song = current_song.get().clone();
        if let Some(current_song) = &current_song {
            title.set(current_song.song.title.clone().unwrap());
            cover_img.set(get_low_img(current_song));

            if let Some(artists) = &current_song.artists {
                artists_list.set(artists.clone())
            }
            return;
        }
        title.set("-".into())
    });

    view! {
        <div class="row no-gutters align-items-center w-100">
            <div class="col-auto mr-3">

                {move || {
                    let cover_img = cover_img.get();
                    view! { <LowImg cover_img=cover_img show_play_button=false play_now=|| {}/> }
                }}

            </div>
            <div class="col text-truncate">
                <div class="row align-items-center justify-content-start">
                    <div class="col-auto w-100 d-flex">
                        <div title=move || title.get() class="text song-title text-truncate mr-2">
                            {move || title.get()}
                        </div>
                    </div>
                </div>

                <div class="row no-gutters">

                    {move || {
                        artists_list
                            .get()
                            .iter()
                            .map(|a| {
                                let artist_name = a.artist_name.clone().unwrap();
                                view! {
                                    <div class="col d-flex">
                                        <div
                                            class="text song-subtitle text-truncate"
                                            title=artist_name.clone()
                                        >
                                            {artist_name}
                                        </div>
                                    </div>
                                }
                            })
                            .collect_view()
                    }}

                </div>
            </div>
        </div>
    }
}

#[component]
pub fn Controls() -> impl IntoView {
    let prev_track_dis = create_rw_signal(true);
    let next_track_dis = create_rw_signal(true);
    let is_play = create_rw_signal(true);
    let is_fav = create_rw_signal(false);
    let is_repeat = create_rw_signal(false);
    let is_shuffle = create_rw_signal(false);
    let current_time_sig = create_rw_signal("".to_string());
    let total_duration_sig = create_rw_signal("".to_string());

    let player_store = use_context::<RwSignal<PlayerStore>>().unwrap();
    let current_song = create_read_slice(player_store, |player_store| {
        player_store.current_song.clone()
    });
    let current_time = create_read_slice(player_store, |player_store| {
        player_store.player_details.current_time
    });
    let (current_state, set_current_state) = create_slice(
        player_store,
        |player| player.player_details.state,
        |player, n| player.set_state(n),
    );

    create_effect(move |_| {
        let current_song = current_song.get();
        if let Some(current_song) = current_song {
            let fmt_dur = format_duration(current_song.song.duration.unwrap_or(-1f64));
            total_duration_sig.set(fmt_dur)
        } else {
            total_duration_sig.set("00:00".to_string())
        }
    });

    create_effect(move |_| {
        let state = current_state.get();
        match state {
            types::ui::player_details::PlayerState::Playing => is_play.set(true),
            types::ui::player_details::PlayerState::Paused => is_play.set(false),
            types::ui::player_details::PlayerState::Stopped => is_play.set(false),
            types::ui::player_details::PlayerState::Loading => is_play.set(false),
        }
    });

    create_effect(move |_| {
        let current_time = current_time.get();
        let fmt_dur = format_duration(current_time);
        current_time_sig.set(fmt_dur)
    });

    view! {
        <div class="row no-gutters align-items-center justify-content-center">
            <div class="col col-button">
                <PrevTrackIcon disabled=prev_track_dis.read_only()/>
            </div>
            <div class="col col-button">

                // TODO: Add repeat once icon
                <RepeatIcon filled=is_repeat.read_only()/>
            </div>
            <div class="col col-play-button">
                <PlayIcon
                    play=is_play.read_only()
                    on:click=move |_| {
                        let is_playing = is_play.read_only().get();
                        if is_playing {
                            set_current_state.set(PlayerState::Paused)
                        } else {
                            set_current_state.set(PlayerState::Playing)
                        }
                    }
                />

            </div>
            <div class="col col-button">
                <NextTrackIcon disabled=next_track_dis.read_only()/>
            </div>
            <div class="col col-button">
                <ShuffleIcon filled=is_shuffle.read_only()/>
            </div>
            <div class="col col-button mr-1">
                <FavIcon filled=is_fav.read_only()/>
            </div>
            <div class="col-md-3 col-5 align-self-center timestamp-container">
                <div class="row no-gutters align-items-center timestamp">
                    <div class="col-auto timestamp">{move || current_time_sig.get()}</div>
                    <div class="col-auto timestamp timestamp-extra ml-1">
                        / {move || total_duration_sig.get()}
                    </div>
                </div>
            </div>
        </div>
    }
}

#[component]
pub fn ExtraControls<T>(musicinfo_cb: T) -> impl IntoView
where
    T: Fn() + 'static,
{
    let is_cut = create_rw_signal(false);

    let player_store = use_context::<RwSignal<PlayerStore>>().unwrap();
    let (current_volume, set_current_volume) = create_slice(
        player_store,
        |player_store| player_store.player_details.volume,
        |player_store, volume| player_store.set_volume(volume),
    );
    view! {
        <div class="row no-gutters align-items-center justify-content-end">
            <div class="col-auto volume-slider-container d-flex">
                <input
                    type="range"
                    min="0"
                    max="100"
                    class="volume-slider w-100 align-self-center"
                    prop:value=move || current_volume.get()
                    on:input=move |ev| {
                        set_current_volume.set(event_target_value(&ev).parse().unwrap())
                    }

                    id="myRange"
                    aria-label="volume"
                    style=move || {
                        format!(
                            "background: linear-gradient(90deg, var(--accent) 0%, var(--accent) {}%, var(--textSecondary) 0%);",
                            current_volume.get(),
                        )
                    }
                />

            </div>
            <div class="col-auto">
                <VolumeIcon cut=is_cut.read_only()/>
            </div>
            <div class="col-auto expand-icon ml-3">
                <ExpandIcon on:click=move |_| musicinfo_cb()/>
            </div>
        </div>
    }
}

#[component]
pub fn Slider() -> impl IntoView {
    let player_store = use_context::<RwSignal<PlayerStore>>().unwrap();
    let current_time = create_read_slice(player_store, |p| p.player_details.current_time);
    let current_song = create_read_slice(player_store, |p| p.current_song.clone());
    let total_time = create_rw_signal(1f64);

    create_effect(move |_| {
        let current_song = current_song.get();
        if let Some(current_song) = current_song {
            if let Some(duration) = current_song.song.duration {
                total_time.set(duration);
            }
        }
    });
    view! {
        <div class="timeline pl-2 pr-2">
            <div
                class="time-slider time-slider-ltr timeline pl-2 pr-2 timeline pl-2 pr-2"
                style="padding: 5px 0px; width: auto; height: 4px;"
            >
                <div class="time-slider-rail">
                    <div
                        class="time-slider-process"
                        style=move || {
                            format!(
                                "height: 100%; top: 0px; left: 0%; width: {}%; transition-property: width, left; transition-duration: 0.1s;",
                                (current_time.get() / total_time.get()) * 100f64,
                            )
                        }
                    >
                    </div>
                    <div
                        class="time-slider-dot"
                        role="slider"
                        tabindex="0"
                        style=move || {
                            format!(
                                "width: 10px; height: 10px; transform: translate(-50%, -50%); top: 50%; left: {}%; transition: left 0.1s ease 0s;",
                                (current_time.get() / total_time.get()) * 100f64,
                            )
                        }
                    >

                        <div class="time-slider-dot-handle"></div>
                    </div>
                </div>
            </div>
        </div>
    }
}

#[component]
pub fn MusicBar() -> impl IntoView {
    let show_musicinfo = create_rw_signal(true);
    view! {
        <div class="musicbar-content d-flex">
            <MusicInfo show=show_musicinfo/>
            <div class="background w-100">
                <div class="musicbar h-100">
                    <AudioStream/>
                    <Slider/>
                    <div class="container-fluid d-flex bar-container h-100 pb-2">
                        <div class="row no-gutters align-items-center justify-content-center align-content-center no-gutters w-100 control-row justify-content-between">
                            <div class="col-4 no-gutters details-col w-100">
                                <Details/>
                            </div>

                            <div class="col align-self-center no-gutters controls-col">
                                <Controls/>
                            </div>
                            <div class="col-lg-auto col-1 align-self-center no-gutters extra-col">
                                <ExtraControls musicinfo_cb=move || {
                                    show_musicinfo.set(!show_musicinfo.get())
                                }/>
                            </div>
                        </div>
                    </div>
                </div>
            </div>

        </div>
    }
}
