use slint::{ComponentHandle, ModelRc};
use songs_proto::moosync::types::{GetEntityOptions, Playlist, PlaylistList, entity_result};
use state_manager::StateManager;
use tracing::debug;

use crate::{
    MainWindow, PlaylistsPageProps, error::UiError, pages::PageHandler, utils::LazySongVecModel,
};

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
    async fn fetch_local_playlists(state_manager: &StateManager) -> Result<Vec<Playlist>, UiError> {
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
    async fn fetch_extension_playlists(
        ext: &extensions::Extension,
    ) -> Result<Vec<Playlist>, UiError> {
        let detail = ext.get_extension_detail();
        let resp = ext
            .get_playlists(
                extensions_proto::moosync::types::RequestedPlaylistsRequest { refresh: false },
            )
            .await?;
        let mut playlists = resp.playlists;
        for p in &mut playlists {
            p.extension = Some(detail.package_name.clone());
            p.icon = detail.extension_icon.clone();
        }
        Ok(playlists)
    }

    #[tracing::instrument(level = "debug", skip_all)]
    async fn fetch_playlists(state_manager: &StateManager) -> Result<Vec<Playlist>, UiError> {
        let local = Self::fetch_local_playlists(state_manager)
            .await
            .unwrap_or_default();
        let ext_handler = state_manager.get_extension_handler().await;
        let playlist_extensions = ext_handler
            .get_extensions_with_scope(
                extensions_proto::moosync::types::ExtensionProviderScope::Playlists,
            )
            .await;

        let all_playlists: Vec<Playlist> = local
            .into_iter()
            .chain(
                futures::future::join_all(
                    playlist_extensions
                        .iter()
                        .map(|ext| Self::fetch_extension_playlists(ext)),
                )
                .await
                .into_iter()
                .filter_map(|r| r.ok())
                .flatten(),
            )
            .collect();

        Ok(all_playlists)
    }

    #[tracing::instrument(level = "debug", skip_all)]
    fn set_playlists(
        main_window: &MainWindow,
        state_manager: &StateManager,
        playlists: Vec<Playlist>,
    ) {
        debug!("Setting playlists");
        let playlist_model = playlists
            .into_iter()
            .map(|playlist| crate::utils::to_playlist_model(&playlist, None))
            .collect::<Vec<_>>();

        let theme = main_window.global::<crate::Theme>();
        let cache_dir = state_manager.get_cache_dir();
        main_window
            .global::<PlaylistsPageProps>()
            .set_playlists(ModelRc::new(LazySongVecModel::new(
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
        tokio::spawn({
            let state_manager = self.state_manager.clone();
            let main_window_weak = self.main_window.as_weak();
            async move {
                if let Ok(playlists) = Self::fetch_playlists(&state_manager).await {
                    let _ = main_window_weak.upgrade_in_event_loop(move |main_window| {
                        Self::set_playlists(&main_window, &state_manager, playlists);
                    });
                }
            }
        });
    }

    #[tracing::instrument(level = "debug", skip_all)]
    fn on_hide(&self) {
        self.main_window
            .global::<PlaylistsPageProps>()
            .set_playlists(ModelRc::default());
    }
}
