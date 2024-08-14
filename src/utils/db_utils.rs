use leptos::{spawn_local, SignalSet};
use serde::Serialize;
use serde_wasm_bindgen::{from_value, to_value};
use types::entities::QueryableAlbum;
use types::entities::QueryableArtist;
use types::entities::QueryableGenre;
use types::{
    entities::{GetEntityOptions, QueryablePlaylist},
    songs::{GetSongOptions, Song},
};

use super::common::invoke;

#[derive(Serialize)]
struct GetSongOptionsArgs {
    options: GetSongOptions,
}

#[derive(Serialize)]
struct GetEntityOptionsArgs {
    options: GetEntityOptions,
}

#[cfg(not(feature = "mock"))]
pub fn get_songs_by_option(
    options: GetSongOptions,
    setter: impl SignalSet<Value = Vec<Song>> + 'static,
) {
    use crate::console_log;

    spawn_local(async move {
        let args = to_value(&GetSongOptionsArgs { options }).unwrap();
        let res = invoke("get_songs_by_options", args).await;
        if res.is_err() {
            console_log!("Failed to load songs {:?}", res.unwrap_err());
            setter.set(vec![]);
            return;
        }
        let songs: Vec<Song> = from_value(res.unwrap()).unwrap();
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
        song.song.duration = Some(96f64);
        song.song.playback_url =
            Some("https://cdn.freesound.org/previews/728/728162_462105-lq.mp3".into());
        songs.push(song);
    }

    setter.set(songs);
}

#[cfg(feature = "mock")]
pub fn get_playlists_by_option(
    options: QueryablePlaylist,
    setter: impl SignalSet<Value = Vec<QueryablePlaylist>> + 'static,
) {
    let mut songs = vec![];
    for i in 0..1000 {
        let mut playlist = QueryablePlaylist::default();
        playlist.playlist_id = Some(format!("playlist_id_{}", i));
        playlist.playlist_name = format!("Playlist {}", i);
        playlist.playlist_coverpath = Some("https://upload.wikimedia.org/wikipedia/commons/thumb/6/66/SMPTE_Color_Bars.svg/200px-SMPTE_Color_Bars.svg.png".to_string());
        songs.push(playlist);
    }

    setter.set(songs);
}

#[cfg(feature = "mock")]
pub fn get_artists_by_option(
    options: QueryableArtist,
    setter: impl SignalSet<Value = Vec<QueryableArtist>> + 'static,
) {
    let mut songs = vec![];
    for i in 0..1000 {
        let mut artist = QueryableArtist::default();
        artist.artist_id = Some(format!("artist_id_{}", i));
        artist.artist_name = Some(format!("Artist {}", i));
        artist.artist_coverpath = Some("https://upload.wikimedia.org/wikipedia/commons/thumb/6/66/SMPTE_Color_Bars.svg/200px-SMPTE_Color_Bars.svg.png".to_string());
        songs.push(artist);
    }

    setter.set(songs);
}

#[cfg(feature = "mock")]
pub fn get_albums_by_option(
    options: QueryableAlbum,
    setter: impl SignalSet<Value = Vec<QueryableAlbum>> + 'static,
) {
    let mut songs = vec![];
    for i in 0..1000 {
        let mut album = QueryableAlbum::default();
        album.album_id = Some(format!("album_id_{}", i));
        album.album_name = Some(format!("Album {}", i));
        album.album_coverpath_high = Some("https://upload.wikimedia.org/wikipedia/commons/thumb/6/66/SMPTE_Color_Bars.svg/200px-SMPTE_Color_Bars.svg.png".to_string());
        songs.push(album);
    }

    setter.set(songs);
}

#[cfg(feature = "mock")]
pub fn get_genres_by_option(
    options: QueryableGenre,
    setter: impl SignalSet<Value = Vec<QueryableGenre>> + 'static,
) {
    let mut songs = vec![];
    for i in 0..1000 {
        let mut album = QueryableGenre::default();
        album.genre_id = Some(format!("genre_id_{}", i));
        album.genre_name = Some(format!("Genre {}", i));
        songs.push(album);
    }

    setter.set(songs);
}

#[cfg(not(feature = "mock"))]
pub fn get_playlists_by_option(
    options: QueryablePlaylist,
    setter: impl SignalSet<Value = Vec<QueryablePlaylist>> + 'static,
) {
    use crate::console_log;

    spawn_local(async move {
        let args = to_value(&GetEntityOptionsArgs {
            options: GetEntityOptions {
                playlist: Some(options),
                ..Default::default()
            },
        })
        .unwrap();
        let res = invoke("get_entity_by_options", args).await;
        if res.is_err() {
            console_log!("Error getting playlists: {:?}", res);
            return;
        }
        let songs: Vec<QueryablePlaylist> = from_value(res.unwrap()).unwrap();
        setter.set(songs);
    });
}

#[cfg(not(feature = "mock"))]
pub fn get_artists_by_option(
    options: QueryableArtist,
    setter: impl SignalSet<Value = Vec<QueryableArtist>> + 'static,
) {
    use crate::console_log;

    spawn_local(async move {
        let args = to_value(&GetEntityOptionsArgs {
            options: GetEntityOptions {
                artist: Some(options),
                ..Default::default()
            },
        })
        .unwrap();
        let res = invoke("get_entity_by_options", args).await;
        if res.is_err() {
            console_log!("Error getting artists: {:?}", res);
            return;
        }
        let songs: Vec<QueryableArtist> = from_value(res.unwrap()).unwrap();
        setter.set(songs);
    });
}

#[cfg(not(feature = "mock"))]
pub fn get_albums_by_option(
    options: QueryableAlbum,
    setter: impl SignalSet<Value = Vec<QueryableAlbum>> + 'static,
) {
    use crate::console_log;

    spawn_local(async move {
        let args = to_value(&GetEntityOptionsArgs {
            options: GetEntityOptions {
                album: Some(options),
                ..Default::default()
            },
        })
        .unwrap();
        let res = invoke("get_entity_by_options", args).await;
        if res.is_err() {
            console_log!("Error getting albums: {:?}", res);
            return;
        }
        let songs: Vec<QueryableAlbum> = from_value(res.unwrap()).unwrap();
        setter.set(songs);
    });
}

#[cfg(not(feature = "mock"))]
pub fn get_genres_by_option(
    options: QueryableGenre,
    setter: impl SignalSet<Value = Vec<QueryableGenre>> + 'static,
) {
    use crate::console_log;

    spawn_local(async move {
        let args = to_value(&GetEntityOptionsArgs {
            options: GetEntityOptions {
                genre: Some(options),
                ..Default::default()
            },
        })
        .unwrap();
        let res = invoke("get_entity_by_options", args).await;
        if res.is_err() {
            console_log!("Error getting genres: {:?}", res);
            return;
        }
        let songs: Vec<QueryableGenre> = from_value(res.unwrap()).unwrap();
        setter.set(songs);
    });
}
