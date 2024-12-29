use types::{mpris::MprisPlayerDetails, songs::Song, ui::player_details::PlayerState};
use wasm_bindgen_futures::spawn_local;

use crate::utils::entities::get_artist_string;

#[tracing::instrument(level = "trace", skip(song))]
pub fn set_metadata(song: &Song) {
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
        let res = crate::utils::invoke::set_metadata(metadata).await;
        if let Err(err) = res {
            tracing::error!("Failed to set mpris metadata {:?}", err);
        }
    })
}

#[tracing::instrument(level = "trace", skip(state))]
pub fn set_playback_state(state: PlayerState) {
    spawn_local(async move {
        let res = crate::utils::invoke::set_playback_state(state).await;
        if let Err(err) = res {
            tracing::error!("Failed to set mpris playback state {:?}", err);
        }
    })
}

#[tracing::instrument(level = "trace", skip(duration))]
pub fn set_position(duration: f64) {
    spawn_local(async move {
        let res = crate::utils::invoke::set_position(duration).await;
        if let Err(err) = res {
            tracing::error!("Failed to set mpris position {:?}", err);
        }
    })
}
