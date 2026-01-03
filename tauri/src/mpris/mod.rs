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

use std::thread;

use crate::macros::generate_command;
use mpris::MprisHolder;
use serde_json::{json, Value};
use tauri::{AppHandle, Emitter, State};
use types::{errors::Result, ui::player_details::PlayerState};

#[tracing::instrument(level = "debug", skip(app))]
pub fn get_mpris_state(app: AppHandle) -> Result<MprisHolder> {
    let mpris_holder = MprisHolder::new()?;

    #[cfg(mobile)]
    mpris_holder.set_app_handle(app.clone());

    let receiver = mpris_holder.event_rx.clone();
    thread::spawn(move || {
        let receiver = receiver.lock().unwrap();
        loop {
            tracing::trace!("Waiting for mpris events");
            let event = receiver.recv().unwrap();
            let data = match event {
                mpris::MediaControlEvent::Play => (0, Value::Null),
                mpris::MediaControlEvent::Pause => (1, Value::Null),
                mpris::MediaControlEvent::Toggle => (13, Value::Null),
                mpris::MediaControlEvent::Next => (6, Value::Null),
                mpris::MediaControlEvent::Previous => (7, Value::Null),
                mpris::MediaControlEvent::Stop => (2, Value::Null),
                mpris::MediaControlEvent::Seek(_) => (12, json!(0)),
                mpris::MediaControlEvent::SeekBy(_dir, pos) => {
                    // match dir {
                    //     mpris::SeekDirection::Forward => (12, json!(pos.as_secs())),
                    //     mpris::SeekDirection::Backward => {
                    //         (12, json!((pos.as_secs() as i128).neg()))
                    //     }
                    // }
                    (12, json!(pos.as_secs()))
                }
                mpris::MediaControlEvent::SetPosition(pos) => (12, json!(pos.0.as_secs())),

                mpris::MediaControlEvent::SetVolume(vol) => (15, json!(vol)),
                mpris::MediaControlEvent::OpenUri(uri) => (16, Value::String(uri)),
                mpris::MediaControlEvent::Raise => (17, Value::Null),
                mpris::MediaControlEvent::Quit => (18, Value::Null),
            };
            let _ = app.emit("media_button_press", data);
        }
    });

    Ok(mpris_holder)
}

// generate_command!(set_metadata, MprisHolder, (), metadata: MprisPlayerDetails);
generate_command!(set_playback_state, MprisHolder, (), state: PlayerState);
generate_command!(set_position, MprisHolder, (), duration: f64);
