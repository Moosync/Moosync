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

use std::{cell::RefCell, ops::Deref, rc::Rc};

use leptos::{
    html::{div, Div},
    prelude::*,
};
use regex::bytes::Regex;
use tokio::sync::oneshot::Sender as OneShotSender;
use types::errors::Result;
use types::{errors::MoosyncError, songs::SongType, ui::player_details::PlayerEvents};
use wasm_bindgen::{closure::Closure, JsCast, JsValue};
use wasm_bindgen_futures::spawn_local;
use web_sys::HtmlDivElement;

use crate::utils::yt_player::YTPlayer;

use super::generic::GenericPlayer;

macro_rules! listen_event {
    ($self:expr, $tx:expr, $event:tt, $data: ident, $handler:expr) => {{
        let tx = Rc::new(RefCell::new($tx));
        let callback = Closure::wrap(Box::new(move |data: $data| {
            let tx = tx.clone();
            spawn_local(async move {
                let tx = tx.borrow_mut();
                let val: Result<PlayerEvents> = $handler(data);
                match val {
                    Ok(val) => {
                        tx(val);
                    }
                    Err(e) => {
                        tracing::warn!("Error sending event: {:?}", e);
                    }
                };
            });
        }) as Box<dyn Fn($data)>);

        let js_value = callback.into_js_value();
        $self.player.on($event, &js_value)
    }};
}

#[derive(Clone)]
pub struct YoutubePlayer {
    player: Rc<YTPlayer>,
    force_play: RwSignal<bool>,
    reload_audio: RwSignal<bool>,
    last_src: RwSignal<Option<String>>,
}

impl YoutubePlayer {
    #[tracing::instrument(level = "debug", skip())]
    pub fn new() -> Self {
        Self {
            player: Rc::new(YTPlayer::new("yt-player")),
            force_play: RwSignal::new(false),
            reload_audio: RwSignal::new(false),
            last_src: RwSignal::new(None),
        }
    }
}

impl std::fmt::Debug for YoutubePlayer {
    #[tracing::instrument(level = "debug", skip(self, f))]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("YoutubePlayer").finish()
    }
}

impl GenericPlayer for YoutubePlayer {
    #[tracing::instrument(level = "debug", skip(self, player_container))]
    fn initialize(&self, player_container: NodeRef<Div>) {
        player_container.on_load(move |elem| {
            let node_ref = NodeRef::new();
            let container_div = div().node_ref(node_ref).id("yt-player");
            let build = container_div.build();
            let el: &HtmlDivElement = build.deref().unchecked_ref();
            elem.append_child(el).unwrap();
        });
        tracing::debug!("Returning from YoutubePlayer initialize");

        let force_play_sig = self.force_play;
        let player = self.player.clone();
        Effect::new(move || {
            let force_play = force_play_sig.get();
            if force_play {
                force_play_sig.set(false);
                player.play();
            }
        });

        let reload_audio_sig = self.reload_audio;
        let last_src = self.last_src;
        let player = self.player.clone();
        Effect::new(move || {
            let reload_audio = reload_audio_sig.get();
            let last_src = last_src.get();
            if let Some(last_src) = last_src {
                if reload_audio {
                    reload_audio_sig.set(false);
                    player.load(last_src.as_str(), true);
                }
            }
        });
    }

    #[tracing::instrument(level = "debug", skip(self))]
    fn key(&self) -> String {
        "youtube".into()
    }

    #[tracing::instrument(level = "debug", skip(self, src, resolver))]
    fn load(&self, src: String, autoplay: bool, resolver: OneShotSender<()>) {
        self.player.load(src.as_str(), false);
        self.last_src.set(Some(src.clone()));
        tracing::debug!("Loaded youtube embed {}, {}", src, autoplay);
        if autoplay {
            self.play().unwrap();
        } else {
            self.pause().unwrap();
        }
        // TODO: Resolve when player state changes
        let res = resolver.send(());
        if res.is_err() {
            tracing::error!("Error sending resolver message: {:?}", res);
        }
    }

    #[tracing::instrument(level = "debug", skip(self))]
    fn play(&self) -> types::errors::Result<()> {
        tracing::debug!("Youtube player playing");
        self.player.play();
        Ok(())
    }

    #[tracing::instrument(level = "debug", skip(self))]
    fn pause(&self) -> types::errors::Result<()> {
        self.player.pause();
        Ok(())
    }

    #[tracing::instrument(level = "debug", skip(self, pos))]
    fn seek(&self, pos: f64) -> types::errors::Result<()> {
        self.player.seek(pos);
        Ok(())
    }

    #[tracing::instrument(level = "debug", skip(self))]
    fn provides(&self) -> &[types::songs::SongType] {
        &[SongType::YOUTUBE, SongType::SPOTIFY]
    }

    #[tracing::instrument(level = "debug", skip(self, song))]
    fn can_play(&self, song: &types::songs::Song) -> bool {
        let re = Regex::new(r"^[0-9A-Za-z_-]{10}[048AEIMQUYcgkosw]$").unwrap();
        re.is_match(song.song.playback_url.clone().unwrap().as_bytes())
    }

    #[tracing::instrument(level = "debug", skip(self, volume))]
    fn set_volume(&self, volume: f64) -> types::errors::Result<()> {
        tracing::debug!("Setting youtube volume {}", volume);
        self.player.setVolume(volume);
        Ok(())
    }

    #[tracing::instrument(level = "debug", skip(self))]
    fn get_volume(&self) -> types::errors::Result<f64> {
        Ok(self.player.getVolume())
    }

    #[tracing::instrument(level = "debug", skip(self, tx))]
    fn add_listeners(&mut self, tx: Rc<Box<dyn Fn(PlayerEvents)>>) {
        let force_play = self.force_play;
        listen_event!(self, tx.clone(), "stateChange", f64, |state| {
            tracing::debug!("Youtube player Emitting {}", state);
            match state {
                0f64 => Ok(PlayerEvents::Ended),
                1f64 => Ok(PlayerEvents::Play),
                2f64 => Ok(PlayerEvents::Pause),
                3f64 => {
                    force_play.set(true);
                    Ok(PlayerEvents::Loading)
                }
                _ => Err(MoosyncError::String(format!(
                    "Youtube player ignoring event: {}",
                    state
                ))),
            }
        });

        listen_event!(self, tx.clone(), "timeUpdate", f64, |time| {
            Ok(PlayerEvents::TimeUpdate(time))
        });

        let reload_audio = self.reload_audio;
        listen_event!(self, tx.clone(), "error", JsValue, |error: JsValue| {
            if let Some(err) = error.as_f64() {
                if err == 2f64 {
                    reload_audio.set(true);
                    return Err("Youtube player error (2), trying to reload audio".into());
                }
            }
            Ok(PlayerEvents::Error(format!("{:?}", error).into()))
        })
    }

    #[tracing::instrument(level = "debug", skip(self))]
    fn stop(&mut self) -> Result<()> {
        self.pause()?;
        self.player.stop();
        self.player.removeAllListeners();
        self.last_src.set(None);
        Ok(())
    }
}
