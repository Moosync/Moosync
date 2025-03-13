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

use std::{cell::RefCell, rc::Rc, sync::Mutex, time::Duration};

use leptos::{
    leptos_dom::helpers::IntervalHandle,
    prelude::{set_interval_with_handle, NodeRef},
    task::spawn_local,
};
use types::{songs::SongType, ui::player_details::PlayerEvents};
use wasm_bindgen::JsValue;

use crate::utils::{
    common::{convert_file_src, listen_event},
    invoke::{rodio_load, rodio_pause, rodio_play, rodio_seek, rodio_set_volume, rodio_stop},
};

use super::generic::{GenericPlayer, PlayerEventsSender};

#[derive(Debug, Clone)]
pub struct RodioPlayer {
    unlisten: Option<js_sys::Function>,
    timer: Rc<Mutex<Option<IntervalHandle>>>,
    time: Rc<Mutex<f64>>,
}

impl RodioPlayer {
    #[tracing::instrument(level = "debug", skip())]
    pub fn new() -> Self {
        Self {
            unlisten: None,
            timer: Default::default(),
            time: Default::default(),
        }
    }
}

impl GenericPlayer for RodioPlayer {
    #[tracing::instrument(level = "debug", skip(self))]
    fn initialize(&self, _: NodeRef<leptos::html::Div>) {}

    #[tracing::instrument(level = "debug", skip(self))]
    fn key(&self) -> String {
        "rodio".into()
    }

    #[tracing::instrument(level = "debug", skip(self, src, resolver))]
    fn load(&self, src: String, autoplay: bool, resolver: tokio::sync::oneshot::Sender<()>) {
        spawn_local(async move {
            let res = rodio_load(src).await;
            if let Err(err) = res {
                tracing::error!("Rodio error {:?}", err);
            } else {
                let _ = rodio_play().await;
            }

            resolver.send(()).unwrap();
        });
    }

    #[tracing::instrument(level = "debug", skip(self))]
    fn stop(&mut self) -> types::errors::Result<()> {
        let unlisten = self.unlisten.take();
        if let Some(unlisten) = &unlisten {
            if let Err(e) = unlisten.call0(&JsValue::NULL) {
                tracing::error!("Error removing listeners {:?}", e);
            }
        }

        spawn_local(async move {
            let res = rodio_stop().await;

            if res.is_err() {
                tracing::error!("Error stopping {:?}", res.unwrap_err());
            }
        });

        let mut timer = self.timer.lock().unwrap();
        if timer.is_some() {
            let handle = timer.unwrap();
            handle.clear();
        }
        *timer = None;

        *self.time.lock().unwrap() = 0f64;

        Ok(())
    }

    #[tracing::instrument(level = "debug", skip(self))]
    fn play(&self) -> types::errors::Result<()> {
        spawn_local(async move {
            let res = rodio_play().await;

            if res.is_err() {
                tracing::error!("Error playing {:?}", res.unwrap_err());
            }
        });
        Ok(())
    }

    #[tracing::instrument(level = "debug", skip(self))]
    fn pause(&self) -> types::errors::Result<()> {
        spawn_local(async move {
            let res = rodio_pause().await;

            if res.is_err() {
                tracing::error!("Error playing {:?}", res.unwrap_err());
            }
        });
        Ok(())
    }

    #[tracing::instrument(level = "debug", skip(self, pos))]
    fn seek(&self, pos: f64) -> types::errors::Result<()> {
        spawn_local(async move {
            let res = rodio_seek(pos).await;
            if res.is_err() {
                tracing::error!("Error playing {:?}", res.unwrap_err());
            }
        });
        Ok(())
    }

    #[tracing::instrument(level = "debug", skip(self))]
    fn provides(&self) -> &[types::songs::SongType] {
        &[
            SongType::LOCAL,
            SongType::URL,
            SongType::YOUTUBE,
            SongType::SPOTIFY,
        ]
    }

    #[tracing::instrument(level = "debug", skip(self, song))]
    fn can_play(&self, song: &types::songs::Song) -> bool {
        let playback_url = song
            .song
            .path
            .clone()
            .map(convert_file_src)
            .or(song.song.playback_url.clone());
        tracing::debug!("Checking playback url {:?}", playback_url);
        if let Some(playback_url) = playback_url {
            return playback_url.starts_with("http://")
                || playback_url.starts_with("https://")
                || playback_url.starts_with("asset");
        }
        false
    }

    #[tracing::instrument(level = "debug", skip(self, volume))]
    fn set_volume(&self, volume: f64) -> types::errors::Result<()> {
        let parsed_volume = volume / 100f64;
        tracing::debug!("Setting volume {}", parsed_volume);
        spawn_local(async move {
            let res = rodio_set_volume(parsed_volume as f32).await;
            if res.is_err() {
                tracing::error!("Error setting volume {}: {:?}", volume, res.unwrap_err());
            }
        });

        Ok(())
    }

    #[tracing::instrument(level = "debug", skip(self))]
    fn get_volume(&self) -> types::errors::Result<f64> {
        Ok(0f64)
    }

    #[tracing::instrument(level = "debug", skip(self, state_setter))]
    fn add_listeners(&mut self, state_setter: PlayerEventsSender) {
        if let Some(unlisten) = self.unlisten.take() {
            unlisten.call0(&JsValue::NULL).unwrap();
        }

        tracing::debug!("Adding rodio listeners");
        let start_timer =
            |timer: Rc<Mutex<Option<IntervalHandle>>>, time: Rc<Mutex<f64>>, tx: Callback| {
                tracing::debug!("Starting timer");
                let mut timer = timer.lock().unwrap();
                if timer.is_some() {
                    let handle = timer.unwrap();
                    handle.clear();
                }
                let res = set_interval_with_handle(
                    move || {
                        let mut time = time.lock().unwrap();
                        *time += 1f64;
                        let tx = tx.borrow_mut();
                        tx(PlayerEvents::TimeUpdate(*time));
                    },
                    Duration::from_secs(1),
                )
                .unwrap();
                *timer = Some(res);
            };

        type Callback = RefCell<PlayerEventsSender>;

        let stop_timer = |timer: Rc<Mutex<Option<IntervalHandle>>>, _, _| {
            tracing::debug!("pausing timer");
            let mut timer = timer.lock().unwrap();
            if timer.is_some() {
                let handle = timer.unwrap();
                handle.clear();
            }
            *timer = None;
        };

        let stop_and_clear_timer =
            |timer: Rc<Mutex<Option<IntervalHandle>>>, time: Rc<Mutex<f64>>, tx: Callback| {
                tracing::debug!("Stopping timer");
                let mut timer = timer.lock().unwrap();
                if timer.is_some() {
                    let handle = timer.unwrap();
                    handle.clear();
                }
                *timer = None;

                *time.lock().unwrap() = 0f64;
                let tx = tx.borrow_mut();
                tx(PlayerEvents::TimeUpdate(0f64));
            };

        let tx = RefCell::new(state_setter);
        let timer = self.timer.clone();
        let time = self.time.clone();

        let unlisten = listen_event("rodio_event", move |data| {
            tracing::debug!("Got rodio event {:?}", data);
            let payload = js_sys::Reflect::get(&data, &JsValue::from_str("payload")).unwrap();
            let event: PlayerEvents = serde_wasm_bindgen::from_value(payload).unwrap();

            match event {
                PlayerEvents::Play => start_timer(timer.clone(), time.clone(), tx.clone()),
                PlayerEvents::Pause => stop_timer(timer.clone(), time.clone(), tx.clone()),
                PlayerEvents::Ended => {
                    stop_and_clear_timer(timer.clone(), time.clone(), tx.clone())
                }
                PlayerEvents::Loading => stop_timer(timer.clone(), time.clone(), tx.clone()),
                PlayerEvents::TimeUpdate(pos) => {
                    let time = time.clone();
                    *time.lock().unwrap() = pos;

                    let tx = tx.borrow_mut();
                    tx(PlayerEvents::TimeUpdate(pos));
                }
                PlayerEvents::Error(_) => stop_timer(timer.clone(), time.clone(), tx.clone()),
            }

            let tx = tx.borrow_mut();
            tx(event);
        });
        self.unlisten = Some(unlisten);
    }
}
