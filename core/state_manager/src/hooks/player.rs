use std::{error::Error, sync::Arc};

use async_trait::async_trait;
use extensions::ExtensionGroupExt;
use extensions_proto::moosync::types::{
    ExtensionProviderScope, PlaybackDetailsRequestedRequest, PlaybackDetailsRequestedResponse,
    PlayerState, PlayerStateChangedRequest, QueueState, SeekedRequest, SongChangedRequest,
    SongQueueChangedRequest, VolumeChangedRequest, player_event::Event as ProtoPlayerEvent,
};
use futures::StreamExt;
use player::{PlayerHandler, SourceResolver, error::PlayerError};
use preferences::keys::{CLEAR_QUEUE, PreferenceItemExt};
use songs_proto::moosync::types::Song;
use tracing::{Instrument, error};

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
        tracing::debug!("Resolving playback details for song: {:?}", song);
        let extension_handler = self.state_manager.get_extension_handler();
        let extensions = extension_handler
            .get_extensions_with_scope(ExtensionProviderScope::PlaybackDetails)
            .await;
        tracing::debug!(
            "Found {} extensions with PlaybackDetails scope",
            extensions.len()
        );
        let song_ref = song;
        let mut stream = extensions.with(|ext| {
            async move {
                ext.playback_details_requested(PlaybackDetailsRequestedRequest {
                    song: Some(song_ref.clone()),
                })
                .await
            }
            .in_current_span()
        });
        while let Some(res) = stream.next().await {
            match res {
                Ok(resp) => return Ok(resp),
                Err(err) => {
                    error!(
                        error = %err,
                        "Failed to fetch playback details from extension"
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
