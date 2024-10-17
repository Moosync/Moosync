use std::{cell::RefCell, rc::Rc};

use leptos::{
    html::{div, Div},
    NodeRef,
};
use regex::bytes::Regex;
use tokio::sync::oneshot::Sender as OneShotSender;
use types::errors::Result;
use types::{errors::MoosyncError, songs::SongType, ui::player_details::PlayerEvents};
use wasm_bindgen::{closure::Closure, JsValue};
use wasm_bindgen_futures::spawn_local;

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
                        tracing::error!("Error sending event: {:?}", e);
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
}

impl YoutubePlayer {
    #[tracing::instrument(level = "trace", skip())]
    pub fn new() -> Self {
        Self {
            player: Rc::new(YTPlayer::new("yt-player")),
        }
    }
}

impl std::fmt::Debug for YoutubePlayer {
    #[tracing::instrument(level = "trace", skip(self, f))]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("YoutubePlayer").finish()
    }
}

impl GenericPlayer for YoutubePlayer {
    #[tracing::instrument(level = "trace", skip(self, player_container))]
    fn initialize(&self, player_container: NodeRef<Div>) {
        player_container.on_load(move |elem| {
            let container_div = div();
            container_div.set_id("yt-player");
            elem.append_child(&container_div).unwrap();
        });
        tracing::debug!("Returning from YoutubePlayer initialize")
    }

    #[tracing::instrument(level = "trace", skip(self))]
    fn key(&self) -> String {
        "youtube".into()
    }

    #[tracing::instrument(level = "trace", skip(self, src, resolver))]
    fn load(&self, src: String, resolver: OneShotSender<()>) {
        self.player.load(src.as_str(), false);
        tracing::debug!("Loaded youtube embed {}", src);
        // TODO: Resolve when player state changes
        let res = resolver.send(());
        if res.is_err() {
            tracing::error!("Error sending resolver message: {:?}", res);
        }
    }

    #[tracing::instrument(level = "trace", skip(self))]
    fn play(&self) -> types::errors::Result<()> {
        self.player.play();
        Ok(())
    }

    #[tracing::instrument(level = "trace", skip(self))]
    fn pause(&self) -> types::errors::Result<()> {
        self.player.pause();
        Ok(())
    }

    #[tracing::instrument(level = "trace", skip(self, pos))]
    fn seek(&self, pos: f64) -> types::errors::Result<()> {
        self.player.seek(pos);
        Ok(())
    }

    #[tracing::instrument(level = "trace", skip(self))]
    fn provides(&self) -> &[types::songs::SongType] {
        &[SongType::YOUTUBE, SongType::SPOTIFY]
    }

    #[tracing::instrument(level = "trace", skip(self, song))]
    fn can_play(&self, song: &types::songs::Song) -> bool {
        let re = Regex::new(r"^[0-9A-Za-z_-]{10}[048AEIMQUYcgkosw]$").unwrap();
        re.is_match(song.song.playback_url.clone().unwrap().as_bytes())
    }

    #[tracing::instrument(level = "trace", skip(self, volume))]
    fn set_volume(&self, volume: f64) -> types::errors::Result<()> {
        tracing::debug!("Setting youtube volume {}", volume);
        self.player.setVolume(volume);
        Ok(())
    }

    #[tracing::instrument(level = "trace", skip(self))]
    fn get_volume(&self) -> types::errors::Result<f64> {
        Ok(self.player.getVolume())
    }

    #[tracing::instrument(level = "trace", skip(self, tx))]
    fn add_listeners(&mut self, tx: Rc<Box<dyn Fn(PlayerEvents)>>) {
        listen_event!(self, tx.clone(), "stateChange", f64, |state| {
            tracing::debug!("Youtube player Emitting {}", state);
            match state {
                0f64 => Ok(PlayerEvents::Ended),
                1f64 => Ok(PlayerEvents::Play),
                2f64 => Ok(PlayerEvents::Pause),
                3f64 => Ok(PlayerEvents::Loading),
                _ => Err(MoosyncError::String(format!(
                    "Youtube player ignoring event: {}",
                    state
                ))),
            }
        });

        listen_event!(self, tx.clone(), "timeUpdate", f64, |time| {
            Ok(PlayerEvents::TimeUpdate(time))
        });

        listen_event!(self, tx.clone(), "error", JsValue, |error| {
            Ok(PlayerEvents::Error(format!("{:?}", error).into()))
        })
    }

    #[tracing::instrument(level = "trace", skip(self))]
    fn stop(&mut self) -> Result<()> {
        self.pause()?;
        self.player.stop();
        self.player.removeAllListeners();
        Ok(())
    }
}
