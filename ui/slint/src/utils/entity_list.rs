use std::marker::PhantomData;

use slint::{ComponentHandle, ModelRc, Weak};
use state_manager::StateManager;
use tracing::Instrument;

use crate::{
    MainWindow,
    error::UiError,
    utils::{LazyModel, make_lazy_card_model},
};

#[async_trait::async_trait]
pub trait EntityListProvider: Send + Sync + 'static {
    type Entity: Send + Sync + 'static;
    type EntityModel: LazyModel + From<Self::Entity> + 'static;

    fn name() -> &'static str;
    fn to_model(entity: Self::Entity) -> Self::EntityModel { entity.into() }
    fn set_models(main_window: &MainWindow, model: ModelRc<Self::EntityModel>);
    fn clear_models(main_window: &MainWindow) { Self::set_models(main_window, ModelRc::default()); }

    async fn fetch_entities(state_manager: &StateManager) -> Result<Vec<Self::Entity>, UiError>;
}

pub struct EntityListCoordinator<P: EntityListProvider> {
    weak: Weak<MainWindow>,
    state_manager: StateManager,
    _phantom: PhantomData<P>,
}

impl<P: EntityListProvider> Clone for EntityListCoordinator<P> {
    #[tracing::instrument(level = "debug", skip_all)]
    fn clone(&self) -> Self {
        Self {
            weak: self.weak.clone(),
            state_manager: self.state_manager.clone(),
            _phantom: PhantomData,
        }
    }
}

impl<P: EntityListProvider> EntityListCoordinator<P> {
    #[tracing::instrument(level = "debug", skip_all)]
    pub fn new(main_window: &MainWindow, state_manager: &StateManager) -> Self {
        Self {
            weak: main_window.as_weak(),
            state_manager: state_manager.clone(),
            _phantom: PhantomData,
        }
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub fn on_show(&self) {
        tracing::debug!("EntityListCoordinator[{}]: on_show triggered", P::name());
        let state_manager = self.state_manager.clone();
        let weak = self.weak.clone();

        tokio::spawn(
            async move {
                match P::fetch_entities(&state_manager).await {
                    Ok(entities) => {
                        tracing::debug!(
                            "EntityListCoordinator[{}]: fetched {} items",
                            P::name(),
                            entities.len()
                        );
                        let _ = weak.upgrade_in_event_loop(move |main_window| {
                            let models: Vec<P::EntityModel> =
                                entities.into_iter().map(P::to_model).collect();
                            let model = make_lazy_card_model(&main_window, &state_manager, models);
                            P::set_models(&main_window, model);
                        });
                    }
                    Err(error) => {
                        tracing::error!(
                            "EntityListCoordinator[{}]: Failed to fetch entity list: {:?}",
                            P::name(),
                            error
                        );
                    }
                }
            }
            .in_current_span(),
        );
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub fn on_hide(&self) {
        tracing::debug!("EntityListCoordinator[{}]: on_hide clearing UI", P::name());
        if let Some(main_window) = self.weak.upgrade() {
            P::clear_models(&main_window);
        }
    }
}
