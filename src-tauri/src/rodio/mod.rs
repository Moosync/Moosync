use std::thread;

use futures::executor::block_on;
use macros::generate_command_async;
use rodio_player::RodioPlayer;
use tauri::{AppHandle, Emitter, Manager, State};
use types::errors::Result;

#[tracing::instrument(level = "trace", skip())]
pub fn get_rodio_state(app: AppHandle) -> RodioPlayer {
    let cache_dir = app.path().app_cache_dir().unwrap();
    let rodio_player = RodioPlayer::new(cache_dir);

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

#[tracing::instrument(level = "trace", skip(app, src))]
#[tauri::command(async)]
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
