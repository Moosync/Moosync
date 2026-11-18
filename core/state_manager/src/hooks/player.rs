use std::{collections::HashSet, error::Error, sync::Arc};

use async_trait::async_trait;
use extensions_proto::moosync::types::{
    ExtensionProviderScope, PlaybackDetailsRequestedRequest, PlaybackDetailsRequestedResponse,
    PlayerState, PlayerStateChangedRequest, QueueState, SeekedRequest, SongChangedRequest,
    SongQueueChangedRequest, VolumeChangedRequest, player_event::Event as ProtoPlayerEvent,
};
use player::{PlayerHandler, SourceResolver, error::PlayerError};
use preferences::keys::{CLEAR_QUEUE, PreferenceItemExt};
use songs_proto::moosync::types::Song;
use tracing::{Instrument, error};
use types::{player::AudioPlayerContext, prelude::ScopeListExt};

use super::Hook;
use crate::StateManager;

pub struct PlayerHook;

impl Default for PlayerHook {
    #[tracing::instrument(level = "debug", skip_all)]
    fn default() -> Self { Self::new() }
}

impl PlayerHook {
    #[tracing::instrument(level = "debug", skip_all)]
    pub fn new() -> Self { Self }

    #[tracing::instrument(level = "debug", skip_all)]
    fn bind_player_data_events(state_manager: &StateManager, player_handler: &Arc<PlayerHandler>) {
        let sm_song = state_manager.clone();
        player_handler.on_song_changed.listen(move |song| {
            let sm = sm_song.clone();
            tokio::spawn(
                async move {
                    let extensions = sm
                        .get_extension_handler()
                        .get_extensions_with_scope(ExtensionProviderScope::PlayerDataEvents)
                        .await;
                    for ext in extensions {
                        if let Err(err) = ext
                            .song_changed(SongChangedRequest { song: song.clone() })
                            .await
                        {
                            error!(
                                "Failed to notify extension {} of song_changed: {:?}",
                                ext.get_package_name(),
                                err
                            );
                        }
                    }
                }
                .in_current_span(),
            );
        });

        let sm_queue = state_manager.clone();
        player_handler.on_queue_updated.listen(move |songs| {
            let sm = sm_queue.clone();
            let index = sm.get_player_handler().get_current_idx() as u64;
            tokio::spawn(
                async move {
                    let extensions = sm
                        .get_extension_handler()
                        .get_extensions_with_scope(ExtensionProviderScope::PlayerDataEvents)
                        .await;
                    let queue_state = Some(QueueState { songs, index });
                    for ext in extensions {
                        if let Err(err) = ext
                            .song_queue_changed(SongQueueChangedRequest {
                                queue_state: queue_state.clone(),
                            })
                            .await
                        {
                            error!(
                                "Failed to notify extension {} of song_queue_changed: {:?}",
                                ext.get_package_name(),
                                err
                            );
                        }
                    }
                }
                .in_current_span(),
            );
        });
    }

    #[tracing::instrument(level = "debug", skip_all)]
    fn bind_player_ui_events(state_manager: &StateManager, player_handler: &Arc<PlayerHandler>) {
        let sm_state = state_manager.clone();
        player_handler.on_player_event.listen(move |player_event| {
            let Some(event) = player_event.event else {
                return;
            };
            let state = match event {
                ProtoPlayerEvent::Play(_) => PlayerState::Playing,
                ProtoPlayerEvent::Pause(_) => PlayerState::Paused,
                ProtoPlayerEvent::Ended(_) => PlayerState::Stopped,
                ProtoPlayerEvent::Loading(_) => PlayerState::Loading,
                ProtoPlayerEvent::TimeUpdate(_) | ProtoPlayerEvent::Error(_) => return,
            };
            let sm = sm_state.clone();
            tokio::spawn(
                async move {
                    let extensions = sm
                        .get_extension_handler()
                        .get_extensions_with_scope(ExtensionProviderScope::PlayerUiEvents)
                        .await;
                    for ext in extensions {
                        if let Err(err) = ext
                            .player_state_changed(PlayerStateChangedRequest {
                                state: state as i32,
                            })
                            .await
                        {
                            error!(
                                "Failed to notify extension {} of player_state_changed: {:?}",
                                ext.get_package_name(),
                                err
                            );
                        }
                    }
                }
                .in_current_span(),
            );
        });

        let sm_vol = state_manager.clone();
        player_handler.on_volume_changed.listen(move |volume| {
            let sm = sm_vol.clone();
            tokio::spawn(
                async move {
                    let extensions = sm
                        .get_extension_handler()
                        .get_extensions_with_scope(ExtensionProviderScope::PlayerUiEvents)
                        .await;
                    for ext in extensions {
                        if let Err(err) = ext
                            .volume_changed(VolumeChangedRequest {
                                volume: volume as f64,
                            })
                            .await
                        {
                            error!(
                                "Failed to notify extension {} of volume_changed: {:?}",
                                ext.get_package_name(),
                                err
                            );
                        }
                    }
                }
                .in_current_span(),
            );
        });

        let sm_seek = state_manager.clone();
        player_handler.on_seeked.listen(move |pos| {
            let sm = sm_seek.clone();
            let position = pos.as_secs_f64();
            tokio::spawn(
                async move {
                    let extensions = sm
                        .get_extension_handler()
                        .get_extensions_with_scope(ExtensionProviderScope::PlayerUiEvents)
                        .await;
                    for ext in extensions {
                        if let Err(err) = ext.seeked(SeekedRequest { position }).await {
                            error!(
                                "Failed to notify extension {} of seeked: {:?}",
                                ext.get_package_name(),
                                err
                            );
                        }
                    }
                }
                .in_current_span(),
            );
        });
    }

    #[tracing::instrument(level = "debug", skip_all)]
    fn bind_extension_players(state_manager: &StateManager, player_handler: &Arc<PlayerHandler>) {
        let sm = state_manager.clone();
        let ph = Arc::clone(player_handler);
        let extension_handler = state_manager.get_extension_handler();

        let sm_clone = sm.clone();
        let ph_clone = Arc::clone(&ph);
        extension_handler.on_extensions_updated.listen(move |_| {
            Self::sync_extension_players(&sm_clone, &ph_clone);
        });

        Self::sync_extension_players(&sm, &ph);
    }

    #[tracing::instrument(level = "debug", skip_all)]
    fn sync_extension_players(state_manager: &StateManager, player_handler: &Arc<PlayerHandler>) {
        let sm_inner = state_manager.clone();
        let ph_inner = Arc::clone(player_handler);
        tokio::spawn(
            async move {
                let extensions = sm_inner
                    .get_extension_handler()
                    .get_extensions_with_scope(ExtensionProviderScope::PlayerProvider)
                    .await;

                let active_ids: HashSet<String> = extensions
                    .iter()
                    .map(|ext| ext.get_id().to_string())
                    .collect();

                for ext in extensions {
                    ph_inner.add_context(ext);
                }

                for player_id in ph_inner.get_player_ids() {
                    if player_id.starts_with("extension:") && !active_ids.contains(&player_id) {
                        ph_inner.remove_context(&player_id);
                    }
                }
            }
            .in_current_span(),
        );
    }
}

struct ExtensionSourceResolver {
    state_manager: StateManager,
}

#[async_trait]
impl SourceResolver for ExtensionSourceResolver {
    #[tracing::instrument(level = "debug", skip_all)]
    async fn resolve(
        &self,
        song: &Song,
    ) -> Result<PlaybackDetailsRequestedResponse, Box<dyn Error + Send + Sync>> {
        let extension_handler = self.state_manager.get_extension_handler();
        let ext_name = song.song.as_ref().and_then(|s| s.extension.as_deref());

        if let Some(ext) = ext_name
            && let Ok(extension) = extension_handler.get_extension(ext)
            && extension
                .manifest
                .contains_scope(ExtensionProviderScope::PlaybackDetails)
            && let Ok(resp) = extension
                .playback_details_requested(PlaybackDetailsRequestedRequest {
                    song: Some(song.clone()),
                })
                .await
            && !resp.url.is_empty()
        {
            return Ok(resp);
        }
        Err(Box::new(PlayerError::NoSrcFound(Box::new(song.clone()))))
    }

    #[tracing::instrument(level = "debug", skip_all)]
    async fn resolve_fallback(
        &self,
        song: &Song,
    ) -> Result<PlaybackDetailsRequestedResponse, Box<dyn Error + Send + Sync>> {
        let extension_handler = self.state_manager.get_extension_handler();
        let ext_name = song.song.as_ref().and_then(|s| s.extension.as_deref());

        let extensions = extension_handler
            .get_extensions_with_scope(ExtensionProviderScope::PlaybackDetails)
            .await;
        for ext in extensions {
            if ext_name == Some(ext.get_package_name()) {
                continue;
            }
            match ext
                .playback_details_requested(PlaybackDetailsRequestedRequest {
                    song: Some(song.clone()),
                })
                .await
            {
                Ok(resp) if !resp.url.is_empty() => return Ok(resp),
                Ok(_) => {}
                Err(err) => {
                    error!(
                        error = %err,
                        "Failed to fetch playback details from fallback extension {}",
                        ext.get_package_name()
                    );
                }
            }
        }
        Err(Box::new(PlayerError::NoSrcFound(Box::new(song.clone()))))
    }
}

#[async_trait]
impl Hook for PlayerHook {
    #[tracing::instrument(level = "debug", skip_all)]
    async fn on_startup(
        &self,
        state_manager: &StateManager,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        let player_handler = state_manager.plugins.get::<PlayerHandler>();
        player_handler.set_resolver(Arc::new(ExtensionSourceResolver {
            state_manager: state_manager.clone(),
        }));
        Self::bind_player_data_events(state_manager, &player_handler);
        Self::bind_player_ui_events(state_manager, &player_handler);
        Self::bind_extension_players(state_manager, &player_handler);

        Ok(())
    }

    #[tracing::instrument(level = "debug", skip_all)]
    async fn on_exit(
        &self,
        state_manager: &StateManager,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        let preferences = state_manager.get_preference_config();
        let should_clear = preferences
            .load(&CLEAR_QUEUE)
            .value::<bool>()
            .unwrap_or(false);
        if should_clear {
            let player_handler = state_manager.plugins.get::<PlayerHandler>();
            // Call twice to also clear the current song
            player_handler.clear_queue().await;
            player_handler.clear_queue().await;
        }
        Ok(())
    }
}
