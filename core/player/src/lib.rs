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

pub(crate) mod audio_source;
mod error;
mod generic;
mod mux_player;
mod rodio;
mod source;

#[cfg(test)]
mod test;

use std::{sync::Arc, time::Duration};

use extensions_proto::moosync::types::{PlayerEvent, player_event::Event};
use songs_proto::moosync::types::Song;
use tokio::{
    sync::mpsc::{UnboundedSender, unbounded_channel},
    time::interval,
};
use tracing::debug;
use types::{
    plugin::{Plugin, PluginContext, RwLock},
    prelude::core_to_proto_duration,
};

use crate::audio_source::AudioSource;

pub type OnEndedCallback = Box<dyn Fn() -> () + Send + Sync + 'static>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RepeatMode {
    None,
    Once,
    Infinite,
}

pub type OnSongChangedCallback = Box<dyn Fn(Option<&Song>) -> () + Send + Sync + 'static>;
pub type OnQueueUpdatedCallback = Box<dyn Fn(&[Song]) -> () + Send + Sync + 'static>;
pub type OnRepeatChangedCallback = Box<dyn Fn(RepeatMode) -> () + Send + Sync + 'static>;
pub type OnPlayerEventCallback = Box<dyn Fn(&PlayerEvent) -> () + Send + Sync + 'static>;

pub struct PlayerHandler {
    pub(crate) song_queue: Vec<Song>,
    pub(crate) current_idx: usize,
    pub(crate) repeat_mode: RepeatMode,
    pub(crate) player: AudioSource,
    on_song_changed_subs: Vec<OnSongChangedCallback>,
    on_queue_updated_subs: Vec<OnQueueUpdatedCallback>,
    on_repeat_changed_subs: Vec<OnRepeatChangedCallback>,
    on_player_event_subs: Vec<OnPlayerEventCallback>,
}

#[plugin_macro::generate]
impl PlayerHandler {
    pub fn new(ended_tx: UnboundedSender<()>) -> Self {
        PlayerHandler {
            song_queue: vec![],
            current_idx: 0,
            repeat_mode: RepeatMode::None,
            player: AudioSource::new(Box::new(move || {
                let _ = ended_tx.send(());
            })),
            on_song_changed_subs: vec![],
            on_queue_updated_subs: vec![],
            on_repeat_changed_subs: vec![],
            on_player_event_subs: vec![],
        }
    }

    pub fn get_player_state(&self) -> i32 { self.player.get_player_state() as i32 }

    pub fn get_volume(&self) -> u8 { self.player.get_volume() }

    pub fn get_queue(&self) -> &[Song] { &self.song_queue }

    pub fn get_current_idx(&self) -> usize { self.current_idx }

    pub fn get_current_pos(&self) -> Result<Duration, crate::error::PlayerError> {
        self.player.get_current_pos()
    }

    pub fn current_song(&self) -> Option<&Song> { self.song_queue.get(self.current_idx) }

    pub fn get_current_song(&self) -> Option<&Song> { self.current_song() }

    pub fn get_repeat_mode(&self) -> RepeatMode { self.repeat_mode }

    pub fn add_to_queue(&mut self, song: Song) {
        self.song_queue.push(song);
        self.trigger_queue_updated();
    }

    pub fn play_now(&mut self, song: Song) {
        debug!("Playing song now: {:?}", song);
        if self.song_queue.is_empty() {
            self.song_queue.push(song.clone());
            self.current_idx = 0;
        } else {
            let insert_pos = self.current_idx;
            self.song_queue.insert(insert_pos, song.clone());
            self.current_idx = insert_pos;
        }
        self.trigger_queue_updated();
        self.trigger_song_changed();

        if let Err(e) = self.play() {
            tracing::error!("Failed to play song: {:?}", e);
        }
    }

    pub fn shuffle(&mut self) {
        if self.song_queue.len() <= 1 {
            return;
        }

        let current_song = self.song_queue.remove(self.current_idx);

        use rand::seq::SliceRandom;
        let mut rng = rand::thread_rng();
        self.song_queue.shuffle(&mut rng);

        self.song_queue.insert(self.current_idx, current_song);
        self.trigger_queue_updated();
    }

    pub fn repeat(&mut self, mode: RepeatMode) {
        self.repeat_mode = mode;
        self.trigger_repeat_changed();
    }

    pub fn play(&mut self) -> Result<(), crate::error::PlayerError> {
        self.player.play()?;
        self.trigger_player_event(PlayerEvent {
            event: Some(Event::Play(true)),
        });
        Ok(())
    }

    pub fn pause(&mut self) -> Result<(), crate::error::PlayerError> {
        self.player.pause()?;
        self.trigger_player_event(PlayerEvent {
            event: Some(Event::Pause(true)),
        });
        Ok(())
    }

    pub fn next(&mut self) {
        if self.song_queue.is_empty() {
            let _ = self.player.stop();
            return;
        }
        if self.current_idx + 1 < self.song_queue.len() {
            self.current_idx += 1;
        } else {
            self.current_idx = 0;
        }
        self.trigger_song_changed();
        if let Err(e) = self.play() {
            tracing::error!("Failed to play song: {:?}", e);
        }
    }

    pub fn prev(&mut self) {
        if self.song_queue.is_empty() {
            return;
        }
        if self.current_idx > 0 {
            self.current_idx -= 1;
            self.trigger_song_changed();
            if let Err(e) = self.play() {
                tracing::error!("Failed to play song: {:?}", e);
            }
        }
    }

    pub fn set_volume(&self, volume: u8) {
        if let Err(e) = self.player.set_volume(volume) {
            tracing::error!("Failed to set volume: {:?}", e)
        }
    }

    pub fn seek(&self, pos: Duration) {
        if let Err(e) = self.player.seek(pos) {
            tracing::error!("Failed to seek: {:?}", e)
        }
        if let Ok(pos) = self.player.get_current_pos() {
            self.trigger_player_event(PlayerEvent {
                event: Some(Event::TimeUpdate(core_to_proto_duration(pos))),
            });
        }
    }

    pub fn on_song_ended(&mut self) {
        self.trigger_player_event(PlayerEvent {
            event: Some(Event::Ended(true)),
        });

        match self.repeat_mode {
            RepeatMode::Once => {
                self.repeat(RepeatMode::None);
                if self.current_idx < self.song_queue.len() {
                    self.trigger_song_changed();
                    if let Err(e) = self.play() {
                        tracing::error!("Failed to play song: {:?}", e);
                    }
                }
            }
            RepeatMode::Infinite => {
                if self.current_idx < self.song_queue.len() {
                    self.trigger_song_changed();
                    if let Err(e) = self.play() {
                        tracing::error!("Failed to play song: {:?}", e);
                    }
                }
            }
            RepeatMode::None => {
                self.next();
            }
        }
    }

    pub fn set_resolver(&self, f: crate::source::SourceResolverFn) { self.player.set_resolver(f); }

    pub fn on_song_changed<F>(&mut self, callback: F)
    where
        F: Fn(Option<&Song>) -> () + Send + Sync + 'static,
    {
        self.on_song_changed_subs.push(Box::new(callback));
    }

    pub fn on_queue_updated<F>(&mut self, callback: F)
    where
        F: Fn(&[Song]) -> () + Send + Sync + 'static,
    {
        self.on_queue_updated_subs.push(Box::new(callback));
    }

    pub fn on_repeat_changed<F>(&mut self, callback: F)
    where
        F: Fn(RepeatMode) -> () + Send + Sync + 'static,
    {
        self.on_repeat_changed_subs.push(Box::new(callback));
    }

    pub fn on_player_event<F>(&mut self, callback: F)
    where
        F: Fn(&PlayerEvent) -> () + Send + Sync + 'static,
    {
        self.on_player_event_subs.push(Box::new(callback));
    }

    fn trigger_song_changed(&mut self) {
        let current = self.current_song().cloned();
        if let Some(song) = &current {
            if let Err(e) = self.player.set_src(song.clone()) {
                tracing::error!("Failed to load song: {:?}", e);
                return;
            }
        }
        for cb in &self.on_song_changed_subs {
            cb(current.as_ref());
        }
        self.trigger_player_event(PlayerEvent {
            event: Some(Event::TimeUpdate(
                extensions_proto::duration_proto::google::protobuf::Duration::default(),
            )),
        });
    }

    fn trigger_queue_updated(&self) {
        for cb in &self.on_queue_updated_subs {
            cb(&self.song_queue);
        }
    }

    fn trigger_repeat_changed(&self) {
        for cb in &self.on_repeat_changed_subs {
            cb(self.repeat_mode);
        }
    }

    fn trigger_player_event(&self, event: PlayerEvent) {
        for cb in &self.on_player_event_subs {
            cb(&event);
        }
    }
}

impl Plugin for PlayerHandler {
    fn init(_context: &PluginContext) -> Arc<RwLock<Self>> {
        let (ended_tx, mut ended_rx) = unbounded_channel();
        let ph = Arc::new(RwLock::new(PlayerHandler::new(ended_tx)));

        let ph_clone = ph.clone();
        tokio::spawn(async move {
            while let Some(_) = ended_rx.recv().await {
                let mut ph = ph_clone.write().await;
                ph.on_song_ended();
            }
        });

        let ph_clone_timer = ph.clone();
        tokio::spawn(async move {
            let mut interval = interval(Duration::from_secs(1));
            loop {
                interval.tick().await;
                let ph = ph_clone_timer.read().await;
                if let Ok(pos) = ph.player.get_current_pos() {
                    ph.trigger_player_event(PlayerEvent {
                        event: Some(Event::TimeUpdate(core_to_proto_duration(pos))),
                    });
                }
            }
        });

        ph
    }
}
