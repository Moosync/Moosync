use leptos::{
    component, create_effect, create_rw_signal, view, IntoView, RwSignal, SignalGet, SignalSet,
};
use types::songs::Song;

use crate::{
    components::{songdetails::SongDetails, songlist::SongList},
    console_log,
};

#[component()]
pub fn SongView(#[prop()] songs: RwSignal<Vec<Song>>) -> impl IntoView {
    let selected_songs: RwSignal<Vec<usize>> = create_rw_signal(vec![]);
    let last_selected_song = create_rw_signal(None::<Song>);

    create_effect(move |_| {
        let selected_song = selected_songs.get().last().cloned();
        if let Some(selected_song) = selected_song {
            let all_songs = songs.get();
            console_log!("selected {:?}", all_songs);
            last_selected_song.set(all_songs.get(selected_song).cloned());
        }
    });

    view! {
        <div class="w-100 h-100">
            <div class="container-fluid song-container h-100">
                <div class="row no-gutters h-100 compact-container">
                    <div class="col-xl-3 col-4 h-100">
                        <SongDetails selected_song=last_selected_song.read_only() />
                    </div>
                    <SongList song_list=songs.read_only() selected_songs_sig=selected_songs />
                </div>
            </div>

        </div>
    }
}
