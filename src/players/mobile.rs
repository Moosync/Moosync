use std::{collections::HashMap, rc::Rc};

use leptos::{
    create_node_ref,
    html::{audio, Div},
    spawn_local, NodeRef,
};

use serde::Deserialize;
use tokio::sync::oneshot::Sender as OneShotSender;
use types::{errors::Result, songs::SongType, ui::player_details::PlayerEvents};
use wasm_bindgen::JsValue;

use crate::utils::{
    common::{addPluginListener, listen_plugin_event},
    invoke::{mobile_load, mobile_pause, mobile_play, mobile_seek, mobile_stop},
};

use super::generic::GenericPlayer;

#[derive(Deserialize)]
struct TimeChangeEvent {
    key: String,
    pos: f64,
}

#[derive(Deserialize)]
struct KeyEvent {
    key: String,
}

#[derive(Clone)]
pub struct MobilePlayer {
    key: String,
    listeners: Vec<js_sys::Function>,
    event_tx: Option<Rc<Box<dyn Fn(PlayerEvents)>>>,
}

macro_rules! listen_event {
    ($self:expr, $tx:expr, $event:expr, $typ:ident, $handler:expr) => {{
        let key = $self.key.clone();
        let unlisten = listen_plugin_event("audioplayer", $event, move |evt| {
            let tx = $tx.clone();
            let data = serde_wasm_bindgen::from_value::<$typ>(evt).unwrap();
            if data.key == key {
                spawn_local(async move {
                    let val = $handler(data);
                    let _ = tx(val);
                    // if let Err(res) = res {
                    //     console_log!("Error sending event: {:?}", res);
                    // }
                });
            }
        });
        $self.listeners.push(unlisten);
    }};
}

macro_rules! generate_event_listeners {
    ($($method:tt => ($event:expr, $typ:ident) => $handler:expr),*) => {
        $(
            fn $method(&mut self, tx: Rc<Box<dyn Fn(PlayerEvents)>>) {
                listen_event!(self, tx, $event, $typ, $handler);
            }
        )*
    };
}

impl std::fmt::Debug for MobilePlayer {
    #[tracing::instrument(level = "trace", skip(self, f))]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LocalPlayer").finish()
    }
}

impl MobilePlayer {
    #[tracing::instrument(level = "trace", skip())]
    pub fn new(key: String) -> Self {
        let mut audio_element = audio();
        let node_ref = create_node_ref();

        audio_element = audio_element.node_ref(node_ref);

        MobilePlayer {
            key,
            listeners: vec![],
            event_tx: None,
        }
    }

    generate_event_listeners!(
        listen_onplay => ("onPlay", KeyEvent) => |_| PlayerEvents::Play,
        listen_onpause => ("onPause", KeyEvent) => |_| PlayerEvents::Pause,
        listen_onended => ("onSongEnded", KeyEvent) => |_| PlayerEvents::Ended,
        listen_ontimeupdate => ("onTimeChange", TimeChangeEvent) => |evt: TimeChangeEvent|{
            PlayerEvents::TimeUpdate(evt.pos)
        }
    );
}

impl GenericPlayer for MobilePlayer {
    #[tracing::instrument(level = "trace", skip(self, _player_container))]
    fn initialize(&self, _player_container: NodeRef<Div>) {
        tracing::debug!("Returning from mobile player initialize")
    }

    #[tracing::instrument(level = "trace", skip(self))]
    fn key(&self) -> String {
        self.key.clone()
    }

    #[tracing::instrument(level = "trace", skip(self, src, resolver))]
    fn load(&self, src: String, autoplay: bool, resolver: OneShotSender<()>) {
        tracing::debug!("Loading audio {}", src);

        let key = self.key.clone();
        spawn_local(async move {
            let res = mobile_load(key, src, autoplay).await;
            if let Err(e) = res {
                tracing::error!("Failed to load audio in mobile player {:?}", e);
            }
            resolver.send(()).expect("Load failed to resolve");
        });
    }

    #[tracing::instrument(level = "trace", skip(self))]
    fn play(&self) -> Result<()> {
        let key = self.key.clone();
        spawn_local(async move {
            let res = mobile_play(key).await;
            if let Err(e) = res {
                tracing::error!("Failed to play in mobile player {:?}", e);
            }
        });
        Ok(())
    }

    #[tracing::instrument(level = "trace", skip(self))]
    fn pause(&self) -> Result<()> {
        let key = self.key.clone();
        spawn_local(async move {
            let res = mobile_pause(key).await;
            if let Err(e) = res {
                tracing::error!("Failed to pause in mobile player {:?}", e);
            }
        });
        Ok(())
    }

    #[tracing::instrument(level = "trace", skip(self))]
    fn provides(&self) -> &[SongType] {
        &[
            SongType::LOCAL,
            SongType::URL,
            SongType::YOUTUBE,
            SongType::SPOTIFY,
        ]
    }

    #[tracing::instrument(level = "trace", skip(self, _volume))]
    fn set_volume(&self, _volume: f64) -> Result<()> {
        Ok(())
    }

    #[tracing::instrument(level = "trace", skip(self))]
    fn get_volume(&self) -> Result<f64> {
        Ok(100f64)
    }

    #[tracing::instrument(level = "trace", skip(self, tx))]
    fn add_listeners(&mut self, tx: Rc<Box<dyn Fn(PlayerEvents)>>) {
        self.listen_onplay(tx.clone());
        self.listen_onpause(tx.clone());
        self.listen_onended(tx.clone());
        self.listen_ontimeupdate(tx.clone());
        self.event_tx = Some(tx);
    }

    #[tracing::instrument(level = "trace", skip(self, pos))]
    fn seek(&self, pos: f64) -> Result<()> {
        let key = self.key.clone();
        spawn_local(async move {
            let res = mobile_seek(key, pos).await;
            if let Err(e) = res {
                tracing::error!("Failed to seek in mobile player {:?}", e);
            }
        });
        Ok(())
    }

    #[tracing::instrument(level = "trace", skip(self, song))]
    fn can_play(&self, song: &types::songs::Song) -> bool {
        let playback_url = song.song.path.clone().or(song.song.playback_url.clone());
        tracing::debug!("Checking playback url {:?}", playback_url);
        if let Some(playback_url) = playback_url {
            if self.key == "LOCAL" {
                return playback_url.starts_with("http://")
                    || playback_url.starts_with("https://")
                    || playback_url.parse::<u64>().is_ok();
            }

            if self.key == "YOUTUBE" {
                return playback_url.len() == 11 || playback_url.starts_with("https://");
            }
        }

        false
    }

    #[tracing::instrument(level = "trace", skip(self))]
    fn stop(&mut self) -> Result<()> {
        let key = self.key.clone();
        spawn_local(async move {
            let res = mobile_stop(key).await;
            if let Err(e) = res {
                tracing::error!("Failed to stop in mobile player {:?}", e);
            }
        });

        for listener in self.listeners.iter() {
            listener.call0(&JsValue::NULL);
        }
        self.listeners.clear();
        self.event_tx = None;
        Ok(())
    }
}
