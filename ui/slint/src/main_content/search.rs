use std::{
    collections::HashSet,
    path::Path,
    sync::atomic::{AtomicU64, Ordering},
};

static SEARCH_ID: AtomicU64 = AtomicU64::new(0);

use extensions::ExtensionGroupExt;
use extensions_proto::moosync::types::{
    ExtensionDetail, ExtensionProviderScope, RequestedSearchResultRequest,
};
use futures::StreamExt;
use slint::{ComponentHandle, ModelRc, VecModel, Weak};
use songs_proto::moosync::types::SearchResult as ProtoSearchResult;
use state_manager::StateManager;
use tracing::Instrument;
use types::prelude::SearchResultExt;

use crate::{
    AppCallbacks, MainWindow, SearchPageProps, Theme,
    error::UiError,
    pages::PageHandler,
    utils::{IntoVec, create_search_result},
};

pub struct SearchPageHandler {
    main_window: &'static MainWindow,
    state_manager: &'static StateManager,
}

impl SearchPageHandler {
    #[tracing::instrument(level = "debug", skip_all)]
    pub fn new(main_window: &'static MainWindow, state_manager: &'static StateManager) -> Self {
        Self {
            main_window,
            state_manager,
        }
    }

    #[tracing::instrument(level = "debug", skip_all)]
    async fn search_local(
        state_manager: &StateManager,
        term: &str,
    ) -> Result<ProtoSearchResult, UiError> {
        let database = state_manager.get_database();
        Ok(database.search_all(term)?)
    }

    #[tracing::instrument(level = "debug", skip_all)]
    fn has_account(state_manager: &StateManager, detail: &ExtensionDetail) -> bool {
        let has_account_scope = detail
            .scopes
            .contains(&(ExtensionProviderScope::Accounts as i32));
        let Ok(ext) = state_manager
            .get_extension_handler()
            .get_extension(&detail.package_name)
        else {
            return has_account_scope;
        };
        !ext.get_accounts().is_empty() || has_account_scope
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub(crate) fn init_providers(window: &MainWindow, state_manager: &StateManager) {
        let theme = window.global::<Theme>();
        let cache_dir = state_manager.get_cache_dir();
        let props = window.global::<SearchPageProps>();
        let has_query = !props.get_search_query().trim().is_empty();
        let current = props.get_provider_results().into_vec();

        let extension_handler = state_manager.get_extension_handler();
        let search_extensions: Vec<ExtensionDetail> = extension_handler
            .get_installed_extensions()
            .into_iter()
            .filter(|detail| {
                detail
                    .scopes
                    .contains(&(ExtensionProviderScope::Search as i32))
            })
            .collect();

        let mut providers = Vec::with_capacity(search_extensions.len() + 1);

        let find_or_create =
            |package_name: &str, detail: Option<&ExtensionDetail>, has_account: bool| {
                if has_query {
                    if let Some(existing) = current.iter().find(|r| r.extension == package_name) {
                        return existing.clone();
                    }
                }
                create_search_result(
                    ProtoSearchResult::default(),
                    detail,
                    has_account,
                    false,
                    &theme,
                    &cache_dir,
                )
            };

        providers.push(find_or_create("", None, false));

        for detail in search_extensions {
            let has_account = Self::has_account(state_manager, &detail);
            providers.push(find_or_create(
                &detail.package_name,
                Some(&detail),
                has_account,
            ));
        }

        props.set_provider_results(ModelRc::new(VecModel::from(providers)));
    }

    #[tracing::instrument(level = "debug", skip_all)]
    fn reset_providers(window: &MainWindow, state_manager: &StateManager, loading: bool) {
        let props = window.global::<SearchPageProps>();
        let mut list = props.get_provider_results().into_vec();

        if list.is_empty() {
            Self::init_providers(window, state_manager);
            list = props.get_provider_results().into_vec();
        }

        for item in list.iter_mut() {
            item.songs = ModelRc::default();
            item.albums = ModelRc::default();
            item.artists = ModelRc::default();
            item.playlists = ModelRc::default();
            item.genres = ModelRc::default();
            item.loading = loading;
        }

        props.set_provider_results(ModelRc::new(VecModel::from(list)));
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub(crate) fn update_provider_result(
        window: &MainWindow,
        cache_dir: &Path,
        res: ProtoSearchResult,
        detail: Option<ExtensionDetail>,
        has_account: bool,
        loading: bool,
    ) {
        let theme = window.global::<Theme>();
        let extension_package = detail
            .as_ref()
            .map(|d| d.package_name.as_str())
            .unwrap_or_default();
        let result_model = create_search_result(
            res,
            detail.as_ref(),
            has_account,
            loading,
            &theme,
            cache_dir,
        );
        let props = window.global::<SearchPageProps>();
        let mut list = props.get_provider_results().into_vec();

        if let Some(position) = list.iter().position(|r| r.extension == extension_package) {
            list[position] = result_model;
        } else {
            list.push(result_model);
        }

        props.set_provider_results(ModelRc::new(VecModel::from(list)));
    }

    #[tracing::instrument(level = "debug", skip_all)]
    fn update_provider_result_if_current(
        main_window_weak: &Weak<MainWindow>,
        cache_dir: &Path,
        search_id: u64,
        res: ProtoSearchResult,
        detail: Option<ExtensionDetail>,
        has_account: bool,
    ) {
        let cache_dir = cache_dir.to_path_buf();
        let _ = main_window_weak.upgrade_in_event_loop(move |window| {
            if search_id == SEARCH_ID.load(Ordering::SeqCst) {
                Self::update_provider_result(&window, &cache_dir, res, detail, has_account, false);
            }
        });
    }

    #[tracing::instrument(level = "debug", skip_all)]
    fn perform_search(
        state_manager: &'static StateManager,
        main_window_weak: Weak<MainWindow>,
        term: String,
    ) {
        let search_id = SEARCH_ID.fetch_add(1, Ordering::SeqCst) + 1;
        tokio::spawn(
            async move {
                let term = term.trim().to_string();
                if term.is_empty() {
                    let _ = main_window_weak.upgrade_in_event_loop(move |window| {
                        Self::reset_providers(&window, state_manager, false);
                    });
                    return;
                }

                tracing::debug!("Performing search for '{}' (id: {})", term, search_id);

                let _ = main_window_weak.upgrade_in_event_loop(move |window| {
                    if search_id == SEARCH_ID.load(Ordering::SeqCst) {
                        Self::reset_providers(&window, state_manager, true);
                    }
                });

                let cache_dir = state_manager.get_cache_dir();
                let local_res = Self::search_local(state_manager, &term)
                    .await
                    .unwrap_or_else(|e| {
                        tracing::error!("Local search failed for query '{}': {:?}", term, e);
                        ProtoSearchResult::default()
                    });
                Self::update_provider_result_if_current(
                    &main_window_weak,
                    &cache_dir,
                    search_id,
                    local_res,
                    None,
                    false,
                );

                let extension_handler = state_manager.get_extension_handler();
                let active_extensions = extension_handler
                    .get_extensions_with_scope(ExtensionProviderScope::Search)
                    .await;

                let active_packages: HashSet<String> = active_extensions
                    .iter()
                    .map(|ext| ext.get_package_name().to_string())
                    .collect();

                for detail in extension_handler.get_installed_extensions() {
                    let is_search = detail
                        .scopes
                        .contains(&(ExtensionProviderScope::Search as i32));
                    if is_search && !active_packages.contains(&detail.package_name) {
                        let has_account = Self::has_account(state_manager, &detail);
                        Self::update_provider_result_if_current(
                            &main_window_weak,
                            &cache_dir,
                            search_id,
                            ProtoSearchResult::default(),
                            Some(detail),
                            has_account,
                        );
                    }
                }

                let req = RequestedSearchResultRequest {
                    query: term.clone(),
                };
                let mut stream = active_extensions.with(|ext| {
                    let req = req.clone();
                    let detail = ext.get_extension_detail();
                    let has_account = !ext.get_accounts().is_empty()
                        || detail
                            .scopes
                            .contains(&(ExtensionProviderScope::Accounts as i32));
                    async move { (detail, has_account, ext.get_search_result(req).await) }
                        .in_current_span()
                });

                while let Some((detail, has_account, res)) = stream.next().await {
                    let proto = match res {
                        Ok(resp) => {
                            let proto = resp.to_songs_proto();
                            tracing::debug!(
                                "Extension search '{}' ({:?})",
                                detail.package_name,
                                proto
                            );
                            proto
                        }
                        Err(e) => {
                            tracing::error!(
                                "Extension search failed for '{}' ({:?}): {:?}",
                                detail.package_name,
                                term,
                                e
                            );
                            ProtoSearchResult::default()
                        }
                    };
                    Self::update_provider_result_if_current(
                        &main_window_weak,
                        &cache_dir,
                        search_id,
                        proto,
                        Some(detail),
                        has_account,
                    );
                }
            }
            .instrument(tracing::debug_span!("slint_cb_perform_search")),
        );
    }

    #[tracing::instrument(level = "debug", skip_all)]
    fn setup_callbacks(&self) {
        let state_manager = self.state_manager;
        let main_window_weak = self.main_window.as_weak();
        self.main_window
            .global::<AppCallbacks>()
            .on_search_term_changed(move |term| {
                if let Some(window) = main_window_weak.upgrade() {
                    let props = window.global::<SearchPageProps>();
                    if props.get_search_query() != term {
                        props.set_search_query(term.clone());
                    }
                }
                if term.trim().is_empty() {
                    SEARCH_ID.fetch_add(1, Ordering::SeqCst);
                    if let Some(window) = main_window_weak.upgrade() {
                        Self::reset_providers(&window, state_manager, false);
                    }
                    return;
                }
                Self::perform_search(state_manager, main_window_weak.clone(), term.to_string());
            });
    }

    #[tracing::instrument(level = "debug", skip_all)]
    fn setup_accounts_listener(&self) {
        let state_manager = self.state_manager;
        let main_window_weak = self.main_window.as_weak();
        tokio::spawn(
            async move {
                let extension_handler = state_manager.get_extension_handler();
                let _cancel = extension_handler.on_accounts_updated.listen({
                    let main_window_weak = main_window_weak.clone();
                    move |_| {
                        let _ = main_window_weak.upgrade_in_event_loop(move |window| {
                            let query = window.global::<SearchPageProps>().get_search_query();
                            if !query.trim().is_empty() {
                                Self::perform_search(
                                    state_manager,
                                    window.as_weak(),
                                    query.to_string(),
                                );
                            }
                        });
                    }
                });
            }
            .in_current_span(),
        );
    }

    #[tracing::instrument(level = "debug", skip_all)]
    fn setup_extensions_listener(&self) {
        let state_manager = self.state_manager;
        let main_window_weak = self.main_window.as_weak();
        tokio::spawn(
            async move {
                let extension_handler = state_manager.get_extension_handler();
                let _cancel = extension_handler.on_extensions_updated.listen({
                    let main_window_weak = main_window_weak.clone();
                    move |_| {
                        let _ = main_window_weak.upgrade_in_event_loop(move |window| {
                            Self::init_providers(&window, state_manager);
                        });
                    }
                });
            }
            .in_current_span(),
        );
    }
}

impl PageHandler for SearchPageHandler {
    #[tracing::instrument(level = "debug", skip_all)]
    fn initialize(&self) {
        Self::init_providers(self.main_window, self.state_manager);
        self.setup_callbacks();
        self.setup_accounts_listener();
        self.setup_extensions_listener();
    }

    #[tracing::instrument(level = "debug", skip_all)]
    fn on_show(&self) { Self::init_providers(self.main_window, self.state_manager); }
}
