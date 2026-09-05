use extensions::Extension;
use extensions_proto::moosync::types::{ExtensionProviderScope, RequestedPlaylistsRequest};
use slint::{ComponentHandle, ModelRc, VecModel, Weak};
use songs_proto::moosync::types::{GetEntityOptions, Playlist, PlaylistList, entity_result};
use state_manager::StateManager;
use tracing::Instrument;

use crate::{
    ContextMenuCallbacks, ContextMenuItem, ContextMenuItems, MainWindow, PlaylistModel,
    PlaylistsPageProps,
    error::UiError,
    pages::PageHandler,
    utils::{EntityListCoordinator, EntityListProvider, IntoVec, make_lazy_card_model},
};

pub struct PlaylistListProvider;

impl PlaylistListProvider {
    #[tracing::instrument(level = "debug", skip_all)]
    async fn get_local_playlists(state_manager: &StateManager) -> Result<Vec<Playlist>, UiError> {
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
    async fn fetch_extension_playlists(ext: &Extension) -> Result<Vec<Playlist>, UiError> {
        let detail = ext.get_extension_detail();
        let resp = ext
            .get_playlists(RequestedPlaylistsRequest { refresh: false })
            .await?;
        let mut playlists = resp.playlists;
        for p in &mut playlists {
            p.extension = Some(detail.package_name.clone());
            p.icon = detail.extension_icon.clone();
        }
        Ok(playlists)
    }

    #[tracing::instrument(level = "debug", skip_all)]
    async fn get_extension_playlists(
        state_manager: &StateManager,
    ) -> Result<Vec<Playlist>, UiError> {
        let mut playlists = Vec::new();
        let ext_handler = state_manager.get_extension_handler().await;
        let playlist_extensions = ext_handler
            .get_extensions_with_scope(ExtensionProviderScope::Playlists)
            .await;
        for ext in playlist_extensions {
            match Self::fetch_extension_playlists(&ext).await {
                Ok(ext_playlists) => {
                    playlists.extend(ext_playlists);
                }
                Err(e) => {
                    tracing::error!(
                        "Failed to fetch playlists from extension {}: {:?}",
                        ext.get_extension_detail().package_name,
                        e
                    );
                }
            }
        }
        Ok(playlists)
    }
}

#[async_trait::async_trait]
impl EntityListProvider for PlaylistListProvider {
    type Entity = Playlist;
    type EntityModel = PlaylistModel;

    #[tracing::instrument(level = "debug", skip_all)]
    fn name() -> &'static str { "Playlists" }

    #[tracing::instrument(level = "debug", skip_all)]
    fn set_models(main_window: &MainWindow, model: ModelRc<PlaylistModel>) {
        main_window
            .global::<PlaylistsPageProps>()
            .set_playlists(model);
    }

    #[tracing::instrument(level = "debug", skip_all)]
    async fn fetch_entities(state_manager: &StateManager) -> Result<Vec<Playlist>, UiError> {
        tracing::debug!("Fetching all playlists (local and extension)");
        let mut playlists = Vec::new();
        match Self::get_local_playlists(state_manager).await {
            Ok(local) => playlists.extend(local),
            Err(e) => tracing::error!("Failed to fetch local playlists: {:?}", e),
        }
        match Self::get_extension_playlists(state_manager).await {
            Ok(extension) => playlists.extend(extension),
            Err(e) => tracing::error!("Failed to fetch extension playlists: {:?}", e),
        }
        Ok(playlists)
    }
}

pub struct PlaylistsPageHandler<'a> {
    main_window: &'a MainWindow,
    state_manager: &'a StateManager,
    coordinator: EntityListCoordinator<PlaylistListProvider>,
}

impl<'a> PlaylistsPageHandler<'a> {
    #[tracing::instrument(level = "debug", skip_all)]
    pub fn new(main_window: &'a MainWindow, state_manager: &'a StateManager) -> Self {
        Self {
            main_window,
            state_manager,
            coordinator: EntityListCoordinator::new(main_window, state_manager),
        }
    }

    #[tracing::instrument(level = "debug", skip_all)]
    async fn handle_playlist_action(
        weak: Weak<MainWindow>,
        state_manager: StateManager,
        playlist_ids: Vec<String>,
        action: String,
    ) {
        tracing::debug!(
            "Handling playlist action '{}' for playlists: {:?}",
            action,
            playlist_ids
        );
        if action == "delete_playlist" {
            let db = state_manager.get_database().await;
            for pid in &playlist_ids {
                match db.remove_playlist(pid) {
                    Ok(()) => {
                        tracing::debug!("Successfully deleted playlist '{}'", pid);
                    }
                    Err(e) => {
                        tracing::error!("Failed to delete playlist '{}': {:?}", pid, e);
                    }
                }
            }
            match PlaylistListProvider::fetch_entities(&state_manager).await {
                Ok(playlists) => {
                    let _ = weak.upgrade_in_event_loop(move |main_window| {
                        let playlists: Vec<PlaylistModel> =
                            playlists.into_iter().map(PlaylistModel::from).collect();
                        let model = make_lazy_card_model(&main_window, &state_manager, playlists);
                        main_window
                            .global::<PlaylistsPageProps>()
                            .set_playlists(model);
                    });
                }
                Err(e) => {
                    tracing::error!("Failed to refresh playlists after deletion: {:?}", e);
                }
            }
        }
    }

    #[tracing::instrument(level = "debug", skip_all)]
    fn register_context_menu_callbacks(&self) {
        let main_window_weak = self.main_window.as_weak();

        self.main_window
            .global::<ContextMenuCallbacks>()
            .on_get_playlist_menu_items(move |_playlist_models| {
                let Some(main_window) = main_window_weak.upgrade() else {
                    return ModelRc::default();
                };

                let all_items: Vec<ContextMenuItem> = main_window
                    .global::<ContextMenuItems>()
                    .invoke_get_playlists_items()
                    .into_vec();

                ModelRc::new(VecModel::from(all_items))
            });

        let state_manager = self.state_manager.clone();
        let main_window_weak = self.main_window.as_weak();
        self.main_window
            .global::<ContextMenuCallbacks>()
            .on_playlist_action(move |playlist_models, action_id| {
                let state_manager = state_manager.clone();
                let playlist_ids: Vec<String> = playlist_models
                    .into_vec()
                    .into_iter()
                    .map(|p| p.id.to_string())
                    .collect();
                let action = action_id.to_string();
                let weak = main_window_weak.clone();

                tokio::spawn(
                    async move {
                        Self::handle_playlist_action(weak, state_manager, playlist_ids, action)
                            .await;
                    }
                    .instrument(tracing::debug_span!("slint_cb_on_playlist_action")),
                );
            });
    }
}

impl<'a> PageHandler for PlaylistsPageHandler<'a> {
    #[tracing::instrument(level = "debug", skip_all)]
    fn initialize(&self) { self.register_context_menu_callbacks(); }

    #[tracing::instrument(level = "debug", skip_all)]
    fn on_show(&self) { self.coordinator.on_show(); }

    #[tracing::instrument(level = "debug", skip_all)]
    fn on_hide(&self) { self.coordinator.on_hide(); }
}
