use leptos::{spawn_local, SignalSet};
use serde::Serialize;
use serde_wasm_bindgen::{from_value, to_value};
use types::songs::{GetSongOptions, QueryableSong, SearchableSong, Song};
use wasm_bindgen::JsValue;

use crate::console_log;

use super::common::invoke;

#[derive(Serialize)]
struct GetSongOptionsArgs {
    options: GetSongOptions,
}

#[cfg(not(feature = "mock"))]
pub fn get_songs_by_option(
    options: GetSongOptions,
    setter: impl SignalSet<Value = Vec<Song>> + 'static,
) {
    spawn_local(async move {
        let args = to_value(&GetSongOptionsArgs { options }).unwrap();
        let res = invoke("get_songs_by_options", args).await;
        let songs: Vec<Song> = from_value(res).unwrap();
        setter.set(songs);
    });
}

#[cfg(feature = "mock")]
pub fn get_songs_by_option(
    options: GetSongOptions,
    setter: impl SignalSet<Value = Vec<Song>> + 'static,
) {
    use types::{entities::QueryableArtist, songs::SongType};

    let mut songs = vec![];
    for i in 0..10 {
        let mut song = Song::default();
        song.song._id = Some(format!("song_id_{}", i));
        song.song.title = Some(format!("hello world {}", i));
        song.song.song_cover_path_low =
            Some("https://i.scdn.co/image/ab67616d0000b2733cf1c1dbcfa3f1ab7282719b".to_string());
        song.artists = Some(vec![QueryableArtist {
            artist_name: Some("Test artist".to_string()),
            ..Default::default()
        }]);
        song.song.type_ = SongType::LOCAL;
        song.song.playback_url =
            Some("https://cdn.freesound.org/previews/728/728162_462105-lq.mp3".into());
        songs.push(song);
    }

    setter.set(songs);
}
