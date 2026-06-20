use std::error::Error;

use async_trait::async_trait;
use player::PlayerHandler;

use super::Hook;
use crate::StateManager;

pub struct PlayerHook;

impl PlayerHook {
    pub fn new() -> Self { Self }
}

#[async_trait]
impl Hook for PlayerHook {
    async fn on_startup(
        &self,
        state_manager: &StateManager,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        let player_handler = state_manager.plugins.get::<PlayerHandler>();
        player_handler
            .read()
            .await
            .set_resolver(Box::new(|_| Ok("".into())));

        Ok(())
    }
}
