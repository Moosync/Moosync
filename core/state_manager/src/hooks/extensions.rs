use std::{error::Error, sync::Arc};

use async_trait::async_trait;

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
        let preferences = state_manager.get_preference_config().await;

        let reply_handler = Arc::new(StateReplyHandler::new(state_manager.clone()));
        extensions.set_reply_handler(reply_handler);

        if let Ok(saved_registries) = preferences.load(preferences::keys::ExtensionRegistries) {
            extensions.set_registries(saved_registries.into_iter().collect());
        }

        preferences.on_preference_changed_immediate(
            {
                let state_manager = state_manager.clone();
                move |_key| {
                    let state_manager = state_manager.clone();
                    tokio::spawn(async move {
                        let preferences = state_manager.get_preference_config().await;
                        if let Ok(registries) =
                            preferences.load(preferences::keys::ExtensionRegistries)
                        {
                            let mut extensions = state_manager.get_extension_handler_mut().await;
                            extensions.set_registries(registries.into_iter().collect());
                            if let Err(e) = extensions.get_extension_manifest().await {
                                tracing::error!("Failed to fetch remote manifests: {:?}", e);
                            }
                            extensions.trigger_extensions_updated();
                        }
                    });
                }
            },
            preferences::keys::ExtensionRegistries,
        );

        let state_manager = state_manager.clone();
        tokio::spawn(async move {
            let extensions = state_manager.get_extension_handler().await;
            if let Err(e) = extensions.find_new_extensions() {
                tracing::error!("Failed to find new extensions: {:?}", e);
            }
            if let Err(e) = extensions.get_extension_manifest().await {
                tracing::error!("Failed to fetch remote manifests on startup: {:?}", e);
            }
            extensions.trigger_extensions_updated();
        });

        Ok(())
    }
}
