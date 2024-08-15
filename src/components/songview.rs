use std::ops::Deref;

use leptos::{
    component, create_effect, create_node_ref, create_rw_signal, event_target, html::Div,
    leptos_dom::Element, view, HtmlElement, IntoView, RwSignal, SignalGet, SignalSet, SignalUpdate,
};
use leptos_use::on_click_outside;
use types::{songs::Song, ui::song_details::SongDetailIcons};
use wasm_bindgen::JsCast;
use web_sys::Node;

use crate::{
    components::{songdetails::SongDetails, songlist::SongList},
    console_log,
};

#[component()]
pub fn SongView(
    #[prop()] songs: RwSignal<Vec<Song>>,
    #[prop()] icons: RwSignal<SongDetailIcons>,
    #[prop()] selected_songs: RwSignal<Vec<usize>>,
) -> impl IntoView {
    let last_selected_song = create_rw_signal(None::<Song>);

    let filtered_selected = create_rw_signal(vec![]);

    create_effect(move |_| {
        let selected_song = selected_songs.get().last().cloned();
        if let Some(selected_song) = selected_song {
            let all_songs = songs.get();
            console_log!("selected {:?}", all_songs.get(selected_song).unwrap());
            last_selected_song.set(all_songs.get(selected_song).cloned());
        } else {
            last_selected_song.set(None);
        }
    });

    let song_details_container = create_node_ref();
    let song_list_container = create_node_ref();
    on_click_outside(song_details_container, move |e| {
        let target = event_target::<Node>(&e);
        let song_details_elem: HtmlElement<Div> = song_list_container.get_untracked().unwrap();

        if !song_details_elem.contains(Some(&target)) {
            selected_songs.update(|s| s.clear());
            filtered_selected.update(|s| s.clear());
        }
    });
    on_click_outside(song_list_container, move |e| {
        let target = event_target::<Node>(&e);
        let song_details_elem: HtmlElement<Div> = song_details_container.get_untracked().unwrap();

        if !song_details_elem.contains(Some(&target)) {
            selected_songs.update(|s| s.clear());
            filtered_selected.update(|s| s.clear());
        }
    });

    view! {
        <div class="w-100 h-100">
            <div class="container-fluid song-container h-100">
                <div class="row no-gutters h-100 compact-container">
                    <div
                        node_ref=song_details_container
                        style="max-height: 100%; height: fit-content;"
                        class="col-xl-3 col-4"
                    >
                        <SongDetails selected_song=last_selected_song.read_only() icons=icons />
                    </div>
                    <div
                        node_ref=song_list_container
                        class="col-xl-9 col-8 h-100 song-list-compact"
                    >
                        <SongList
                            song_list=songs.read_only()
                            selected_songs_sig=selected_songs
                            filtered_selected=filtered_selected
                        />
                    </div>
                </div>
            </div>

        </div>
    }
}
