use std::{error::Error, sync::Arc};

use async_trait::async_trait;
use extensions_proto::moosync::types::{ExtensionProviderScope, SongAddedRequest};
use songs_proto::moosync::types::Song;
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

#[tracing::instrument(level = "debug", skip_all)]
async fn notify_extensions_songs_added(state_manager: StateManager, songs: Vec<Song>) {
    let extensions = state_manager.get_extension_handler();
    let targets = extensions
        .get_extensions_with_scope(ExtensionProviderScope::DatabaseSongEvents)
        .await;
    for ext in targets {
        if let Err(e) = ext
            .song_added(SongAddedRequest {
                songs: songs.clone(),
            })
            .await
        {
            tracing::error!(
                "Failed to send song_added event to extension {}: {:?}",
                ext.manifest.package_name,
                e
            );
        }
    }
}

#[async_trait]
impl Hook for ExtensionsHook {
    #[tracing::instrument(level = "debug", skip_all)]
    async fn on_startup(
        &self,
        state_manager: &StateManager,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        let extensions = state_manager.get_extension_handler();
        let reply_handler = Arc::new(StateReplyHandler::new(state_manager.clone()));
        extensions.set_reply_handler(reply_handler);

        let sm_for_db = state_manager.clone();
        state_manager
            .get_database()
            .on_songs_added
            .listen(move |songs| {
                if songs.is_empty() {
                    return;
                }
                let sm = sm_for_db.clone();
                tokio::spawn(notify_extensions_songs_added(sm, songs).in_current_span());
            });

        let state_manager = state_manager.clone();
        tokio::spawn(
            async move {
                let extensions = state_manager.get_extension_handler();
                if let Err(e) = extensions.find_new_extensions() {
                    tracing::error!("Failed to find new extensions: {:?}", e);
                }
                extensions.trigger_extensions_updated();
            }
            .in_current_span(),
        );

        Ok(())
    }

    #[tracing::instrument(level = "debug", skip_all)]
    async fn on_exit(
        &self,
        state_manager: &StateManager,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        state_manager
            .get_extension_handler()
            .notify_app_exit()
            .await;
        Ok(())
    }
}
