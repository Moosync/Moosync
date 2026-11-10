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
        Arc, RwLock,
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
    source::{SourceResolver, SourceResolverHolder, ValidSrc, get_valid_src},
};

pub struct MuxPlayer {
    players: RwLock<Vec<(String, Arc<dyn AudioPlayerContext>)>>,
    active_idx: AtomicUsize,
    events_tx: RwLock<UnboundedSender<PlayerEvent>>,
    pub on_ended_callback: SubscriberList<()>,
    source_resolver: SourceResolverHolder,
}

impl MuxPlayer {
    #[tracing::instrument(level = "debug", skip_all)]
    pub fn new_with_contexts(
        on_ended_callback: SubscriberList<()>,
        players: Vec<(String, Arc<dyn AudioPlayerContext>)>,
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
            players: RwLock::new(players),
            active_idx: AtomicUsize::new(0),
            events_tx: RwLock::new(events_tx),
            on_ended_callback,
            source_resolver: SourceResolverHolder::new(),
        }
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub fn add_player(&self, id: String, player: Arc<dyn AudioPlayerContext>) {
        let mut players = self.players.write().unwrap();
        players.retain(|(existing_id, _)| existing_id != &id);
        players.push((id, player));
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub fn remove_player(&self, id: &str) {
        let mut players = self.players.write().unwrap();
        players.retain(|(existing_id, _)| existing_id != id);
        let current_idx = self.active_idx.load(Ordering::SeqCst);
        if current_idx >= players.len() && !players.is_empty() {
            self.active_idx.store(0, Ordering::SeqCst);
        }
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub fn get_player_ids(&self) -> Vec<String> {
        self.players
            .read()
            .unwrap()
            .iter()
            .map(|(id, _)| id.clone())
            .collect()
    }

    #[tracing::instrument(level = "debug", skip_all)]
    fn get_active_player(&self) -> Option<Arc<dyn AudioPlayerContext>> {
        let players = self.players.read().unwrap();
        let idx = self.active_idx.load(Ordering::SeqCst);
        players.get(idx).map(|(_, player)| player.clone())
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub fn get_player_state(&self) -> PlayerState {
        self.get_active_player()
            .and_then(|p| p.get_player_state().ok())
            .unwrap_or(PlayerState::Stopped)
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub fn get_volume(&self) -> u8 {
        self.get_active_player()
            .and_then(|p| p.get_volume().ok())
            .unwrap_or(100)
    }

    #[cfg(test)]
    #[tracing::instrument(level = "debug", skip_all)]
    pub async fn can_play(&self, src: ValidSrc<'_>) -> bool {
        if let Some(player) = self.get_active_player() {
            player.can_play(&src).await
        } else {
            false
        }
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub fn set_resolver(&self, resolver: Arc<dyn SourceResolver>) {
        self.source_resolver.set_resolver(resolver);
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub(crate) fn create_new_events_tx(&self) -> UnboundedSender<PlayerEvent> {
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
    pub async fn load_src(&self, src: ValidSrc<'_>) -> Result<(), PlayerError> {
        let (current_idx, active_player, all_players) = {
            let players = self.players.read().unwrap();
            let idx = self.active_idx.load(Ordering::SeqCst);
            let active = players.get(idx).map(|(_, p)| p.clone());
            let all: Vec<(usize, Arc<dyn AudioPlayerContext>)> = players
                .iter()
                .enumerate()
                .map(|(i, (_, p))| (i, p.clone()))
                .collect();
            (idx, active, all)
        };

        if let Some(player) = active_player
            && player.can_play(&src).await
        {
            let events_tx = self.events_tx.read().unwrap().clone();
            match player.set_src(src.clone().into_owned(), events_tx).await {
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

        for (idx, player) in all_players {
            if idx == current_idx {
                continue;
            }
            if player.can_play(&src).await {
                let new_events_tx = self.create_new_events_tx();
                match player
                    .set_src(src.clone().into_owned(), new_events_tx)
                    .await
                {
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
    pub async fn load(&self, song: &Song) -> Result<(), PlayerError> {
        let src = get_valid_src(song)?;
        self.load_src(src)
            .await
            .map_err(|_| PlayerError::NoPlayerFound(Box::new(song.clone())))
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub async fn set_src(&self, song: &Song) -> Result<(), PlayerError> {
        self.load_song(song).await
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub async fn load_song(&self, song: &Song) -> Result<(), PlayerError> {
        let res = self.load(song).await;
        if let Err(e) = res {
            match e {
                PlayerError::NoPlayerFound(_) | PlayerError::NoSrcFound(_) => {
                    let resp = self.source_resolver.resolve(song).await?;
                    tracing::debug!("Resolved playback details for: {:?}", resp);
                    let src = ValidSrc::new_url_with_key(resp.url, resp.decryption_key);
                    self.load_src(src)
                        .await
                        .map_err(|_| PlayerError::NoPlayerFound(Box::new(song.clone())))?;
                }
                _ => return Err(e),
            }
        }
        Ok(())
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub async fn play(&self) -> Result<(), PlayerError> {
        let Some(player) = self.get_active_player() else {
            return Err(PlayerError::Custom("No active player".to_string()));
        };
        player.play().await
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub async fn pause(&self) -> Result<(), PlayerError> {
        let Some(player) = self.get_active_player() else {
            return Err(PlayerError::Custom("No active player".to_string()));
        };
        player.pause().await
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub async fn stop(&self) -> Result<(), PlayerError> {
        let Some(player) = self.get_active_player() else {
            return Err(PlayerError::Custom("No active player".to_string()));
        };
        player.stop().await
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub async fn set_volume(&self, volume: u8) -> Result<(), PlayerError> {
        let Some(player) = self.get_active_player() else {
            return Err(PlayerError::Custom("No active player".to_string()));
        };
        player.set_volume(volume).await
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub async fn seek(&self, position: Duration) -> Result<(), PlayerError> {
        let Some(player) = self.get_active_player() else {
            return Err(PlayerError::Custom("No active player".to_string()));
        };
        player.seek(position).await
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub fn get_current_pos(&self) -> Result<Duration, PlayerError> {
        let Some(player) = self.get_active_player() else {
            return Ok(Duration::default());
        };
        player.get_current_pos()
    }
}
