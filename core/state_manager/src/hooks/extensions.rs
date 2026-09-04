use std::{error::Error, sync::Arc};

use async_trait::async_trait;
use tracing::Instrument;

use super::Hook;
use crate::{StateManager, reply_handler::StateReplyHandler};

pub struct ExtensionsHook;

impl Default for ExtensionsHook {
    fn default() -> Self { Self::new() }
}

impl ExtensionsHook {
    #[tracing::instrument(level = "debug", skip_all)]
    pub fn new() -> Self { Self }
}

#[async_trait]
impl Hook for ExtensionsHook {
    #[tracing::instrument(level = "debug", skip_all)]
    async fn on_startup(
        &self,
        state_manager: &StateManager,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        let mut extensions = state_manager.get_extension_handler_mut().await;
        let reply_handler = Arc::new(StateReplyHandler::new(state_manager.clone()));
        extensions.set_reply_handler(reply_handler);

        let state_manager = state_manager.clone();
        tokio::spawn(
            async move {
                let extensions = state_manager.get_extension_handler().await;
                if let Err(e) = extensions.find_new_extensions() {
                    tracing::error!("Failed to find new extensions: {:?}", e);
                }
                extensions.trigger_extensions_updated();
            }
            .in_current_span(),
        );

        Ok(())
    }
}
