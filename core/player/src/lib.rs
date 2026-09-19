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
pub mod context;
pub mod error;
mod generic;
mod mux_player;
mod rodio;
mod source;

#[cfg(test)]
mod audio_source_test;
#[cfg(test)]
mod lib_test;
#[cfg(test)]
mod lib_test_smoke;
#[cfg(test)]
mod mux_player_test;
#[cfg(test)]
mod source_test;

use std::{path::PathBuf, sync::Arc, time::Duration};

pub use context::{AudioPlayerContext, DummyAudioPlayerContext, RodioPlayerContext};
use extensions_proto::moosync::types::{PlayerEvent, PlayerState, player_event::Event};
use player_proto::moosync::types::{PlayerData, RepeatMode};
use songs_proto::moosync::types::Song;
use tokio::{
    sync::mpsc::{UnboundedSender, unbounded_channel},
    time::interval,
};
use tracing::{Instrument, debug};
use types::{
    plugin::{Plugin, PluginContext, RwLock},
    prelude::core_to_proto_duration,
    subscription::SubscriberList,
};

use crate::{
    audio_source::AudioSource,
    context::{PersistContext, dummy::DummyPersistContext, file_persist::FilePersist},
    error::PlayerError,
};

pub type OnEndedCallback = Box<dyn Fn() + Send + Sync + 'static>;

pub type OnSongChangedCallback = Box<dyn Fn(Option<&Song>) + Send + Sync + 'static>;
pub type OnQueueUpdatedCallback = Box<dyn Fn(&[Song]) + Send + Sync + 'static>;
pub type OnRepeatChangedCallback = Box<dyn Fn(RepeatMode) + Send + Sync + 'static>;
pub type OnPlayerEventCallback = Box<dyn Fn(&PlayerEvent) + Send + Sync + 'static>;

pub struct PlayerHandler {
    pub(crate) player_data: PlayerData,
    pub(crate) player: AudioSource,
    pub(crate) persist_context: Box<dyn PersistContext>,
    pub(crate) on_song_changed: SubscriberList<OnSongChangedCallback>,
    pub(crate) on_queue_updated: SubscriberList<OnQueueUpdatedCallback>,
    pub(crate) on_repeat_changed: SubscriberList<OnRepeatChangedCallback>,
    pub(crate) on_player_event: SubscriberList<OnPlayerEventCallback>,
}

#[plugin_macro::generate]
impl PlayerHandler {
    #[tracing::instrument(level = "debug", skip_all)]
    pub fn new(ended_tx: UnboundedSender<()>, data_dir: PathBuf) -> Self {
        Self::new_with_context(
            ended_tx,
            Box::new(RodioPlayerContext::new()),
            Box::new(FilePersist::new(data_dir)),
        )
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub fn new_with_context(
        ended_tx: UnboundedSender<()>,
        context: Box<dyn AudioPlayerContext>,
        persist_context: Box<dyn PersistContext>,
    ) -> Self {
        let mut player_data = match persist_context.load() {
            Ok(data) => data,
            Err(e) => {
                tracing::error!("Failed to load player data: {:?}", e);
                PlayerData::default()
            }
        };
        if player_data.current_idx as usize >= player_data.song_queue.len() {
            player_data.current_idx = 0;
        }
        PlayerHandler {
            player_data,
            player: AudioSource::new_with_context(
                Box::new(move || {
                    let _ = ended_tx.send(());
                }),
                context,
            ),
            persist_context,
            on_song_changed: SubscriberList::new(),
            on_queue_updated: SubscriberList::new(),
            on_repeat_changed: SubscriberList::new(),
            on_player_event: SubscriberList::new(),
        }
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub fn new_dummy(ended_tx: UnboundedSender<()>) -> Self {
        Self::new_with_context(
            ended_tx,
            Box::new(DummyAudioPlayerContext::new()),
            Box::new(DummyPersistContext {}),
        )
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub fn set_context(&mut self, context: Box<dyn AudioPlayerContext>) {
        self.player = AudioSource::new_with_context(Box::new(|| {}), context);
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub fn get_player_state(&self) -> i32 { self.player.get_player_state() as i32 }

    #[tracing::instrument(level = "debug", skip_all)]
    pub fn get_volume(&self) -> u8 { self.player.get_volume() }

    #[tracing::instrument(level = "debug", skip_all)]
    pub fn get_queue(&self) -> &[Song] { &self.player_data.song_queue }

    #[tracing::instrument(level = "debug", skip_all)]
    pub fn get_current_idx(&self) -> usize { self.player_data.current_idx as usize }

    #[tracing::instrument(level = "debug", skip_all)]
    pub fn get_current_pos(&self) -> Result<Duration, PlayerError> { self.player.get_current_pos() }

    #[tracing::instrument(level = "debug", skip_all)]
    pub fn current_song(&self) -> Option<&Song> {
        self.player_data
            .song_queue
            .get(self.player_data.current_idx as usize)
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub fn get_current_song(&self) -> Option<&Song> { self.current_song() }

    #[tracing::instrument(level = "debug", skip_all)]
    pub fn get_repeat_mode(&self) -> RepeatMode {
        RepeatMode::try_from(self.player_data.repeat_mode).unwrap_or_default()
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub fn add_to_queue(&mut self, songs: Vec<Song>) {
        if songs.is_empty() {
            return;
        }

        if self.current_song().is_none() {
            self.play_now(songs);
            return;
        }

        self.player_data.song_queue.extend(songs);
        self.trigger_queue_changed();
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub fn play_next(&mut self, songs: Vec<Song>) {
        if songs.is_empty() {
            return;
        }

        if self.current_song().is_none() {
            self.play_now(songs);
            return;
        }

        let insert_pos = (self.player_data.current_idx as usize) + 1;
        self.player_data
            .song_queue
            .splice(insert_pos..insert_pos, songs);
        self.trigger_queue_changed();
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub fn clear_and_play(&mut self, songs: Vec<Song>) {
        self.player_data.song_queue.clear();
        self.player_data.current_idx = 0;
        self.play_now(songs);
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub fn play_now(&mut self, songs: Vec<Song>) {
        if songs.is_empty() {
            return;
        }

        debug!("Playing songs now: {:?}", songs);
        if self.current_song().is_none() {
            self.player_data.song_queue = songs;
            self.player_data.current_idx = 0;
            self.trigger_queue_changed();
            self.trigger_song_changed();
            if let Err(e) = self.play() {
                tracing::error!("Failed to play song: {:?}", e);
            }
            return;
        }

        let insert_pos = self.player_data.current_idx as usize;
        self.player_data
            .song_queue
            .splice(insert_pos..insert_pos, songs);
        self.player_data.current_idx = insert_pos as u64;
        self.trigger_queue_changed();
        self.trigger_song_changed();

        if let Err(e) = self.play() {
            tracing::error!("Failed to play song: {:?}", e);
        }
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub fn shuffle(&mut self) {
        if self.player_data.song_queue.len() <= 1 {
            return;
        }

        let current_song = self
            .player_data
            .song_queue
            .remove(self.player_data.current_idx as usize);

        use rand::seq::SliceRandom;
        let mut rng = rand::thread_rng();
        self.player_data.song_queue.shuffle(&mut rng);

        self.player_data
            .song_queue
            .insert(self.player_data.current_idx as usize, current_song);
        self.trigger_queue_changed();
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub fn repeat(&mut self, mode: RepeatMode) {
        self.player_data.repeat_mode = mode.into();
        self.trigger_repeat_changed();
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub fn play(&mut self) -> Result<(), PlayerError> {
        if self.player.get_player_state() == PlayerState::Stopped
            && let Some(mut song) = self.current_song().cloned()
            && let Err(e) = self.player.set_src(&mut song)
        {
            tracing::error!("Failed to load song on play: {:?}", e);
        }
        self.player.play()?;
        self.trigger_player_event(Event::Play(true));
        Ok(())
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub fn pause(&mut self) -> Result<(), PlayerError> {
        self.player.pause()?;
        self.trigger_player_event(Event::Pause(true));
        Ok(())
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub fn next(&mut self) {
        if self.player_data.song_queue.is_empty() {
            let _ = self.player.stop();
            return;
        }
        if (self.player_data.current_idx as usize) + 1 < self.player_data.song_queue.len() {
            self.player_data.current_idx += 1;
        } else {
            self.player_data.current_idx = 0;
        }
        self.trigger_song_changed();
        if let Err(e) = self.play() {
            tracing::error!("Failed to play song: {:?}", e);
        }
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub fn prev(&mut self) {
        if self.player_data.song_queue.is_empty() {
            return;
        }
        if self.player_data.current_idx > 0 {
            self.player_data.current_idx -= 1;
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
            self.trigger_player_event(Event::TimeUpdate(core_to_proto_duration(pos)));
        }
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub fn play_index(&mut self, idx: usize) {
        if idx < self.player_data.song_queue.len() {
            self.player_data.current_idx = idx as u64;
            self.trigger_song_changed();
            if let Err(e) = self.play() {
                tracing::error!("Failed to play song at index {}: {:?}", idx, e);
            }
        }
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub fn remove_from_queue(&mut self, idx: usize) {
        if idx < self.player_data.song_queue.len() {
            self.player_data.song_queue.remove(idx);
            if (self.player_data.current_idx as usize) >= self.player_data.song_queue.len()
                && !self.player_data.song_queue.is_empty()
            {
                self.player_data.current_idx = (self.player_data.song_queue.len() - 1) as u64;
            }
            self.trigger_queue_changed();
        }
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub fn clear_queue(&mut self) {
        if self.player_data.song_queue.len() > 1 {
            if (self.player_data.current_idx as usize) < self.player_data.song_queue.len() {
                let song = self
                    .player_data
                    .song_queue
                    .remove(self.player_data.current_idx as usize);
                self.player_data.song_queue.clear();
                self.player_data.song_queue.push(song);
                self.player_data.current_idx = 0;
                self.trigger_queue_changed();
            }
            return;
        }

        if self.player_data.song_queue.len() == 1 {
            self.player_data.song_queue.clear();
            self.player_data.current_idx = 0;
            self.trigger_queue_changed();
            self.trigger_song_changed();
        }
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub fn move_queue_item(&mut self, from_idx: usize, to_idx: usize) {
        if from_idx < self.player_data.song_queue.len()
            && to_idx < self.player_data.song_queue.len()
        {
            let song = self.player_data.song_queue.remove(from_idx);
            self.player_data.song_queue.insert(to_idx, song);
            if (self.player_data.current_idx as usize) == from_idx {
                self.player_data.current_idx = to_idx as u64;
            } else if from_idx < (self.player_data.current_idx as usize)
                && to_idx >= (self.player_data.current_idx as usize)
            {
                self.player_data.current_idx -= 1;
            } else if from_idx > (self.player_data.current_idx as usize)
                && to_idx <= (self.player_data.current_idx as usize)
            {
                self.player_data.current_idx += 1;
            }
            self.trigger_queue_changed();
        }
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub fn on_song_ended(&mut self) {
        self.trigger_player_event(Event::Ended(true));

        match RepeatMode::try_from(self.player_data.repeat_mode).unwrap_or_default() {
            RepeatMode::RepeatOnce => {
                self.repeat(RepeatMode::RepeatNone);
                if (self.player_data.current_idx as usize) < self.player_data.song_queue.len() {
                    self.trigger_song_changed();
                    if let Err(e) = self.play() {
                        tracing::error!("Failed to play song: {:?}", e);
                    }
                }
            }
            RepeatMode::RepeatInfinite => {
                if (self.player_data.current_idx as usize) < self.player_data.song_queue.len() {
                    self.trigger_song_changed();
                    if let Err(e) = self.play() {
                        tracing::error!("Failed to play song: {:?}", e);
                    }
                }
            }
            RepeatMode::RepeatNone => {
                self.next();
            }
        }
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub fn set_resolver(&self, f: crate::source::SourceResolverFn) { self.player.set_resolver(f); }

    #[tracing::instrument(level = "debug", skip_all)]
    fn trigger_queue_changed(&self) {
        self.on_queue_updated
            .run_all(|cb| cb(&self.player_data.song_queue));
        if let Err(e) = self.persist_context.persist(&self.player_data) {
            tracing::error!("Failed to persist player data: {}", e);
        }
    }

    #[tracing::instrument(level = "debug", skip_all)]
    fn trigger_repeat_changed(&self) {
        let mode = RepeatMode::try_from(self.player_data.repeat_mode).unwrap_or_default();
        self.on_repeat_changed.run_all(|cb| cb(mode));
        if let Err(e) = self.persist_context.persist(&self.player_data) {
            tracing::error!("Failed to persist player data: {}", e);
        }
    }

    #[tracing::instrument(level = "debug", skip_all)]
    fn trigger_player_event(&self, event: Event) {
        let player_event = PlayerEvent { event: Some(event) };
        self.on_player_event.run_all(|cb| cb(&player_event));
    }

    #[tracing::instrument(level = "debug", skip_all)]
    fn trigger_song_changed(&mut self) {
        let mut current = self.current_song().cloned();
        if current.is_none() {
            let _ = self.player.stop();
        }

        if let Some(song) = current.as_mut()
            && let Err(e) = self.player.set_src(song)
        {
            tracing::error!("Failed to load song: {:?}", e);
            return;
        }
        self.on_song_changed.run_all(|cb| {
            cb(current.as_ref());
        });
        self.trigger_player_event(Event::TimeUpdate(Default::default()));
        if let Err(e) = self.persist_context.persist(&self.player_data) {
            tracing::error!("Failed to persist player data: {}", e);
        }
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
    fn init(context: &PluginContext) -> Arc<RwLock<Self>> {
        let (ended_tx, mut ended_rx) = unbounded_channel();
        let ph = Arc::new(RwLock::new(PlayerHandler::new(
            ended_tx,
            context.data_dir.clone(),
        )));

        let ph_clone = ph.clone();
        tokio::spawn(
            async move {
                while ended_rx.recv().await.is_some() {
                    let mut ph = ph_clone.write().await;
                    ph.on_song_ended();
                }
            }
            .in_current_span(),
        );

        let ph_clone_timer = ph.clone();
        tokio::spawn(
            async move {
                let mut interval = interval(Duration::from_secs(1));
                loop {
                    interval.tick().await;
                    let ph = ph_clone_timer.read().await;
                    if let Ok(pos) = ph.player.get_current_pos() {
                        ph.trigger_player_event(Event::TimeUpdate(core_to_proto_duration(pos)));
                    }
                }
            }
            .in_current_span(),
        );

        ph
    }
}
