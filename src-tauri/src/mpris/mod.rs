use std::thread;

use macros::generate_command;
use mpris::MprisHolder;
use serde_json::{json, Value};
use tauri::{AppHandle, Emitter, State};
use types::{errors::errors::Result, mpris::MprisPlayerDetails, ui::player_details::PlayerState};

pub fn get_mpris_state(app: AppHandle) -> Result<MprisHolder> {
    let mpris_holder = MprisHolder::new()?;

    let receiver = mpris_holder.event_rx.clone();
    thread::spawn(move || {
        let receiver = receiver.lock().unwrap();
        loop {
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

generate_command!(set_metadata, MprisHolder, (), metadata: MprisPlayerDetails);
generate_command!(set_playback_state, MprisHolder, (), state: PlayerState);
generate_command!(set_position, MprisHolder, (), duration: f64);
