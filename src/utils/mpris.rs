use serde::Serialize;
use types::{mpris::MprisPlayerDetails, songs::Song, ui::player_details::PlayerState};
use wasm_bindgen_futures::spawn_local;

use crate::{
    console_log,
    utils::{
        common::invoke,
        entities::get_artist_string,
    },
};

pub fn set_metadata(song: &Song) {
    #[derive(Serialize)]
    struct SetMetadataArgs {
        metadata: MprisPlayerDetails,
    }

    let metadata = MprisPlayerDetails {
        title: song.song.title.clone(),
        id: song.song._id.clone(),
        artist_name: Some(get_artist_string(song.artists.clone())),
        album_name: song.album.clone().map(|a| a.album_name.unwrap_or_default()),
        album_artist: None,
        genres: None,
        duration: song.song.duration,
        thumbnail: song.song.song_cover_path_high.clone(),
    };
    spawn_local(async move {
        let res = invoke(
            "set_metadata",
            serde_wasm_bindgen::to_value(&SetMetadataArgs { metadata }).unwrap(),
        )
        .await;

        if let Err(err) = res {
            console_log!("Failed to set mpris metadata {:?}", err);
        }
    })
}

pub fn set_playback_state(state: PlayerState) {
    #[derive(Serialize)]
    struct SetPlaybackStateArgs {
        state: PlayerState,
    }

    spawn_local(async move {
        let res = invoke(
            "set_playback_state",
            serde_wasm_bindgen::to_value(&SetPlaybackStateArgs { state }).unwrap(),
        )
        .await;

        if let Err(err) = res {
            console_log!("Failed to set mpris playback state {:?}", err);
        }
    })
}

pub fn set_position(duration: f64) {
    #[derive(Serialize)]
    struct SetPositionArgs {
        duration: f64,
    }

    spawn_local(async move {
        let res = invoke(
            "set_position",
            serde_wasm_bindgen::to_value(&SetPositionArgs { duration }).unwrap(),
        )
        .await;

        if let Err(err) = res {
            console_log!("Failed to set mpris position {:?}", err);
        }
    })
}
