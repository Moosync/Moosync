use leptos::{
    component, create_read_slice, create_write_slice, expect_context, view, CollectView, IntoView,
    RwSignal, SignalGet, SignalSet,
};
use types::songs::Song;

use crate::{
    components::{low_img::LowImg, provider_icon::ProviderIcon, songdetails::SongDetails},
    icons::{cross_icon::CrossIcon, trash_icon::TrashIcon},
    store::player_store::PlayerStore,
    utils::common::get_low_img,
};

#[component]
pub fn QueueItem(#[prop()] song: Song, index: usize) -> impl IntoView {
    let player_store = expect_context::<RwSignal<PlayerStore>>();
    let play_now = create_write_slice(player_store, |p, val| p.play_now(val));
    let remove_from_queue = create_write_slice(player_store, |p, val| p.remove_from_queue(val));
    let song_cloned = song.clone();

    view! {
        <div class="container-fluid item-container">
            <div class="row item-row">
                <LowImg
                    cover_img=get_low_img(&song)
                    play_now=move || play_now.set(song_cloned.clone())
                />
                <div class="col-lg-7 col-xl-8 col-5">
                    <div class="d-flex">
                        <div class="text-left song-title text-truncate">{song.song.title}</div>
                        <ProviderIcon extension=song.song.provider_extension/>
                    </div>
                    <div class="text-left song-subtitle text-truncate">
                        {song
                            .artists
                            .unwrap_or_default()
                            .iter()
                            .map(|a| a.artist_name.clone().unwrap_or_default())
                            .collect::<Vec<String>>()
                            .join(", ")}
                    </div>

                </div>
                <div class="col-auto text-right ml-auto d-flex align-items-center">
                    <div class="ml-auto remove-button">
                        <TrashIcon on:click=move |_| remove_from_queue.set(index)/>
                    </div>
                </div>
            </div>
        </div>
    }
}

#[component]
pub fn MusicInfo(#[prop()] show: RwSignal<bool>) -> impl IntoView {
    let player_store = expect_context::<RwSignal<PlayerStore>>();
    let current_song = create_read_slice(player_store, move |p| p.current_song.clone());
    let queue_songs = create_read_slice(player_store, move |p| p.get_queue_songs());
    view! {
        <div
            class="slider"
            class:musicinfo-open=move || show.get()
            class:musicinfo-close=move || !show.get()
        >

            <div class="h-100 w-100">
                // Canvas
                <div class="dark-overlay" style="top: 0px;"></div>
                <div class="container-fluid w-100 h-100 music-info-container">
                    <div class="row no-gutters justify-content-end">
                        // Close button
                        <div class="col-auto">
                            <div class="cross-icon button-grow">
                                <CrossIcon/>
                            </div>
                        </div>
                    </div>

                    <div class="row no-gutters justify-content-center h-100 flex-nowrap">
                        // Song details
                        <div class="col-4">
                            <SongDetails show_icons=false selected_song=current_song/>
                        </div>
                        <div class="col-7 offset-1 right-container h-100">
                            <div class="h-100">
                                <div class="row">
                                    <div class="col-auto d-flex">
                                        <div class="rounded-btn">Save as playlist</div>
                                        <div class="rounded-btn">Clear</div>
                                    </div>
                                </div>
                                <div class="row queue-container-outer">
                                    <div class="col w-100 h-100 mr-4 queue-container">
                                        <div class="w-100 h-100">

                                            {move || {
                                                queue_songs
                                                    .get()
                                                    .into_iter()
                                                    .enumerate()
                                                    .map(|(index, s)| {
                                                        view! { <QueueItem song=s index=index/> }
                                                    })
                                                    .collect_view()
                                            }}

                                        </div>
                                    </div>
                                </div>
                            </div>
                        </div>
                    </div>
                </div>
            </div>

        </div>
    }
}
