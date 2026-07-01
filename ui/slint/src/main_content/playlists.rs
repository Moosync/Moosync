use slint::{ComponentHandle, ModelRc};
use songs_proto::moosync::types::{GetEntityOptions, Playlist, PlaylistList, entity_result};
use state_manager::StateManager;
use tracing::debug;

use crate::{MainWindow, Pages, error::UiError, pages::PageHandler, utils::LazySongVecModel};

pub struct PlaylistsPageHandler<'a> {
    main_window: &'a MainWindow,
    state_manager: &'a StateManager,
}

impl<'a> PlaylistsPageHandler<'a> {
    #[tracing::instrument(level = "debug", skip_all)]
    pub fn new(main_window: &'a MainWindow, state_manager: &'a StateManager) -> Self {
        Self {
            main_window,
            state_manager,
        }
    }

    #[tracing::instrument(level = "debug", skip_all)]
    async fn get_playlists_from_db(state_manager: &StateManager) -> Result<Vec<Playlist>, UiError> {
        let database = state_manager.get_database().await;
        let playlists_res = database.get_entity_by_options(GetEntityOptions {
            playlist: Some(Playlist::default()),
            ..Default::default()
        })?;

        match playlists_res.result {
            Some(entity_result::Result::Playlists(PlaylistList { playlists })) => Ok(playlists),
            _ => Err(UiError::EntityParseFailed),
        }
    }

    #[tracing::instrument(level = "debug", skip_all)]
    async fn fetch_playlists(state_manager: &StateManager) -> Result<Vec<Playlist>, UiError> {
        Self::get_playlists_from_db(state_manager).await
    }

    #[tracing::instrument(level = "debug", skip_all)]
    fn set_playlists(
        main_window: &MainWindow,
        playlists: Vec<Playlist>,
        cache_dir: std::path::PathBuf,
    ) {
        debug!("Setting playlists");
        let playlist_model = playlists
            .into_iter()
            .map(|playlist| crate::utils::to_playlist_model(&playlist))
            .collect::<Vec<_>>();

        let theme = main_window.global::<crate::Theme>();
        main_window.set_playlists(ModelRc::new(LazySongVecModel::new(
            playlist_model,
            theme.get_cardHeight() as usize,
            theme.get_cardWidth() as usize,
            cache_dir,
        )));
    }
}

impl<'a> PageHandler for PlaylistsPageHandler<'a> {
    #[tracing::instrument(level = "debug", skip_all)]
    fn initialize(&self) {}

    #[tracing::instrument(level = "debug", skip_all)]
    fn on_show(&self) {
        let state_manager = self.state_manager.clone();
        let main_window_weak = self.main_window.as_weak();
        tokio::spawn(async move {
            if let Ok(playlists) = Self::fetch_playlists(&state_manager).await {
                let cache_dir = state_manager.get_cache_dir();
                let _ = slint::invoke_from_event_loop(move || {
                    if let Some(main_window) = main_window_weak.upgrade() {
                        if main_window.get_active_page() == Pages::Playlists {
                            Self::set_playlists(&main_window, playlists, cache_dir);
                        }
                    }
                });
            }
        });
    }

    #[tracing::instrument(level = "debug", skip_all)]
    fn on_hide(&self) { self.main_window.set_playlists(ModelRc::default()); }
}
