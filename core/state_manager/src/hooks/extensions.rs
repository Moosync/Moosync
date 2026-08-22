use std::{error::Error, sync::Arc};

use async_trait::async_trait;
use extensions::ExtensionHandler;

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
        let extensions = state_manager.plugins.get::<ExtensionHandler>();
        let extensions_cl = extensions.clone();

        let reply_handler = Arc::new(StateReplyHandler::new(state_manager.clone()));
        let mut ext_handle = extensions.write().await;
        ext_handle.set_reply_handler(reply_handler);

        tokio::spawn(async move {
            let ext_handle = extensions_cl.read().await;
            if let Err(e) = ext_handle.find_new_extensions() {
                tracing::error!("Failed to find new extensions: {:?}", e);
            }
        });

        Ok(())
    }
}
