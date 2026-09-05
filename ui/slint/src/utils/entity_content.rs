use std::{
    collections::HashMap,
    future::Future,
    marker::PhantomData,
    sync::{Arc, Mutex},
};

use extensions::ExtensionHandler;
use extensions_proto::moosync::types::{ExtensionDetail, ExtensionProviderScope};
use slint::{ComponentHandle, ModelRc, VecModel, Weak};
use songs_proto::moosync::types::Song;
use state_manager::StateManager;
use tracing::Instrument;

use crate::{
    ExtensionProviderItem, MainWindow, SongModel, error::UiError, utils::make_lazy_song_model,
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
    pub fn remove_extension(&mut self, package_name: &str) { self.extensions.remove(package_name); }
}

pub trait EntitySongProvider: Send + Sync + 'static {
    type Entity: Clone + Send + 'static;

    fn extension_scope() -> Option<ExtensionProviderScope>;
    fn get_entity(main_window: &MainWindow) -> (Self::Entity, String);
    fn get_songs(main_window: &MainWindow) -> Vec<SongModel>;
    fn set_songs(main_window: &MainWindow, model: ModelRc<SongModel>);
    fn update_extensions_enabled(main_window: &MainWindow, package_name: &str, enabled: bool);
    fn set_extensions(main_window: &MainWindow, extensions: ModelRc<ExtensionProviderItem>);
    fn clear_ui(main_window: &MainWindow);

    fn fetch_local_songs(
        state_manager: &StateManager,
        entity: Self::Entity,
    ) -> impl Future<Output = Result<Vec<Song>, UiError>> + Send;

    fn fetch_extension_songs(
        state_manager: &StateManager,
        entity: Self::Entity,
        extension: String,
        page_token: Option<String>,
    ) -> impl Future<Output = Result<(Vec<Song>, Option<String>), UiError>> + Send;
}

pub struct EntityContentCoordinator<P: EntitySongProvider> {
    weak: Weak<MainWindow>,
    state_manager: StateManager,
    pagination: Arc<Mutex<ExtensionPaginationManager>>,
    _phantom: PhantomData<P>,
}

impl<P: EntitySongProvider> Clone for EntityContentCoordinator<P> {
    #[tracing::instrument(level = "debug", skip_all)]
    fn clone(&self) -> Self {
        Self {
            weak: self.weak.clone(),
            state_manager: self.state_manager.clone(),
            pagination: self.pagination.clone(),
            _phantom: PhantomData,
        }
    }
}

impl<P: EntitySongProvider> EntityContentCoordinator<P> {
    #[tracing::instrument(level = "debug", skip_all)]
    pub fn new(main_window: &MainWindow, state_manager: &StateManager) -> Self {
        Self {
            weak: main_window.as_weak(),
            state_manager: state_manager.clone(),
            pagination: Arc::new(Mutex::new(ExtensionPaginationManager::default())),
            _phantom: PhantomData,
        }
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub fn on_toggle_extension(&self, package_name: String, enabled: bool) {
        if !enabled {
            self.pagination
                .lock()
                .unwrap()
                .remove_extension(&package_name);
            let state_manager = self.state_manager.clone();
            let _ = self.weak.upgrade_in_event_loop(move |main_window| {
                P::update_extensions_enabled(&main_window, &package_name, false);
                let current = P::get_songs(&main_window);
                let filtered: Vec<SongModel> = current
                    .into_iter()
                    .filter(|s| s.extension != package_name)
                    .collect();
                let model = make_lazy_song_model(&main_window, &state_manager, filtered);
                P::set_songs(&main_window, model);
            });
            return;
        }

        let Some(main_window) = self.weak.upgrade() else {
            return;
        };
        P::update_extensions_enabled(&main_window, &package_name, true);
        let (entity, _) = P::get_entity(&main_window);
        self.pagination
            .lock()
            .unwrap()
            .register_extension(&package_name);

        let state_manager = self.state_manager.clone();
        let weak = self.weak.clone();
        let pagination = self.pagination.clone();

        tokio::spawn(
            async move {
                let ext = match state_manager
                    .get_extension_handler()
                    .await
                    .get_extension(&package_name)
                {
                    Ok(ext) => ext,
                    Err(e) => {
                        tracing::error!("Failed to get extension {} detail: {:?}", package_name, e);
                        pagination.lock().unwrap().remove_extension(&package_name);
                        let _ = weak.upgrade_in_event_loop(move |window| {
                            P::update_extensions_enabled(&window, &package_name, false);
                        });
                        return;
                    }
                };
                let detail = ext.get_extension_detail();

                let (new_songs, next_token) = match P::fetch_extension_songs(
                    &state_manager,
                    entity,
                    package_name.clone(),
                    None,
                )
                .await
                {
                    Ok(result) => result,
                    Err(error) => {
                        tracing::error!(
                            "Failed to fetch songs from extension {}: {:?}",
                            package_name,
                            error
                        );
                        pagination.lock().unwrap().remove_extension(&package_name);
                        let _ = weak.upgrade_in_event_loop(move |window| {
                            P::update_extensions_enabled(&window, &package_name, false);
                        });
                        return;
                    }
                };

                let _ = weak.upgrade_in_event_loop(move |main_window| {
                    let mut current = P::get_songs(&main_window);
                    current.extend(map_songs_to_models(new_songs, Some(&detail)));
                    let model = make_lazy_song_model(&main_window, &state_manager, current);
                    P::set_songs(&main_window, model);

                    pagination
                        .lock()
                        .unwrap()
                        .update_after_fetch(&package_name, next_token);
                });
            }
            .in_current_span(),
        );
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub fn on_load_more_songs(&self) {
        let extensions_to_fetch = self.pagination.lock().unwrap().load_more();
        if extensions_to_fetch.is_empty() {
            return;
        }

        let Some(main_window) = self.weak.upgrade() else {
            return;
        };
        let (entity, _) = P::get_entity(&main_window);

        for (package_name, page_token) in extensions_to_fetch {
            let state_manager = self.state_manager.clone();
            let weak = self.weak.clone();
            let pagination = self.pagination.clone();
            let entity = entity.clone();

            tokio::spawn(
                async move {
                    let (new_songs, next_token) = match P::fetch_extension_songs(
                        &state_manager,
                        entity,
                        package_name.clone(),
                        page_token,
                    )
                    .await
                    {
                        Ok(result) => result,
                        Err(error) => {
                            tracing::error!(
                                "Failed to fetch songs from extension {}: {:?}",
                                package_name,
                                error
                            );
                            pagination.lock().unwrap().mark_error(&package_name);
                            return;
                        }
                    };

                    let detail = state_manager
                        .get_extension_handler()
                        .await
                        .get_extension(&package_name)
                        .ok()
                        .map(|e| e.get_extension_detail());

                    let _ = weak.upgrade_in_event_loop(move |window| {
                        let mut current = P::get_songs(&window);
                        current.extend(map_songs_to_models(new_songs, detail.as_ref()));
                        let model = make_lazy_song_model(&window, &state_manager, current);
                        P::set_songs(&window, model);

                        pagination
                            .lock()
                            .unwrap()
                            .update_after_fetch(&package_name, next_token);
                    });
                }
                .in_current_span(),
            );
        }
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub fn on_show(&self) {
        self.pagination.lock().unwrap().reset();

        let Some(main_window) = self.weak.upgrade() else {
            return;
        };
        let (entity, extension) = P::get_entity(&main_window);

        if !extension.is_empty() {
            self.pagination
                .lock()
                .unwrap()
                .register_extension(&extension);
        }

        let state_manager = self.state_manager.clone();
        let weak = self.weak.clone();
        let pagination = self.pagination.clone();

        tokio::spawn(
            async move {
                let ext_handler = state_manager.get_extension_handler().await;
                let (extensions, detail) =
                    fetch_scope_providers(&ext_handler, P::extension_scope(), &extension).await;

                let (songs, next_token) = if !extension.is_empty() {
                    match P::fetch_extension_songs(&state_manager, entity, extension.clone(), None)
                        .await
                    {
                        Ok(result) => result,
                        Err(error) => {
                            tracing::error!("Failed to fetch songs from extension: {:?}", error);
                            return;
                        }
                    }
                } else {
                    match P::fetch_local_songs(&state_manager, entity).await {
                        Ok(songs) => (songs, None),
                        Err(error) => {
                            tracing::error!("Failed to fetch local songs: {:?}", error);
                            return;
                        }
                    }
                };

                let _ = weak.upgrade_in_event_loop(move |main_window| {
                    P::set_extensions(&main_window, ModelRc::new(VecModel::from(extensions)));
                    let song_models = map_songs_to_models(songs, detail.as_ref());
                    let model = make_lazy_song_model(&main_window, &state_manager, song_models);
                    P::set_songs(&main_window, model);

                    if !extension.is_empty() {
                        pagination
                            .lock()
                            .unwrap()
                            .update_after_fetch(&extension, next_token);
                    }
                });
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
pub fn map_songs_to_models(songs: Vec<Song>, detail: Option<&ExtensionDetail>) -> Vec<SongModel> {
    songs.into_iter().map(|s| (s, detail).into()).collect()
}

#[tracing::instrument(level = "debug", skip_all)]
pub async fn fetch_scope_providers(
    ext_handler: &ExtensionHandler,
    scope: Option<ExtensionProviderScope>,
    current_extension: &str,
) -> (Vec<ExtensionProviderItem>, Option<ExtensionDetail>) {
    let Some(scope) = scope else {
        return (Vec::new(), None);
    };

    let active_exts = ext_handler.get_extensions_with_scope(scope).await;

    let providers: Vec<ExtensionProviderItem> = active_exts
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
        .collect();

    let detail = if !current_extension.is_empty() {
        ext_handler
            .get_extension(current_extension)
            .ok()
            .map(|e| e.get_extension_detail())
    } else {
        None
    };

    (providers, detail)
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
