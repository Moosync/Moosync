use std::{
    sync::{
        Arc, RwLock,
        atomic::{AtomicUsize, Ordering},
    },
    time::Duration,
};

use extensions_proto::moosync::types::{
    PlayerEvent as ProtoPlayerEvent, PlayerState,
    player_event::Event::{self as PlayerEvent, Ended},
};
use songs_proto::moosync::types::Song;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender, unbounded_channel};
use tracing::Instrument;
use types::subscription::SubscriberList;

use crate::{
    context::AudioPlayerContext,
    error::PlayerError,
    source::{SourceResolver, SourceResolverHolder, ValidSrc, get_valid_src},
};

pub struct MuxPlayer {
    players: RwLock<Vec<Arc<dyn AudioPlayerContext>>>,
    active_idx: AtomicUsize,
    events_tx: RwLock<UnboundedSender<PlayerEvent>>,
    pub on_ended_callback: SubscriberList<()>,
    pub on_player_event: SubscriberList<ProtoPlayerEvent>,
    source_resolver: Arc<SourceResolverHolder>,
}

impl MuxPlayer {
    #[tracing::instrument(level = "debug", skip_all)]
    fn spawn_events_listener(
        mut events_rx: UnboundedReceiver<PlayerEvent>,
        on_ended_callback: SubscriberList<()>,
        on_player_event: SubscriberList<ProtoPlayerEvent>,
    ) {
        tokio::spawn(
            async move {
                while let Some(event) = events_rx.recv().await {
                    if let Ended(_) = event {
                        on_ended_callback.emit(());
                    }
                    on_player_event.emit(ProtoPlayerEvent { event: Some(event) });
                }
            }
            .in_current_span(),
        );
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub fn new_with_contexts(
        on_ended_callback: SubscriberList<()>,
        players: Vec<Arc<dyn AudioPlayerContext>>,
    ) -> Self {
        assert!(
            !players.is_empty(),
            "MuxPlayer requires at least one player"
        );
        let (events_tx, events_rx) = unbounded_channel();
        let on_player_event = SubscriberList::new();
        Self::spawn_events_listener(
            events_rx,
            on_ended_callback.clone(),
            on_player_event.clone(),
        );

        Self {
            players: RwLock::new(players),
            active_idx: AtomicUsize::new(0),
            events_tx: RwLock::new(events_tx),
            on_ended_callback,
            on_player_event,
            source_resolver: Arc::new(SourceResolverHolder::new()),
        }
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub(crate) fn get_resolver(&self) -> Arc<SourceResolverHolder> { self.source_resolver.clone() }

    #[tracing::instrument(level = "debug", skip_all)]
    pub fn add_player(&self, player: Arc<dyn AudioPlayerContext>) {
        let id = player.get_id().to_string();
        let mut players = self.players.write().unwrap();
        players.retain(|existing| existing.get_id() != id);
        players.push(player);
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub fn remove_player(&self, id: &str) {
        let mut players = self.players.write().unwrap();
        players.retain(|existing| existing.get_id() != id);
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
            .map(|p| p.get_id().to_string())
            .collect()
    }

    #[tracing::instrument(level = "debug", skip_all)]
    fn get_active_player(&self) -> Option<Arc<dyn AudioPlayerContext>> {
        let players = self.players.read().unwrap();
        let idx = self.active_idx.load(Ordering::SeqCst);
        players.get(idx).cloned()
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
        let (events_tx, events_rx) = unbounded_channel();
        Self::spawn_events_listener(
            events_rx,
            self.on_ended_callback.clone(),
            self.on_player_event.clone(),
        );
        *self.events_tx.write().unwrap() = events_tx.clone();
        events_tx
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub async fn load_src<'a>(&self, src: ValidSrc<'a>) -> Result<(), PlayerError> {
        let current_idx = self.active_idx.load(Ordering::SeqCst);
        let active_player = {
            let players = self.players.read().unwrap();
            players.get(current_idx).cloned()
        };

        if let Some(player) = active_player
            && player.can_play(&src).await
        {
            let events_tx = self.events_tx.read().unwrap().clone();
            match player.set_src(src.clone(), events_tx).await {
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

        let players = self.players.read().unwrap().clone();
        for (idx, player) in players.into_iter().enumerate() {
            if idx == current_idx {
                continue;
            }
            if player.can_play(&src).await {
                let new_events_tx = self.create_new_events_tx();
                match player.set_src(src.clone(), new_events_tx).await {
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
    async fn try_player_by_id(&self, target_id: &str, src: &ValidSrc<'_>) -> bool {
        let players = self.players.read().unwrap().clone();
        for (idx, player) in players.into_iter().enumerate() {
            if player.get_id() == target_id && player.can_play(src).await {
                let events_tx = self.create_new_events_tx();
                if player.set_src(src.clone(), events_tx).await.is_ok() {
                    self.active_idx.store(idx, Ordering::SeqCst);
                    return true;
                }
            }
        }
        false
    }

    #[tracing::instrument(level = "debug", skip_all)]
    async fn try_other_extension_players(
        &self,
        exclude_id: Option<&str>,
        src: &ValidSrc<'_>,
    ) -> bool {
        let players = self.players.read().unwrap().clone();
        for (idx, player) in players.into_iter().enumerate() {
            let pid = player.get_id();
            if pid.starts_with("extension:")
                && exclude_id != Some(pid)
                && player.can_play(src).await
            {
                let events_tx = self.create_new_events_tx();
                if player.set_src(src.clone(), events_tx).await.is_ok() {
                    self.active_idx.store(idx, Ordering::SeqCst);
                    return true;
                }
            }
        }
        false
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub async fn load_song(&self, song: &Song) -> Result<(), PlayerError> {
        // 1. Check if we already have a playback url/valid src, if yes try to
        //    play it
        if let Ok(src) = get_valid_src(song)
            && self.load_src(src).await.is_ok()
        {
            return Ok(());
        }

        // 2. If no, check if the same extension which gave us the song can
        //    resolve the URL, if yes then get the URL from that extension and
        //    play it
        if let Ok(resp) = self.source_resolver.resolve(song, false).await {
            let src = ValidSrc::new_url_with_key(resp.url, resp.decryption_key);
            if self.load_src(src).await.is_ok() {
                return Ok(());
            }
        }

        let origin_ext = song.song.as_ref().and_then(|s| s.extension.as_deref());
        let origin_player_id = origin_ext.map(|ext| format!("extension:{ext}"));

        // Candidate src from song url for extension players (Rules 3 and 4)
        let candidate_src = song
            .song
            .as_ref()
            .and_then(|s| s.url.as_deref())
            .map(ValidSrc::new_url);

        // 3. If no, check if the extension implements Player provider scope, if
        //    yes try to play the song through that extension (rely on can_play)
        if let Some(ref target_id) = origin_player_id
            && let Some(ref src) = candidate_src
            && self.try_player_by_id(target_id, src).await
        {
            return Ok(());
        }

        // 4. If no, fall back to other extensions to handle the song: 4a. Fall
        //    back to other extensions to resolve the URL
        if let Ok(resp) = self.source_resolver.resolve_fallback(song, false).await {
            let src = ValidSrc::new_url_with_key(resp.url, resp.decryption_key);
            if self.load_src(src).await.is_ok() {
                return Ok(());
            }
        }

        //    4b. Fall back to other extensions to play using their player
        // provider
        if let Some(ref src) = candidate_src
            && self
                .try_other_extension_players(origin_player_id.as_deref(), src)
                .await
        {
            return Ok(());
        }

        Err(PlayerError::NoPlayerFound(Box::new(song.clone())))
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub async fn play(&self) -> Result<(), PlayerError> {
        let Some(player) = self.get_active_player() else {
            return Err(PlayerError::NoActivePlayer);
        };
        player.play().await
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub async fn pause(&self) -> Result<(), PlayerError> {
        let Some(player) = self.get_active_player() else {
            return Err(PlayerError::NoActivePlayer);
        };
        player.pause().await
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub async fn stop(&self) -> Result<(), PlayerError> {
        let Some(player) = self.get_active_player() else {
            return Err(PlayerError::NoActivePlayer);
        };
        player.stop().await
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub async fn set_volume(&self, volume: u8) -> Result<(), PlayerError> {
        let Some(player) = self.get_active_player() else {
            return Err(PlayerError::NoActivePlayer);
        };
        player.set_volume(volume).await
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub async fn seek(&self, position: Duration) -> Result<(), PlayerError> {
        let Some(player) = self.get_active_player() else {
            return Err(PlayerError::NoActivePlayer);
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
