use extensions::Extension;
use extensions_proto::moosync::types::{
    ExtensionDetail, ExtensionProviderScope, RequestedSearchResultRequest,
};
use slint::{ComponentHandle, ModelRc, VecModel, Weak};
use songs_proto::moosync::types::SearchResult as ProtoSearchResult;
use state_manager::StateManager;
use types::prelude::SearchResultExt;

use crate::{
    AppCallbacks, MainWindow, SearchPageProps, Theme,
    error::UiError,
    pages::PageHandler,
    utils::{cache_image, default_folder_icon, load_icon},
};

type SearchResultItem = (ExtensionDetail, Option<String>, ProtoSearchResult);

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
    async fn search_extensions(state_manager: &StateManager, term: &str) -> Vec<SearchResultItem> {
        let ext_handler = state_manager.get_extension_handler().await;
        let active_extensions = ext_handler
            .get_extensions_with_scope(ExtensionProviderScope::Search)
            .await;

        let cache_dir = state_manager.get_cache_dir();
        let mut results = Vec::new();

        for ext in active_extensions {
            let detail = ext.get_extension_detail();
            let icon_path = detail.extension_icon.clone();
            let cached_path = match icon_path {
                Some(ref p) if !p.is_empty() => cache_image(p, &cache_dir)
                    .await
                    .map(|p| p.to_string_lossy().to_string()),
                _ => None,
            };
            let res = Self::search_extension(term, &ext).await;
            if let Ok(r) = res {
                results.push((detail, cached_path, r));
            }
        }
        results
    }

    #[tracing::instrument(level = "debug", skip_all)]
    fn update_ui(
        main_window: &MainWindow,
        state_manager: &StateManager,
        local_res: Result<ProtoSearchResult, UiError>,
        ext_results: Vec<SearchResultItem>,
    ) {
        let theme = main_window.global::<Theme>();
        let cache_dir = state_manager.get_cache_dir();
        let mut list = Vec::new();

        if let Ok(local) = local_res {
            let local_icon = default_folder_icon();
            let results = (local, None, local_icon, &theme, cache_dir.as_path()).into();
            list.push(results);
        }

        for (detail, cached_path, res) in ext_results {
            let icon = load_icon(cached_path.as_deref().unwrap_or(""));
            let results = (res, Some(&detail), icon, &theme, cache_dir.as_path()).into();
            list.push(results);
        }

        main_window
            .global::<SearchPageProps>()
            .set_provider_results(ModelRc::new(VecModel::from(list)));
    }

    #[tracing::instrument(level = "debug", skip_all)]
    fn perform_search(
        state_manager: StateManager,
        main_window_weak: Weak<MainWindow>,
        term: String,
    ) {
        tokio::spawn(async move {
            let local_res = Self::search_local(&state_manager, &term).await;
            let ext_results = Self::search_extensions(&state_manager, &term).await;

            let _ = slint::invoke_from_event_loop(move || {
                if let Some(main_window) = main_window_weak.upgrade() {
                    Self::update_ui(&main_window, &state_manager, local_res, ext_results);
                }
            });
        });
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

    #[tracing::instrument(level = "debug", skip_all)]
    fn on_show(&self) {}

    #[tracing::instrument(level = "debug", skip_all)]
    fn on_hide(&self) {}
}
