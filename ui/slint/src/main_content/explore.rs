use slint::{ComponentHandle, ModelRc, VecModel};
use state_manager::StateManager;

use crate::{
    ExplorePageProps, MainWindow, ProviderRecommendations,
    error::UiError,
    pages::PageHandler,
    utils::{LazySongVecModel, cache_image, load_local_icon, to_song_model},
};

pub struct ExplorePageHandler<'a> {
    main_window: &'a MainWindow,
    state_manager: &'a StateManager,
}

impl<'a> ExplorePageHandler<'a> {
    #[tracing::instrument(level = "debug", skip_all)]
    pub fn new(main_window: &'a MainWindow, state_manager: &'a StateManager) -> Self {
        Self {
            main_window,
            state_manager,
        }
    }

    #[tracing::instrument(level = "debug", skip_all)]
    async fn fetch_extension_recommendations(
        ext: &extensions::Extension,
    ) -> Result<Vec<songs_proto::moosync::types::Song>, UiError> {
        let resp = ext
            .get_recommendations(
                extensions_proto::moosync::types::RequestedRecommendationsRequest {
                    refresh: false,
                },
            )
            .await?;
        Ok(resp.songs)
    }

    #[tracing::instrument(level = "debug", skip_all)]
    async fn fetch_all_recommendations(
        state_manager: &StateManager,
    ) -> Result<
        Vec<(
            extensions_proto::moosync::types::ExtensionDetail,
            Option<String>,
            Vec<songs_proto::moosync::types::Song>,
        )>,
        UiError,
    > {
        let mut results = Vec::new();
        let ext_handler = state_manager.get_extension_handler().await;
        let rec_extensions = ext_handler
            .get_extensions_with_scope(
                extensions_proto::moosync::types::ExtensionProviderScope::Recommendations,
            )
            .await;

        let cache_dir = state_manager.get_cache_dir();

        for ext in rec_extensions {
            let detail = ext.get_extension_detail();
            let icon_path = detail.extension_icon.clone();
            let cached_path = match icon_path {
                Some(p) if !p.is_empty() => cache_image(&p, &cache_dir)
                    .await
                    .map(|p| p.to_string_lossy().to_string()),
                _ => None,
            };
            let res = Self::fetch_extension_recommendations(&ext).await;
            if let Ok(songs) = res {
                if !songs.is_empty() {
                    results.push((detail, cached_path, songs));
                }
            }
        }
        Ok(results)
    }
}

impl<'a> PageHandler for ExplorePageHandler<'a> {
    #[tracing::instrument(level = "debug", skip_all)]
    fn initialize(&self) {}

    #[tracing::instrument(level = "debug", skip_all)]
    fn on_show(&self) {
        tokio::spawn({
            let state_manager = self.state_manager.clone();
            let main_window_weak = self.main_window.as_weak();
            async move {
                if let Ok(recommendations) = Self::fetch_all_recommendations(&state_manager).await {
                    let _ = slint::invoke_from_event_loop(move || {
                        if let Some(main_window) = main_window_weak.upgrade() {
                            let cache_dir = state_manager.get_cache_dir();
                            let theme = main_window.global::<crate::Theme>();
                            let mut list = Vec::new();
                            for (detail, cached_path, songs) in recommendations {
                                let icon = load_local_icon(cached_path.as_deref().unwrap_or(""));
                                let song_models = songs
                                    .iter()
                                    .map(|s| to_song_model(s, Some(&detail)))
                                    .collect::<Vec<_>>();
                                let mapped_songs = ModelRc::new(LazySongVecModel::new(
                                    song_models,
                                    theme.get_songListItemHeight() as usize,
                                    theme.get_songListItemWidth() as usize,
                                    cache_dir.clone(),
                                ));
                                list.push(ProviderRecommendations {
                                    provider_name: detail.name.clone().into(),
                                    provider_icon: icon,
                                    songs: mapped_songs,
                                });
                            }
                            main_window
                                .global::<ExplorePageProps>()
                                .set_provider_recommendations(ModelRc::new(VecModel::from(list)));
                        }
                    });
                }
            }
        });
    }

    #[tracing::instrument(level = "debug", skip_all)]
    fn on_hide(&self) {
        self.main_window
            .global::<ExplorePageProps>()
            .set_provider_recommendations(ModelRc::default());
    }
}
