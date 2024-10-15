use std::{cell::RefCell, rc::Rc, sync::Mutex, time::Duration};

use leptos::{leptos_dom::helpers::IntervalHandle, set_interval_with_handle, spawn_local};
use serde::Serialize;
use types::{songs::SongType, ui::player_details::PlayerEvents};
use wasm_bindgen::JsValue;

use crate::{
    console_log,
    utils::common::{convert_file_src, invoke, listen_event},
};

use super::generic::GenericPlayer;

#[derive(Debug, Clone)]
pub struct RodioPlayer {
    unlisten: Option<js_sys::Function>,
    timer: Rc<Mutex<Option<IntervalHandle>>>,
    time: Rc<Mutex<f64>>,
}

impl RodioPlayer {
    pub fn new() -> Self {
        Self {
            unlisten: None,
            timer: Default::default(),
            time: Default::default(),
        }
    }
}

impl GenericPlayer for RodioPlayer {
    fn initialize(&self, _: leptos::NodeRef<leptos::html::Div>) {}

    fn key(&self) -> String {
        "rodio".into()
    }

    fn load(&self, src: String, resolver: tokio::sync::oneshot::Sender<()>) {
        spawn_local(async move {
            #[derive(Serialize)]
            struct RodioLoadArgs {
                src: String,
            }
            let res = invoke(
                "rodio_load",
                serde_wasm_bindgen::to_value(&RodioLoadArgs { src }).unwrap(),
            )
            .await;

            if let Err(err) = res {
                console_log!("Rodio error {:?}", err);
            }

            resolver.send(()).unwrap();
        });
    }

    fn stop(&mut self) -> types::errors::Result<()> {
        spawn_local(async move {
            let res = invoke("rodio_stop", JsValue::undefined()).await;

            if res.is_err() {
                console_log!("Error stopping {:?}", res.unwrap_err());
            }
        });

        let mut timer = self.timer.lock().unwrap();
        if timer.is_some() {
            let handle = timer.unwrap();
            handle.clear();
        }
        *timer = None;

        *self.time.lock().unwrap() = 0f64;

        let unlisten = self.unlisten.take();
        if let Some(unlisten) = &unlisten {
            if let Err(e) = unlisten.call0(&JsValue::undefined()) {
                console_log!("Error removing listeners {:?}", e);
            }
        }
        Ok(())
    }

    fn play(&self) -> types::errors::Result<()> {
        spawn_local(async move {
            let res = invoke("rodio_play", JsValue::undefined()).await;

            if res.is_err() {
                console_log!("Error playing {:?}", res.unwrap_err());
            }
        });
        Ok(())
    }

    fn pause(&self) -> types::errors::Result<()> {
        spawn_local(async move {
            let res = invoke("rodio_pause", JsValue::undefined()).await;

            if res.is_err() {
                console_log!("Error playing {:?}", res.unwrap_err());
            }
        });
        Ok(())
    }

    fn seek(&self, pos: f64) -> types::errors::Result<()> {
        spawn_local(async move {
            #[derive(Serialize)]
            struct SeekArgs {
                pos: f64,
            }

            let res = invoke(
                "rodio_seek",
                serde_wasm_bindgen::to_value(&SeekArgs { pos }).unwrap(),
            )
            .await;

            if res.is_err() {
                console_log!("Error playing {:?}", res.unwrap_err());
            }
        });
        Ok(())
    }

    fn provides(&self) -> &[types::songs::SongType] {
        &[
            SongType::LOCAL,
            SongType::URL,
            SongType::YOUTUBE,
            SongType::SPOTIFY,
        ]
    }

    fn can_play(&self, song: &types::songs::Song) -> bool {
        let playback_url = song
            .song
            .path
            .clone()
            .map(convert_file_src)
            .or(song.song.playback_url.clone());
        console_log!("Checking playback url {:?}", playback_url);
        if let Some(playback_url) = playback_url {
            return playback_url.starts_with("http://")
                || playback_url.starts_with("https://")
                || playback_url.starts_with("asset");
        }
        false
    }

    fn set_volume(&self, volume: f64) -> types::errors::Result<()> {
        let parsed_volume = volume / 100f64;
        console_log!("Setting volume {}", parsed_volume);
        spawn_local(async move {
            #[derive(Serialize)]
            struct VolumeArgs {
                volume: f64,
            }
            let res = invoke(
                "rodio_set_volume",
                serde_wasm_bindgen::to_value(&VolumeArgs {
                    volume: parsed_volume,
                })
                .unwrap(),
            )
            .await;

            if res.is_err() {
                console_log!("Error setting volume {}: {:?}", volume, res.unwrap_err());
            }
        });

        Ok(())
    }

    fn get_volume(&self) -> types::errors::Result<f64> {
        Ok(0f64)
    }

    fn add_listeners(
        &mut self,
        state_setter: std::rc::Rc<Box<dyn Fn(types::ui::player_details::PlayerEvents)>>,
    ) {
        console_log!("Adding rodio listeners");
        let start_timer =
            |timer: Rc<Mutex<Option<IntervalHandle>>>, time: Rc<Mutex<f64>>, tx: Callback| {
                console_log!("Starting timer");
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

        type Callback = RefCell<Rc<Box<dyn Fn(PlayerEvents)>>>;

        let stop_timer = |timer: Rc<Mutex<Option<IntervalHandle>>>, _, _| {
            console_log!("pausing timer");
            let mut timer = timer.lock().unwrap();
            if timer.is_some() {
                let handle = timer.unwrap();
                handle.clear();
            }
            *timer = None;
        };

        let stop_and_clear_timer =
            |timer: Rc<Mutex<Option<IntervalHandle>>>, time: Rc<Mutex<f64>>, tx: Callback| {
                console_log!("Stopping timer");
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
            console_log!("Got rodio event {:?}", data);
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
