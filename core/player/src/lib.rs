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

pub use context::{AudioPlayerContext, RodioPlayerContext};
#[cfg(any(test, feature = "test"))]
pub use context::{DummyAudioPlayerContext, DummyPersistContext};
use extensions_proto::moosync::types::{
    LoadingEvent, PlayerEvent, PlayerState, player_event::Event,
};
pub use mux_player::MuxPlayer;
pub use player_data_ext::PlayerDataExt;
use player_proto::moosync::types::{PlayerData, RepeatMode};
use songs_proto::moosync::types::{PlaybackUrl, Song};
pub use source::SourceResolver;
use source::get_valid_src;
use tokio::{
    sync::mpsc::{UnboundedSender, unbounded_channel},
    time::interval,
};
use tracing::{Instrument, debug};
use types::{
    plugin::{Plugin, PluginContext},
    prelude::{SongsExt, core_to_proto_duration},
    subscription::SubscriberList,
};

use crate::{
    context::{PersistContext, file_persist::FilePersist},
    error::PlayerError,
};

pub struct PlayerHandler {
    pub(crate) player_data: Arc<RwLock<PlayerData>>,
    pub(crate) player: MuxPlayer,
    pub(crate) persist_context: Arc<Box<dyn PersistContext>>,
    pub on_song_changed: SubscriberList<Option<Song>>,
    pub on_queue_updated: SubscriberList<Vec<Song>>,
    pub on_repeat_changed: SubscriberList<RepeatMode>,
    pub on_player_event: SubscriberList<PlayerEvent>,
    pub on_volume_changed: SubscriberList<u8>,
    pub on_seeked: SubscriberList<Duration>,
}

#[plugin_macro::generate]
impl PlayerHandler {
    #[tracing::instrument(level = "debug", skip_all)]
    pub fn new(ended_tx: UnboundedSender<()>, data_dir: PathBuf) -> Self {
        Self::new_with_contexts(
            ended_tx,
            vec![
                Arc::new(RodioPlayerContext::new()),
                #[cfg(any(test, feature = "test"))]
                Arc::new(DummyAudioPlayerContext::new()),
            ],
            Box::new(FilePersist::new(data_dir)),
        )
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub fn new_with_contexts(
        ended_tx: UnboundedSender<()>,
        contexts: Vec<Arc<dyn AudioPlayerContext>>,
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
            player_data: Arc::new(RwLock::new(player_data)),
            player: MuxPlayer::new_with_contexts(on_ended_list, contexts),
            persist_context: Arc::new(persist_context),
            on_song_changed: SubscriberList::new(),
            on_queue_updated: SubscriberList::new(),
            on_repeat_changed: SubscriberList::new(),
            on_player_event: SubscriberList::new(),
            on_volume_changed: SubscriberList::new(),
            on_seeked: SubscriberList::new(),
        }
    }

    #[cfg(any(test, feature = "test"))]
    #[tracing::instrument(level = "debug", skip_all)]
    pub fn new_dummy(ended_tx: UnboundedSender<()>) -> Self {
        Self::new_with_contexts(
            ended_tx,
            vec![Arc::new(DummyAudioPlayerContext::new())],
            Box::new(DummyPersistContext {}),
        )
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub fn add_context(&self, context: Arc<dyn AudioPlayerContext>) {
        self.player.add_player(context);
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub fn remove_context(&self, id: &str) { self.player.remove_player(id); }

    #[tracing::instrument(level = "debug", skip_all)]
    pub fn get_player_ids(&self) -> Vec<String> { self.player.get_player_ids() }

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
    fn persist_state(&self, is_playing: bool) {
        let pd = {
            let mut data = self.player_data.write().unwrap();
            data.is_playing = is_playing;
            data.clone()
        };
        if let Err(e) = self.persist_context.persist(&pd) {
            tracing::error!("Failed to persist player data: {}", e);
        }
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub async fn load_initial_song(&self, persist_playback_state: bool) {
        let (song, was_playing) = {
            let data = self.player_data.read().unwrap();
            (self.current_song(), data.is_playing)
        };
        let Some(song) = song else {
            return;
        };

        self.trigger_player_event(Event::Loading(LoadingEvent {}));
        if let Err(e) = self.player.set_src(&song).await {
            tracing::error!("Failed to preload initial song on startup: {:?}", e);
            self.trigger_player_event(Event::Pause(true));
            return;
        }

        if persist_playback_state && was_playing {
            if let Err(e) = self.play().await {
                tracing::error!("Failed to resume playback on startup: {:?}", e);
            }
        } else {
            self.trigger_player_event(Event::Pause(true));
        }
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub async fn play(&self) -> Result<(), PlayerError> {
        if self.player.get_player_state() == PlayerState::Stopped
            && let Some(song) = self.current_song()
        {
            self.trigger_player_event(Event::Loading(LoadingEvent {}));
            if let Err(e) = self.player.set_src(&song).await {
                self.trigger_player_event(Event::Pause(true));
                self.persist_state(false);
                return Err(PlayerError::PlaybackUrlResolutionFailed(format!(
                    "Failed to load song on play: {:?}",
                    e
                )));
            }
        }
        if let Err(e) = self.player.play().await {
            self.trigger_player_event(Event::Pause(true));
            self.persist_state(false);
            return Err(e);
        }
        self.persist_state(true);
        self.trigger_player_event(Event::Play(true));
        Ok(())
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub async fn pause(&self) -> Result<(), PlayerError> {
        self.player.pause().await?;
        self.persist_state(false);
        self.trigger_player_event(Event::Pause(true));
        Ok(())
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub async fn stop(&self) -> Result<(), PlayerError> {
        self.player.stop().await?;
        self.persist_state(false);
        self.trigger_player_event(Event::Pause(true));
        Ok(())
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub async fn next(&self) {
        if self.player_data.read().unwrap().song_queue.is_empty() {
            let _ = self.player.stop().await;
            self.trigger_player_event(Event::Pause(true));
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
    pub async fn set_volume(&self, volume: u8) {
        if let Err(e) = self.player.set_volume(volume).await {
            tracing::error!("Failed to set volume: {:?}", e);
            return;
        }
        self.on_volume_changed.emit(volume);
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub async fn seek(&self, pos: Duration) {
        if let Err(e) = self.player.seek(pos).await {
            tracing::error!("Failed to seek: {:?}", e);
        }
        if let Ok(pos) = self.player.get_current_pos() {
            self.trigger_player_event(Event::TimeUpdate(core_to_proto_duration(pos)));
            self.on_seeked.emit(pos);
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
    pub async fn remove_from_queue(&self, idx: usize) {
        if idx >= self.get_queue().len() {
            return;
        }

        let is_current = idx == self.get_current_idx();
        let was_playing = self.player.get_player_state() == PlayerState::Playing;

        {
            let mut data = self.player_data.write().unwrap();
            data.remove_at_index(idx);
        }
        self.trigger_queue_changed();

        if is_current {
            self.handle_current_removed(was_playing).await;
        }
    }

    #[tracing::instrument(level = "debug", skip_all)]
    async fn handle_current_removed(&self, was_playing: bool) {
        self.trigger_song_changed().await;
        if self.current_song().is_none() {
            return;
        }

        if was_playing {
            if let Err(e) = self.play().await {
                tracing::error!("Failed to play song: {:?}", e);
            }
            return;
        }

        let _ = self.pause().await;
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
    pub fn set_resolver(&self, resolver: Arc<dyn SourceResolver>) {
        self.player.set_resolver(resolver);
    }

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
        self.on_song_changed.emit(current.clone());
        self.trigger_player_event(Event::TimeUpdate(Default::default()));

        if let Some(song) = current.as_ref() {
            self.trigger_player_event(Event::Loading(LoadingEvent {}));
            if let Err(e) = self.player.set_src(song).await {
                tracing::error!("Failed to load song: {:?}", e);
                let _ = self.player.stop().await;
                self.trigger_player_event(Event::Pause(true));
            }
        } else {
            let _ = self.player.stop().await;
            self.trigger_player_event(Event::Pause(true));
        }

        let pd = self.player_data.read().unwrap().clone();
        if let Err(e) = self.persist_context.persist(&pd) {
            tracing::error!("Failed to persist player data: {}", e);
        }
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub fn update_song_in_queue(&self, updated_song: &Song) {
        let Some(target_id) = updated_song.song.as_ref().and_then(|s| s.id.as_deref()) else {
            return;
        };
        let mut merged_current: Option<Song> = None;
        let mut changed = false;
        {
            let mut data = self.player_data.write().unwrap();
            let current_idx = data.current_idx as usize;
            for (idx, queued) in data.song_queue.iter_mut().enumerate() {
                if queued.song.as_ref().and_then(|s| s.id.as_deref()) == Some(target_id) {
                    queued.merge(updated_song);
                    changed = true;
                    if idx == current_idx {
                        merged_current = Some(queued.clone());
                    }
                }
            }
        }
        if changed {
            let pd = self.player_data.read().unwrap().clone();
            if let Err(e) = self.persist_context.persist(&pd) {
                tracing::error!("Failed to persist player data: {}", e);
            }
        }
        if let Some(current) = merged_current {
            self.on_song_changed.emit(Some(current));
        }
    }
}

impl Plugin for PlayerHandler {
    #[tracing::instrument(level = "debug", skip_all)]
    fn init(context: &PluginContext) -> Arc<Self> {
        let (ended_tx, ended_rx) = unbounded_channel();
        let ph = Arc::new(PlayerHandler::new(ended_tx, context.data_dir.clone()));

        let ph_worker = ph.clone();
        tokio::spawn(
            async move {
                ph_worker.run_worker_loop(ended_rx).await;
            }
            .in_current_span(),
        );

        ph
    }
}

impl PlayerHandler {
    #[tracing::instrument(level = "debug", skip_all)]
    async fn run_worker_loop(&self, mut ended_rx: tokio::sync::mpsc::UnboundedReceiver<()>) {
        let mut position_ticker = interval(Duration::from_millis(250));
        let mut prefetch_ticker = interval(Duration::from_secs(2));
        loop {
            tokio::select! {
                ended = ended_rx.recv() => {
                    if ended.is_none() {
                        break;
                    }
                    self.on_song_ended().await;
                }
                _ = position_ticker.tick() => {
                    self.emit_current_position();
                }
                _ = prefetch_ticker.tick() => {
                    for song in self.upcoming_songs_to_prefetch() {
                        self.prefetch_song(&song).await;
                    }
                }
            }
        }
    }

    #[tracing::instrument(level = "debug", skip_all)]
    fn emit_current_position(&self) {
        if self.player.get_player_state() != PlayerState::Playing {
            return;
        }
        if let Ok(pos) = self.player.get_current_pos() {
            self.trigger_player_event(Event::TimeUpdate(core_to_proto_duration(pos)));
        }
    }

    #[tracing::instrument(level = "debug", skip_all)]
    fn upcoming_songs_to_prefetch(&self) -> Vec<Song> {
        let data = self.player_data.read().unwrap();
        let start = (data.current_idx as usize).saturating_add(1);
        let end = (start + 15).min(data.song_queue.len());
        if start >= end {
            return Vec::new();
        }
        data.song_queue[start..end].to_vec()
    }

    #[tracing::instrument(level = "debug", skip_all)]
    async fn prefetch_song(&self, song: &Song) {
        if get_valid_src(song).is_ok() {
            return;
        }
        let Ok(resp) = self.player.get_resolver().resolve(song, true).await else {
            return;
        };
        if resp.url.is_empty() {
            return;
        }
        let Some(song_id) = song.get_id() else {
            return;
        };

        let new_pb = PlaybackUrl {
            url: resp.url,
            expires_at: resp.expires_at,
            decryption_key: resp.decryption_key,
        };
        let updated_pd = {
            let mut data = self.player_data.write().unwrap();
            let mut updated = false;
            for queued in &mut data.song_queue {
                if queued.get_id().as_deref() == Some(song_id.as_ref())
                    && let Some(ref mut inner) = queued.song
                {
                    inner.playback_url = Some(new_pb.clone());
                    updated = true;
                }
            }
            if updated { Some(data.clone()) } else { None }
        };
        if let Some(pd) = updated_pd
            && let Err(e) = self.persist_context.persist(&pd)
        {
            tracing::error!("Failed to persist prefetched playback URL: {}", e);
        }
    }
}
