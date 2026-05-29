use std::sync::Arc;

use songs_proto::moosync::types::Song;
use types::prelude::SongsExt;

use crate::{
    error::PlayerError,
    generic::PlayerExt,
    rodio::RodioPlayer,
    source::{ValidSrc, get_valid_src},
};

pub(crate) struct MuxPlayer {
    players: Vec<Arc<Box<dyn PlayerExt>>>,
    active_player: Arc<Box<dyn PlayerExt>>,
}

// Common holder for all players
// Decides which player to use for a given song and abstracts calls on the active player
impl MuxPlayer {
    pub fn new() -> Self {
        let players: Vec<Arc<Box<dyn PlayerExt>>> = vec![
            Arc::new(Box::new(RodioPlayer::new())),
            // Player::Spotify(LibrespotHolder::new()),
        ];
        Self {
            active_player: players[0].clone(),
            players,
        }
    }

    fn set_active_player(&mut self, player: Arc<Box<dyn PlayerExt>>) {
        self.active_player = player;
    }

    fn find_best_player(&mut self, song: &Song) -> Result<(), PlayerError> {
        let src = get_valid_src(&song)?;
        for player in &self.players {
            if player.can_play(src.clone()) {
                self.set_active_player(player.clone());
                return Ok(());
            }
        }

        Err(PlayerError::NoPlayerFound(song.to_string()))
    }

    pub fn load(&mut self, song: &Song) -> Result<(), PlayerError> {
        let src = get_valid_src(song)?;
        self.find_best_player(song)?;
        // SAFETY: find_best_player should throw an error if no player was found. At this point active player is not None.
        self.active_player.set_src(src)?;
        Ok(())
    }
}

impl PlayerExt for MuxPlayer {
    fn play(&self) -> Result<(), PlayerError> {
        self.active_player.play()
    }

    fn pause(&self) -> Result<(), PlayerError> {
        self.active_player.pause()
    }

    fn stop(&self) -> Result<(), PlayerError> {
        self.active_player.stop()
    }

    fn set_volume(&self, volume: f32) -> Result<(), PlayerError> {
        self.active_player.set_volume(volume)
    }

    fn seek(&self, position: f64) -> Result<(), PlayerError> {
        self.active_player.seek(position)
    }

    fn set_src(&self, src: ValidSrc) -> Result<(), PlayerError> {
        self.active_player.set_src(src)
    }

    // WARN: Do not call can_play on the mux player, it will always return false
    // can_play should only be called on the individual players
    fn can_play(&self, _: ValidSrc) -> bool {
        tracing::warn!("Do not call can_play on the mux player, it will always return false");
        false
    }
}

impl Default for MuxPlayer {
    fn default() -> Self {
        Self::new()
    }
}
