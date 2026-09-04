// Moosync
// Copyright (C) 2024, 2025  Moosync <support@moosync.app>
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <http://www.gnu.org/licenses/>.

use std::error::Error;

use async_trait::async_trait;
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

    #[tracing::instrument(level = "debug", skip_all)]
    async fn fetch_and_update_manifests(state_manager: &StateManager) {
        let mut extensions = state_manager.get_extension_handler_mut().await;
        if let Ok(manifests) = extensions.get_extension_manifest().await {
            extensions.set_remote_manifests(manifests);
            extensions.check_for_updates();
            extensions.trigger_extensions_updated();
            return;
        }

        tracing::error!("Failed to fetch remote manifests");
    }
}

#[async_trait]
impl Hook for ExtensionRegistriesHook {
    #[tracing::instrument(level = "debug", skip_all)]
    async fn on_startup(
        &self,
        state_manager: &StateManager,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        let preferences = state_manager.get_preference_config().await;

        if let Ok(saved_registries) = preferences.load(preferences::keys::ExtensionRegistries) {
            let mut extensions = state_manager.get_extension_handler_mut().await;
            extensions.set_registries(saved_registries.into_iter().collect());
        }

        preferences.on_preference_changed_immediate(
            {
                let state_manager = state_manager.clone();
                move |_key| {
                    let state_manager = state_manager.clone();
                    tokio::spawn(
                        async move {
                            let preferences = state_manager.get_preference_config().await;
                            let Ok(registries) =
                                preferences.load(preferences::keys::ExtensionRegistries)
                            else {
                                return;
                            };
                            {
                                let mut extensions =
                                    state_manager.get_extension_handler_mut().await;
                                extensions.set_registries(registries.into_iter().collect());
                            }
                            Self::fetch_and_update_manifests(&state_manager).await;
                        }
                        .in_current_span(),
                    );
                }
            },
            preferences::keys::ExtensionRegistries,
        );

        let state_manager = state_manager.clone();
        tokio::spawn(
            async move {
                Self::fetch_and_update_manifests(&state_manager).await;
            }
            .in_current_span(),
        );

        Ok(())
    }
}
