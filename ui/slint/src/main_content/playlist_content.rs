use extensions_proto::moosync::types::{ExtensionProviderScope, RequestedPlaylistSongsRequest};
use slint::{ComponentHandle, ModelRc, VecModel};
use songs_proto::moosync::types::{GetSongOptions, Playlist, Song};
use state_manager::StateManager;

use crate::{
    AppCallbacks, ExtensionProviderItem, MainWindow, PlaylistContentPageProps, PlaylistsPageProps,
    SongModel,
    error::UiError,
    pages::PageHandler,
    utils::{EntityContentCoordinator, EntitySongProvider, IntoVec, update_provider_list_enabled},
};

#[derive(Clone)]
pub struct PlaylistSongProvider;

impl EntitySongProvider for PlaylistSongProvider {
    type Entity = Playlist;

    #[tracing::instrument(level = "debug", skip_all)]
    fn scope() -> ExtensionProviderScope { ExtensionProviderScope::PlaylistSongs }

    #[tracing::instrument(level = "debug", skip_all)]
    fn get_entity(main_window: &MainWindow) -> (Playlist, String) {
        let playlist: Playlist = main_window
            .global::<PlaylistsPageProps>()
            .get_selected_playlist()
            .into();
        let extension = playlist.extension.clone().unwrap_or_default();
        (playlist, extension)
    }

    #[tracing::instrument(level = "debug", skip_all)]
    fn get_songs(main_window: &MainWindow) -> Vec<SongModel> {
        main_window
            .global::<PlaylistContentPageProps>()
            .get_songs()
            .into_vec()
    }

    #[tracing::instrument(level = "debug", skip_all)]
    fn set_songs(main_window: &MainWindow, model: ModelRc<SongModel>) {
        main_window
            .global::<PlaylistContentPageProps>()
            .set_songs(model);
    }

    #[tracing::instrument(level = "debug", skip_all)]
    fn update_extensions_enabled(main_window: &MainWindow, package_name: &str, enabled: bool) {
        let props = main_window.global::<PlaylistContentPageProps>();
        let extensions = props.get_extension_providers().into_vec();
        let updated = update_provider_list_enabled(&extensions, package_name, enabled);
        props.set_extension_providers(ModelRc::new(VecModel::from(updated)));
    }

    #[tracing::instrument(level = "debug", skip_all)]
    fn set_extensions(main_window: &MainWindow, extensions: ModelRc<ExtensionProviderItem>) {
        main_window
            .global::<PlaylistContentPageProps>()
            .set_extension_providers(extensions);
    }

    #[tracing::instrument(level = "debug", skip_all)]
    fn clear_ui(main_window: &MainWindow) {
        let props = main_window.global::<PlaylistContentPageProps>();
        props.set_songs(ModelRc::default());
        props.set_extension_providers(ModelRc::default());
    }

    #[tracing::instrument(level = "debug", skip_all)]
    async fn fetch_local_songs(
        state_manager: &StateManager,
        playlist: Playlist,
    ) -> Result<Vec<Song>, UiError> {
        let database = state_manager.get_database().await;
        let options = GetSongOptions {
            playlist: Some(playlist),
            ..Default::default()
        };
        database.get_songs_by_options(options).map_err(|e| e.into())
    }

    #[tracing::instrument(level = "debug", skip_all)]
    async fn fetch_extension_songs(
        state_manager: &StateManager,
        playlist: Playlist,
        extension: String,
        page_token: Option<String>,
    ) -> Result<(Vec<Song>, Option<String>), UiError> {
        let ext = state_manager
            .get_extension_handler()
            .await
            .get_extension(&extension)?;
        let resp = ext
            .get_playlist_songs(RequestedPlaylistSongsRequest {
                id: playlist.playlist_id.unwrap_or_default(),
                refresh: false,
                page_token,
            })
            .await?;
        Ok((resp.songs, resp.next_page_token))
    }
}

pub struct PlaylistContentPageHandler<'a> {
    main_window: &'a MainWindow,
    coordinator: EntityContentCoordinator<PlaylistSongProvider>,
}

impl<'a> PlaylistContentPageHandler<'a> {
    #[tracing::instrument(level = "debug", skip_all)]
    pub fn new(main_window: &'a MainWindow, state_manager: &'a StateManager) -> Self {
        Self {
            main_window,
            coordinator: EntityContentCoordinator::new(main_window, state_manager),
        }
    }
}

impl<'a> PageHandler for PlaylistContentPageHandler<'a> {
    #[tracing::instrument(level = "debug", skip_all)]
    fn initialize(&self) {
        let coordinator = self.coordinator.clone();
        self.main_window
            .global::<AppCallbacks>()
            .on_toggle_playlist_content_extension(move |pkg, enabled| {
                coordinator.on_toggle_extension(pkg.to_string(), enabled);
            });

        let coordinator = self.coordinator.clone();
        self.main_window
            .global::<AppCallbacks>()
            .on_load_more_playlist_content(move || {
                coordinator.on_load_more_songs();
            });
    }

    #[tracing::instrument(level = "debug", skip_all)]
    fn on_show(&self) { self.coordinator.on_show(); }

    #[tracing::instrument(level = "debug", skip_all)]
    fn on_hide(&self) { self.coordinator.on_hide(); }
}
