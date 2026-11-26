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
    AppProps, ExplorePageProps, MainWindow, Pages, ProviderRecommendations, SongModel,
    error::UiError,
    pages::PageHandler,
    utils::{IntoVec, load_image, make_lazy_song_model, resolve_image},
};

type RecommendationItem = (ExtensionDetail, Option<String>, Vec<Song>);

pub struct ExplorePageHandler {
    main_window: MainWindow,
    state_manager: StateManager,
}

impl ExplorePageHandler {
    #[tracing::instrument(level = "debug", skip_all)]
    pub fn new(main_window: &MainWindow, state_manager: &StateManager) -> Self {
        Self {
            main_window: main_window.clone_strong(),
            state_manager: state_manager.clone(),
        }
    }

    #[tracing::instrument(level = "debug", skip_all)]
    async fn fetch_extension_recommendations(ext: &Extension) -> Result<Vec<Song>, UiError> {
        let resp = ext
            .get_recommendations(RequestedRecommendationsRequest {
                refresh: false,
                related_songs: vec![],
            })
            .await?;
        Ok(resp.songs)
    }

    #[tracing::instrument(level = "debug", skip_all)]
    fn build_provider_recommendation(
        main_window: &MainWindow,
        state_manager: &StateManager,
        item: RecommendationItem,
    ) -> ProviderRecommendations {
        let (detail, cached_path, songs) = item;
        let icon = load_image(cached_path.as_deref().unwrap_or(""));
        let song_models = songs
            .into_iter()
            .map(|s| (s, Some(&detail)).into())
            .collect::<Vec<SongModel>>();
        let mapped_songs = make_lazy_song_model(main_window, state_manager, song_models);
        ProviderRecommendations {
            provider_name: detail.name.into(),
            provider_icon: icon,
            songs: mapped_songs,
        }
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub(crate) fn append_recommendation(
        main_window: &MainWindow,
        state_manager: &StateManager,
        item: RecommendationItem,
    ) {
        let props = main_window.global::<ExplorePageProps>();
        let mut list = props.get_provider_recommendations().into_vec();
        list.push(Self::build_provider_recommendation(
            main_window,
            state_manager,
            item,
        ));
        props.set_provider_recommendations(ModelRc::new(VecModel::from(list)));
    }
}

impl PageHandler for ExplorePageHandler {
    #[tracing::instrument(level = "debug", skip_all)]
    fn on_show(&self) {
        tracing::debug!("ExplorePage: on_show triggered");
        let props = self.main_window.global::<ExplorePageProps>();
        props.set_provider_recommendations(ModelRc::default());
        props.set_is_loading(true);

        tokio::spawn({
            let state_manager = self.state_manager.clone();
            let main_window_weak = self.main_window.as_weak();
            async move {
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
                            if !songs.is_empty() {
                                let cached_path = match &detail.extension_icon {
                                    Some(p) if !p.is_empty() => resolve_image(p, &cache_dir)
                                        .await
                                        .map(|p| p.to_string_lossy().to_string()),
                                    _ => None,
                                };
                                let state_manager_ui = state_manager.clone();
                                let _ =
                                    main_window_weak.upgrade_in_event_loop(move |main_window| {
                                        if main_window.global::<AppProps>().get_active_page()
                                            != Pages::Explore
                                        {
                                            return;
                                        }
                                        Self::append_recommendation(
                                            &main_window,
                                            &state_manager_ui,
                                            (detail, cached_path, songs),
                                        );
                                    });
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

                let _ = main_window_weak.upgrade_in_event_loop(move |main_window| {
                    main_window
                        .global::<ExplorePageProps>()
                        .set_is_loading(false);
                });
            }
            .in_current_span()
        });
    }

    #[tracing::instrument(level = "debug", skip_all)]
    fn on_hide(&self) {
        tracing::debug!("ExplorePage: on_hide clearing recommendations");
        let props = self.main_window.global::<ExplorePageProps>();
        props.set_provider_recommendations(ModelRc::default());
        props.set_is_loading(false);
    }
}
