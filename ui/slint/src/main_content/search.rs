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
    AppCallbacks, MainWindow, SearchPageProps, SearchResult, Theme,
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
    pub(crate) fn append_search_result_to_window(
        window: &MainWindow,
        cache_dir: &std::path::Path,
        res: ProtoSearchResult,
        detail: Option<ExtensionDetail>,
        has_account: bool,
    ) {
        let theme = window.global::<Theme>();
        let result_model =
            create_search_result(res, detail.as_ref(), has_account, &theme, cache_dir);
        let props = window.global::<SearchPageProps>();
        let mut list: Vec<SearchResult> = props.get_provider_results().into_vec();
        list.push(result_model);
        props.set_provider_results(ModelRc::new(VecModel::from(list)));
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub(crate) fn append_search_result(
        main_window_weak: &Weak<MainWindow>,
        state_manager: &StateManager,
        res: ProtoSearchResult,
        detail: Option<ExtensionDetail>,
        has_account: bool,
    ) {
        let cache_dir = state_manager.get_cache_dir();
        let _ = main_window_weak.upgrade_in_event_loop(move |window| {
            Self::append_search_result_to_window(&window, &cache_dir, res, detail, has_account);
        });
    }

    #[tracing::instrument(level = "debug", skip_all)]
    fn perform_search(
        state_manager: &'static StateManager,
        main_window_weak: Weak<MainWindow>,
        term: String,
    ) {
        tokio::spawn(
            async move {
                let term = term.trim().to_string();
                if term.is_empty() {
                    let _ = main_window_weak.upgrade_in_event_loop(|window| {
                        window
                            .global::<SearchPageProps>()
                            .set_provider_results(ModelRc::default());
                    });
                    return;
                }

                tracing::debug!("Performing search for '{}'", term);

                match Self::search_local(state_manager, &term).await {
                    Ok(local) => {
                        let _ = main_window_weak.upgrade_in_event_loop(|window| {
                            window
                                .global::<SearchPageProps>()
                                .set_provider_results(ModelRc::default());
                        });
                        Self::append_search_result(
                            &main_window_weak,
                            state_manager,
                            local,
                            None,
                            false,
                        );
                    }
                    Err(e) => {
                        tracing::error!("Local search failed for query '{}': {:?}", term, e);
                    }
                }

                let ext_handler = state_manager.get_extension_handler();
                let active_extensions = ext_handler
                    .get_extensions_with_scope(ExtensionProviderScope::Search)
                    .await;

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
                    match res {
                        Ok(resp) => {
                            let res = resp.to_songs_proto();
                            tracing::debug!(
                                "Extension search '{}' ({:?})",
                                detail.package_name,
                                res
                            );
                            Self::append_search_result(
                                &main_window_weak,
                                state_manager,
                                res,
                                Some(detail),
                                has_account,
                            );
                        }
                        Err(e) => {
                            tracing::error!(
                                "Extension search failed for '{}' ({:?}): {:?}",
                                detail.package_name,
                                term,
                                e
                            );
                            if has_account {
                                Self::append_search_result(
                                    &main_window_weak,
                                    state_manager,
                                    ProtoSearchResult::default(),
                                    Some(detail),
                                    has_account,
                                );
                            }
                        }
                    }
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
                if term.trim().is_empty() {
                    if let Some(window) = main_window_weak.upgrade() {
                        window
                            .global::<SearchPageProps>()
                            .set_provider_results(ModelRc::default());
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
}

impl PageHandler for SearchPageHandler {
    #[tracing::instrument(level = "debug", skip_all)]
    fn initialize(&self) {
        self.setup_callbacks();
        self.setup_accounts_listener();
    }
}
