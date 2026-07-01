use std::error::Error;

use async_trait::async_trait;

use crate::StateManager;

pub mod extensions;
pub mod player;
pub mod scanner;

#[async_trait]
pub trait Hook: Send + Sync {
    #[tracing::instrument(level = "debug", skip_all)]
    async fn on_startup(
        &self,
        _state_manager: &StateManager,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        Ok(())
    }

    #[tracing::instrument(level = "debug", skip_all)]
    async fn on_delayed_startup(
        &self,
        _state_manager: &StateManager,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        Ok(())
    }

    #[tracing::instrument(level = "debug", skip_all)]
    async fn on_exit(
        &self,
        _state_manager: &StateManager,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        Ok(())
    }
}
