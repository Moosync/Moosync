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
pub mod error;
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
    subscription::SubscriberList,
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
    pub(crate) on_song_changed: SubscriberList<OnSongChangedCallback>,
    pub(crate) on_queue_updated: SubscriberList<OnQueueUpdatedCallback>,
    pub(crate) on_repeat_changed: SubscriberList<OnRepeatChangedCallback>,
    pub(crate) on_player_event: SubscriberList<OnPlayerEventCallback>,
}

#[plugin_macro::generate]
impl PlayerHandler {
    #[tracing::instrument(level = "debug", skip_all)]
    pub fn new(ended_tx: UnboundedSender<()>) -> Self {
        PlayerHandler {
            song_queue: vec![],
            current_idx: 0,
            repeat_mode: RepeatMode::None,
            player: AudioSource::new(Box::new(move || {
                let _ = ended_tx.send(());
            })),
            on_song_changed: SubscriberList::new(),
            on_queue_updated: SubscriberList::new(),
            on_repeat_changed: SubscriberList::new(),
            on_player_event: SubscriberList::new(),
        }
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub fn get_player_state(&self) -> i32 { self.player.get_player_state() as i32 }

    #[tracing::instrument(level = "debug", skip_all)]
    pub fn get_volume(&self) -> u8 { self.player.get_volume() }

    #[tracing::instrument(level = "debug", skip_all)]
    pub fn get_queue(&self) -> &[Song] { &self.song_queue }

    #[tracing::instrument(level = "debug", skip_all)]
    pub fn get_current_idx(&self) -> usize { self.current_idx }

    #[tracing::instrument(level = "debug", skip_all)]
    pub fn get_current_pos(&self) -> Result<Duration, crate::error::PlayerError> {
        self.player.get_current_pos()
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub fn current_song(&self) -> Option<&Song> { self.song_queue.get(self.current_idx) }

    #[tracing::instrument(level = "debug", skip_all)]
    pub fn get_current_song(&self) -> Option<&Song> { self.current_song() }

    #[tracing::instrument(level = "debug", skip_all)]
    pub fn get_repeat_mode(&self) -> RepeatMode { self.repeat_mode }

    #[tracing::instrument(level = "debug", skip_all)]
    pub fn add_to_queue(&mut self, song: Song) {
        if self.current_song().is_none() {
            self.play_now(song);
        } else {
            self.song_queue.push(song);
            self.on_queue_updated.run_all(|cb| cb(&self.song_queue));
        }
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub fn play_now(&mut self, song: Song) {
        debug!("Playing song now: {:?}", song);
        if self.current_song().is_none() {
            self.song_queue.push(song.clone());
            self.current_idx = 0;
        } else {
            let insert_pos = self.current_idx;
            self.song_queue.insert(insert_pos, song.clone());
            self.current_idx = insert_pos;
        }
        self.on_queue_updated.run_all(|cb| cb(&self.song_queue));
        self.trigger_song_changed();

        if let Err(e) = self.play() {
            tracing::error!("Failed to play song: {:?}", e);
        }
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub fn shuffle(&mut self) {
        if self.song_queue.len() <= 1 {
            return;
        }

        let current_song = self.song_queue.remove(self.current_idx);

        use rand::seq::SliceRandom;
        let mut rng = rand::thread_rng();
        self.song_queue.shuffle(&mut rng);

        self.song_queue.insert(self.current_idx, current_song);
        self.on_queue_updated.run_all(|cb| cb(&self.song_queue));
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub fn repeat(&mut self, mode: RepeatMode) {
        self.repeat_mode = mode;
        self.on_repeat_changed.run_all(|cb| cb(self.repeat_mode));
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub fn play(&mut self) -> Result<(), crate::error::PlayerError> {
        self.player.play()?;
        self.on_player_event.run_all(|cb| {
            cb(&PlayerEvent {
                event: Some(Event::Play(true)),
            });
        });
        Ok(())
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub fn pause(&mut self) -> Result<(), crate::error::PlayerError> {
        self.player.pause()?;
        self.on_player_event.run_all(|cb| {
            cb(&PlayerEvent {
                event: Some(Event::Pause(true)),
            });
        });
        Ok(())
    }

    #[tracing::instrument(level = "debug", skip_all)]
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

    #[tracing::instrument(level = "debug", skip_all)]
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

    #[tracing::instrument(level = "debug", skip_all)]
    pub fn set_volume(&self, volume: u8) {
        if let Err(e) = self.player.set_volume(volume) {
            tracing::error!("Failed to set volume: {:?}", e)
        }
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub fn seek(&self, pos: Duration) {
        if let Err(e) = self.player.seek(pos) {
            tracing::error!("Failed to seek: {:?}", e)
        }
        if let Ok(pos) = self.player.get_current_pos() {
            self.on_player_event.run_all(|cb| {
                cb(&PlayerEvent {
                    event: Some(Event::TimeUpdate(core_to_proto_duration(pos))),
                });
            });
        }
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub fn play_index(&mut self, idx: usize) {
        if idx < self.song_queue.len() {
            self.current_idx = idx;
            self.trigger_song_changed();
            if let Err(e) = self.play() {
                tracing::error!("Failed to play song at index {}: {:?}", idx, e);
            }
        }
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub fn remove_from_queue(&mut self, idx: usize) {
        if idx < self.song_queue.len() {
            self.song_queue.remove(idx);
            if self.current_idx >= self.song_queue.len() && !self.song_queue.is_empty() {
                self.current_idx = self.song_queue.len() - 1;
            }
            self.on_queue_updated.run_all(|cb| cb(&self.song_queue));
        }
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub fn clear_queue(&mut self) {
        self.song_queue.clear();
        self.current_idx = 0;
        self.on_queue_updated.run_all(|cb| cb(&self.song_queue));
        let _ = self.player.stop();
        self.trigger_song_changed();
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub fn move_queue_item(&mut self, from_idx: usize, to_idx: usize) {
        if from_idx < self.song_queue.len() && to_idx < self.song_queue.len() {
            let song = self.song_queue.remove(from_idx);
            self.song_queue.insert(to_idx, song);
            if self.current_idx == from_idx {
                self.current_idx = to_idx;
            } else if from_idx < self.current_idx && to_idx >= self.current_idx {
                self.current_idx -= 1;
            } else if from_idx > self.current_idx && to_idx <= self.current_idx {
                self.current_idx += 1;
            }
            self.on_queue_updated.run_all(|cb| cb(&self.song_queue));
        }
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub fn on_song_ended(&mut self) {
        self.on_player_event.run_all(|cb| {
            cb(&PlayerEvent {
                event: Some(Event::Ended(true)),
            });
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

    #[tracing::instrument(level = "debug", skip_all)]
    pub fn set_resolver(&self, f: crate::source::SourceResolverFn) { self.player.set_resolver(f); }

    #[tracing::instrument(level = "debug", skip_all)]
    fn trigger_song_changed(&mut self) {
        let current = self.current_song().cloned();
        if let Some(song) = &current {
            if let Err(e) = self.player.set_src(song.clone()) {
                tracing::error!("Failed to load song: {:?}", e);
                return;
            }
        }
        self.on_song_changed.run_all(|cb| {
            cb(current.as_ref());
        });
        self.on_player_event.run_all(|cb| {
            cb(&PlayerEvent {
                event: Some(Event::TimeUpdate(
                    extensions_proto::duration_proto::google::protobuf::Duration::default(),
                )),
            });
        });
    }
}

types::generate_on_event_impl!(
    PlayerHandler;
    on_song_changed, Option<&Song>;
    on_queue_updated, &[Song];
    on_repeat_changed, RepeatMode;
    on_player_event, &PlayerEvent;
);

impl Plugin for PlayerHandler {
    #[tracing::instrument(level = "debug", skip_all)]
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
                    ph.on_player_event.run_all(|cb| {
                        cb(&PlayerEvent {
                            event: Some(Event::TimeUpdate(core_to_proto_duration(pos))),
                        });
                    });
                }
            }
        });

        ph
    }
}
