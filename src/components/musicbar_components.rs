// Moosync
// Copyright (C) 2024, 2025  Moosync <support@moosync.app>
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <http://www.gnu.org/licenses/>.

use std::time::Duration;

use ev::{mousemove, mouseup};
use leptos::ev::{pointermove, touchend, touchmove};
use leptos::*;
use leptos::{component, prelude::*, view, IntoView};
use leptos_dom::helpers::TimeoutHandle;
use leptos_use::{use_document, use_event_listener};
use task::spawn_local;
use types::entities::{QueryableArtist, QueryablePlaylist};
use types::ui::player_details::PlayerState;

use crate::components::artist_list::ArtistList;
use crate::components::low_img::LowImg;
use crate::icons::expand_icon::ExpandIcon;
use crate::icons::fav_icon::FavIcon;
use crate::icons::next_track_icon::NextTrackIcon;
use crate::icons::play_icon::PlayIcon;
use crate::icons::prev_track_icon::PrevTrackIcon;
use crate::icons::repeat_icon::RepeatIcon;
use crate::icons::shuffle_icon::ShuffleIcon;
use crate::icons::volume_icon::VolumeIcon;
use crate::store::player_store::PlayerStore;
use crate::store::ui_store::UiStore;
use crate::utils::common::{format_duration, get_low_img};

#[tracing::instrument(level = "debug", skip())]
#[component]
pub fn Details() -> impl IntoView {
    let player_store = use_context::<RwSignal<PlayerStore>>().unwrap();

    let current_song =
        create_read_slice(player_store, |player_store| player_store.get_current_song());

    let title = RwSignal::new("-".to_string());
    let artists_list = RwSignal::<Vec<QueryableArtist>>::new(vec![]);
    let cover_img = RwSignal::new("".to_string());

    Effect::new(move || {
        let current_song = current_song.get().clone();
        if let Some(current_song) = &current_song {
            title.set(current_song.song.title.clone().unwrap());
            cover_img.set(get_low_img(current_song));

            if let Some(artists) = &current_song.artists {
                artists_list.set(artists.clone())
            }
            return;
        }
        title.set("-".into());
        artists_list.set(vec![]);
        cover_img.set("".to_string());
    });

    view! {
        <div class="row no-gutters align-items-center w-100">
            <div class="col-auto mr-3">

                {move || {
                    let cover_img = cover_img.get();
                    view! {
                        <LowImg
                            show_eq=|| false
                            eq_playing=|| false
                            cover_img=cover_img
                            show_play_button=false
                            play_now=|| {}
                        />
                    }
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

                <div class="row no-gutters w-100 flex-nowrap text-truncate">
                    {move || {
                        let artists = artists_list.get();
                        view! { <ArtistList artists=Some(artists) /> }
                    }}

                </div>
            </div>
        </div>
    }
}

#[tracing::instrument(level = "debug", skip())]
#[component]
pub fn Controls(
    #[prop(optional, default = true)] show_time: bool,
    #[prop(optional, default = true)] show_fav: bool,
) -> impl IntoView {
    let player_store = use_context::<RwSignal<PlayerStore>>().unwrap();

    let current_song = create_read_slice(player_store, |p| p.get_current_song());

    let prev_track_dis = create_read_slice(player_store, |p| p.get_queue_len() <= 1);
    let next_track_dis = create_read_slice(player_store, |p| p.get_queue_len() <= 1);
    let is_play = create_read_slice(player_store, |p| {
        p.get_player_state() == PlayerState::Playing
    });
    let is_fav = RwSignal::new(false);
    let (repeat_mode, toggle_repeat) =
        create_slice(player_store, |p| p.get_repeat(), |p, _| p.toggle_repeat());
    let is_shuffle = RwSignal::new(true);
    let shuffle_queue = create_write_slice(player_store, |p, _| p.shuffle_queue());

    let (current_time_sig, total_duration_sig) = if show_time {
        let current_time_sig = create_read_slice(player_store, |p| {
            format_duration(p.get_current_time(), false)
        });
        let total_duration_sig = create_read_slice(player_store, |p| {
            if let Some(current_song) = p.get_current_song() {
                format_duration(current_song.song.duration.unwrap_or(-1f64), false)
            } else {
                "00:00".to_string()
            }
        });

        (current_time_sig, total_duration_sig)
    } else {
        (Default::default(), Default::default())
    };

    let set_current_state = create_write_slice(player_store, |player, n| player.set_state(n));
    let next_song_setter = create_write_slice(player_store, |p, _| p.next_song());
    let prev_song_setter = create_write_slice(player_store, |p, _| p.prev_song());

    Effect::new(move || {
        let current_song = current_song.get();
        if let Some(current_song) = current_song {
            spawn_local(async move {
                tracing::debug!("Checking song in favorites");
                let res = crate::utils::invoke::is_song_in_playlist(
                    "favorite".into(),
                    current_song.song._id.unwrap_or_default(),
                )
                .await;
                match res {
                    Ok(res) => {
                        tracing::debug!("song in favorites: {}", res);
                        is_fav.set(res);
                    }
                    Err(e) => {
                        tracing::error!("Failed to check song in favs: {:?}", e);
                        is_fav.set(false);
                    }
                }
            });
        } else {
            is_fav.set(false);
        }
    });

    let add_to_fav = move |_| {
        let current_song = current_song.get();
        let is_fav_val = is_fav.get();
        if let Some(current_song) = current_song {
            spawn_local(async move {
                // Don't care if favorites playlist already exists
                let _ = crate::utils::invoke::create_playlist(QueryablePlaylist {
                    playlist_id: Some("favorite".into()),
                    playlist_name: "Favorites".into(),
                    playlist_coverpath: Some("favorites".into()),
                    ..Default::default()
                })
                .await;

                let res = if !is_fav_val {
                    crate::utils::invoke::add_to_playlist("favorite".into(), vec![current_song])
                        .await
                } else {
                    crate::utils::invoke::remove_from_playlist(
                        "favorite".into(),
                        vec![current_song.song._id.unwrap_or_default()],
                    )
                    .await
                };
                match res {
                    Err(e) => tracing::error!("Failed to add to favorites playlist {:?}", e),
                    Ok(_) => is_fav.set(!is_fav_val),
                }
            });
        }
    };

    view! {
        <div class="row no-gutters controls-container">
            <div class="col col-button prev-button">
                <PrevTrackIcon
                    disabled=prev_track_dis
                    on:click=move |_| {
                        if !prev_track_dis.get() {
                            prev_song_setter.set(())
                        }
                    }
                />
            </div>
            <div class="col col-button repeat-button">
                <RepeatIcon mode=repeat_mode on:click=move |_| { toggle_repeat.set(()) } />
            </div>
            <div class="col col-play-button play-pause-button">
                <PlayIcon
                    play=is_play
                    on:click=move |_| {
                        let is_playing = is_play.get();
                        if is_playing {
                            set_current_state.set(PlayerState::Paused)
                        } else {
                            set_current_state.set(PlayerState::Playing)
                        }
                    }
                />

            </div>
            <div class="col col-button next-button">
                <NextTrackIcon
                    disabled=next_track_dis
                    on:click=move |_| {
                        if !next_track_dis.get() {
                            next_song_setter.set(())
                        }
                    }
                />
            </div>
            <div class="col col-button shuffle-button">
                <ShuffleIcon
                    filled=is_shuffle.read_only()
                    on:click=move |_| {
                        shuffle_queue.set(());
                    }
                />
            </div>
            {move || {
                if show_fav {
                    view! {
                        <div class="col col-button mr-1 fav-button" on:click=add_to_fav>
                            <FavIcon filled=is_fav.read_only() />
                        </div>
                    }
                        .into_any()
                } else {
                    ().into_any()
                }
            }}

            {move || {
                if show_time {
                    view! {
                        <div class="col-md-3 col-5 align-self-center timestamp-container">
                            <div class="row no-gutters align-items-center timestamp">
                                <div class="col-auto timestamp">
                                    {move || current_time_sig.get()}
                                </div>
                                <div class="col-auto timestamp timestamp-extra ml-1">
                                    / {move || total_duration_sig.get()}
                                </div>
                            </div>
                        </div>
                    }
                        .into_any()
                } else {
                    ().into_any()
                }
            }}

        </div>
    }
}

#[tracing::instrument(level = "debug", skip())]
#[component]
pub fn ExtraControls() -> impl IntoView {
    let player_store = use_context::<RwSignal<PlayerStore>>().unwrap();
    let (current_volume, set_current_volume) = create_slice(
        player_store,
        |player_store| player_store.get_raw_volume(),
        |player_store, volume| player_store.set_volume(volume),
    );

    let is_cut = Memo::new(move |_| current_volume.get() == 0f64);
    let ui_store = expect_context::<RwSignal<UiStore>>();

    let toggle_mute_sig = create_write_slice(player_store, |p, _| {
        p.toggle_mute();
    });

    let toggle_mute = move |_| {
        toggle_mute_sig.set(());
    };

    let show_popup_volume = RwSignal::new(false);
    let interval = RwSignal::<Option<TimeoutHandle>>::new(None);

    view! {
        <div class="row no-gutters align-items-center justify-content-end">
            <div
                class="col-auto volume-slider-container d-flex"
                class:volume-slider-show=move || show_popup_volume.get()
                on:mouseover=move |_| {
                    if show_popup_volume.get_untracked() {
                        interval
                            .update(|i| {
                                if let Some(i) = i.take() {
                                    i.clear();
                                }
                            });
                    }
                }
                on:mouseout=move |_| {
                    interval
                        .update(|i| {
                            if let Some(i) = i.take() {
                                i.clear();
                            }
                            *i = Some(
                                set_timeout_with_handle(
                                        move || {
                                            show_popup_volume.set(false);
                                        },
                                        Duration::from_millis(800),
                                    )
                                    .unwrap(),
                            );
                        });
                }
            >
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
                <VolumeIcon
                    on:click=toggle_mute
                    cut=is_cut
                    on:mouseover=move |_| {
                        interval
                            .update(|i| {
                                if let Some(i) = i.take() {
                                    i.clear();
                                }
                            });
                        show_popup_volume.set(true);
                    }
                    on:mouseout=move |_| {
                        interval
                            .update(|i| {
                                if let Some(i) = i.take() {
                                    i.clear();
                                }
                                *i = Some(
                                    set_timeout_with_handle(
                                            move || {
                                                show_popup_volume.set(false);
                                            },
                                            Duration::from_millis(800),
                                        )
                                        .unwrap(),
                                );
                            });
                    }
                />
            </div>
            <div class="col-auto expand-icon ml-3">
                <ExpandIcon on:click=move |_| { ui_store.update(move |s| s.toggle_show_queue()) } />
            </div>
        </div>
    }
}

#[tracing::instrument(level = "debug", skip())]
#[component]
pub fn Slider() -> impl IntoView {
    let player_store = use_context::<RwSignal<PlayerStore>>().unwrap();
    let slider_process: NodeRef<html::Div> = NodeRef::new();
    let offset_width = RwSignal::new(0f64);

    slider_process.on_load(move |s| {
        offset_width.set(s.offset_width() as f64);
    });
    let (current_time, set_current_time) = create_slice(
        player_store,
        |p| p.get_current_time(),
        move |p, val: f64| {
            p.force_seek_percent(
                val / slider_process.get_untracked().unwrap().offset_width() as f64,
            )
        },
    );

    let current_song = create_read_slice(player_store, |p| p.get_current_song());
    let total_time = RwSignal::new(1f64);

    let is_dragging = RwSignal::new(false);

    let display_time = RwSignal::new(0f64);
    Effect::new(move || {
        if !is_dragging.get() {
            let current_time = current_time.get();
            display_time.set(current_time);
        }
    });

    let listener = move |client_x| {
        if is_dragging.get_untracked() {
            tracing::debug!("dragging stop {}", client_x);
            set_current_time.set(client_x as f64);
            is_dragging.set(false);
        }
    };

    let _ = use_event_listener(use_document(), mouseup, move |evt| {
        listener(evt.client_x());
    });
    let _ = use_event_listener(use_document(), touchend, move |evt| {
        let client_x = evt.changed_touches().item(0).unwrap().client_x();
        listener(client_x);
    });

    let listener = move |offset_x| {
        if is_dragging.get() {
            display_time.set(
                (offset_x as f64 / slider_process.get_untracked().unwrap().offset_width() as f64)
                    * total_time.get(),
            );
        }
    };

    let _ = use_event_listener(use_document(), pointermove, move |evt| {
        listener(evt.offset_x());
    });

    Effect::new(move || {
        let current_song = current_song.get();
        if let Some(current_song) = current_song {
            if let Some(duration) = current_song.song.duration {
                total_time.set(duration);
            }
        }
    });

    view! {
        <div class="timeline pl-2 pr-2">
            <div class="time-slider time-slider-ltr timeline">
                <div
                    class="time-slider-rail"
                    node_ref=slider_process
                    on:click=move |ev| {
                        tracing::debug!("offset {}", ev.offset_x());
                        set_current_time.set(ev.offset_x() as f64);
                    }
                    on:mousedown=move |ev| {
                        ev.stop_propagation();
                        is_dragging.set(true);
                        tracing::debug!("dragging start");
                    }

                    on:touchstart=move |ev| {
                        ev.stop_propagation();
                        is_dragging.set(true);
                        tracing::debug!("dragging start");
                    }
                >

                    <div class="time-slider-bg">
                        <div
                            class="time-slider-process"
                            style=move || {
                                format!(
                                    "height: 100%; top: 0px; left: 0%; width: {}%; transition-property: width, left; transition-duration: 0.1s;",
                                    (display_time.get() / total_time.get()) * 100f64,
                                )
                            }
                        ></div>
                        <div
                            class="time-slider-dot"
                            role="slider"
                            tabindex="0"
                            style=move || {
                                format!(
                                    "width: 10px; height: 10px; transform: translate(-50%, -50%); top: 50%; left: {}%; transition: left 0.1s ease 0s;",
                                    (display_time.get() / total_time.get()) * 100f64,
                                )
                            }
                        >

                            <div class="time-slider-dot-handle"></div>
                        </div>
                    </div>
                </div>
            </div>
        </div>
    }
}
