use leptos::{
    component, create_rw_signal, view, IntoView,
};
use types::songs::{GetSongOptions};

use crate::{
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
