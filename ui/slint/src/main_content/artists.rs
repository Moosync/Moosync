use slint::{ComponentHandle, ModelRc};
use songs_proto::moosync::types::{Artist, ArtistList, GetEntityOptions, entity_result};
use state_manager::StateManager;
use tracing::debug;

use crate::{
    ArtistModel, ArtistsPageProps, MainWindow, Theme, error::UiError, pages::PageHandler,
    utils::LazySongVecModel,
};

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
    async fn fetch_artists(state_manager: &StateManager) -> Result<Vec<Artist>, UiError> {
        let database = state_manager.get_database().await;
        let artists = database.get_entity_by_options(GetEntityOptions {
            artist: Some(Artist::default()),
            ..Default::default()
        })?;
        if let Some(entity_result::Result::Artists(ArtistList { artists })) = artists.result {
            return Ok(artists);
        }
        Ok(vec![])
    }

    #[tracing::instrument(level = "debug", skip_all)]
    fn set_artists(main_window: &MainWindow, state_manager: &StateManager, artists: Vec<Artist>) {
        debug!("Setting artists");
        let artist_model = artists
            .into_iter()
            .map(ArtistModel::from)
            .collect::<Vec<_>>();

        let theme = main_window.global::<Theme>();
        let cache_dir = state_manager.get_cache_dir();
        main_window
            .global::<ArtistsPageProps>()
            .set_artists(ModelRc::new(LazySongVecModel::new(
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
        tokio::spawn({
            let state_manager = self.state_manager.clone();
            let main_window_weak = self.main_window.as_weak();
            async move {
                if let Ok(artists) = Self::fetch_artists(&state_manager).await {
                    let _ = main_window_weak.upgrade_in_event_loop(move |main_window| {
                        Self::set_artists(&main_window, &state_manager, artists);
                    });
                }
            }
        });
    }

    #[tracing::instrument(level = "debug", skip_all)]
    fn on_hide(&self) {
        self.main_window
            .global::<ArtistsPageProps>()
            .set_artists(ModelRc::default());
    }
}
