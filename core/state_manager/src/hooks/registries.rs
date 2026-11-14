use std::error::Error;

use async_trait::async_trait;
use preferences::keys::{EXTENSION_REGISTRIES, PreferenceItemExt, THEME_REGISTRIES};
use tracing::Instrument;

use super::Hook;
use crate::StateManager;

pub struct ExtensionRegistriesHook;

impl Default for ExtensionRegistriesHook {
    fn default() -> Self { Self::new() }
}

impl ExtensionRegistriesHook {
    #[tracing::instrument(level = "debug", skip_all)]
    pub fn new() -> Self { Self }
}

#[async_trait]
impl Hook for ExtensionRegistriesHook {
    #[tracing::instrument(level = "debug", skip_all)]
    async fn on_startup(
        &self,
        state_manager: &StateManager,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        let preferences = state_manager.get_preference_config();

        preferences.on_preference_changed.listen_filtered_immediate(
            {
                let state_manager = state_manager.clone();
                move |_key| {
                    let state_manager = state_manager.clone();
                    tokio::spawn(
                        async move {
                            let preferences = state_manager.get_preference_config();
                            let registries = preferences
                                .load(&EXTENSION_REGISTRIES)
                                .value::<Vec<String>>()
                                .unwrap_or_default();
                            let extensions = state_manager.get_extension_handler();
                            if let Err(e) = extensions
                                .fetch_remote_manifests(registries.into_iter().collect())
                                .await
                            {
                                tracing::error!("Failed to fetch remote manifests: {:?}", e);
                            }
                        }
                        .in_current_span(),
                    );
                }
            },
            EXTENSION_REGISTRIES.id.clone(),
        );

        Ok(())
    }
}

pub struct ThemeRegistriesHook;

impl Default for ThemeRegistriesHook {
    fn default() -> Self { Self::new() }
}

impl ThemeRegistriesHook {
    #[tracing::instrument(level = "debug", skip_all)]
    pub fn new() -> Self { Self }
}

#[async_trait]
impl Hook for ThemeRegistriesHook {
    #[tracing::instrument(level = "debug", skip_all)]
    async fn on_startup(
        &self,
        state_manager: &StateManager,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        let preferences = state_manager.get_preference_config();

        preferences.on_preference_changed.listen_filtered_immediate(
            {
                let state_manager = state_manager.clone();
                move |_key| {
                    let state_manager = state_manager.clone();
                    tokio::spawn(
                        async move {
                            let preferences = state_manager.get_preference_config();
                            let registries = preferences
                                .load(&THEME_REGISTRIES)
                                .value::<Vec<String>>()
                                .unwrap_or_default();
                            let themes = state_manager.get_theme_holder();
                            if let Err(e) = themes
                                .fetch_remote_manifests(registries.into_iter().collect())
                                .await
                            {
                                tracing::error!("Failed to fetch remote theme manifests: {:?}", e);
                            }
                        }
                        .in_current_span(),
                    );
                }
            },
            THEME_REGISTRIES.id.clone(),
        );

        Ok(())
    }
}
