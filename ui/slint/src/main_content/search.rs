use extensions::Extension;
use extensions_proto::moosync::types::{
    ExtensionDetail, ExtensionProviderScope, RequestedSearchResultRequest,
};
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

pub struct SearchPageHandler<'a> {
    main_window: &'a MainWindow,
    state_manager: &'a StateManager,
}

impl<'a> SearchPageHandler<'a> {
    #[tracing::instrument(level = "debug", skip_all)]
    pub fn new(main_window: &'a MainWindow, state_manager: &'a StateManager) -> Self {
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
        let database = state_manager.get_database().await;
        Ok(database.search_all(term)?)
    }

    #[tracing::instrument(level = "debug", skip_all)]
    async fn search_extension(term: &str, ext: &Extension) -> Result<ProtoSearchResult, UiError> {
        let resp = ext
            .get_search_result(RequestedSearchResultRequest {
                query: term.to_string(),
            })
            .await?;
        Ok(resp.to_songs_proto())
    }

    #[tracing::instrument(level = "debug", skip_all)]
    fn append_search_result(
        main_window_weak: &Weak<MainWindow>,
        state_manager: &StateManager,
        res: ProtoSearchResult,
        detail: Option<ExtensionDetail>,
    ) {
        let cache_dir = state_manager.get_cache_dir();
        let _ = main_window_weak.upgrade_in_event_loop(move |window| {
            let theme = window.global::<Theme>();
            let result_model = create_search_result(res, detail.as_ref(), &theme, &cache_dir);
            let props = window.global::<SearchPageProps>();
            let mut list: Vec<SearchResult> = props.get_provider_results().into_vec();
            list.push(result_model);
            props.set_provider_results(ModelRc::new(VecModel::from(list)));
        });
    }

    #[tracing::instrument(level = "debug", skip_all)]
    fn perform_search(
        state_manager: StateManager,
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

                match Self::search_local(&state_manager, &term).await {
                    Ok(local) => {
                        let _ = main_window_weak.upgrade_in_event_loop(|window| {
                            window
                                .global::<SearchPageProps>()
                                .set_provider_results(ModelRc::default());
                        });
                        Self::append_search_result(&main_window_weak, &state_manager, local, None);
                    }
                    Err(e) => {
                        tracing::error!("Local search failed for query '{}': {:?}", term, e);
                    }
                }

                let ext_handler = state_manager.get_extension_handler().await;
                let active_extensions = ext_handler
                    .get_extensions_with_scope(ExtensionProviderScope::Search)
                    .await;

                for ext in active_extensions {
                    let state_manager = state_manager.clone();
                    let main_window_weak = main_window_weak.clone();
                    let term = term.clone();

                    tokio::spawn(async move {
                        let detail = ext.get_extension_detail();
                        match Self::search_extension(&term, &ext).await {
                            Ok(res) => {
                                Self::append_search_result(
                                    &main_window_weak,
                                    &state_manager,
                                    res,
                                    Some(detail),
                                );
                            }
                            Err(e) => {
                                tracing::error!(
                                    "Extension search failed for '{}' ({:?}): {:?}",
                                    detail.package_name,
                                    term,
                                    e
                                );
                            }
                        }
                    });
                }
            }
            .instrument(tracing::debug_span!("slint_cb_perform_search")),
        );
    }
}

impl<'a> PageHandler for SearchPageHandler<'a> {
    #[tracing::instrument(level = "debug", skip_all)]
    fn initialize(&self) {
        let state_manager = self.state_manager.clone();
        let main_window_weak = self.main_window.as_weak();
        self.main_window
            .global::<AppCallbacks>()
            .on_search_term_changed(move |term| {
                Self::perform_search(
                    state_manager.clone(),
                    main_window_weak.clone(),
                    term.to_string(),
                );
            });
    }
}
