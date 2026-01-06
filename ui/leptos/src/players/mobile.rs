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

use leptos::{html::Div, prelude::*, task::spawn_local};

use crate::utils::error::Result;
use extensions_proto::moosync::types::player_event::Event as PlayerEvents;
use serde::Deserialize;
use songs_proto::moosync::types::{Song, SongType};
use tokio::sync::oneshot::Sender as OneShotSender;
use types::prelude::SongsExt;
use wasm_bindgen::JsValue;

use crate::utils::{
    common::listen_plugin_event,
    invoke::{mobile_load, mobile_pause, mobile_play, mobile_seek, mobile_stop},
};

use super::generic::{GenericPlayer, PlayerEventsSender};

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
    event_tx: Option<PlayerEventsSender>,
}

macro_rules! listen_event {
    ($self:expr, $tx:expr, $event:expr, $typ:ident, $handler:expr) => {{
        let key = $self.key.clone();
        let unlisten = listen_plugin_event("audioplayer", $event, move |evt| {
            let tx = $tx.clone();
            let key = key.clone();
            let data = serde_wasm_bindgen::from_value::<$typ>(evt).unwrap();
            if data.key == key {
                spawn_local(async move {
                    let val = $handler(data);
                    let _ = tx(key, val);
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
            fn $method(&mut self, tx: PlayerEventsSender) {
                listen_event!(self, tx, $event, $typ, $handler);
            }
        )*
    };
}

impl std::fmt::Debug for MobilePlayer {
    #[tracing::instrument(level = "debug", skip(self, f))]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LocalPlayer").finish()
    }
}

impl MobilePlayer {
    #[tracing::instrument(level = "debug", skip())]
    pub fn new(key: String) -> Self {
        MobilePlayer {
            key,
            listeners: vec![],
            event_tx: None,
        }
    }

    generate_event_listeners!(
        listen_onplay => ("onPlay", KeyEvent) => |_| PlayerEvents::Play(true),
        listen_onpause => ("onPause", KeyEvent) => |_| PlayerEvents::Pause(true),
        listen_onended => ("onSongEnded", KeyEvent) => |_| PlayerEvents::Ended(true),
        listen_ontimeupdate => ("onTimeChange", TimeChangeEvent) => |evt: TimeChangeEvent|{
            PlayerEvents::TimeUpdate(evt.pos / 1000f64)
        }
    );
}

impl GenericPlayer for MobilePlayer {
    #[tracing::instrument(level = "debug", skip(self, _player_container))]
    fn initialize(&self, _player_container: NodeRef<Div>) {
        tracing::debug!("Returning from mobile player initialize")
    }

    #[tracing::instrument(level = "debug", skip(self))]
    fn key(&self) -> String {
        self.key.clone()
    }

    #[tracing::instrument(level = "debug", skip(self, src, resolver))]
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

    #[tracing::instrument(level = "debug", skip(self))]
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

    #[tracing::instrument(level = "debug", skip(self))]
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

    #[tracing::instrument(level = "debug", skip(self))]
    fn provides(&self) -> &[SongType] {
        &[SongType::Local, SongType::Url, SongType::Spotify]
    }

    #[tracing::instrument(level = "debug", skip(self, _volume))]
    fn set_volume(&self, _volume: f64) -> Result<()> {
        Ok(())
    }

    #[tracing::instrument(level = "debug", skip(self))]
    fn get_volume(&self) -> Result<f64> {
        Ok(100f64)
    }

    #[tracing::instrument(level = "debug", skip(self, tx))]
    fn add_listeners(&mut self, tx: PlayerEventsSender) {
        self.listen_onplay(tx.clone());
        self.listen_onpause(tx.clone());
        self.listen_onended(tx.clone());
        self.listen_ontimeupdate(tx.clone());
        self.event_tx = Some(tx);
    }

    #[tracing::instrument(level = "debug", skip(self, pos))]
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

    #[tracing::instrument(level = "debug", skip(self, song))]
    fn can_play(&self, song: &Song) -> bool {
        let playback_url = song.get_path().or(song.get_playback_url());
        tracing::debug!("Checking playback url {:?}", playback_url);
        if let Some(playback_url) = playback_url {
            if self.key == "LOCAL" {
                return playback_url.starts_with("http://")
                    || playback_url.starts_with("https://")
                    || playback_url.parse::<u64>().is_ok();
            }

            if self.key == "LIBRESPOT" {
                return playback_url.starts_with("spotify:");
            }
        }

        false
    }

    #[tracing::instrument(level = "debug", skip(self))]
    fn stop(&mut self) -> Result<()> {
        let key = self.key.clone();
        spawn_local(async move {
            let res = mobile_stop(key).await;
            if let Err(e) = res {
                tracing::error!("Failed to stop in mobile player {:?}", e);
            }
        });

        for listener in self.listeners.iter() {
            let _ = listener.call0(&JsValue::NULL);
        }
        self.listeners.clear();
        self.event_tx = None;
        Ok(())
    }
}
