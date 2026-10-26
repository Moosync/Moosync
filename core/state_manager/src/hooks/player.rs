use std::{error::Error, sync::Arc};

use async_trait::async_trait;
use extensions::ExtensionGroupExt;
use extensions_proto::moosync::types::{
    ExtensionProviderScope, PlaybackDetailsRequestedRequest, PlaybackDetailsRequestedResponse,
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
    fn default() -> Self { Self::new() }
}

impl PlayerHook {
    #[tracing::instrument(level = "debug", skip_all)]
    pub fn new() -> Self { Self }
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
        // TODO: Bind source resolver to extensions
        player_handler.set_resolver(Arc::new(ExtensionSourceResolver {
            state_manager: state_manager.clone(),
        }));

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
