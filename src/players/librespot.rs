use std::{cell::RefCell, rc::Rc, sync::Mutex, time::Duration, u16};

use leptos::{leptos_dom::helpers::IntervalHandle, set_interval_with_handle};
use serde::Serialize;
use types::{preferences::CheckboxPreference, ui::player_details::PlayerEvents};
use wasm_bindgen::JsValue;
use wasm_bindgen_futures::spawn_local;

use crate::{
    console_log,
    utils::{
        common::{invoke, listen_event},
        prefs::load_selective_async,
    },
};

use super::generic::GenericPlayer;

macro_rules! listen_events {
    ($self:expr, $tx:expr, $(($event_name:expr, $player_event:expr $(, $modify_self:expr)?)),* $(,)?) => {{
        let tx = RefCell::new($tx);

        $(
            let _timer = $self.timer.clone();
            let _time = $self.time.clone();
            let unlisten = listen_event($event_name, {
                let tx = tx.clone();
                move |_| {
                    let tx_clone = tx.clone();
                    $(
                        let timer = _timer.clone();
                        let time = _time.clone();
                        let tx = tx.clone();
                        $modify_self(timer, time, tx);
                    )?
                    let tx = tx_clone.borrow_mut();
                    tx($player_event);

                }
            });

            $self.listeners.push(unlisten);
        )*
    }};
}

macro_rules! register_events {
    ($($event_name:expr),* $(,)?) => {
        spawn_local(async move {
            $(
                invoke(
                    "register_event",
                    serde_wasm_bindgen::to_value(&RegisterEventArgs {
                        event: $event_name.into(),
                    })
                    .unwrap(),
                )
                .await
                .unwrap();
            )*
        });
    };
}

pub struct LibrespotPlayer {
    listeners: Vec<js_sys::Function>,
    timer: Rc<Mutex<Option<IntervalHandle>>>,
    time: Rc<Mutex<f64>>,
}

static ENABLED: Mutex<bool> = Mutex::new(false);
static HAS_USERNAME: Mutex<bool> = Mutex::new(false);
static HAS_PASSWORD: Mutex<bool> = Mutex::new(false);
static INITIALIZED: Mutex<bool> = Mutex::new(false);

impl std::fmt::Debug for LibrespotPlayer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LibrespotPlayer").finish()
    }
}

impl LibrespotPlayer {
    pub fn new() -> Self {
        LibrespotPlayer {
            listeners: vec![],
            timer: Default::default(),
            time: Default::default(),
        }
    }

    pub fn set_enabled(en: bool) {
        *(ENABLED.lock().unwrap()) = en;
        LibrespotPlayer::initialize_librespot();
    }

    pub fn set_has_username(en: bool) {
        *(HAS_USERNAME.lock().unwrap()) = en;
        LibrespotPlayer::initialize_librespot();
    }

    pub fn set_has_password(en: bool) {
        *(HAS_PASSWORD.lock().unwrap()) = en;
        LibrespotPlayer::initialize_librespot();
    }

    fn initialize_librespot() {
        if *ENABLED.lock().unwrap()
            && *HAS_USERNAME.lock().unwrap()
            && *HAS_PASSWORD.lock().unwrap()
        {
            spawn_local(async move {
                let res = invoke("initialize_librespot", JsValue::undefined()).await;
                if res.is_err() {
                    console_log!("Error initializing librespot: {:?}", res.unwrap_err());
                    return;
                }
                *INITIALIZED.lock().unwrap() = true;
            })
        }
    }
}

impl GenericPlayer for LibrespotPlayer {
    fn initialize(&self, element: leptos::NodeRef<leptos::html::Div>) {
        spawn_local(async move {
            let enabled: Vec<CheckboxPreference> =
                load_selective_async("spotify.enable".into()).await.unwrap();
            for pref in enabled {
                if pref.key == "enable" {
                    LibrespotPlayer::set_enabled(pref.enabled)
                }
            }
        });

        spawn_local(async move {
            let enabled: String = load_selective_async("spotify.username".into())
                .await
                .unwrap();
            LibrespotPlayer::set_has_username(!enabled.is_empty())
        });

        spawn_local(async move {
            let enabled: String = load_selective_async("spotify.password".into())
                .await
                .unwrap();
            LibrespotPlayer::set_has_password(!enabled.is_empty())
        });
    }

    fn key(&self) -> String {
        "spotify".into()
    }

    fn load(&self, src: String, resolver: tokio::sync::oneshot::Sender<()>) {
        spawn_local(async move {
            #[derive(Serialize)]
            struct LoadArgs {
                uri: String,
                autoplay: bool,
            }
            let res = invoke(
                "librespot_load",
                serde_wasm_bindgen::to_value(&LoadArgs {
                    uri: src.clone(),
                    autoplay: false,
                })
                .unwrap(),
            )
            .await;

            if res.is_err() {
                console_log!("Error loading track {}: {:?}", src, res.unwrap_err());
            }

            resolver.send(()).unwrap();
        });
    }

    fn play(&self) -> types::errors::errors::Result<()> {
        spawn_local(async move {
            let res = invoke("librespot_play", JsValue::undefined()).await;

            if res.is_err() {
                console_log!("Error playing {:?}", res.unwrap_err());
            }
        });
        Ok(())
    }

    fn pause(&self) -> types::errors::errors::Result<()> {
        spawn_local(async move {
            let res = invoke("librespot_pause", JsValue::undefined()).await;

            if res.is_err() {
                console_log!("Error pausing {:?}", res.unwrap_err());
            }
        });
        Ok(())
    }

    fn seek(&self, pos: f64) -> types::errors::errors::Result<()> {
        let time = self.time.clone();
        spawn_local(async move {
            #[derive(Serialize)]
            struct SeekArgs {
                pos: u32,
            }
            let res = invoke(
                "librespot_seek",
                serde_wasm_bindgen::to_value(&SeekArgs {
                    pos: (pos * 1000f64) as u32,
                })
                .unwrap(),
            )
            .await;

            if res.is_err() {
                console_log!("Error seeking to {}: {:?}", pos, res.unwrap_err());
                return;
            }

            *time.lock().unwrap() = pos;
        });

        Ok(())
    }

    fn provides(&self) -> &[types::songs::SongType] {
        &[types::songs::SongType::SPOTIFY]
    }

    fn can_play(&self, song: &types::songs::Song) -> bool {
        *INITIALIZED.lock().unwrap() && song.song.type_ == types::songs::SongType::SPOTIFY
    }

    fn set_volume(&self, volume: f64) -> types::errors::errors::Result<()> {
        let parsed_volume = (volume / 100f64 * (u16::MAX as f64)) as u16;
        spawn_local(async move {
            #[derive(Serialize)]
            struct VolumeArgs {
                volume: u16,
            }
            let res = invoke(
                "librespot_volume",
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

    fn get_volume(&self) -> types::errors::errors::Result<f64> {
        Ok(0f64)
    }

    fn add_listeners(
        &mut self,
        tx: std::rc::Rc<Box<dyn Fn(types::ui::player_details::PlayerEvents)>>,
    ) {
        #[derive(Serialize)]
        struct RegisterEventArgs {
            event: String,
        }
        register_events!(
            "librespot_event_Playing",
            "librespot_event_Paused",
            "librespot_event_Stopped",
            "librespot_event_Loading",
            "librespot_event_EndOfTrack",
            "librespot_event_Unavailable",
            "librespot_event_TrackChanged",
            "SessionDisconnected"
        );

        let start_timer =
            |timer: Rc<Mutex<Option<IntervalHandle>>>, time: Rc<Mutex<f64>>, tx: Callback| {
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
            let mut timer = timer.lock().unwrap();
            if timer.is_some() {
                let handle = timer.unwrap();
                handle.clear();
            }
            *timer = None;
        };

        let stop_and_clear_timer =
            |timer: Rc<Mutex<Option<IntervalHandle>>>, time: Rc<Mutex<f64>>, tx: Callback| {
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

        listen_events!(
            self,
            tx.clone(),
            (
                "librespot_event_Stopped",
                PlayerEvents::Ended,
                stop_and_clear_timer
            ),
            ("librespot_event_Playing", PlayerEvents::Play, start_timer),
            ("librespot_event_Paused", PlayerEvents::Pause, stop_timer),
            ("librespot_event_Loading", PlayerEvents::Loading, stop_timer),
            (
                "librespot_event_EndOfTrack",
                PlayerEvents::Ended,
                stop_and_clear_timer
            ),
            (
                "librespot_event_Unavailable",
                PlayerEvents::Ended,
                stop_and_clear_timer
            ),
            (
                "librespot_event_TrackChanged",
                PlayerEvents::Ended,
                stop_and_clear_timer
            ),
            ("SessionDisconnected", PlayerEvents::Ended, stop_timer)
        );
    }

    fn stop(&mut self) -> types::errors::errors::Result<()> {
        self.pause()?;

        for listener in &self.listeners {
            listener.call0(&JsValue::undefined());
        }

        self.listeners.clear();

        Ok(())
    }
}
