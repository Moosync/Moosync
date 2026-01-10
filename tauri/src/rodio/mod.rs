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

use crate::macros::generate_command_async;
use futures::executor::block_on;
use rodio_player::RodioPlayer;
use tauri::{AppHandle, Emitter, Manager, State};
use types::errors::Result;

#[tracing::instrument(level = "debug", skip())]
pub fn get_rodio_state(app: AppHandle) -> RodioPlayer {
    let rodio_player = RodioPlayer::new();

    let events_rx = rodio_player.get_events_rx();
    thread::spawn(move || {
        let events_rx = events_rx.lock().unwrap();
        while let Ok(event) = events_rx.recv() {
            tracing::info!("Sending rodio event {:?}", event);
            let res = app.emit("rodio_event", event);
            if res.is_err() {
                tracing::error!("Error sending rodio event {:?}", res);
            }
        }
    });

    rodio_player
}

#[tracing::instrument(level = "debug", skip(app, src))]
#[tauri::command(async)]
#[tauri_invoke_proc::parse_tauri_command]
pub fn rodio_load(app: AppHandle, src: String) -> Result<()> {
    tauri::async_runtime::spawn_blocking(move || {
        let rodio: State<'_, RodioPlayer> = app.state();
        block_on(rodio.rodio_load(src)).unwrap();
    });
    Ok(())
}

// generate_command_async!(rodio_load, RodioPlayer, (), src: String);
generate_command_async!(rodio_play, RodioPlayer, (),);
generate_command_async!(rodio_pause, RodioPlayer, (),);
generate_command_async!(rodio_stop, RodioPlayer, (),);
generate_command_async!(rodio_seek, RodioPlayer, (), pos: f64);
generate_command_async!(rodio_set_volume, RodioPlayer, (), volume: f32);
generate_command_async!(rodio_get_volume, RodioPlayer, f32,);
