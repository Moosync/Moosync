use extensions::{Extension, ExtensionGroupExt};
use extensions_proto::moosync::types::{
    ExtensionDetail, ExtensionProviderScope, RequestedRecommendationsRequest,
};
use futures::StreamExt;
use slint::{ComponentHandle, ModelRc, VecModel};
use songs_proto::moosync::types::Song;
use state_manager::StateManager;
use tracing::Instrument;

use crate::{
    ExplorePageProps, MainWindow, ProviderRecommendations, SongModel,
    error::UiError,
    pages::PageHandler,
    utils::{cache_image, load_icon, make_lazy_song_model},
};

type RecommendationItem = (ExtensionDetail, Option<String>, Vec<Song>);

pub struct ExplorePageHandler {
    main_window: &'static MainWindow,
    state_manager: &'static StateManager,
}

impl ExplorePageHandler {
    #[tracing::instrument(level = "debug", skip_all)]
    pub fn new(main_window: &'static MainWindow, state_manager: &'static StateManager) -> Self {
        Self {
            main_window,
            state_manager,
        }
    }

    #[tracing::instrument(level = "debug", skip_all)]
    async fn fetch_extension_recommendations(ext: &Extension) -> Result<Vec<Song>, UiError> {
        let resp = ext
            .get_recommendations(RequestedRecommendationsRequest { refresh: false })
            .await?;
        Ok(resp.songs)
    }

    #[tracing::instrument(level = "debug", skip_all)]
    async fn fetch_all_recommendations(
        state_manager: &StateManager,
    ) -> Result<Vec<RecommendationItem>, UiError> {
        tracing::debug!("Fetching recommendations across all active providers");
        let mut results = Vec::new();
        let ext_handler = state_manager.get_extension_handler();
        let rec_extensions = ext_handler
            .get_extensions_with_scope(ExtensionProviderScope::Recommendations)
            .await;

        let cache_dir = state_manager.get_cache_dir();

        let mut stream = rec_extensions.with(|ext| {
            async move {
                (
                    ext.clone(),
                    Self::fetch_extension_recommendations(&ext).await,
                )
            }
            .in_current_span()
        });

        while let Some((ext, res)) = stream.next().await {
            let detail = ext.get_extension_detail();
            match res {
                Ok(songs) => {
                    tracing::debug!(
                        "Fetched {} recommendations from extension '{}'",
                        songs.len(),
                        detail.package_name
                    );
                    if !songs.is_empty() {
                        let cached_path = match &detail.extension_icon {
                            Some(p) if !p.is_empty() => cache_image(p, &cache_dir)
                                .await
                                .map(|p| p.to_string_lossy().to_string()),
                            _ => None,
                        };
                        results.push((detail, cached_path, songs));
                    }
                }
                Err(e) => {
                    tracing::error!(
                        "Failed to fetch recommendations from extension '{}': {:?}",
                        detail.package_name,
                        e
                    );
                }
            }
        }
        Ok(results)
    }
}

impl PageHandler for ExplorePageHandler {
    #[tracing::instrument(level = "debug", skip_all)]
    fn on_show(&self) {
        tracing::debug!("ExplorePage: on_show triggered");
        tokio::spawn({
            let state_manager = self.state_manager;
            let main_window_weak = self.main_window.as_weak();
            async move {
                match Self::fetch_all_recommendations(state_manager).await {
                    Ok(recommendations) => {
                        let _ = slint::invoke_from_event_loop(move || {
                            if let Some(main_window) = main_window_weak.upgrade() {
                                let mut list = Vec::new();
                                for (detail, cached_path, songs) in recommendations {
                                    let icon = load_icon(cached_path.as_deref().unwrap_or(""));
                                    let song_models = songs
                                        .into_iter()
                                        .map(|s| (s, Some(&detail)).into())
                                        .collect::<Vec<SongModel>>();
                                    let mapped_songs = make_lazy_song_model(
                                        &main_window,
                                        state_manager,
                                        song_models,
                                    );
                                    list.push(ProviderRecommendations {
                                        provider_name: detail.name.clone().into(),
                                        provider_icon: icon,
                                        songs: mapped_songs,
                                    });
                                }
                                main_window
                                    .global::<ExplorePageProps>()
                                    .set_provider_recommendations(ModelRc::new(VecModel::from(
                                        list,
                                    )));
                            }
                        });
                    }
                    Err(e) => {
                        tracing::error!("Failed to fetch recommendations: {:?}", e);
                    }
                }
            }
            .in_current_span()
        });
    }

    #[tracing::instrument(level = "debug", skip_all)]
    fn on_hide(&self) {
        tracing::debug!("ExplorePage: on_hide clearing recommendations");
        self.main_window
            .global::<ExplorePageProps>()
            .set_provider_recommendations(ModelRc::default());
    }
}
