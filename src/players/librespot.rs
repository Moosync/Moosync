use std::{cell::RefCell, rc::Rc, sync::Mutex, time::Duration};

use leptos::{leptos_dom::helpers::IntervalHandle, set_interval_with_handle};
use types::{preferences::CheckboxPreference, ui::player_details::PlayerEvents};
use wasm_bindgen::JsValue;
use wasm_bindgen_futures::spawn_local;

use crate::utils::{
    common::listen_event,
    invoke::{
        is_initialized, librespot_load, librespot_pause, librespot_play, librespot_seek,
        librespot_volume, load_selective,
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
                crate::utils::invoke::register_event($event_name.into())
                .await
                .unwrap();
            )*
        });
    };
}

type PlayerStateSender = Option<Rc<Box<dyn Fn(PlayerEvents)>>>;

#[derive(Clone)]
pub struct LibrespotPlayer {
    listeners: Vec<js_sys::Function>,
    timer: Rc<Mutex<Option<IntervalHandle>>>,
    time: Rc<Mutex<f64>>,
    player_state_tx: PlayerStateSender,
}

static ENABLED: Mutex<bool> = Mutex::new(false);
static INITIALIZED: Mutex<bool> = Mutex::new(false);

impl std::fmt::Debug for LibrespotPlayer {
    #[tracing::instrument(level = "trace", skip(self, f))]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LibrespotPlayer").finish()
    }
}

type Callback = RefCell<Rc<Box<dyn Fn(PlayerEvents)>>>;

impl LibrespotPlayer {
    #[tracing::instrument(level = "trace", skip())]
    pub fn new() -> Self {
        LibrespotPlayer {
            listeners: vec![],
            timer: Default::default(),
            time: Default::default(),
            player_state_tx: None,
        }
    }

    #[tracing::instrument(level = "trace", skip(en))]
    pub fn set_enabled(en: bool) {
        *(ENABLED.lock().unwrap()) = en;
        LibrespotPlayer::initialize_librespot();
    }

    #[tracing::instrument(level = "trace", skip(init))]
    pub fn set_initialized(init: bool) {
        *(INITIALIZED.lock().unwrap()) = init;
    }

    #[tracing::instrument(level = "trace", skip())]
    fn initialize_librespot() {
        if *ENABLED.lock().unwrap() {
            spawn_local(async move {
                let res = is_initialized().await;
                tracing::debug!("Librespot initialized: {:?}", res);
                if let Ok(initialized) = res {
                    *INITIALIZED.lock().unwrap() = initialized;
                    return;
                }

                *INITIALIZED.lock().unwrap() = false;
            })
        }
    }

    #[tracing::instrument(level = "trace", skip(timer, time, tx))]
    fn start_timer(timer: Rc<Mutex<Option<IntervalHandle>>>, time: Rc<Mutex<f64>>, tx: Callback) {
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
    }

    #[tracing::instrument(level = "trace", skip(timer))]
    fn stop_timer(timer: Rc<Mutex<Option<IntervalHandle>>>, _: Rc<Mutex<f64>>, _: Callback) {
        let mut timer = timer.lock().unwrap();
        if timer.is_some() {
            let handle = timer.unwrap();
            handle.clear();
        }
        *timer = None;
    }

    #[tracing::instrument(level = "trace", skip(timer, time, tx))]
    fn stop_and_clear_timer(
        timer: Rc<Mutex<Option<IntervalHandle>>>,
        time: Rc<Mutex<f64>>,
        tx: Callback,
    ) {
        let mut timer = timer.lock().unwrap();
        if timer.is_some() {
            let handle = timer.unwrap();
            handle.clear();
        }
        *timer = None;

        *time.lock().unwrap() = 0f64;
        let tx = tx.borrow_mut();
        tx(PlayerEvents::TimeUpdate(0f64));
    }
}

impl GenericPlayer for LibrespotPlayer {
    #[tracing::instrument(level = "trace", skip(self))]
    fn initialize(&self, _: leptos::NodeRef<leptos::html::Div>) {
        spawn_local(async move {
            let data = load_selective("spotify.enable".into()).await;

            let enabled: Vec<CheckboxPreference> = if let Ok(data) = data {
                serde_wasm_bindgen::from_value(data).unwrap_or(vec![CheckboxPreference {
                    key: "enable".into(),
                    enabled: true,
                }])
            } else {
                vec![CheckboxPreference {
                    key: "enable".into(),
                    enabled: true,
                }]
            };
            for pref in enabled {
                if pref.key == "enable" {
                    LibrespotPlayer::set_enabled(pref.enabled)
                }
            }
        });
    }

    #[tracing::instrument(level = "trace", skip(self))]
    fn key(&self) -> String {
        "spotify".into()
    }

    #[tracing::instrument(level = "trace", skip(self, src, resolver))]
    fn load(&self, src: String, autoplay: bool, resolver: tokio::sync::oneshot::Sender<()>) {
        let player_state_tx = self.player_state_tx.clone();
        spawn_local(async move {
            let res = librespot_load(src.clone(), false).await;
            if let Err(err) = res {
                if let Some(player_state_tx) = player_state_tx {
                    player_state_tx(PlayerEvents::Error(format!("{:?}", err).into()));
                }
            } else if autoplay {
                let _ = librespot_play().await;
            }

            resolver.send(()).unwrap();
        });
    }

    #[tracing::instrument(level = "trace", skip(self))]
    fn play(&self) -> types::errors::Result<()> {
        spawn_local(async move {
            let res = librespot_play().await;

            if res.is_err() {
                tracing::error!("Error playing {:?}", res.unwrap_err());
            }
        });
        Ok(())
    }

    #[tracing::instrument(level = "trace", skip(self))]
    fn pause(&self) -> types::errors::Result<()> {
        spawn_local(async move {
            let res = librespot_pause().await;

            if res.is_err() {
                tracing::error!("Error pausing {:?}", res.unwrap_err());
            }
        });
        Ok(())
    }

    #[tracing::instrument(level = "trace", skip(self, pos))]
    fn seek(&self, pos: f64) -> types::errors::Result<()> {
        let time = self.time.clone();
        spawn_local(async move {
            let res = librespot_seek((pos * 1000f64) as u32).await;
            if res.is_err() {
                tracing::error!("Error seeking to {}: {:?}", pos, res.unwrap_err());
                return;
            }

            *time.lock().unwrap() = pos;
        });

        Ok(())
    }

    #[tracing::instrument(level = "trace", skip(self))]
    fn provides(&self) -> &[types::songs::SongType] {
        &[types::songs::SongType::SPOTIFY]
    }

    #[tracing::instrument(level = "trace", skip(self, song))]
    fn can_play(&self, song: &types::songs::Song) -> bool {
        Self::initialize_librespot();
        *INITIALIZED.lock().unwrap() && song.song.type_ == types::songs::SongType::SPOTIFY
    }

    #[tracing::instrument(level = "trace", skip(self, volume))]
    fn set_volume(&self, volume: f64) -> types::errors::Result<()> {
        let parsed_volume = (volume / 100f64 * (u16::MAX as f64)) as u16;
        spawn_local(async move {
            let res = librespot_volume(parsed_volume).await;
            if res.is_err() {
                tracing::error!("Error setting volume {}: {:?}", volume, res.unwrap_err());
            }
        });

        Ok(())
    }

    #[tracing::instrument(level = "trace", skip(self))]
    fn get_volume(&self) -> types::errors::Result<f64> {
        Ok(0f64)
    }

    #[tracing::instrument(level = "trace", skip(self, tx))]
    fn add_listeners(
        &mut self,
        tx: std::rc::Rc<Box<dyn Fn(types::ui::player_details::PlayerEvents)>>,
    ) {
        self.player_state_tx = Some(tx.clone());
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

        listen_events!(
            self,
            tx.clone(),
            (
                "librespot_event_Stopped",
                PlayerEvents::Ended,
                Self::stop_and_clear_timer
            ),
            (
                "librespot_event_Playing",
                PlayerEvents::Play,
                Self::start_timer
            ),
            (
                "librespot_event_Paused",
                PlayerEvents::Pause,
                Self::stop_timer
            ),
            (
                "librespot_event_Loading",
                PlayerEvents::Loading,
                Self::stop_timer
            ),
            (
                "librespot_event_EndOfTrack",
                PlayerEvents::Ended,
                Self::stop_and_clear_timer
            ),
            (
                "librespot_event_Unavailable",
                PlayerEvents::Error("Track unavailable".into()),
                Self::stop_and_clear_timer
            ),
            (
                "librespot_event_TrackChanged",
                PlayerEvents::Loading,
                Self::stop_and_clear_timer
            ),
            (
                "SessionDisconnected",
                PlayerEvents::Error("Session ended".into()),
                Self::stop_timer
            )
        );
    }

    #[tracing::instrument(level = "trace", skip(self))]
    fn stop(&mut self) -> types::errors::Result<()> {
        self.pause()?;

        let timer = self.timer.clone();
        let time = self.time.clone();
        let tx = self.player_state_tx.clone();
        if let Some(tx) = tx {
            Self::stop_and_clear_timer(timer, time, RefCell::new(tx));
        }

        for listener in &self.listeners {
            let _ = listener.call0(&JsValue::undefined());
        }

        self.listeners.clear();

        Ok(())
    }
}
