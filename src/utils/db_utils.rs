use std::rc::Rc;

use futures::lock::Mutex;
use indexed_db_futures::IdbDatabase;
use indexed_db_futures::IdbQuerySource;
use leptos::{spawn_local, SignalSet, SignalUpdate};
use serde::Serialize;
use serde_wasm_bindgen::{from_value, to_value};
use types::entities::QueryableAlbum;
use types::entities::QueryableArtist;
use types::entities::QueryableGenre;
use types::{
    entities::{GetEntityOptions, QueryablePlaylist},
    songs::{GetSongOptions, Song},
};
use wasm_bindgen::JsValue;
use web_sys::DomException;
use web_sys::IdbTransactionMode;

use crate::console_log;

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
pub fn get_playlists_local<T>(setter: T)
where
    T: SignalSet<Value = Vec<QueryablePlaylist>>
        + SignalUpdate<Value = Vec<QueryablePlaylist>>
        + 'static,
{
    spawn_local(async move {
        let args = to_value(&GetEntityOptionsArgs {
            options: GetEntityOptions {
                playlist: Some(QueryablePlaylist::default()),
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
pub fn get_playlists_by_option<T>(options: QueryablePlaylist, setter: T)
where
    T: SignalSet<Value = Vec<QueryablePlaylist>>
        + SignalUpdate<Value = Vec<QueryablePlaylist>>
        + 'static,
{
    use std::rc::Rc;

    use leptos::expect_context;

    use crate::{console_log, store::provider_store::ProviderStore, utils::common::fetch_infinite};

    let provider_store = expect_context::<Rc<ProviderStore>>();
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

        for key in provider_store.get_provider_keys() {
            console_log!("Fetching playlists from {}", key);
            fetch_infinite!(provider_store, key, fetch_user_playlists, setter,);
        }

        setter.update(|p| p.dedup_by(|a, b| a == b));
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

pub fn add_songs_to_library(songs: Vec<Song>) {
    #[derive(Serialize)]
    struct AddSongsArgs {
        songs: Vec<Song>,
    }

    spawn_local(async move {
        let res = invoke(
            "insert_songs",
            serde_wasm_bindgen::to_value(&AddSongsArgs { songs }).unwrap(),
        )
        .await;
        if res.is_err() {
            console_log!("Error adding songs: {:?}", res);
        }
    });
}

pub fn remove_songs_from_library(songs: Vec<Song>) {
    #[derive(Serialize)]
    struct RemoveSongsArgs {
        songs: Vec<String>,
    }
    spawn_local(async move {
        let res = invoke(
            "remove_songs",
            serde_wasm_bindgen::to_value(&RemoveSongsArgs {
                songs: songs
                    .iter()
                    .map(|s| s.song._id.clone().unwrap_or_default())
                    .collect(),
            })
            .unwrap(),
        )
        .await;
        if res.is_err() {
            console_log!("Error removing songs: {:?}", res);
        }
    });
}

pub fn add_to_playlist(id: String, songs: Vec<Song>) {
    #[derive(Serialize)]
    struct AddToPlaylistArgs {
        id: String,
        songs: Vec<Song>,
    }
    spawn_local(async move {
        let res = invoke(
            "add_to_playlist",
            serde_wasm_bindgen::to_value(&AddToPlaylistArgs { id, songs }).unwrap(),
        )
        .await;
        if res.is_err() {
            console_log!("Error adding to playlist: {:?}", res);
        }
    });
}

pub fn create_playlist(playlist: QueryablePlaylist) {
    spawn_local(async move {
        #[derive(Serialize)]
        struct CreatePlaylistArgs {
            playlist: QueryablePlaylist,
        }

        let res = invoke(
            "create_playlist",
            serde_wasm_bindgen::to_value(&CreatePlaylistArgs { playlist }).unwrap(),
        )
        .await;
        if let Err(res) = res {
            console_log!("Failed to create playlist: {:?}", res);
        }
    });
}

pub fn remove_playlist(playlist: QueryablePlaylist) {
    if playlist.playlist_id.is_none() {
        return;
    }

    spawn_local(async move {
        #[derive(Serialize)]
        struct RemovePlaylistArgs {
            id: String,
        }

        let res = invoke(
            "remove_playlist",
            serde_wasm_bindgen::to_value(&RemovePlaylistArgs {
                id: playlist.playlist_id.unwrap(),
            })
            .unwrap(),
        )
        .await;
        if let Err(res) = res {
            console_log!("Failed to remove playlist: {:?}", res);
        }
    });
}

pub fn export_playlist(playlist: QueryablePlaylist) {
    spawn_local(async move {
        #[derive(Serialize)]
        struct ExportPlaylistArgs {
            id: String,
        }

        let res = invoke(
            "export_playlist",
            serde_wasm_bindgen::to_value(&ExportPlaylistArgs {
                id: playlist.playlist_id.unwrap(),
            })
            .unwrap(),
        )
        .await;
        if let Err(res) = res {
            console_log!("Failed to export playlist: {:?}", res);
        }
    });
}

pub async fn write_to_indexed_db(
    db: Rc<Mutex<Option<Rc<IdbDatabase>>>>,
    store: &str,
    key: &str,
    value: &JsValue,
) -> Result<(), DomException> {
    let db = db.lock().await.clone();
    if let Some(db) = db {
        let tx = db.transaction_on_one_with_mode(store, IdbTransactionMode::Readwrite)?;
        let store = tx.object_store(store)?;
        store.put_key_val_owned(key, value)?.await?;
        console_log!("Wrote to indexed db");
    }
    Ok(())
}

pub async fn read_from_indexed_db(
    db: Rc<Mutex<Option<Rc<IdbDatabase>>>>,
    store: &str,
    key: &str,
) -> Result<Option<JsValue>, DomException> {
    let db = db.lock().await.clone();
    if let Some(db) = db {
        let tx = db.transaction_on_one(store)?;
        let store = tx.object_store(store)?;
        let res = store.get_owned(key)?.await?;
        return Ok(res);
    }
    Ok(None)
}
