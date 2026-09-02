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

use std::{sync::Arc, time::Duration};

use extensions_proto::moosync::types::player_event::Event as PlayerEvent;
use songs_proto::moosync::types::Song;
use tokio::sync::mpsc::UnboundedSender;

#[cfg(test)]
use crate::source::ValidSrc;
use crate::{
    context::AudioPlayerContext, error::PlayerError, generic::PlayerExt, rodio::RodioPlayer,
    source::get_valid_src,
};

pub(crate) struct MuxPlayer {
    players: Vec<Arc<Box<dyn PlayerExt>>>,
    active_player: Arc<Box<dyn PlayerExt>>,
}

// Common holder for all players
// Decides which player to use for a given song and abstracts calls on the
// active player
impl MuxPlayer {
    #[tracing::instrument(level = "debug", skip_all)]
    pub fn new(events_tx: UnboundedSender<PlayerEvent>) -> Self {
        let players: Vec<Arc<Box<dyn PlayerExt>>> =
            vec![Arc::new(Box::new(RodioPlayer::new(events_tx)))];

        Self {
            active_player: players[0].clone(),
            players,
        }
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub fn new_with_context(
        events_tx: UnboundedSender<PlayerEvent>,
        context: Box<dyn AudioPlayerContext>,
    ) -> Self {
        let players: Vec<Arc<Box<dyn PlayerExt>>> = vec![Arc::new(Box::new(
            RodioPlayer::new_with_context(context, events_tx),
        ))];

        Self {
            active_player: players[0].clone(),
            players,
        }
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub fn get_player_state(&self) -> extensions_proto::moosync::types::PlayerState {
        self.active_player
            .get_player_state()
            .unwrap_or(extensions_proto::moosync::types::PlayerState::Stopped)
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub fn get_volume(&self) -> u8 { self.active_player.get_volume().unwrap_or(100) }

    #[tracing::instrument(level = "debug", skip_all)]
    fn set_active_player(&mut self, player: Arc<Box<dyn PlayerExt>>) {
        self.active_player = player;
    }

    #[tracing::instrument(level = "debug", skip_all)]
    fn find_best_player(&mut self, song: &Song) -> Result<(), PlayerError> {
        let src = get_valid_src(song)?;
        for player in &self.players {
            if player.can_play(src.clone()) {
                self.set_active_player(player.clone());
                return Ok(());
            }
        }

        Err(PlayerError::NoPlayerFound(Box::new(song.clone())))
    }

    #[cfg(test)]
    #[tracing::instrument(level = "debug", skip_all)]
    pub fn can_play(&self, src: ValidSrc) -> bool { self.active_player.can_play(src) }

    #[tracing::instrument(level = "debug", skip_all)]
    pub fn load(&mut self, song: &Song) -> Result<(), PlayerError> {
        self.find_best_player(song)?;
        let src = get_valid_src(song)?;
        self.active_player.set_src(src)
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub fn play(&self) -> Result<(), PlayerError> { self.active_player.play() }

    #[tracing::instrument(level = "debug", skip_all)]
    pub fn pause(&self) -> Result<(), PlayerError> { self.active_player.pause() }

    #[tracing::instrument(level = "debug", skip_all)]
    pub fn stop(&self) -> Result<(), PlayerError> { self.active_player.stop() }

    #[tracing::instrument(level = "debug", skip_all)]
    pub fn set_volume(&self, volume: u8) -> Result<(), PlayerError> {
        self.active_player.set_volume(volume)
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub fn seek(&self, position: Duration) -> Result<(), PlayerError> {
        self.active_player.seek(position)
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub fn get_current_pos(&self) -> Result<Duration, PlayerError> {
        self.active_player.get_current_pos()
    }
}
