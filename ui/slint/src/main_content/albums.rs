use slint::{ComponentHandle, ModelRc};
use songs_proto::moosync::types::{Album, AlbumList, GetEntityOptions, entity_result};
use state_manager::StateManager;
use tracing::debug;

use crate::{MainWindow, Pages, error::UiError, pages::PageHandler, utils::LazySongVecModel};

pub struct AlbumsPageHandler<'a> {
    main_window: &'a MainWindow,
    state_manager: &'a StateManager,
}

impl<'a> AlbumsPageHandler<'a> {
    #[tracing::instrument(level = "debug", skip_all)]
    pub fn new(main_window: &'a MainWindow, state_manager: &'a StateManager) -> Self {
        Self {
            main_window,
            state_manager,
        }
    }

    #[tracing::instrument(level = "debug", skip_all)]
    async fn get_albums_from_db(state_manager: &StateManager) -> Result<Vec<Album>, UiError> {
        let database = state_manager.get_database().await;
        let albums_res = database.get_entity_by_options(GetEntityOptions {
            album: Some(Album::default()),
            ..Default::default()
        })?;

        match albums_res.result {
            Some(entity_result::Result::Albums(AlbumList { albums })) => Ok(albums),
            _ => Err(UiError::EntityParseFailed),
        }
    }

    #[tracing::instrument(level = "debug", skip_all)]
    async fn fetch_albums(state_manager: &StateManager) -> Result<Vec<Album>, UiError> {
        Self::get_albums_from_db(state_manager).await
    }

    #[tracing::instrument(level = "debug", skip_all)]
    fn set_albums(main_window: &MainWindow, albums: Vec<Album>, cache_dir: std::path::PathBuf) {
        debug!("Setting albums");
        let album_model = albums
            .into_iter()
            .map(|album| crate::utils::to_album_model(&album))
            .collect::<Vec<_>>();

        let theme = main_window.global::<crate::Theme>();
        main_window.set_albums(ModelRc::new(LazySongVecModel::new(
            album_model,
            theme.get_cardHeight() as usize,
            theme.get_cardWidth() as usize,
            cache_dir,
        )));
    }
}

impl<'a> PageHandler for AlbumsPageHandler<'a> {
    #[tracing::instrument(level = "debug", skip_all)]
    fn initialize(&self) {}

    #[tracing::instrument(level = "debug", skip_all)]
    fn on_show(&self) {
        let state_manager = self.state_manager.clone();
        let main_window_weak = self.main_window.as_weak();
        tokio::spawn(async move {
            if let Ok(albums) = Self::fetch_albums(&state_manager).await {
                let cache_dir = state_manager.get_cache_dir();
                let _ = slint::invoke_from_event_loop(move || {
                    if let Some(main_window) = main_window_weak.upgrade() {
                        if main_window.get_active_page() == Pages::Albums {
                            Self::set_albums(&main_window, albums, cache_dir);
                        }
                    }
                });
            }
        });
    }

    #[tracing::instrument(level = "debug", skip_all)]
    fn on_hide(&self) { self.main_window.set_albums(ModelRc::default()); }
}
