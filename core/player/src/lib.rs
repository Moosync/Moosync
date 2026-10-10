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

pub mod context;
pub mod error;
pub mod mux_player;
pub mod player_data_ext;
mod rodio;
mod source;

#[cfg(test)]
mod lib_test;
#[cfg(test)]
mod lib_test_smoke;
#[cfg(test)]
mod mux_player_test;
#[cfg(test)]
mod player_data_ext_test;
#[cfg(test)]
mod source_test;

use std::{
    path::PathBuf,
    sync::{Arc, RwLock},
    time::Duration,
};

pub use context::{AudioPlayerContext, DummyAudioPlayerContext, RodioPlayerContext};
use extensions_proto::moosync::types::{PlayerEvent, PlayerState, player_event::Event};
pub use mux_player::MuxPlayer;
pub use player_data_ext::PlayerDataExt;
use player_proto::moosync::types::{PlayerData, RepeatMode};
use songs_proto::moosync::types::Song;
use tokio::{
    sync::mpsc::{UnboundedSender, unbounded_channel},
    time::interval,
};
use tracing::{Instrument, debug};
use types::{
    plugin::{Plugin, PluginContext},
    prelude::core_to_proto_duration,
    subscription::SubscriberList,
};

use crate::{
    context::{PersistContext, dummy::DummyPersistContext, file_persist::FilePersist},
    error::PlayerError,
    source::SourceResolverFn,
};

pub struct PlayerHandler {
    pub(crate) player_data: RwLock<PlayerData>,
    pub(crate) player: MuxPlayer,
    pub(crate) persist_context: Box<dyn PersistContext>,
    pub on_song_changed: SubscriberList<Option<Song>>,
    pub on_queue_updated: SubscriberList<Vec<Song>>,
    pub on_repeat_changed: SubscriberList<RepeatMode>,
    pub on_player_event: SubscriberList<PlayerEvent>,
}

#[plugin_macro::generate]
impl PlayerHandler {
    #[tracing::instrument(level = "debug", skip_all)]
    pub fn new(ended_tx: UnboundedSender<()>, data_dir: PathBuf) -> Self {
        Self::new_with_contexts(
            ended_tx,
            vec![
                Box::new(RodioPlayerContext::new()),
                Box::new(DummyAudioPlayerContext::new()),
            ],
            Box::new(FilePersist::new(data_dir)),
        )
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub fn new_with_contexts(
        ended_tx: UnboundedSender<()>,
        contexts: Vec<Box<dyn AudioPlayerContext>>,
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
        let on_ended_list = SubscriberList::new();
        on_ended_list.listen(move |()| {
            let _ = ended_tx.send(());
        });
        PlayerHandler {
            player_data: RwLock::new(player_data),
            player: MuxPlayer::new_with_contexts(on_ended_list, contexts),
            persist_context,
            on_song_changed: SubscriberList::new(),
            on_queue_updated: SubscriberList::new(),
            on_repeat_changed: SubscriberList::new(),
            on_player_event: SubscriberList::new(),
        }
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub fn new_dummy(ended_tx: UnboundedSender<()>) -> Self {
        Self::new_with_contexts(
            ended_tx,
            vec![Box::new(DummyAudioPlayerContext::new())],
            Box::new(DummyPersistContext {}),
        )
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub fn get_player_state(&self) -> i32 { self.player.get_player_state() as i32 }

    #[tracing::instrument(level = "debug", skip_all)]
    pub fn get_volume(&self) -> u8 { self.player.get_volume() }

    #[tracing::instrument(level = "debug", skip_all)]
    pub fn get_queue(&self) -> Vec<Song> { self.player_data.read().unwrap().song_queue.clone() }

    #[tracing::instrument(level = "debug", skip_all)]
    pub fn get_current_idx(&self) -> usize { self.player_data.read().unwrap().current_idx as usize }

    #[tracing::instrument(level = "debug", skip_all)]
    pub fn get_current_pos(&self) -> Result<Duration, PlayerError> { self.player.get_current_pos() }

    #[tracing::instrument(level = "debug", skip_all)]
    pub fn current_song(&self) -> Option<Song> {
        self.player_data.read().unwrap().current_song().cloned()
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub fn get_current_song(&self) -> Option<Song> { self.current_song() }

    #[tracing::instrument(level = "debug", skip_all)]
    pub fn get_repeat_mode(&self) -> RepeatMode {
        RepeatMode::try_from(self.player_data.read().unwrap().repeat_mode).unwrap_or_default()
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub async fn add_to_queue(&self, songs: Vec<Song>) {
        if songs.is_empty() {
            return;
        }

        if self.current_song().is_none() {
            self.play_now(songs).await;
            return;
        }

        self.player_data.write().unwrap().append_songs(songs);
        self.trigger_queue_changed();
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub async fn play_next(&self, songs: Vec<Song>) {
        if songs.is_empty() {
            return;
        }

        if self.current_song().is_none() {
            self.play_now(songs).await;
            return;
        }

        self.player_data.write().unwrap().insert_songs_next(songs);
        self.trigger_queue_changed();
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub async fn clear_and_play(&self, songs: Vec<Song>) {
        self.player_data.write().unwrap().clear_and_set(Vec::new());
        self.play_now(songs).await;
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub async fn play_now(&self, songs: Vec<Song>) {
        if songs.is_empty() {
            return;
        }

        debug!("Playing songs now: {:?}", songs);
        if self.current_song().is_none() {
            self.player_data.write().unwrap().clear_and_set(songs);
            self.trigger_queue_changed();
            self.trigger_song_changed().await;
            if let Err(e) = self.play().await {
                tracing::error!("Failed to play song: {:?}", e);
            }
            return;
        }

        let insert_pos = self.get_current_idx();
        {
            let mut pd = self.player_data.write().unwrap();
            pd.insert_songs_at(insert_pos, songs);
            pd.set_index(insert_pos);
        }
        self.trigger_queue_changed();
        self.trigger_song_changed().await;

        if let Err(e) = self.play().await {
            tracing::error!("Failed to play song: {:?}", e);
        }
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub fn shuffle(&self) {
        self.player_data.write().unwrap().shuffle_queue();
        self.trigger_queue_changed();
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub fn repeat(&self, mode: RepeatMode) {
        self.player_data.write().unwrap().set_repeat_mode(mode);
        self.trigger_repeat_changed();
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub async fn play(&self) -> Result<(), PlayerError> {
        if self.player.get_player_state() == PlayerState::Stopped
            && let Some(song) = self.current_song()
            && let Err(e) = self.player.set_src(&song).await
        {
            return Err(PlayerError::PlaybackUrlResolutionFailed(format!(
                "Failed to load song on play: {:?}",
                e
            )));
        }
        self.player.play()?;
        self.trigger_player_event(Event::Play(true));
        Ok(())
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub fn pause(&self) -> Result<(), PlayerError> {
        self.player.pause()?;
        self.trigger_player_event(Event::Pause(true));
        Ok(())
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub async fn next(&self) {
        if self.player_data.read().unwrap().song_queue.is_empty() {
            let _ = self.player.stop();
            return;
        }
        self.player_data.write().unwrap().advance_index();
        self.trigger_song_changed().await;
        if let Err(e) = self.play().await {
            tracing::error!("Failed to play song: {:?}", e);
        }
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub async fn prev(&self) {
        if self.player_data.read().unwrap().song_queue.is_empty() {
            return;
        }
        if self.get_current_idx() > 0 {
            self.player_data.write().unwrap().rewind_index();
            self.trigger_song_changed().await;
            if let Err(e) = self.play().await {
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
    pub async fn play_index(&self, idx: usize) {
        if idx < self.player_data.read().unwrap().song_queue.len() {
            self.player_data.write().unwrap().set_index(idx);
            self.trigger_song_changed().await;
            if let Err(e) = self.play().await {
                tracing::error!("Failed to play song at index {}: {:?}", idx, e);
            }
        }
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub fn remove_from_queue(&self, idx: usize) {
        self.player_data.write().unwrap().remove_at_index(idx);
        self.trigger_queue_changed();
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub async fn clear_queue(&self) {
        let is_single = self.player_data.read().unwrap().song_queue.len() == 1;
        self.player_data.write().unwrap().clear_except_current();
        self.trigger_queue_changed();
        if is_single {
            self.trigger_song_changed().await;
        }
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub fn move_queue_item(&self, from_idx: usize, to_idx: usize) {
        self.player_data.write().unwrap().reorder(from_idx, to_idx);
        self.trigger_queue_changed();
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub async fn on_song_ended(&self) {
        self.trigger_player_event(Event::Ended(true));

        match self.get_repeat_mode() {
            RepeatMode::RepeatOnce => {
                self.repeat(RepeatMode::RepeatNone);
                if self.get_current_idx() < self.player_data.read().unwrap().song_queue.len() {
                    self.trigger_song_changed().await;
                    if let Err(e) = self.play().await {
                        tracing::error!("Failed to play song: {:?}", e);
                    }
                }
            }
            RepeatMode::RepeatInfinite => {
                if self.get_current_idx() < self.player_data.read().unwrap().song_queue.len() {
                    self.trigger_song_changed().await;
                    if let Err(e) = self.play().await {
                        tracing::error!("Failed to play song: {:?}", e);
                    }
                }
            }
            RepeatMode::RepeatNone => {
                self.next().await;
            }
        }
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub fn set_resolver(&self, f: SourceResolverFn) { self.player.set_resolver(f); }

    #[tracing::instrument(level = "debug", skip_all)]
    fn trigger_queue_changed(&self) {
        let queue = self.player_data.read().unwrap().song_queue.clone();
        self.on_queue_updated.emit(queue);
        let pd = self.player_data.read().unwrap().clone();
        if let Err(e) = self.persist_context.persist(&pd) {
            tracing::error!("Failed to persist player data: {}", e);
        }
    }

    #[tracing::instrument(level = "debug", skip_all)]
    fn trigger_repeat_changed(&self) {
        let mode = self.get_repeat_mode();
        self.on_repeat_changed.emit(mode);
        let pd = self.player_data.read().unwrap().clone();
        if let Err(e) = self.persist_context.persist(&pd) {
            tracing::error!("Failed to persist player data: {}", e);
        }
    }

    #[tracing::instrument(level = "debug", skip_all)]
    fn trigger_player_event(&self, event: Event) {
        let player_event = PlayerEvent { event: Some(event) };
        self.on_player_event.emit(player_event);
    }

    #[tracing::instrument(level = "debug", skip_all)]
    async fn trigger_song_changed(&self) {
        let current = self.current_song();
        if current.is_none() {
            let _ = self.player.stop();
        }

        if let Some(song) = current.as_ref()
            && let Err(e) = self.player.set_src(song).await
        {
            tracing::error!("Failed to load song: {:?}", e);
            return;
        }
        self.on_song_changed.emit(current);
        self.trigger_player_event(Event::TimeUpdate(Default::default()));
        let pd = self.player_data.read().unwrap().clone();
        if let Err(e) = self.persist_context.persist(&pd) {
            tracing::error!("Failed to persist player data: {}", e);
        }
    }
}

impl Plugin for PlayerHandler {
    #[tracing::instrument(level = "debug", skip_all)]
    fn init(context: &PluginContext) -> Arc<Self> {
        let (ended_tx, mut ended_rx) = unbounded_channel();
        let ph = Arc::new(PlayerHandler::new(ended_tx, context.data_dir.clone()));

        let ph_clone = ph.clone();
        tokio::spawn(
            async move {
                while ended_rx.recv().await.is_some() {
                    ph_clone.on_song_ended().await;
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
                    if let Ok(pos) = ph_clone_timer.player.get_current_pos() {
                        ph_clone_timer
                            .trigger_player_event(Event::TimeUpdate(core_to_proto_duration(pos)));
                    }
                }
            }
            .in_current_span(),
        );

        ph
    }
}
