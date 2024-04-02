use leptos::{spawn_local, SignalSet};
use serde::Serialize;
use serde_wasm_bindgen::{from_value, to_value};
use types::songs::{GetSongOptions, Song};

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
    for i in 0..1000 {
        let mut song = Song::default();
        song.song._id = Some(format!("song_id_{}", i));
        song.song.title = Some(format!("hello world {}", i));
        song.song.song_cover_path_low = Some("https://upload.wikimedia.org/wikipedia/commons/thumb/6/66/SMPTE_Color_Bars.svg/200px-SMPTE_Color_Bars.svg.png".to_string());
        song.song.song_cover_path_high = Some("https://upload.wikimedia.org/wikipedia/commons/thumb/6/66/SMPTE_Color_Bars.svg/200px-SMPTE_Color_Bars.svg.png".to_string());
        song.artists = Some(vec![QueryableArtist {
            artist_name: Some("Test artist".to_string()),
            ..Default::default()
        }]);
        song.song.type_ = SongType::LOCAL;
        song.song.duration = Some(200f64);
        song.song.playback_url =
            Some("https://cdn.freesound.org/previews/728/728162_462105-lq.mp3".into());
        songs.push(song);
    }

    setter.set(songs);
}
