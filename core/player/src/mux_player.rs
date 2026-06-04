use std::{sync::Arc, time::Duration};

use extensions_proto::moosync::types::player_event::Event as PlayerEvent;
use songs_proto::moosync::types::Song;
use tokio::sync::mpsc::UnboundedSender;

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
    pub fn new(events_tx: UnboundedSender<PlayerEvent>) -> Self {
        let players: Vec<Arc<Box<dyn PlayerExt>>> = vec![
            Arc::new(Box::new(RodioPlayer::new(events_tx))),
            // Player::Spotify(LibrespotHolder::new()),
        ];

        Self {
            active_player: players[0].clone(),
            players,
        }
    }

    pub fn get_player_state(&self) -> extensions_proto::moosync::types::PlayerState {
        self.active_player.get_player_state().unwrap_or(extensions_proto::moosync::types::PlayerState::Stopped)
    }

    pub fn get_volume(&self) -> u8 {
        self.active_player.get_volume().unwrap_or(100)
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

        Err(PlayerError::NoPlayerFound(song.clone()))
    }

    pub fn load(&mut self, song: &Song) -> Result<(), PlayerError> {
        let src = get_valid_src(song)?;
        self.find_best_player(song)?;
        // SAFETY: find_best_player should throw an error if no player was found.
        // At this point active player is not None.
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

    fn set_volume(&self, volume: u8) -> Result<(), PlayerError> {
        self.active_player.set_volume(volume)
    }

    fn seek(&self, position: Duration) -> Result<(), PlayerError> {
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

    fn get_current_pos(&self) -> Result<Duration, PlayerError> {
        self.active_player.get_current_pos()
    }

    fn get_volume(&self) -> Result<u8, PlayerError> {
        self.active_player.get_volume()
    }

    fn get_player_state(&self) -> Result<extensions_proto::moosync::types::PlayerState, PlayerError> {
        self.active_player.get_player_state()
    }
}
