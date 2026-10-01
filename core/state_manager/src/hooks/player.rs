use std::error::Error;

use async_trait::async_trait;
use extensions_proto::moosync::types::{ExtensionProviderScope, PlaybackDetailsRequestedRequest};
use player::{PlayerHandler, error::PlayerError};
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

#[async_trait]
impl Hook for PlayerHook {
    #[tracing::instrument(level = "debug", skip_all)]
    async fn on_startup(
        &self,
        state_manager: &StateManager,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        let player_handler = state_manager.plugins.get::<PlayerHandler>();
        // TODO: Bind source resolver to extensions
        player_handler.set_resolver({
            let state_manager = state_manager.clone();
            Box::new(move |song| {
                let state_manager = state_manager.clone();
                let song = song.clone();
                Box::new(Box::pin(
                    async move {
                        let extension_handler = state_manager.get_extension_handler();
                        let extensions = extension_handler
                            .get_extensions_with_scope(ExtensionProviderScope::PlaybackDetails)
                            .await;
                        for ext in extensions {
                            match ext
                                .playback_details_requested(PlaybackDetailsRequestedRequest {
                                    song: Some(song.clone()),
                                })
                                .await
                            {
                                Ok(resp) => return Ok(resp),
                                Err(err) => {
                                    error!(
                                        error = %err,
                                        "Failed to fetch playback details from extension"
                                    );
                                }
                            }
                        }
                        Err(Box::new(PlayerError::NoSrcFound(Box::new(song.clone())))
                            as Box<dyn Error + Send + Sync>)
                    }
                    .in_current_span(),
                ))
            })
        });

        Ok(())
    }
}
