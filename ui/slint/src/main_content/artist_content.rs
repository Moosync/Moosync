use extensions_proto::moosync::types::{ExtensionProviderScope, RequestedArtistSongsRequest};
use slint::{ComponentHandle, ModelRc, VecModel, Weak};
use songs_proto::moosync::types::{Artist, GetSongOptions, Song};
use state_manager::StateManager;
use tracing::Instrument;

use crate::{
    AppCallbacks, ArtistContentPageProps, ArtistsPageProps, MainWindow, SongModel,
    error::UiError,
    pages::PageHandler,
    utils::{
        IntoVec, fetch_scope_providers, make_lazy_song_model, map_songs_to_models,
        update_provider_list_enabled,
    },
};

pub struct ArtistContentPageHandler<'a> {
    main_window: &'a MainWindow,
    state_manager: &'a StateManager,
}

impl<'a> ArtistContentPageHandler<'a> {
    #[tracing::instrument(level = "debug", skip_all)]
    pub fn new(main_window: &'a MainWindow, state_manager: &'a StateManager) -> Self {
        Self {
            main_window,
            state_manager,
        }
    }

    #[tracing::instrument(level = "debug", skip_all)]
    async fn fetch_local_songs(
        state_manager: &StateManager,
        artist: Artist,
    ) -> Result<Vec<Song>, UiError> {
        let database = state_manager.get_database().await;
        let options = GetSongOptions {
            artist: Some(artist),
            ..Default::default()
        };
        database.get_songs_by_options(options).map_err(|e| e.into())
    }

    #[tracing::instrument(level = "debug", skip_all)]
    async fn fetch_extension_songs(
        state_manager: &StateManager,
        artist: Artist,
        extension: String,
    ) -> Result<Vec<Song>, UiError> {
        let handler = state_manager.get_extension_handler().await;
        let ext = handler.get_extension(&extension)?;
        let resp = ext
            .get_artist_songs(RequestedArtistSongsRequest {
                artist: Some(artist),
                page_token: None,
            })
            .await?;
        Ok(resp.songs)
    }

    #[tracing::instrument(level = "debug", skip_all)]
    async fn fetch_songs(
        state_manager: &StateManager,
        artist: Artist,
        extension: String,
    ) -> Result<Vec<Song>, UiError> {
        if !extension.is_empty() {
            return Self::fetch_extension_songs(state_manager, artist, extension).await;
        }
        Self::fetch_local_songs(state_manager, artist).await
    }

    #[tracing::instrument(level = "debug", skip_all)]
    fn update_providers_enabled(main_window: &MainWindow, package_name: &str, enabled: bool) {
        let props = main_window.global::<ArtistContentPageProps>();
        let providers = props.get_extension_providers().into_vec();
        let updated = update_provider_list_enabled(&providers, package_name, enabled);
        props.set_extension_providers(ModelRc::new(VecModel::from(updated)));
    }

    #[tracing::instrument(level = "debug", skip_all)]
    fn toggle_extension(
        weak: Weak<MainWindow>,
        state_manager: StateManager,
        package_name: String,
        enabled: bool,
    ) {
        if !enabled {
            let _ = weak.upgrade_in_event_loop(move |main_window| {
                Self::update_providers_enabled(&main_window, &package_name, false);
                let current = main_window
                    .global::<ArtistContentPageProps>()
                    .get_songs()
                    .into_vec();
                let filtered: Vec<SongModel> = current
                    .into_iter()
                    .filter(|s| s.extension != package_name)
                    .collect();
                let model = make_lazy_song_model(&main_window, &state_manager, filtered);
                main_window
                    .global::<ArtistContentPageProps>()
                    .set_songs(model);
            });
            return;
        }

        let Some(main_window) = weak.upgrade() else {
            return;
        };
        Self::update_providers_enabled(&main_window, &package_name, true);
        let artist: Artist = main_window
            .global::<ArtistsPageProps>()
            .get_selected_artist()
            .into();
        drop(main_window);

        tokio::spawn({
            let state_manager = state_manager.clone();
            let weak = weak.clone();
            async move {
                let handler = state_manager.get_extension_handler().await;
                let Ok(ext) = handler.get_extension(&package_name) else {
                    tracing::error!("Extension {} not found", package_name);
                    let _ = weak.upgrade_in_event_loop(move |window| {
                        Self::update_providers_enabled(&window, &package_name, false);
                    });
                    return;
                };
                let detail = ext.get_extension_detail();

                let Ok(new_songs) =
                    Self::fetch_extension_songs(&state_manager, artist, package_name.clone()).await
                else {
                    tracing::error!(
                        "Failed to fetch artist songs from extension {}",
                        package_name
                    );
                    let _ = weak.upgrade_in_event_loop(move |window| {
                        Self::update_providers_enabled(&window, &package_name, false);
                    });
                    return;
                };

                let _ = weak.upgrade_in_event_loop(move |main_window| {
                    let mut current = main_window
                        .global::<ArtistContentPageProps>()
                        .get_songs()
                        .into_vec();
                    current.extend(map_songs_to_models(new_songs, Some(&detail)));
                    let model = make_lazy_song_model(&main_window, &state_manager, current);
                    main_window
                        .global::<ArtistContentPageProps>()
                        .set_songs(model);
                });
            }
            .in_current_span()
        });
    }
}

impl<'a> PageHandler for ArtistContentPageHandler<'a> {
    #[tracing::instrument(level = "debug", skip_all)]
    fn initialize(&self) {
        let state_manager = self.state_manager.clone();
        let main_window_weak = self.main_window.as_weak();
        self.main_window
            .global::<AppCallbacks>()
            .on_toggle_artist_content_extension(move |pkg, enabled| {
                Self::toggle_extension(
                    main_window_weak.clone(),
                    state_manager.clone(),
                    pkg.to_string(),
                    enabled,
                );
            });
    }

    #[tracing::instrument(level = "debug", skip_all)]
    fn on_show(&self) {
        let selected = self
            .main_window
            .global::<ArtistsPageProps>()
            .get_selected_artist();
        let artist: Artist = selected.into();
        let extension = artist.extension.clone().unwrap_or_default();

        tokio::spawn({
            let state_manager = self.state_manager.clone();
            let main_window_weak = self.main_window.as_weak();
            async move {
                let ext_handler = state_manager.get_extension_handler().await;
                let (providers, detail) = fetch_scope_providers(
                    &ext_handler,
                    ExtensionProviderScope::ArtistSongs,
                    &extension,
                )
                .await;

                let Ok(songs) = Self::fetch_songs(&state_manager, artist, extension).await else {
                    tracing::error!("Failed to fetch artist songs");
                    return;
                };

                let _ = main_window_weak.upgrade_in_event_loop(move |main_window| {
                    main_window
                        .global::<ArtistContentPageProps>()
                        .set_extension_providers(ModelRc::new(VecModel::from(providers)));
                    let song_models = map_songs_to_models(songs, detail.as_ref());
                    let model = make_lazy_song_model(&main_window, &state_manager, song_models);
                    main_window
                        .global::<ArtistContentPageProps>()
                        .set_songs(model);
                });
            }
            .in_current_span()
        });
    }

    #[tracing::instrument(level = "debug", skip_all)]
    fn on_hide(&self) {
        self.main_window
            .global::<ArtistContentPageProps>()
            .set_songs(ModelRc::default());
        self.main_window
            .global::<ArtistContentPageProps>()
            .set_extension_providers(ModelRc::default());
    }
}
