use std::rc::Rc;

use leptos::{
    create_node_ref,
    ev::{ended, loadeddata, loadstart, pause, play, timeupdate},
    event_target,
    html::{audio, Audio, Div},
    spawn_local, HtmlElement, NodeRef,
};

use leptos_use::use_event_listener;
use tokio::sync::oneshot::Sender as OneShotSender;
use types::{errors::errors::Result, songs::SongType, ui::player_details::PlayerEvents};
use wasm_bindgen_futures::JsFuture;

use crate::{
    console_log,
    utils::common::{convert_file_src, get_blob_url},
};

use super::generic::GenericPlayer;

macro_rules! listen_event {
    ($self:expr, $tx:expr, $event:ident, $handler:expr) => {{
        let unlisten = use_event_listener($self.node_ref, $event, move |evt| {
            let tx = $tx.clone();
            spawn_local(async move {
                let val = $handler(evt);
                let res = tx(val);
                // if let Err(res) = res {
                //     console_log!("Error sending event: {:?}", res);
                // }
            });
        });
        $self.listeners.push(Box::new(unlisten));
    }};
}

macro_rules! generate_event_listeners {
    ($($method:tt => $event:ident => $handler:expr),*) => {
        $(
            fn $method(&mut self, tx: Rc<Box<dyn Fn(PlayerEvents)>>) {
                listen_event!(self, tx, $event, $handler);
            }
        )*
    };
}

pub struct LocalPlayer {
    pub audio_element: HtmlElement<Audio>,
    node_ref: NodeRef<Audio>,
    listeners: Vec<Box<dyn Fn()>>,
}

impl std::fmt::Debug for LocalPlayer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LocalPlayer")
            .field("audio_element", &self.audio_element.tag_name())
            .finish()
    }
}

impl LocalPlayer {
    pub fn new() -> Self {
        let mut audio_element = audio();
        let node_ref = create_node_ref();

        audio_element = audio_element.node_ref(node_ref);

        LocalPlayer {
            audio_element,
            node_ref,
            listeners: vec![],
        }
    }

    generate_event_listeners!(
        listen_onplay => play => |_| PlayerEvents::Play,
        listen_onpause => pause => |_| PlayerEvents::Pause,
        listen_onended => ended => |_| PlayerEvents::Ended,
        listen_onloadstart => loadstart => |_| PlayerEvents::Loading,
        listen_onloadend => loadeddata => |_| PlayerEvents::Play,
        listen_ontimeupdate => timeupdate => |evt|{
            let target = event_target::<leptos::web_sys::HtmlAudioElement>(&evt);
            let time = target.current_time();
            PlayerEvents::TimeUpdate(time)
        }
    );
}

impl GenericPlayer for LocalPlayer {
    fn initialize(&self, player_container: NodeRef<Div>) {
        let node_ref = self.node_ref;
        player_container.on_load(move |elem| {
            let audio_elem = node_ref.get().unwrap();
            if let Err(e) = elem.append_child(&audio_elem) {
                console_log!("Error initializing local player: {:?}", e);
            }
        });
        console_log!("Returning from local player initialize")
    }

    fn key(&self) -> String {
        "local".into()
    }

    fn load(&self, src: String, resolver: OneShotSender<()>) {
        let mut src = convert_file_src(src);
        console_log!("Loading audio {}", src);

        let audio_element = self.audio_element.clone();
        spawn_local(async move {
            if src.starts_with("asset") {
                src = get_blob_url(src).await;
            }

            audio_element.set_src(src.as_str());
            audio_element.load();

            resolver.send(()).expect("Load failed to resolve");
        });
    }

    fn play(&self) -> Result<()> {
        let promise = self.audio_element.play()?;
        spawn_local(async move {
            JsFuture::from(promise).await.unwrap();
        });
        Ok(())
    }

    fn pause(&self) -> Result<()> {
        self.audio_element.pause()?;
        Ok(())
    }

    fn provides(&self) -> &[SongType] {
        &[
            SongType::LOCAL,
            SongType::URL,
            SongType::YOUTUBE,
            SongType::SPOTIFY,
        ]
    }

    fn set_volume(&self, volume: f64) -> Result<()> {
        self.audio_element.set_volume(volume);
        Ok(())
    }

    fn get_volume(&self) -> Result<f64> {
        Ok(self.audio_element.volume())
    }

    fn add_listeners(&mut self, tx: Rc<Box<dyn Fn(PlayerEvents)>>) {
        self.listen_onplay(tx.clone());
        self.listen_onpause(tx.clone());
        self.listen_onended(tx.clone());
        self.listen_onloadstart(tx.clone());
        self.listen_onloadend(tx.clone());
        self.listen_ontimeupdate(tx.clone());
    }

    fn seek(&self, pos: f64) -> Result<()> {
        Ok(self.audio_element.fast_seek(pos)?)
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
}
