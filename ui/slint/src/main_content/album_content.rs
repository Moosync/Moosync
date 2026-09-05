use extensions_proto::moosync::types::{ExtensionProviderScope, RequestedAlbumSongsRequest};
use slint::{ComponentHandle, ModelRc};
use songs_proto::moosync::types::{Album, GetSongOptions, Song};
use state_manager::StateManager;

use crate::{
    AlbumContentPageProps, AlbumsPageProps, AppCallbacks, ExtensionProviderItem, MainWindow,
    SongModel,
    error::UiError,
    pages::PageHandler,
    utils::{EntityContentCoordinator, EntitySongProvider, IntoVec},
};

#[derive(Clone)]
pub struct AlbumSongProvider;

#[async_trait::async_trait]
impl EntitySongProvider for AlbumSongProvider {
    type Entity = Album;

    #[tracing::instrument(level = "debug", skip_all)]
    fn extension_scope() -> Option<ExtensionProviderScope> {
        Some(ExtensionProviderScope::AlbumSongs)
    }

    #[tracing::instrument(level = "debug", skip_all)]
    fn get_entity(main_window: &MainWindow) -> (Album, String) {
        let album: Album = main_window
            .global::<AlbumsPageProps>()
            .get_selected_album()
            .into();
        let extension = album.extension.clone().unwrap_or_default();
        (album, extension)
    }

    #[tracing::instrument(level = "debug", skip_all)]
    fn get_songs(main_window: &MainWindow) -> Vec<SongModel> {
        main_window
            .global::<AlbumContentPageProps>()
            .get_songs()
            .into_vec()
    }

    #[tracing::instrument(level = "debug", skip_all)]
    fn set_songs(main_window: &MainWindow, model: ModelRc<SongModel>) {
        main_window
            .global::<AlbumContentPageProps>()
            .set_songs(model);
    }

    #[tracing::instrument(level = "debug", skip_all)]
    fn get_extensions(main_window: &MainWindow) -> ModelRc<ExtensionProviderItem> {
        main_window
            .global::<AlbumContentPageProps>()
            .get_extension_providers()
    }

    #[tracing::instrument(level = "debug", skip_all)]
    fn set_extensions(main_window: &MainWindow, extensions: ModelRc<ExtensionProviderItem>) {
        main_window
            .global::<AlbumContentPageProps>()
            .set_extension_providers(extensions);
    }

    #[tracing::instrument(level = "debug", skip_all)]
    async fn fetch_local_songs(
        state_manager: &StateManager,
        album: Album,
    ) -> Result<Vec<Song>, UiError> {
        let database = state_manager.get_database().await;
        let options = GetSongOptions {
            album: Some(album),
            ..Default::default()
        };
        database.get_songs_by_options(options).map_err(|e| e.into())
    }

    #[tracing::instrument(level = "debug", skip_all)]
    async fn fetch_extension_songs(
        state_manager: &StateManager,
        album: Album,
        extension: String,
        page_token: Option<String>,
    ) -> Result<(Vec<Song>, Option<String>), UiError> {
        let ext = state_manager
            .get_extension_handler()
            .await
            .get_extension(&extension)?;
        let resp = ext
            .get_album_songs(RequestedAlbumSongsRequest {
                album: Some(album),
                page_token,
            })
            .await?;
        Ok((resp.songs, resp.next_page_token))
    }
}

pub struct AlbumContentPageHandler<'a> {
    main_window: &'a MainWindow,
    coordinator: EntityContentCoordinator<AlbumSongProvider>,
}

impl<'a> AlbumContentPageHandler<'a> {
    #[tracing::instrument(level = "debug", skip_all)]
    pub fn new(main_window: &'a MainWindow, state_manager: &'a StateManager) -> Self {
        Self {
            main_window,
            coordinator: EntityContentCoordinator::new(main_window, state_manager),
        }
    }
}

impl<'a> PageHandler for AlbumContentPageHandler<'a> {
    #[tracing::instrument(level = "debug", skip_all)]
    fn initialize(&self) {
        let coordinator = self.coordinator.clone();
        self.main_window
            .global::<AppCallbacks>()
            .on_toggle_album_content_extension(move |pkg, enabled| {
                coordinator.on_toggle_extension(pkg.to_string(), enabled);
            });

        let coordinator = self.coordinator.clone();
        self.main_window
            .global::<AppCallbacks>()
            .on_load_more_album_content(move || {
                coordinator.on_load_more_songs();
            });
    }

    #[tracing::instrument(level = "debug", skip_all)]
    fn on_show(&self) { self.coordinator.on_show(); }

    #[tracing::instrument(level = "debug", skip_all)]
    fn on_hide(&self) { self.coordinator.on_hide(); }
}
