use leptos::{
    component, create_effect, create_rw_signal, spawn_local, view, IntoView, RwSignal, SignalGet,
    SignalSet,
};
use types::songs::{GetSongOptions, Song};

use crate::{
    components::{songdetails::SongDetails, songlist::SongList},
    console_log,
    utils::db_utils::get_songs_by_option,
};

use crate::components::songview::SongView;

#[component()]
pub fn AllSongs() -> impl IntoView {
    let songs = create_rw_signal(vec![]);
    get_songs_by_option(GetSongOptions::default(), songs);

    view! {
        <SongView songs=songs />
    }
}
