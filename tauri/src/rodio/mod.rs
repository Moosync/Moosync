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

#[cfg(desktop)]
use std::thread;

#[cfg(desktop)]
use crate::macros::generate_command_async;
#[cfg(desktop)]
use futures::executor::block_on;
#[cfg(desktop)]
use rodio_player::RodioPlayer;
use tauri::AppHandle;
#[cfg(desktop)]
use tauri::{Emitter, Manager, State};
use types::errors::Result;

// Desktop implementation
#[cfg(desktop)]
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

#[cfg(desktop)]
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

#[cfg(desktop)]
generate_command_async!(rodio_play, RodioPlayer, (),);
#[cfg(desktop)]
generate_command_async!(rodio_pause, RodioPlayer, (),);
#[cfg(desktop)]
generate_command_async!(rodio_stop, RodioPlayer, (),);
#[cfg(desktop)]
generate_command_async!(rodio_seek, RodioPlayer, (), pos: f64);
#[cfg(desktop)]
generate_command_async!(rodio_set_volume, RodioPlayer, (), volume: f32);
#[cfg(desktop)]
generate_command_async!(rodio_get_volume, RodioPlayer, f32,);

// Mobile stubs - rodio is not used on mobile, audio is handled by tauri-plugin-audioplayer
#[cfg(mobile)]
pub struct RodioPlayerStub;

#[cfg(mobile)]
pub fn get_rodio_state(_app: AppHandle) -> RodioPlayerStub {
    RodioPlayerStub
}

#[cfg(mobile)]
#[tauri::command(async)]
#[tauri_invoke_proc::parse_tauri_command]
pub fn rodio_load(_app: AppHandle, _src: String) -> Result<()> {
    Ok(())
}

#[cfg(mobile)]
#[tauri::command(async)]
#[tauri_invoke_proc::parse_tauri_command]
pub fn rodio_play(_app: AppHandle) -> Result<()> {
    Ok(())
}

#[cfg(mobile)]
#[tauri::command(async)]
#[tauri_invoke_proc::parse_tauri_command]
pub fn rodio_pause(_app: AppHandle) -> Result<()> {
    Ok(())
}

#[cfg(mobile)]
#[tauri::command(async)]
#[tauri_invoke_proc::parse_tauri_command]
pub fn rodio_stop(_app: AppHandle) -> Result<()> {
    Ok(())
}

#[cfg(mobile)]
#[tauri::command(async)]
#[tauri_invoke_proc::parse_tauri_command]
pub fn rodio_seek(_app: AppHandle, _pos: f64) -> Result<()> {
    Ok(())
}

#[cfg(mobile)]
#[tauri::command(async)]
#[tauri_invoke_proc::parse_tauri_command]
pub fn rodio_set_volume(_app: AppHandle, _volume: f32) -> Result<()> {
    Ok(())
}

#[cfg(mobile)]
#[tauri::command(async)]
#[tauri_invoke_proc::parse_tauri_command]
pub fn rodio_get_volume(_app: AppHandle) -> Result<f32> {
    Ok(0.0)
}
