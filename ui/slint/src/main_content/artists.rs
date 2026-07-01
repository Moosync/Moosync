use slint::{ComponentHandle, ModelRc};
use songs_proto::moosync::types::{Artist, ArtistList, GetEntityOptions, entity_result};
use state_manager::StateManager;
use tracing::debug;

use crate::{MainWindow, Pages, error::UiError, pages::PageHandler, utils::LazySongVecModel};

pub struct ArtistsPageHandler<'a> {
    main_window: &'a MainWindow,
    state_manager: &'a StateManager,
}

impl<'a> ArtistsPageHandler<'a> {
    #[tracing::instrument(level = "debug", skip_all)]
    pub fn new(main_window: &'a MainWindow, state_manager: &'a StateManager) -> Self {
        Self {
            main_window,
            state_manager,
        }
    }

    #[tracing::instrument(level = "debug", skip_all)]
    async fn get_artists_from_db(state_manager: &StateManager) -> Result<Vec<Artist>, UiError> {
        let database = state_manager.get_database().await;
        let artists_res = database.get_entity_by_options(GetEntityOptions {
            artist: Some(Artist::default()),
            ..Default::default()
        })?;

        match artists_res.result {
            Some(entity_result::Result::Artists(ArtistList { artists })) => Ok(artists),
            _ => Err(UiError::EntityParseFailed),
        }
    }

    #[tracing::instrument(level = "debug", skip_all)]
    async fn fetch_artists(state_manager: &StateManager) -> Result<Vec<Artist>, UiError> {
        Self::get_artists_from_db(state_manager).await
    }

    #[tracing::instrument(level = "debug", skip_all)]
    fn set_artists(main_window: &MainWindow, artists: Vec<Artist>, cache_dir: std::path::PathBuf) {
        debug!("Setting artists");
        let artist_model = artists
            .into_iter()
            .map(|artist| crate::utils::to_artist_model(&artist))
            .collect::<Vec<_>>();

        let theme = main_window.global::<crate::Theme>();
        main_window.set_artists(ModelRc::new(LazySongVecModel::new(
            artist_model,
            theme.get_cardHeight() as usize,
            theme.get_cardWidth() as usize,
            cache_dir,
        )));
    }
}

impl<'a> PageHandler for ArtistsPageHandler<'a> {
    #[tracing::instrument(level = "debug", skip_all)]
    fn initialize(&self) {}

    #[tracing::instrument(level = "debug", skip_all)]
    fn on_show(&self) {
        let state_manager = self.state_manager.clone();
        let main_window_weak = self.main_window.as_weak();
        tokio::spawn(async move {
            if let Ok(artists) = Self::fetch_artists(&state_manager).await {
                let cache_dir = state_manager.get_cache_dir();
                let _ = slint::invoke_from_event_loop(move || {
                    if let Some(main_window) = main_window_weak.upgrade() {
                        if main_window.get_active_page() == Pages::Artists {
                            Self::set_artists(&main_window, artists, cache_dir);
                        }
                    }
                });
            }
        });
    }

    #[tracing::instrument(level = "debug", skip_all)]
    fn on_hide(&self) { self.main_window.set_artists(ModelRc::default()); }
}
