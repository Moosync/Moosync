use slint::{ComponentHandle, Image, ModelRc, VecModel};
use state_manager::StateManager;
use types::prelude::SearchResultExt;

use crate::{
    AppCallbacks, MainWindow, SearchPageProps,
    error::UiError,
    pages::PageHandler,
    utils::{cache_image, load_local_icon, to_search_result},
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
    ) -> Result<songs_proto::moosync::types::SearchResult, UiError> {
        let database = state_manager.get_database().await;
        Ok(database.search_all(term)?)
    }

    #[tracing::instrument(level = "debug", skip_all)]
    async fn search_extension(
        term: &str,
        ext: &extensions::Extension,
    ) -> Result<songs_proto::moosync::types::SearchResult, UiError> {
        let resp = ext
            .get_search_result(
                extensions_proto::moosync::types::RequestedSearchResultRequest {
                    query: term.to_string(),
                },
            )
            .await?;
        Ok(resp.to_songs_proto())
    }

    #[tracing::instrument(level = "debug", skip_all)]
    async fn search_extensions(
        state_manager: &StateManager,
        term: &str,
    ) -> Vec<(
        extensions_proto::moosync::types::ExtensionDetail,
        Option<String>,
        songs_proto::moosync::types::SearchResult,
    )> {
        let ext_handler = state_manager.get_extension_handler().await;
        let active_extensions = ext_handler
            .get_extensions_with_scope(
                extensions_proto::moosync::types::ExtensionProviderScope::Search,
            )
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
        local_res: Result<songs_proto::moosync::types::SearchResult, UiError>,
        ext_results: Vec<(
            extensions_proto::moosync::types::ExtensionDetail,
            Option<String>,
            songs_proto::moosync::types::SearchResult,
        )>,
    ) {
        let theme = main_window.global::<crate::Theme>();
        let cache_dir = state_manager.get_cache_dir();
        let mut list = Vec::new();

        if let Ok(local) = local_res {
            let local_icon =
                Image::load_from_svg_data(include_bytes!("../icons/folder.svg")).unwrap();
            let results = to_search_result(local, None, local_icon, &theme, &cache_dir);
            list.push(results);
        }

        for (detail, cached_path, res) in ext_results {
            let icon = load_local_icon(cached_path.as_deref().unwrap_or(""));
            let results = to_search_result(res, Some(&detail), icon, &theme, &cache_dir);
            list.push(results);
        }

        main_window
            .global::<SearchPageProps>()
            .set_provider_results(ModelRc::new(VecModel::from(list)));
    }

    #[tracing::instrument(level = "debug", skip_all)]
    fn perform_search(
        state_manager: StateManager,
        main_window_weak: slint::Weak<MainWindow>,
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
