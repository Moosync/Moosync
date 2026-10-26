use std::{
    collections::{HashMap, HashSet},
    marker::PhantomData,
    sync::{Arc, Mutex},
};

use extensions::{Extension, ExtensionGroupExt, ExtensionHandler};
use extensions_proto::moosync::types::{ExtensionDetail, ExtensionProviderScope};
use futures::StreamExt;
use slint::{ComponentHandle, ModelRc, VecModel, Weak};
use songs_proto::moosync::types::Song;
use state_manager::StateManager;
use tracing::Instrument;

use crate::{
    ExtensionProviderItem, MainWindow, SongModel,
    error::UiError,
    utils::{IntoVec, make_lazy_song_model},
};

#[derive(Debug, Default, Clone)]
pub struct ExtensionPaginationState {
    pub next_page_token: Option<String>,
    pub has_more: bool,
    pub is_loading: bool,
}

#[derive(Debug, Default)]
pub struct ExtensionPaginationManager {
    extensions: HashMap<String, ExtensionPaginationState>,
}

impl ExtensionPaginationManager {
    #[tracing::instrument(level = "debug", skip_all)]
    pub fn reset(&mut self) { self.extensions.clear(); }

    #[tracing::instrument(level = "debug", skip_all)]
    pub fn register_extension(&mut self, package_name: &str) {
        self.extensions.insert(
            package_name.to_string(),
            ExtensionPaginationState {
                next_page_token: None,
                has_more: true,
                is_loading: true,
            },
        );
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub fn update_after_fetch(&mut self, package_name: &str, next_page_token: Option<String>) {
        let has_more = next_page_token.as_ref().is_some_and(|t| !t.is_empty());
        self.extensions.insert(
            package_name.to_string(),
            ExtensionPaginationState {
                next_page_token,
                has_more,
                is_loading: false,
            },
        );
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub fn load_more(&mut self) -> Vec<(String, Option<String>)> {
        let mut to_load = Vec::new();
        for (pkg, state) in self.extensions.iter_mut() {
            if state.has_more && !state.is_loading {
                state.is_loading = true;
                to_load.push((pkg.clone(), state.next_page_token.clone()));
            }
        }
        to_load
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub fn mark_error(&mut self, package_name: &str) {
        if let Some(state) = self.extensions.get_mut(package_name) {
            state.is_loading = false;
        }
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub fn contains_extension(&self, package_name: &str) -> bool {
        self.extensions.contains_key(package_name)
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub fn remove_extension(&mut self, package_name: &str) { self.extensions.remove(package_name); }
}

#[async_trait::async_trait]
pub trait EntitySongProvider: Clone + Send + Sync + 'static {
    type Entity: Clone + Send + Sync + 'static;

    fn extension_scope() -> Option<ExtensionProviderScope> { None }
    fn get_entity(main_window: &MainWindow) -> (Self::Entity, String);
    fn get_songs(_main_window: &MainWindow) -> Vec<SongModel> { Vec::new() }
    fn set_songs(main_window: &MainWindow, model: ModelRc<SongModel>);
    fn get_extensions(_main_window: &MainWindow) -> ModelRc<ExtensionProviderItem> {
        ModelRc::default()
    }
    fn set_extensions(_main_window: &MainWindow, _extensions: ModelRc<ExtensionProviderItem>) {}
    fn update_extensions_enabled(main_window: &MainWindow, package_name: &str, enabled: bool) {
        let extensions = Self::get_extensions(main_window).into_vec();
        let updated = update_provider_list_enabled(&extensions, package_name, enabled);
        Self::set_extensions(main_window, ModelRc::new(VecModel::from(updated)));
    }
    fn clear_ui(main_window: &MainWindow) {
        Self::set_songs(main_window, ModelRc::default());
        Self::set_extensions(main_window, ModelRc::default());
    }

    async fn fetch_local_songs(
        state_manager: &StateManager,
        entity: Self::Entity,
    ) -> Result<Vec<Song>, UiError>;

    async fn fetch_extension_songs(
        _ext: &Extension,
        _entity: Self::Entity,
        _page_token: Option<String>,
    ) -> Result<(Vec<Song>, Option<String>), UiError> {
        Ok((Vec::new(), None))
    }
}

#[derive(Clone)]
pub struct EntityContentCoordinator<P: EntitySongProvider> {
    weak: Weak<MainWindow>,
    state_manager: &'static StateManager,
    pagination: Arc<Mutex<ExtensionPaginationManager>>,
    _phantom: PhantomData<P>,
}

impl<P: EntitySongProvider> EntityContentCoordinator<P> {
    #[tracing::instrument(level = "debug", skip_all)]
    pub fn new(main_window: &MainWindow, state_manager: &'static StateManager) -> Self {
        Self {
            weak: main_window.as_weak(),
            state_manager,
            pagination: Arc::new(Mutex::new(ExtensionPaginationManager::default())),
            _phantom: PhantomData,
        }
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub(crate) fn disable_extension_in_ui(
        main_window: &MainWindow,
        state_manager: &'static StateManager,
        package_name: &str,
    ) {
        P::update_extensions_enabled(main_window, package_name, false);
        let current = P::get_songs(main_window);
        let filtered: Vec<SongModel> = current
            .into_iter()
            .filter(|s| s.extension != package_name)
            .collect();
        let model = make_lazy_song_model(main_window, state_manager, filtered);
        P::set_songs(main_window, model);
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub fn on_toggle_extension(&self, package_name: String, enabled: bool) {
        if !enabled {
            self.pagination
                .lock()
                .unwrap()
                .remove_extension(&package_name);
            let state_manager = self.state_manager;
            let _ = self.weak.upgrade_in_event_loop(move |main_window| {
                Self::disable_extension_in_ui(&main_window, state_manager, &package_name);
            });
            return;
        }

        let Some(main_window) = self.weak.upgrade() else {
            return;
        };
        P::update_extensions_enabled(&main_window, &package_name, true);
        let (entity, _) = P::get_entity(&main_window);
        let enabled_packages: HashSet<String> = P::get_extensions(&main_window)
            .into_vec()
            .into_iter()
            .filter(|p| p.enabled)
            .map(|p| p.id.to_string())
            .collect();

        let coordinator = self.clone();
        tokio::spawn(
            async move {
                coordinator
                    .fetch_enabled_extension_songs(entity, enabled_packages)
                    .await;
            }
            .in_current_span(),
        );
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub async fn fetch_enabled_extension_songs(
        &self,
        entity: P::Entity,
        enabled_packages: HashSet<String>,
    ) {
        let Some(scope) = P::extension_scope() else {
            return;
        };
        if enabled_packages.is_empty() {
            return;
        }

        let active_exts = self
            .state_manager
            .get_extension_handler()
            .get_extensions_with_scope(scope)
            .await;

        let to_fetch: Vec<Arc<Extension>> = {
            let pagination = self.pagination.lock().unwrap();
            active_exts
                .into_iter()
                .filter(|ext| {
                    let pkg = ext.get_package_name();
                    enabled_packages.contains(pkg) && !pagination.contains_extension(pkg)
                })
                .collect()
        };

        if to_fetch.is_empty() {
            return;
        }

        {
            let mut pagination = self.pagination.lock().unwrap();
            for ext in &to_fetch {
                pagination.register_extension(ext.get_package_name());
            }
        }

        let mut stream = to_fetch.with(|ext| {
            let entity = entity.clone();
            async move {
                let detail = ext.get_extension_detail();
                let result = P::fetch_extension_songs(&ext, entity, None).await;
                (detail, result)
            }
            .in_current_span()
        });

        let state_manager = self.state_manager;
        let weak = self.weak.clone();
        let pagination = self.pagination.clone();

        while let Some((detail, result)) = stream.next().await {
            match result {
                Ok((songs, next_token)) => {
                    let pagination = pagination.clone();
                    let package_name = detail.package_name.clone();
                    let _ = weak.upgrade_in_event_loop(move |window| {
                        let mut current = P::get_songs(&window);
                        current.extend(map_songs_to_models(songs, Some(&detail)));
                        let model = make_lazy_song_model(&window, state_manager, current);
                        P::set_songs(&window, model);

                        pagination
                            .lock()
                            .unwrap()
                            .update_after_fetch(&package_name, next_token);
                    });
                }
                Err(error) => {
                    tracing::error!(
                        "Failed to fetch songs from extension {}: {:?}",
                        detail.package_name,
                        error
                    );
                    pagination
                        .lock()
                        .unwrap()
                        .remove_extension(&detail.package_name);
                    let package_name = detail.package_name;
                    let _ = weak.upgrade_in_event_loop(move |window| {
                        P::update_extensions_enabled(&window, &package_name, false);
                    });
                }
            }
        }
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub fn on_load_more_songs(&self) {
        let Some(scope) = P::extension_scope() else {
            return;
        };
        let extensions_to_fetch: HashMap<String, Option<String>> = self
            .pagination
            .lock()
            .unwrap()
            .load_more()
            .into_iter()
            .collect();
        if extensions_to_fetch.is_empty() {
            return;
        }

        let Some(main_window) = self.weak.upgrade() else {
            return;
        };
        let (entity, _) = P::get_entity(&main_window);

        let state_manager = self.state_manager;
        let weak = self.weak.clone();
        let pagination = self.pagination.clone();

        tokio::spawn(
            async move {
                let active_exts = state_manager
                    .get_extension_handler()
                    .get_extensions_with_scope(scope)
                    .await;
                let to_fetch: Vec<Arc<Extension>> = active_exts
                    .into_iter()
                    .filter(|ext| extensions_to_fetch.contains_key(ext.get_package_name()))
                    .collect();

                if to_fetch.is_empty() {
                    return;
                }

                let mut stream = to_fetch.with(|ext| {
                    let entity = entity.clone();
                    let page_token = extensions_to_fetch
                        .get(ext.get_package_name())
                        .cloned()
                        .flatten();
                    async move {
                        let detail = ext.get_extension_detail();
                        let result = P::fetch_extension_songs(&ext, entity, page_token).await;
                        (detail, result)
                    }
                    .in_current_span()
                });

                while let Some((detail, result)) = stream.next().await {
                    match result {
                        Ok((songs, next_token)) => {
                            let pagination = pagination.clone();
                            let package_name = detail.package_name.clone();
                            let _ = weak.upgrade_in_event_loop(move |window| {
                                let mut current = P::get_songs(&window);
                                current.extend(map_songs_to_models(songs, Some(&detail)));
                                let model = make_lazy_song_model(&window, state_manager, current);
                                P::set_songs(&window, model);

                                pagination
                                    .lock()
                                    .unwrap()
                                    .update_after_fetch(&package_name, next_token);
                            });
                        }
                        Err(error) => {
                            tracing::error!(
                                "Failed to fetch more songs from extension {}: {:?}",
                                detail.package_name,
                                error
                            );
                            pagination.lock().unwrap().mark_error(&detail.package_name);
                        }
                    }
                }
            }
            .in_current_span(),
        );
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub fn on_show(&self) {
        self.pagination.lock().unwrap().reset();

        let Some(main_window) = self.weak.upgrade() else {
            return;
        };
        let (entity, extension) = P::get_entity(&main_window);

        let state_manager = self.state_manager;
        let weak = self.weak.clone();
        let coordinator = self.clone();

        tokio::spawn(
            async move {
                let ext_handler = state_manager.get_extension_handler();
                let providers =
                    fetch_scope_providers(&ext_handler, P::extension_scope(), &extension).await;

                let providers_for_ui = providers.clone();
                let _ = weak.upgrade_in_event_loop(move |main_window| {
                    P::set_extensions(&main_window, ModelRc::new(VecModel::from(providers_for_ui)));
                });

                if let Ok(songs) = P::fetch_local_songs(state_manager, entity.clone()).await {
                    let _ = weak.upgrade_in_event_loop(move |main_window| {
                        let song_models = map_songs_to_models(songs, None);
                        let model = make_lazy_song_model(&main_window, state_manager, song_models);
                        P::set_songs(&main_window, model);
                    });
                }

                let enabled_packages: HashSet<String> = providers
                    .into_iter()
                    .filter(|p| p.enabled)
                    .map(|p| p.id.to_string())
                    .collect();

                coordinator
                    .fetch_enabled_extension_songs(entity, enabled_packages)
                    .await;
            }
            .in_current_span(),
        );
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub fn on_hide(&self) {
        self.pagination.lock().unwrap().reset();
        if let Some(main_window) = self.weak.upgrade() {
            P::clear_ui(&main_window);
        }
    }
}

#[tracing::instrument(level = "debug", skip_all)]
pub(crate) fn map_songs_to_models(
    songs: Vec<Song>,
    detail: Option<&ExtensionDetail>,
) -> Vec<SongModel> {
    songs.into_iter().map(|s| (s, detail).into()).collect()
}

#[tracing::instrument(level = "debug", skip_all)]
pub(crate) async fn fetch_scope_providers(
    ext_handler: &ExtensionHandler,
    scope: Option<ExtensionProviderScope>,
    current_extension: &str,
) -> Vec<ExtensionProviderItem> {
    let Some(scope) = scope else {
        return Vec::new();
    };

    ext_handler
        .get_extensions_with_scope(scope)
        .await
        .into_iter()
        .map(|ext| {
            let detail = ext.get_extension_detail();
            let enabled = !current_extension.is_empty() && current_extension == detail.package_name;
            ExtensionProviderItem {
                id: detail.package_name.into(),
                name: detail.name.into(),
                enabled,
            }
        })
        .collect()
}

#[tracing::instrument(level = "debug", skip_all)]
pub fn update_provider_list_enabled(
    providers: &[ExtensionProviderItem],
    package_name: &str,
    enabled: bool,
) -> Vec<ExtensionProviderItem> {
    providers
        .iter()
        .cloned()
        .map(|mut p| {
            if p.id == package_name {
                p.enabled = enabled;
            }
            p
        })
        .collect()
}
