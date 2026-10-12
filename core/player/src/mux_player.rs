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

use std::{
    sync::{
        RwLock,
        atomic::{AtomicUsize, Ordering},
    },
    time::Duration,
};

use extensions_proto::moosync::types::{
    PlayerState,
    player_event::Event::{self as PlayerEvent, Ended},
};
use songs_proto::moosync::types::Song;
use tokio::sync::mpsc::{UnboundedSender, unbounded_channel};
use tracing::Instrument;
use types::subscription::SubscriberList;

use crate::{
    context::AudioPlayerContext,
    error::PlayerError,
    source::{SourceResolver, SourceResolverFn, ValidSrc, get_valid_src},
};

pub struct MuxPlayer {
    players: Vec<Box<dyn AudioPlayerContext>>,
    active_idx: AtomicUsize,
    events_tx: RwLock<UnboundedSender<PlayerEvent>>,
    pub on_ended_callback: SubscriberList<()>,
    source_resolver: SourceResolver,
}

impl MuxPlayer {
    #[tracing::instrument(level = "debug", skip_all)]
    pub fn new_with_contexts(
        on_ended_callback: SubscriberList<()>,
        players: Vec<Box<dyn AudioPlayerContext>>,
    ) -> Self {
        assert!(
            !players.is_empty(),
            "MuxPlayer requires at least one player"
        );
        let (events_tx, mut events_rx) = unbounded_channel();
        let callback_clone = on_ended_callback.clone();
        tokio::spawn(
            async move {
                while let Some(event) = events_rx.recv().await {
                    if let Ended(_) = event {
                        callback_clone.emit(());
                    }
                }
            }
            .in_current_span(),
        );

        Self {
            players,
            active_idx: AtomicUsize::new(0),
            events_tx: RwLock::new(events_tx),
            on_ended_callback,
            source_resolver: SourceResolver::new(),
        }
    }

    #[tracing::instrument(level = "debug", skip_all)]
    fn active_player(&self) -> &dyn AudioPlayerContext {
        let idx = self.active_idx.load(Ordering::SeqCst);
        self.players[idx].as_ref()
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub fn get_player_state(&self) -> PlayerState {
        self.active_player()
            .get_player_state()
            .unwrap_or(PlayerState::Stopped)
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub fn get_volume(&self) -> u8 { self.active_player().get_volume().unwrap_or(100) }

    #[cfg(test)]
    #[tracing::instrument(level = "debug", skip_all)]
    pub fn can_play(&self, src: ValidSrc) -> bool { self.active_player().can_play(&src) }

    #[tracing::instrument(level = "debug", skip_all)]
    pub fn set_resolver(&self, f: SourceResolverFn) { self.source_resolver.set_resolver(f); }

    #[tracing::instrument(level = "debug", skip_all)]
    fn create_new_events_tx(&self) -> UnboundedSender<PlayerEvent> {
        let (events_tx, mut events_rx) = unbounded_channel();
        let callback_clone = self.on_ended_callback.clone();
        tokio::spawn(
            async move {
                while let Some(event) = events_rx.recv().await {
                    if let Ended(_) = event {
                        callback_clone.emit(());
                    }
                }
            }
            .in_current_span(),
        );
        *self.events_tx.write().unwrap() = events_tx.clone();
        events_tx
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub fn load_src(&self, src: ValidSrc) -> Result<(), PlayerError> {
        let current_idx = self.active_idx.load(Ordering::SeqCst);

        if self.players[current_idx].can_play(&src) {
            let events_tx = self.events_tx.read().unwrap().clone();
            match self.players[current_idx].set_src(src.clone(), events_tx) {
                Ok(()) => return Ok(()),
                Err(e) => {
                    tracing::error!(
                        "Failed to set src on active player {}: {:?}",
                        current_idx,
                        e
                    );
                }
            }
        }

        for (idx, player) in self.players.iter().enumerate() {
            if idx == current_idx {
                continue;
            }
            if player.can_play(&src) {
                let new_events_tx = self.create_new_events_tx();
                match player.set_src(src.clone(), new_events_tx) {
                    Ok(()) => {
                        self.active_idx.store(idx, Ordering::SeqCst);
                        return Ok(());
                    }
                    Err(e) => {
                        tracing::error!("Failed to set src on player {}: {:?}", idx, e);
                    }
                }
            }
        }
        Err(PlayerError::NoPlayerFound(Box::default()))
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub fn load(&self, song: &Song) -> Result<(), PlayerError> {
        let src = get_valid_src(song)?;
        self.load_src(src)
            .map_err(|_| PlayerError::NoPlayerFound(Box::new(song.clone())))
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub async fn set_src(&self, song: &Song) -> Result<(), PlayerError> {
        self.load_song(song).await
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub async fn load_song(&self, song: &Song) -> Result<(), PlayerError> {
        let res = self.load(song);
        if let Err(e) = res {
            match e {
                PlayerError::NoPlayerFound(_) | PlayerError::NoSrcFound(_) => {
                    let resp = self.source_resolver.resolve(song).await?;
                    tracing::debug!("Resolved playback details for: {:?}", resp);
                    let src = ValidSrc::new_url_with_key(resp.url, resp.decryption_key);
                    self.load_src(src)
                        .map_err(|_| PlayerError::NoPlayerFound(Box::new(song.clone())))?;
                }
                _ => return Err(e),
            }
        }
        Ok(())
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub fn play(&self) -> Result<(), PlayerError> { self.active_player().play() }

    #[tracing::instrument(level = "debug", skip_all)]
    pub fn pause(&self) -> Result<(), PlayerError> { self.active_player().pause() }

    #[tracing::instrument(level = "debug", skip_all)]
    pub fn stop(&self) -> Result<(), PlayerError> { self.active_player().stop() }

    #[tracing::instrument(level = "debug", skip_all)]
    pub fn set_volume(&self, volume: u8) -> Result<(), PlayerError> {
        self.active_player().set_volume(volume)
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub fn seek(&self, position: Duration) -> Result<(), PlayerError> {
        self.active_player().seek(position)
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub fn get_current_pos(&self) -> Result<Duration, PlayerError> {
        self.active_player().get_current_pos()
    }
}
