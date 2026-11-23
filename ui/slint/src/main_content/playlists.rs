use extensions::{Extension, ExtensionGroupExt};
use extensions_proto::moosync::types::{ExtensionProviderScope, RequestedPlaylistsRequest};
use futures::StreamExt;
use slint::{ComponentHandle, ModelRc, Weak};
use songs_proto::moosync::types::Playlist;
use state_manager::StateManager;
use tracing::Instrument;

use crate::{
    AppProps, ContextMenuCallbacks, MainWindow, Pages, PlaylistModel, PlaylistsPageProps,
    error::UiError,
    pages::PageHandler,
    utils::{
        EntityListCoordinator, EntityListProvider, IntoVec, build_playlist_context_menu_items,
        handle_extension_context_action, make_lazy_card_model,
    },
};

pub struct PlaylistListProvider;

impl PlaylistListProvider {
    /// Fetches all local playlists from the database.
    #[tracing::instrument(level = "debug", skip_all)]
    pub(crate) async fn get_local_playlists(
        state_manager: &StateManager,
    ) -> Result<Vec<Playlist>, UiError> {
        let database = state_manager.get_database();
        Ok(database.get_playlists(Playlist::default(), false).await?)
    }

    /// Requests playlists from a single extension and attaches its metadata.
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

    /// Fetches playlists across all extensions supporting the `Playlists`
    /// scope.
    #[tracing::instrument(level = "debug", skip_all)]
    async fn get_extension_playlists(
        state_manager: &StateManager,
    ) -> Result<Vec<Playlist>, UiError> {
        let mut playlists = Vec::new();
        let ext_handler = state_manager.get_extension_handler();
        let playlist_extensions = ext_handler
            .get_extensions_with_scope(ExtensionProviderScope::Playlists)
            .await;
        let mut stream = playlist_extensions.with(|ext| {
            async move { (ext.clone(), Self::fetch_extension_playlists(&ext).await) }
                .in_current_span()
        });
        while let Some((ext, res)) = stream.next().await {
            match res {
                Ok(ext_playlists) => playlists.extend(ext_playlists),
                Err(e) => {
                    tracing::error!(
                        "Failed to fetch playlists from extension {}: {:?}",
                        ext.get_package_name(),
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

    /// Returns the display name of the entity provider.
    #[tracing::instrument(level = "debug", skip_all)]
    fn name() -> &'static str { "Playlists" }

    /// Updates the playlists model on the `PlaylistsPageProps` global.
    #[tracing::instrument(level = "debug", skip_all)]
    fn set_models(main_window: &MainWindow, model: ModelRc<PlaylistModel>) {
        main_window
            .global::<PlaylistsPageProps>()
            .set_playlists(model);
    }

    /// Fetches both local and extension-provided playlists.
    #[tracing::instrument(level = "debug", skip_all)]
    async fn fetch_entities(state_manager: &StateManager) -> Result<Vec<Playlist>, UiError> {
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

pub struct PlaylistsPageHandler {
    main_window: MainWindow,
    state_manager: StateManager,
    coordinator: EntityListCoordinator<PlaylistListProvider>,
}

impl PlaylistsPageHandler {
    /// Creates a new `PlaylistsPageHandler` bound to the main window and state
    /// manager.
    #[tracing::instrument(level = "debug", skip_all)]
    pub fn new(main_window: &MainWindow, state_manager: &StateManager) -> Self {
        Self {
            main_window: main_window.clone_strong(),
            state_manager: state_manager.clone(),
            coordinator: EntityListCoordinator::new(main_window, state_manager),
        }
    }

    /// Handles a selected playlist context menu action.
    #[tracing::instrument(level = "debug", skip_all)]
    pub(crate) async fn handle_playlist_action(
        weak: Weak<MainWindow>,
        state_manager: StateManager,
        playlist_ids: Vec<String>,
        action: String,
    ) {
        if handle_extension_context_action(&state_manager, &action).await
            || action != "delete_playlist"
        {
            return;
        }

        let db = state_manager.get_database();
        for pid in &playlist_ids {
            if let Err(e) = db.remove_playlist(pid).await {
                tracing::error!("Failed to delete playlist '{}': {:?}", pid, e);
            }
        }

        Self::refresh_playlists(weak, state_manager).await;
    }

    /// Re-fetches all playlists and updates the playlists page model.
    #[tracing::instrument(level = "debug", skip_all)]
    pub(crate) async fn refresh_playlists(weak: Weak<MainWindow>, state_manager: StateManager) {
        match PlaylistListProvider::fetch_entities(&state_manager).await {
            Ok(playlists) => {
                let _ = weak.upgrade_in_event_loop(move |main_window| {
                    if main_window.global::<AppProps>().get_active_page() != Pages::Playlists {
                        return;
                    }
                    let playlists: Vec<PlaylistModel> =
                        playlists.into_iter().map(PlaylistModel::from).collect();
                    let model = make_lazy_card_model(&main_window, &state_manager, playlists);
                    main_window
                        .global::<PlaylistsPageProps>()
                        .set_playlists(model);
                });
            }
            Err(e) => {
                tracing::error!("Failed to refresh playlists: {:?}", e);
            }
        }
    }

    /// Wires Slint callbacks for building and dispatching playlist context menu
    /// actions.
    #[tracing::instrument(level = "debug", skip_all)]
    fn register_context_menu_callbacks(&self) {
        let main_window_weak = self.main_window.as_weak();
        let state_manager = self.state_manager.clone();

        self.main_window
            .global::<ContextMenuCallbacks>()
            .on_get_playlist_menu_items(move |playlist_models| {
                let Some(main_window) = main_window_weak.upgrade() else {
                    return ModelRc::default();
                };

                build_playlist_context_menu_items(
                    &main_window,
                    state_manager.clone(),
                    &playlist_models,
                )
            });

        let state_manager = self.state_manager.clone();
        let main_window_weak = self.main_window.as_weak();
        self.main_window
            .global::<ContextMenuCallbacks>()
            .on_playlist_action(move |playlist_models, action_id| {
                let playlist_ids: Vec<String> = playlist_models
                    .into_vec()
                    .into_iter()
                    .map(|p| p.id.to_string())
                    .collect();
                let action = action_id.to_string();
                let weak = main_window_weak.clone();
                let state_manager = state_manager.clone();

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

impl PageHandler for PlaylistsPageHandler {
    /// Initializes playlist context menu callbacks.
    #[tracing::instrument(level = "debug", skip_all)]
    fn initialize(&self) { self.register_context_menu_callbacks(); }

    /// Loads playlists when the page becomes visible.
    #[tracing::instrument(level = "debug", skip_all)]
    fn on_show(&self) { self.coordinator.on_show(); }

    /// Cleans up subscriptions when the page is hidden.
    #[tracing::instrument(level = "debug", skip_all)]
    fn on_hide(&self) { self.coordinator.on_hide(); }
}
