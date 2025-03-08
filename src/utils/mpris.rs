// Moosync
// Copyright (C) 2024, 2025  Moosync <support@moosync.app>
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <http://www.gnu.org/licenses/>.

use std::sync::Mutex;

use crate::utils::entities::get_artist_string;
use lazy_static::lazy_static;
use leptos::task::spawn_local;
use types::{mpris::MprisPlayerDetails, songs::Song, ui::player_details::PlayerState};

#[tracing::instrument(level = "debug", skip(song))]
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

#[tracing::instrument(level = "debug", skip(state))]
pub fn set_playback_state(state: PlayerState) {
    spawn_local(async move {
        let res = crate::utils::invoke::set_playback_state(state).await;
        if let Err(err) = res {
            tracing::error!("Failed to set mpris playback state {:?}", err);
        }
    })
}

lazy_static! {
    static ref last_time_update: Mutex<wasm_timer::Instant> =
        Mutex::new(wasm_timer::Instant::now());
}
#[tracing::instrument(level = "debug", skip(duration))]
pub fn set_position(duration: f64) {
    let should_update = {
        let mut last_time_update_lock = last_time_update.lock().unwrap();
        if last_time_update_lock.elapsed().as_secs() > 10 {
            *last_time_update_lock = wasm_timer::Instant::now();
            true
        } else {
            false
        }
    };
    if should_update {
        spawn_local(async move {
            let res = crate::utils::invoke::set_position(duration).await;
            if let Err(err) = res {
                tracing::error!("Failed to set mpris position {:?}", err);
            }
        });
    }
}
